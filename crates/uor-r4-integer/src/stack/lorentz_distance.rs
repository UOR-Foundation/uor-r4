//! Validated component boundary for the stack's existing Lorentz distance.
//!
//! Coordinates are signed Q16, with their full magnitude and orientation.
//! Exact dot/norm sums feed floor-Q32 lifts; their product and aligned dot
//! yield signed Q64 excess. The nonnegative excess is floored to a Q32 code,
//! clamped to 429, and read through the existing octave-grid Q24 arcosh table.
//! This preserves the stack kernel's rounding and clamp policy; it does not
//! normalize vectors or adopt the older recurrent model's Q8 metric contract.
//!
//! Width 1..4096 and full i32 coordinates bound each norm/dot by 2^74, lift
//! radicands below 2^107, and the excess intermediates below 2^108. These
//! bounds make the reused kernel arithmetic exact before its declared floor
//! operations. Argument codes remain below 2^76, inside the existing table.
//! Large nearly parallel vectors can still be sensitive to Q32 lift rounding;
//! the signed excess and clamp flag expose that effect rather than hiding it.
//!
//! Construction allocates and validates table shape/order. The caller must
//! bind the table's mathematical producer and exact bytes in its artifact;
//! monotonicity alone does not authenticate an arcosh table. Lift preparation
//! and distance evaluation allocate nothing. Learned beta/offset, age, NoRead,
//! softmax, value mixing and full-stack serving are outside this component.

use std::fmt;

use super::kernels::{
    stack_arcosh1p_q24, stack_dot, stack_lift, stack_mul_u128, stack_query_tables,
    ARCOSH_CODE_BITS, ARCOSH_TABLE_LEN,
};
use crate::math::scale_pow2;

pub const LORENTZ_MAX_HEAD_WIDTH: usize = 4096;
/// Existing stack approximation to the excess floor 1e-7 at Q32.
pub const LORENTZ_MIN_EXCESS_CODE: u128 = 429;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LorentzDistanceError {
    InvalidWidth { width: usize },
    TableLength { expected: usize, actual: usize },
    TableOrder { index: usize },
    VectorWidth { expected: usize, actual: usize },
    ArithmeticOverflow,
    ArgumentOutOfRange,
}

impl fmt::Display for LorentzDistanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWidth { width } => write!(
                f,
                "Lorentz head width {width} is outside 1..={LORENTZ_MAX_HEAD_WIDTH}"
            ),
            Self::TableLength { expected, actual } => {
                write!(f, "Lorentz table needs {expected} entries, got {actual}")
            }
            Self::TableOrder { index } => write!(f, "Lorentz table decreases at entry {index}"),
            Self::VectorWidth { expected, actual } => {
                write!(f, "Lorentz vector needs width {expected}, got {actual}")
            }
            Self::ArithmeticOverflow => f.write_str("Lorentz distance arithmetic overflow"),
            Self::ArgumentOutOfRange => {
                f.write_str("Lorentz excess code is outside the arcosh table")
            }
        }
    }
}

impl std::error::Error for LorentzDistanceError {}

/// The immutable coordinate borrow binds a cached lift to its exact input.
/// Fields are private: callers cannot supply a fabricated or stale lift.
#[derive(Clone, Copy, Debug)]
pub struct LiftedVector<'a> {
    coordinates: &'a [i32],
    lift_q32: u64,
}

