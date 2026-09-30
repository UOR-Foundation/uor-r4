//! Finite spherical-harmonic features for offline B2 learning.
//!
//! The proposed kernel and normalization policy require pre-registration before
//! model compute. A feature identity is not checkpoint or language parity.
//!
//! For unit input rows x,y, band h_l has inner product P_l(x.y), where P_l is
//! the Gegenbauer polynomial normalized at one. The corresponding harmonics
//! orthonormal under spherical probability measure are sqrt(N_l) * h_l.
//! A band itself is therefore addition-formula normalized, not a list of
//! functions each having unit L2 norm under the probability measure.
//!
//! forward concatenates sqrt(b_l)*h_l and realizes
//! delta + ((1 + x.y)/2)^L, L in 1..=3. It assumes unit input rows, retains no
//! recurrent state, and makes no GQA, sparse-selection, or serving claim.

use std::fmt;

use candle_core::{DType, Device, Tensor};

/// Coordinate ordering is part of the eventual learned artifact contract.
pub const FEATURE_ORDER_VERSION: &str = "harmonic-packed-householder-v1";

#[derive(Debug)]
pub enum HarmonicError {
    Tensor(candle_core::Error),
    InvalidConfig(String),
    InvalidInput(String),
}

impl fmt::Display for HarmonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tensor(error) => write!(f, "harmonic tensor operation: {error}"),
            Self::InvalidConfig(message) => write!(f, "harmonic configuration: {message}"),
            Self::InvalidInput(message) => write!(f, "harmonic input: {message}"),
        }
    }
}

impl std::error::Error for HarmonicError {}

impl From<candle_core::Error> for HarmonicError {
    fn from(error: candle_core::Error) -> Self {
        Self::Tensor(error)
    }
}

pub type Result<T> = std::result::Result<T, HarmonicError>;

/// Unit rows plus observable uses of the fixed e0 fallback.
pub struct NormalizedRows {
    pub unit: Tensor,
    pub fallback_rows: usize,
    /// Detached squared norms of the original input, one per row. These are
    /// retained for diagnostics of the amplitude discarded by normalization.
    pub squared_norms: Vec<f32>,
}

/// Normalize nonempty F32 [rows, dimensions] input exactly onto the sphere in
/// real arithmetic. If norm < epsilon, use e0. Both branches remain finite at
/// zero, so the unused division cannot introduce a NaN into backward.
///
/// This helper copies only the row squared norms to the host for fallback and
/// finite-value reporting. Nonfinite/overflowed squared norms are explicit
/// errors, not unreported fallback rows. At the threshold, the ordinary branch
/// is selected. The fallback branch has zero input derivative by construction.
pub fn normalize_rows(input: &Tensor, epsilon: f64) -> Result<NormalizedRows> {
    let (rows, dimensions) = input.dims2()?;
    if input.dtype() != DType::F32 || rows == 0 || dimensions == 0 {
        return Err(HarmonicError::InvalidInput(
            "normalization requires nonempty F32 [rows, dimensions]".into(),
        ));
    }
    let epsilon_squared = (epsilon * epsilon) as f32;
    if !epsilon.is_finite()
        || epsilon <= 0.0
        || !epsilon_squared.is_finite()
        || epsilon_squared <= 0.0
    {
        return Err(HarmonicError::InvalidConfig(
            "epsilon and its F32 square must be finite and strictly positive".into(),
        ));
    }
    let squared = input.sqr()?.sum_keepdim(1)?;
    let squared_norms = squared.detach().flatten_all()?.to_vec1::<f32>()?;
    if squared_norms.iter().any(|value| !value.is_finite()) {
        return Err(HarmonicError::InvalidInput(
            "a row has a nonfinite or overflowed squared norm".into(),
        ));
    }
    let fallback_rows = squared_norms
        .iter()
        .filter(|value| **value < epsilon_squared)
        .count();
    let denominator = squared
        .clamp(epsilon_squared as f64, f32::MAX as f64)?
        .sqrt()?;
    let ordinary = input.broadcast_div(&denominator)?;
    let first = Tensor::ones((rows, 1), DType::F32, input.device())?;
    let fallback = if dimensions == 1 {
        first
    } else {
        let tail = Tensor::zeros((rows, dimensions - 1), DType::F32, input.device())?;
        Tensor::cat(&[&first, &tail], 1)?
    };
    let ordinary_mask = squared
        .ge(epsilon_squared as f64)?
        .broadcast_as((rows, dimensions))?;
    let unit = ordinary_mask.where_cond(&ordinary, &fallback)?;
    Ok(NormalizedRows {
        unit,
        fallback_rows,
        squared_norms,
    })
}

/// H = I - w*w^T, with w = sqrt(2/(v.v))*v and v = c/||c|| - e0.
/// Only w is stored: no dense dimension-by-dimension basis is allocated.
struct TraceRemoval {
    width: usize,
    reflection: Tensor,
}

impl TraceRemoval {
    fn new(trace_direction: &[f64], device: &Device) -> Result<Self> {
        if trace_direction.len() < 2 || trace_direction.iter().any(|value| !value.is_finite()) {
            return Err(HarmonicError::InvalidConfig(
                "trace direction must contain at least two finite coordinates".into(),
            ));
        }
        let norm = trace_direction.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !norm.is_finite() || norm == 0.0 {
            return Err(HarmonicError::InvalidConfig(
                "trace direction must have a finite positive norm".into(),
            ));
        }
        let mut vector = trace_direction
            .iter()
            .enumerate()
            .map(|(index, value)| value / norm - if index == 0 { 1.0 } else { 0.0 })
            .collect::<Vec<_>>();
        let norm_squared = vector.iter().map(|value| value * value).sum::<f64>();
        if !norm_squared.is_finite() || norm_squared <= 0.0 {
            return Err(HarmonicError::InvalidConfig(
                "trace direction is parallel to e0; this reflection is undefined".into(),
            ));
        }
        let scale = (2.0 / norm_squared).sqrt();
        for value in &mut vector {
            *value *= scale;
        }
        let values = vector
            .into_iter()
            .map(|value| value as f32)
            .collect::<Vec<_>>();
        let width = values.len();
        Ok(Self {
            width,
            reflection: Tensor::from_vec(values, (1, width), device)?,
        })
    }

    /// Each row is one packed group. Reflect, then discard its trace coordinate.
    fn forward(&self, groups: &Tensor) -> Result<Tensor> {
        let (_, width) = groups.dims2()?;
        if width != self.width {
            return Err(HarmonicError::InvalidInput(
                "packed trace group has the wrong width".into(),
            ));
        }
        let projection = groups.broadcast_mul(&self.reflection)?.sum_keepdim(1)?;
        let reflected = groups.sub(&projection.broadcast_mul(&self.reflection)?)?;
        Ok(reflected.narrow(1, 1, width - 1)?)
    }
}

struct DegreeTwo {
    left: Tensor,
    right: Tensor,
    diagonal_trace: TraceRemoval,
}

impl DegreeTwo {
    fn new(dimensions: usize, device: &Device) -> Result<Self> {
        let mut left = Vec::<u32>::new();
        let mut right = Vec::<u32>::new();
        for i in 0..dimensions {
            for j in i + 1..dimensions {
                left.push(i as u32);
                right.push(j as u32);
            }
        }
        let count = left.len();
        Ok(Self {
            left: Tensor::from_vec(left, count, device)?,
            right: Tensor::from_vec(right, count, device)?,
            diagonal_trace: TraceRemoval::new(&vec![1.0; dimensions], device)?,
        })
    }

    fn forward(&self, unit: &Tensor, dimensions: usize) -> Result<Tensor> {
        let diagonal = self.diagonal_trace.forward(&unit.sqr()?)?;
        let off_diagonal = unit
            .index_select(&self.left, 1)?
            .mul(&unit.index_select(&self.right, 1)?)?
            .affine(2.0f64.sqrt(), 0.0)?;
        Ok(Tensor::cat(&[&diagonal, &off_diagonal], 1)?
            .affine((dimensions as f64 / (dimensions - 1) as f64).sqrt(), 0.0)?)
    }
}

struct DegreeThree {
    distinct_i: Tensor,
    distinct_j: Tensor,
    distinct_k: Tensor,
    singleton: Tensor,
    repeated: Tensor,
    group_scale: Tensor,
    group_trace: TraceRemoval,
}