impl<'a> LiftedVector<'a> {
    pub fn coordinates(self) -> &'a [i32] {
        self.coordinates
    }

    pub fn lift_q32(self) -> u64 {
        self.lift_q32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LorentzDistanceResult {
    pub distance_q24: u32,
    pub signed_excess_q64: i128,
    pub code_q32: u128,
    /// True when the floored nonnegative code was below 429 and the minimum
    /// changed it. An unchanged code exactly equal to 429 is not clamped.
    pub clamped: bool,
}

/// One reusable head-width/table binding and preallocated query scratch.
pub struct IntegerLorentzDistance {
    width: usize,
    arcosh: Vec<u32>,
    query_tables: Vec<[i64; 16]>,
}

impl IntegerLorentzDistance {
    pub fn new(width: usize, table: Vec<u32>) -> Result<Self, LorentzDistanceError> {
        if width == 0 || width > LORENTZ_MAX_HEAD_WIDTH {
            return Err(LorentzDistanceError::InvalidWidth { width });
        }
        if table.len() != ARCOSH_TABLE_LEN {
            return Err(LorentzDistanceError::TableLength {
                expected: ARCOSH_TABLE_LEN,
                actual: table.len(),
            });
        }
        if let Some(index) = table.windows(2).position(|pair| pair[0] > pair[1]) {
            return Err(LorentzDistanceError::TableOrder { index: index + 1 });
        }
        Ok(Self {
            width,
            arcosh: table,
            query_tables: vec![[0; 16]; width],
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    fn validate_width(&self, coordinates: &[i32]) -> Result<(), LorentzDistanceError> {
        if coordinates.len() != self.width {
            return Err(LorentzDistanceError::VectorWidth {
                expected: self.width,
                actual: coordinates.len(),
            });
        }
        Ok(())
    }

    /// Compute the existing floor-Q32 lift once. The returned borrow prevents
    /// coordinate mutation while a cached lift is in use, without copying or
    /// allocating. It can be used by another component of the same head width.
    #[inline(never)]
    pub fn lift<'a>(
        &self,
        coordinates: &'a [i32],
    ) -> Result<LiftedVector<'a>, LorentzDistanceError> {
        self.validate_width(coordinates)?;
        Ok(LiftedVector {
            coordinates,
            lift_q32: stack_lift(coordinates),
        })
    }

    /// Distance between prepared full Q16 vectors. Widths are revalidated for
    /// cross-component use. Only private scratch changes; vectors are immutable
    /// and no partially written result escapes on an error.
    #[inline(never)]
    pub fn evaluate(
        &mut self,
        query: LiftedVector<'_>,
        key: LiftedVector<'_>,
    ) -> Result<LorentzDistanceResult, LorentzDistanceError> {
        self.validate_width(query.coordinates)?;
        self.validate_width(key.coordinates)?;
        stack_query_tables(query.coordinates, &mut self.query_tables);
        let dot = stack_dot(&self.query_tables, key.coordinates);
        // Validated i32 coordinates and bounded widths establish that the
        // reused product fits below 2^107; the signed operations stay checked.
        let product = i128::try_from(stack_mul_u128(
            u128::from(query.lift_q32),
            u128::from(key.lift_q32),
        ))
        .map_err(|_| LorentzDistanceError::ArithmeticOverflow)?;
        let aligned_dot =
            scale_pow2(dot, 32).map_err(|_| LorentzDistanceError::ArithmeticOverflow)?;
        let signed_excess_q64 = product
            .checked_sub(aligned_dot)
            .and_then(|value| value.checked_sub(1i128 << 64))
            .ok_or(LorentzDistanceError::ArithmeticOverflow)?;
        let floored_code = (signed_excess_q64.max(0) >> 32) as u128;
        let code_q32 = floored_code.max(LORENTZ_MIN_EXCESS_CODE);
        Ok(LorentzDistanceResult {
            distance_q24: self.lookup(code_q32)?,
            signed_excess_q64,
            code_q32,
            clamped: floored_code < LORENTZ_MIN_EXCESS_CODE,
        })
    }

    fn lookup(&self, code: u128) -> Result<u32, LorentzDistanceError> {
        // Avoid the legacy helper's table-end saturation. The coordinate bound
        // is much tighter than this range, but retain an explicit boundary.
        if code >= 1u128 << ARCOSH_CODE_BITS {
            return Err(LorentzDistanceError::ArgumentOutOfRange);
        }
        Ok(stack_arcosh1p_q24(code, &self.arcosh))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::kernels::{stack_lorentz_distance, ARCOSH_MANTISSA_BITS};

    // Offline test fixture only; production receives the existing exported
    // table bytes. This is the same declared grid/formula, not a runtime path.
    fn grid(index: usize) -> u128 {
        let direct = 1usize << ARCOSH_MANTISSA_BITS;
        if index < direct {
            return index as u128;
        }
        let octave = (index - direct) >> ARCOSH_MANTISSA_BITS;
        let mantissa = (index - direct) & (direct - 1);
        ((direct + mantissa) as u128) << octave
    }

    fn table() -> Vec<u32> {
        (0..ARCOSH_TABLE_LEN)
            .map(|index| {
                let e = grid(index) as f64 * 2f64.powi(-32);
                ((e + (e * (e + 2.0)).sqrt()).ln_1p() * 2f64.powi(24)).round() as u32
            })
            .collect()
    }

    #[test]
    fn lorentz_distance_preserves_zero_directions_radii_and_units(
    ) -> Result<(), LorentzDistanceError> {
        let mut distance = IntegerLorentzDistance::new(2, table())?;
        let zero = [0, 0];
        let unit = [1 << 16, 0];
        let opposite = [-(1 << 16), 0];
        let orthogonal = [0, 1 << 16];
        let radius_two = [2 << 16, 0];
        let z = distance.lift(&zero)?;
        assert_eq!(z.lift_q32(), 1 << 32);
        assert_eq!(z.coordinates(), &zero);
        let q = distance.lift(&unit)?;
        let zero_result = distance.evaluate(z, z)?;
        assert_eq!(zero_result.signed_excess_q64, 0);
        assert_eq!(zero_result.code_q32, LORENTZ_MIN_EXCESS_CODE);
        assert!(zero_result.clamped);
        let identical = distance.evaluate(q, q)?;
        assert!(identical.signed_excess_q64 <= 0 && identical.clamped);
        assert_eq!(identical.distance_q24, zero_result.distance_q24);
        let mut results = Vec::new();
        for (key, expected) in [
            (&zero, 1f64.asinh()),
            (&opposite, 2.0 * 1f64.asinh()),
            (&orthogonal, 2f64.acosh()),
            (&radius_two, 2f64.asinh() - 1f64.asinh()),
        ] {
            let k = distance.lift(key)?;
            let result = distance.evaluate(q, k)?;
            let real = f64::from(result.distance_q24) * 2f64.powi(-24);
            assert!((real - expected).abs() < 1e-6, "{real} vs {expected}");
            assert!(!result.clamped);
            assert_eq!(result, distance.evaluate(k, q)?, "distance symmetry");
            results.push(result.distance_q24);
        }
        assert!(results[1] > results[2] && results[2] > results[0] && results[0] > results[3]);
        Ok(())
    }

    #[test]
    fn lorentz_distance_matches_existing_kernel_at_bounds_and_cancellation(
    ) -> Result<(), LorentzDistanceError> {
        for width in [1, 16, LORENTZ_MAX_HEAD_WIDTH] {
            let mut distance = IntegerLorentzDistance::new(width, table())?;
            let query: Vec<i32> = (0..width)
                .map(|i| if i & 1 == 0 { i32::MIN } else { i32::MAX })
                .collect();
            let key: Vec<i32> = query
                .iter()
                .map(|&x| if x < 0 { i32::MAX } else { i32::MIN })
                .collect();
            let q = distance.lift(&query)?;
            let k = distance.lift(&key)?;
            let result = distance.evaluate(q, k)?;
            let dot: i128 = query
                .iter()
                .zip(&key)
                .map(|(&a, &b)| i128::from(a) * i128::from(b))
                .sum();
            let expected_excess =
                i128::from(q.lift_q32()) * i128::from(k.lift_q32()) - (dot << 32) - (1i128 << 64);
            assert_eq!(result.signed_excess_q64, expected_excess);
            assert_eq!(
                result.distance_q24,
                stack_lorentz_distance(q.lift_q32(), k.lift_q32(), dot, &distance.arcosh)
            );
            assert!(result.code_q32 < 1u128 << 76 && !result.clamped);
            let identical = distance.evaluate(q, q)?;
            assert!(identical.signed_excess_q64 <= 0 && identical.clamped);
            let mut nearby = query.clone();
            nearby[0] += 1;
            let nearby = distance.lift(&nearby)?;
            let result = distance.evaluate(q, nearby)?;
            let dot: i128 = query
                .iter()
                .zip(nearby.coordinates())
                .map(|(&a, &b)| i128::from(a) * i128::from(b))
                .sum();
            assert_eq!(
                result.distance_q24,
                stack_lorentz_distance(q.lift_q32(), nearby.lift_q32(), dot, &distance.arcosh)
            );
        }
        Ok(())
    }

    #[test]
    fn lorentz_distance_table_grid_and_interpolation_match_declared_policy(
    ) -> Result<(), LorentzDistanceError> {
        // A monotone synthetic table makes indexing and interpolation visible;
        // this test does not claim its entries are mathematical distances.
        let distance = IntegerLorentzDistance::new(1, (0..ARCOSH_TABLE_LEN as u32).collect())?;
        for index in [
            0,
            1,
            429,
            1023,
            1024,
            2047,
            2048,
            4096,
            ARCOSH_TABLE_LEN - 2,
        ] {
            assert_eq!(distance.lookup(grid(index))?, index as u32);
        }
        let index = 40_000;
        let low = grid(index);
        let high = grid(index + 1);
        let midpoint = low + ((high - low) >> 1);
        assert_eq!(distance.lookup(midpoint - 1)?, index as u32);
        assert_eq!(
            distance.lookup(midpoint)?,
            index as u32 + 1,
            "half rounds up"
        );
        assert_eq!(
            distance.lookup(1u128 << ARCOSH_CODE_BITS),
            Err(LorentzDistanceError::ArgumentOutOfRange)
        );
        Ok(())
    }

    #[test]
    fn lorentz_distance_rejects_bad_tables_widths_and_cross_width_cached_vectors(
    ) -> Result<(), LorentzDistanceError> {
        for width in [0, LORENTZ_MAX_HEAD_WIDTH + 1] {
            assert!(matches!(
                IntegerLorentzDistance::new(width, table()),
                Err(LorentzDistanceError::InvalidWidth { .. })
            ));
        }
        assert!(matches!(
            IntegerLorentzDistance::new(1, vec![]),
            Err(LorentzDistanceError::TableLength { .. })
        ));
        let mut bad_table = table();
        bad_table[2] = 0;
        assert!(matches!(
            IntegerLorentzDistance::new(1, bad_table),
            Err(LorentzDistanceError::TableOrder { index: 2 })
        ));
        let other = IntegerLorentzDistance::new(1, table())?;
        let wrong_width = other.lift(&[0])?;
        let mut distance = IntegerLorentzDistance::new(2, table())?;
        assert!(matches!(
            distance.lift(&[0]),
            Err(LorentzDistanceError::VectorWidth { .. })
        ));
        let good = distance.lift(&[0, 0])?;
        let expected = distance.evaluate(good, good)?;
        assert!(matches!(
            distance.evaluate(wrong_width, good),
            Err(LorentzDistanceError::VectorWidth { .. })
        ));
        assert!(matches!(
            distance.evaluate(good, wrong_width),
            Err(LorentzDistanceError::VectorWidth { .. })
        ));
        assert_eq!(distance.evaluate(good, good)?, expected);
        Ok(())
    }
}