impl DegreeThree {
    fn new(dimensions: usize, device: &Device) -> Result<Self> {
        let mut distinct_i = Vec::<u32>::new();
        let mut distinct_j = Vec::<u32>::new();
        let mut distinct_k = Vec::<u32>::new();
        for i in 0..dimensions {
            for j in i + 1..dimensions {
                for k in j + 1..dimensions {
                    distinct_i.push(i as u32);
                    distinct_j.push(j as u32);
                    distinct_k.push(k as u32);
                }
            }
        }
        let distinct_count = distinct_i.len();
        let mut singleton = Vec::<u32>::with_capacity(dimensions * dimensions);
        let mut repeated = Vec::<u32>::with_capacity(dimensions * dimensions);
        let mut group_scale = Vec::<f32>::with_capacity(dimensions * dimensions);
        // Group i: x_i^3 first; then sqrt(3)*x_i*x_j^2 for ascending j != i.
        for i in 0..dimensions {
            singleton.push(i as u32);
            repeated.push(i as u32);
            group_scale.push(1.0);
            for j in 0..dimensions {
                if i != j {
                    singleton.push(i as u32);
                    repeated.push(j as u32);
                    group_scale.push(3.0f32.sqrt());
                }
            }
        }
        let mut trace_direction = vec![1.0 / 3.0f64.sqrt(); dimensions];
        if let Some(first) = trace_direction.first_mut() {
            *first = 1.0;
        }
        Ok(Self {
            distinct_i: Tensor::from_vec(distinct_i, distinct_count, device)?,
            distinct_j: Tensor::from_vec(distinct_j, distinct_count, device)?,
            distinct_k: Tensor::from_vec(distinct_k, distinct_count, device)?,
            singleton: Tensor::from_vec(singleton, dimensions * dimensions, device)?,
            repeated: Tensor::from_vec(repeated, dimensions * dimensions, device)?,
            group_scale: Tensor::from_vec(group_scale, (1, dimensions * dimensions), device)?,
            group_trace: TraceRemoval::new(&trace_direction, device)?,
        })
    }

    fn forward(&self, unit: &Tensor, dimensions: usize) -> Result<Tensor> {
        let (rows, _) = unit.dims2()?;
        let distinct = unit
            .index_select(&self.distinct_i, 1)?
            .mul(&unit.index_select(&self.distinct_j, 1)?)?
            .mul(&unit.index_select(&self.distinct_k, 1)?)?
            .affine(6.0f64.sqrt(), 0.0)?;
        let groups = unit
            .index_select(&self.singleton, 1)?
            .mul(&unit.index_select(&self.repeated, 1)?.sqr()?)?
            .broadcast_mul(&self.group_scale)?
            .reshape((rows * dimensions, dimensions))?;
        let trace_free = self
            .group_trace
            .forward(&groups)?
            .contiguous()?
            .reshape((rows, dimensions * (dimensions - 1)))?;
        Ok(Tensor::cat(&[&distinct, &trace_free], 1)?.affine(
            ((dimensions + 2) as f64 / (dimensions - 1) as f64).sqrt(),
            0.0,
        )?)
    }
}

/// Fixed positive kernel feature map. Parameters are mathematical/configuration
/// constants; only input tensors carry learned projection/LoRA derivatives.
/// Positivity describes the ideal real-arithmetic kernel. F32 cancellation can
/// erode its delta floor, and tiny deltas can round out of the constant band.
/// Consumers must measure actual scores/denominators and reject nonpositive or
/// nonfinite values; this constructor does not promise numerical positivity.
pub struct HarmonicFeatures {
    dimensions: usize,
    degree: usize,
    delta: f64,
    band_weights: Vec<f64>,
    degree_two: Option<DegreeTwo>,
    degree_three: Option<DegreeThree>,
}

impl HarmonicFeatures {
    pub fn new(dimensions: usize, degree: usize, delta: f64, device: &Device) -> Result<Self> {
        if !matches!(dimensions, 16 | 32) || !(1..=3).contains(&degree) {
            return Err(HarmonicError::InvalidConfig(
                "registered arms require dimensions 16/32 and degree 1/2/3".into(),
            ));
        }
        if !delta.is_finite()
            || delta <= 0.0
            || !(delta as f32).is_finite()
            || (delta as f32) <= 0.0
        {
            return Err(HarmonicError::InvalidConfig(
                "delta must be finite and strictly positive in F32".into(),
            ));
        }
        let d = dimensions as f64;
        let band_weights = match degree {
            1 => vec![delta + 0.5, 0.5],
            2 => vec![delta + (d + 1.0) / (4.0 * d), 0.5, (d - 1.0) / (4.0 * d)],
            3 => vec![
                delta + (d + 3.0) / (8.0 * d),
                3.0 * (d + 3.0) / (8.0 * (d + 2.0)),
                3.0 * (d - 1.0) / (8.0 * d),
                (d - 1.0) / (8.0 * (d + 2.0)),
            ],
            _ => return Err(HarmonicError::InvalidConfig("unsupported degree".into())),
        };
        Ok(Self {
            dimensions,
            degree,
            delta,
            band_weights,
            degree_two: if degree >= 2 {
                Some(DegreeTwo::new(dimensions, device)?)
            } else {
                None
            },
            degree_three: if degree >= 3 {
                Some(DegreeThree::new(dimensions, device)?)
            } else {
                None
            },
        })
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
    pub fn degree(&self) -> usize {
        self.degree
    }
    pub fn delta(&self) -> f64 {
        self.delta
    }
    pub fn band_weights(&self) -> &[f64] {
        &self.band_weights
    }

    pub fn band_dimensions(&self) -> Vec<usize> {
        let d = self.dimensions;
        let mut result = vec![1, d];
        if self.degree >= 2 {
            result.push(d * (d + 1) / 2 - 1);
        }
        if self.degree >= 3 {
            result.push(d * (d + 1) * (d + 2) / 6 - d);
        }
        result
    }

    pub fn feature_dimensions(&self) -> usize {
        self.band_dimensions().into_iter().sum()
    }

    /// PRECONDITION: every row lies on S^(d-1), within measured F32 error.
    /// This function checks shape and dtype but does not silently renormalize.
    /// Use normalize_rows once after projection, and record its fallback count.
    ///
    /// Coordinate order: h0; h1 in input order; h2 retained reflected diagonal
    /// then lexicographic i<j; h3 lexicographic i<j<k then ascending singleton
    /// groups with their first reflected coordinate removed.
    pub fn bands(&self, unit: &Tensor) -> Result<Vec<Tensor>> {
        let (rows, dimensions) = unit.dims2()?;
        if rows == 0 || dimensions != self.dimensions || unit.dtype() != DType::F32 {
            return Err(HarmonicError::InvalidInput(
                "features require nonempty F32 [rows, configured dimensions] unit rows".into(),
            ));
        }
        let mut bands = vec![
            Tensor::ones((rows, 1), DType::F32, unit.device())?,
            unit.clone(),
        ];
        if let Some(map) = &self.degree_two {
            bands.push(map.forward(unit, dimensions)?);
        }
        if let Some(map) = &self.degree_three {
            bands.push(map.forward(unit, dimensions)?);
        }
        Ok(bands)
    }

    /// PRECONDITION: unit rows, as described by bands().
    pub fn forward(&self, unit: &Tensor) -> Result<Tensor> {
        let weighted = self
            .bands(unit)?
            .into_iter()
            .zip(&self.band_weights)
            .map(|(band, weight)| band.affine(weight.sqrt(), 0.0))
            .collect::<candle_core::Result<Vec<_>>>()?;
        Ok(Tensor::cat(&weighted, 1)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Var;

    fn points(dimensions: usize) -> Result<Tensor> {
        let mut points = vec![0f32; 5 * dimensions];
        points[0] = 1.0;
        points[dimensions + 1] = 1.0;
        points[2 * dimensions] = -1.0;
        for row in 3..5 {
            let values = (0..dimensions)
                .map(|i| ((i * 7 + row * 3) % 19) as f64 - 9.0)
                .collect::<Vec<_>>();
            let norm = values.iter().map(|v| v * v).sum::<f64>().sqrt();
            for (i, value) in values.into_iter().enumerate() {
                points[row * dimensions + i] = (value / norm) as f32;
            }
        }
        Ok(Tensor::from_vec(points, (5, dimensions), &Device::Cpu)?)
    }

    fn polynomial(degree: usize, dot: f64, dimensions: usize) -> f64 {
        let d = dimensions as f64;
        match degree {
            0 => 1.0,
            1 => dot,
            2 => (d * dot * dot - 1.0) / (d - 1.0),
            3 => ((d + 2.0) * dot * dot * dot - 3.0 * dot) / (d - 1.0),
            _ => f64::NAN,
        }
    }

    #[test]
    fn packed_band_grams_match_normalized_gegenbauer() -> Result<()> {
        for dimensions in [16, 32] {
            let unit = points(dimensions)?;
            let dot = unit.matmul(&unit.t()?)?.to_vec2::<f32>()?;
            let map = HarmonicFeatures::new(dimensions, 3, 1e-6, &Device::Cpu)?;
            for (degree, band) in map.bands(&unit)?.iter().enumerate() {
                let gram = band.matmul(&band.t()?)?.to_vec2::<f32>()?;
                for (actual_row, dot_row) in gram.iter().zip(&dot) {
                    for (&actual, &t) in actual_row.iter().zip(dot_row) {
                        let expected = polynomial(degree, t as f64, dimensions);
                        assert!(
                            (actual as f64 - expected).abs() < 4e-5,
                            "d={dimensions}, l={degree}, got={actual}, expected={expected}"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn all_arms_have_minimal_dimension_and_positive_shifted_power_kernel() -> Result<()> {
        for (dimensions, expected_dimensions) in [(16, [17, 152, 952]), (32, [33, 560, 6512])] {
            let unit = points(dimensions)?;
            let dot = unit.matmul(&unit.t()?)?.to_vec2::<f32>()?;
            for (index, expected_dimension) in expected_dimensions.into_iter().enumerate() {
                let degree = index + 1;
                let map = HarmonicFeatures::new(dimensions, degree, 1e-6, &Device::Cpu)?;
                let feature = map.forward(&unit)?;
                assert_eq!(map.feature_dimensions(), expected_dimension);
                assert_eq!(feature.dims(), &[5, expected_dimension]);
                let gram = feature.matmul(&feature.t()?)?.to_vec2::<f32>()?;
                for (actual_row, dot_row) in gram.iter().zip(&dot) {
                    for (&actual, &t) in actual_row.iter().zip(dot_row) {
                        let expected = 1e-6 + ((1.0 + t as f64) / 2.0).powi(degree as i32);
                        assert!(actual > 0.0, "d={dimensions}, l={degree}, score={actual}");
                        assert!((actual as f64 - expected).abs() < 4e-5);
                    }
                }
            }
        }
        Ok(())
    }

    fn feature_gradient_matches_finite_difference(
        dimensions: usize,
        device: &Device,
    ) -> Result<()> {
        let input_values = (0..dimensions)
            .map(|i| (i as f32 - 5.0) / 10.0)
            .collect::<Vec<_>>();
        let input = Var::from_tensor(&Tensor::from_slice(&input_values, (1, dimensions), device)?)?;
        let map = HarmonicFeatures::new(dimensions, 3, 1e-6, device)?;
        let reference = points(dimensions)?.narrow(0, 4, 1)?.to_device(device)?;
        let target = map.forward(&reference)?.detach();
        let objective = |tensor: &Tensor| -> Result<Tensor> {
            let normalized = normalize_rows(tensor, 1e-6)?;
            Ok(map.forward(&normalized.unit)?.mul(&target)?.sum_all()?)
        };
        let gradients = objective(input.as_tensor())?.backward()?;
        let gradient = gradients
            .get(&input)
            .ok_or_else(|| {
                HarmonicError::InvalidInput(
                    "expected an input gradient through normalization/gathers/Householder".into(),
                )
            })?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(gradient.iter().all(|value| value.is_finite()));
        assert!(gradient.iter().any(|value| value.abs() > 1e-6));
        let epsilon = 1e-3f32;
        let mut plus = input_values.clone();
        let mut minus = input_values;
        plus[3] += epsilon;
        minus[3] -= epsilon;
        let upper =
            objective(&Tensor::from_vec(plus, (1, dimensions), device)?)?.to_scalar::<f32>()?;
        let lower =
            objective(&Tensor::from_vec(minus, (1, dimensions), device)?)?.to_scalar::<f32>()?;
        let finite_difference = (upper - lower) / (2.0 * epsilon);
        assert!(
            (gradient[3] - finite_difference).abs() < 2e-4,
            "autodiff={}, finite_difference={finite_difference}",
            gradient[3]
        );
        Ok(())
    }

    #[test]
    fn packed_features_backpropagate_through_normalization_and_match_finite_difference(
    ) -> Result<()> {
        feature_gradient_matches_finite_difference(16, &Device::Cpu)
    }

    #[cfg(feature = "metal")]
    #[test]
    fn largest_harmonic_arm_backpropagates_on_metal() -> Result<()> {
        feature_gradient_matches_finite_difference(32, &Device::new_metal(0)?)
    }

    #[test]
    fn zero_and_small_rows_use_counted_finite_unit_fallback() -> Result<()> {
        let mut values = vec![0f32; 3 * 16];
        values[16] = 1e-8;
        values[32 + 1] = 2.0;
        let variable = Var::from_tensor(&Tensor::from_vec(values, (3, 16), &Device::Cpu)?)?;
        let normalized = normalize_rows(variable.as_tensor(), 1e-6)?;
        assert_eq!(normalized.fallback_rows, 2);
        let rows = normalized.unit.to_vec2::<f32>()?;
        assert_eq!(rows[0][0], 1.0);
        assert_eq!(rows[1][0], 1.0);
        assert_eq!(rows[2][1], 1.0);
        for row in &rows {
            assert!(row.iter().all(|value| value.is_finite()));
            assert!((row.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-6);
        }
        let gradients = normalized.unit.sum_all()?.backward()?;
        let gradient = gradients
            .get(&variable)
            .ok_or_else(|| HarmonicError::InvalidInput("normalization gradient missing".into()))?
            .to_vec2::<f32>()?;
        for row in &gradient[..2] {
            assert!(row.iter().all(|value| value.is_finite() && *value == 0.0));
        }
        assert!(HarmonicFeatures::new(8, 2, 1e-6, &Device::Cpu).is_err());
        assert!(HarmonicFeatures::new(16, 0, 1e-6, &Device::Cpu).is_err());
        assert!(HarmonicFeatures::new(16, 2, 0.0, &Device::Cpu).is_err());
        Ok(())
    }

    fn distributed_antipodes(device: &Device) -> Result<()> {
        for dimensions in [16, 32] {
            let mut values = (0..dimensions)
                .map(|i| ((i * 13 + 7) % 23) as f32 - 11.0)
                .collect::<Vec<_>>();
            let norm = values
                .iter()
                .map(|x| f64::from(*x).powi(2))
                .sum::<f64>()
                .sqrt();
            for value in &mut values {
                *value = (f64::from(*value) / norm) as f32;
            }
            for perturbation in [0.0f32, 1e-4, 1e-2] {
                let mut opposite = values.iter().map(|x| -*x).collect::<Vec<_>>();
                opposite[3] += perturbation;
                let opposite_norm = opposite
                    .iter()
                    .map(|x| f64::from(*x).powi(2))
                    .sum::<f64>()
                    .sqrt();
                for value in &mut opposite {
                    *value = (f64::from(*value) / opposite_norm) as f32;
                }
                let both =
                    Tensor::from_vec([values.clone(), opposite].concat(), (2, dimensions), device)?;
                let unit = normalize_rows(&both, 1e-6)?.unit;
                let rows = unit.to_vec2::<f32>()?;
                let dot = rows[0]
                    .iter()
                    .zip(&rows[1])
                    .map(|(a, b)| f64::from(*a) * f64::from(*b))
                    .sum::<f64>();
                let norm_product = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|x| f64::from(*x).powi(2))
                            .sum::<f64>()
                            .sqrt()
                    })
                    .product::<f64>();
                let cosine = (dot / norm_product).clamp(-1.0, 1.0);
                for degree in 1..=3 {
                    let map = HarmonicFeatures::new(dimensions, degree, 1e-6, device)?;
                    let lifted = map.forward(&unit)?;
                    let actual = lifted.matmul(&lifted.t()?)?.to_vec2::<f32>()?[0][1] as f64;
                    let expected = 1e-6 + ((1.0 + cosine) / 2.0).powi(degree as i32);
                    // Separate floor-sensitive check: the general Gram tolerance
                    // alone exceeds delta by forty times and cannot certify it.
                    assert!(actual.is_finite() && actual > 0.0,
                        "nonpositive score d={dimensions},L={degree},perturb={perturbation}: {actual}");
                    assert!((actual-expected).abs() <= 5e-7 + 2e-5*(expected-1e-6),
                        "floor error d={dimensions},L={degree},perturb={perturbation}: actual={actual},expected={expected}");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn distributed_antipodal_scores_preserve_measured_floor_cpu() -> Result<()> {
        distributed_antipodes(&Device::Cpu)
    }

    #[cfg(feature = "metal")]
    #[test]
    fn distributed_antipodal_scores_preserve_measured_floor_metal() -> Result<()> {
        distributed_antipodes(&Device::new_metal(0)?)
    }

    /// Diagnostic only: keep the acceptance test and its tolerance unchanged.
    /// Replaying identical normalized rows separates feature construction from
    /// the final dot-product reduction on the observed failing Metal fixture.
    #[cfg(feature = "metal")]
    #[test]
    #[ignore = "explicit cross-backend cancellation diagnostic; not an acceptance gate"]
    fn diagnose_metal_antipodal_floor_reduction() -> Result<()> {
        let dimensions = 32;
        let mut values = (0..dimensions)
            .map(|i| ((i * 13 + 7) % 23) as f32 - 11.0)
            .collect::<Vec<_>>();
        let norm = values
            .iter()
            .map(|v| f64::from(*v).powi(2))
            .sum::<f64>()
            .sqrt();
        for v in &mut values {
            *v = (f64::from(*v) / norm) as f32;
        }
        let mut opposite = values.iter().map(|v| -*v).collect::<Vec<_>>();
        opposite[3] += 0.01;
        let norm = opposite
            .iter()
            .map(|v| f64::from(*v).powi(2))
            .sum::<f64>()
            .sqrt();
        for v in &mut opposite {
            *v = (f64::from(*v) / norm) as f32;
        }
        let metal = Device::new_metal(0)?;
        let both = Tensor::from_vec([values, opposite].concat(), (2, dimensions), &metal)?;
        let unit = normalize_rows(&both, 1e-6)?.unit;
        let rows = unit.to_vec2::<f32>()?;
        let dot = |a: &[f32], b: &[f32]| -> f64 {
            a.iter()
                .zip(b)
                .map(|(x, y)| f64::from(*x) * f64::from(*y))
                .sum()
        };
        let cosine = (dot(&rows[0], &rows[1])
            / (dot(&rows[0], &rows[0]) * dot(&rows[1], &rows[1])).sqrt())
        .clamp(-1.0, 1.0);
        for (name, device) in [("cpu", Device::Cpu), ("metal", metal.clone())] {
            // No second normalization: each backend receives identical F32 bytes.
            let pinned = unit.to_device(&device)?;
            let features = HarmonicFeatures::new(dimensions, 3, 1e-6, &device)?.forward(&pinned)?;
            let host = features.to_vec2::<f32>()?;
            let metal_features = features.to_device(&metal)?;
            let cpu_features = features.to_device(&Device::Cpu)?;
            let products = metal_features
                .narrow(0, 0, 1)?
                .mul(&metal_features.narrow(0, 1, 1)?)?;
            println!(
                "{}",
                serde_json::json!({
                    "schema": "uor-r4.harmonic-floor-diagnostic/1",
                    "construction_backend": name,
                    "dimensions": dimensions, "degree": 3, "perturbation": 0.01,
                    "feature_count": host[0].len(),
                    "expected_unit_kernel": 1e-6 + ((1.0 + cosine) / 2.0).powi(3),
                    "host_f64_dot": dot(&host[0], &host[1]),
                    "cpu_matmul": cpu_features.matmul(&cpu_features.t()?)?.to_vec2::<f32>()?[0][1],
                    "metal_matmul": metal_features.matmul(&metal_features.t()?)?.to_vec2::<f32>()?[0][1],
                    "metal_elementwise_sum": products.sum_all()?.to_scalar::<f32>()?,
                    "acceptance": "DIAGNOSTIC_ONLY"
                })
            );
        }
        Ok(())
    }

    #[test]
    fn normalization_and_feature_boundaries_reject_invalid_values() -> Result<()> {
        let valid = Tensor::zeros((1, 16), DType::F32, &Device::Cpu)?;
        for epsilon in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e-100] {
            assert!(normalize_rows(&valid, epsilon).is_err());
        }
        for value in [f32::NAN, f32::INFINITY, f32::MAX] {
            let input = Tensor::full(value, (1, 16), &Device::Cpu)?;
            assert!(normalize_rows(&input, 1e-6).is_err());
        }
        let map = HarmonicFeatures::new(16, 2, 1e-6, &Device::Cpu)?;
        assert!(map
            .forward(&Tensor::zeros((1, 32), DType::F32, &Device::Cpu)?)
            .is_err());
        assert!(map.forward(&valid.to_dtype(DType::F64)?).is_err());
        Ok(())
    }
}
