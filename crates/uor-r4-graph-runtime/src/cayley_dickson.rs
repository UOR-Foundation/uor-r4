//! Cayley-Dickson (\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O} \subset \mathbb{V}) Endomorphic Routing
//! Implementation based on N. Furey (arXiv:2607.18450v1).

/// Discrete 16-dimensional vector in Cayley-Dickson space V = e_i O \oplus e_5 H \oplus e_6 C \oplus e_7 R \oplus R
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CayleyDicksonVector {
    pub coords: [i32; 16],
}

impl CayleyDicksonVector {
    pub const ZERO: Self = Self { coords: [0; 16] };

    pub fn from_u32(token: u32) -> Self {
        let mut coords = [0i32; 16];
        let hash = wrapping_mul_u32(token, 0x9E37_79B1);
        for (i, coord) in coords.iter_mut().enumerate() {
            let bit = (hash >> (i << 1)) & 0x03;
            *coord = (bit as i32) - 1;
        }
        Self { coords }
    }

    /// Integer norm / trace product in Cl(0,8) endomorphism space
    pub fn endomorphism_scalar_product(&self, other: &Self) -> i32 {
        let mut sum = 0i32;
        for (a, b) in self.coords.iter().zip(other.coords.iter()) {
            sum = sum.saturating_add(saturating_mul_small(*a, *b));
        }
        sum
    }
}

fn wrapping_mul_u32(mut lhs: u32, mut rhs: u32) -> u32 {
    let mut acc = 0u32;
    while rhs != 0 {
        if (rhs & 1) != 0 {
            acc = acc.wrapping_add(lhs);
        }
        lhs = lhs.wrapping_shl(1);
        rhs >>= 1;
    }
    acc
}

fn saturating_mul_small(lhs: i32, rhs: i32) -> i32 {
    if lhs == 0 || rhs == 0 {
        return 0;
    }

    let negative = (lhs < 0) ^ (rhs < 0);
    let mut a = lhs.unsigned_abs() as i64;
    let mut b = rhs.unsigned_abs() as i64;
    let limit = if negative {
        (i32::MAX as i64).saturating_add(1)
    } else {
        i32::MAX as i64
    };
    let mut acc = 0i64;

    while b != 0 {
        if (b & 1) != 0 {
            acc = acc.saturating_add(a);
            if acc > limit {
                return if negative { i32::MIN } else { i32::MAX };
            }
        }

        b >>= 1;
        if b != 0 {
            a = a.saturating_add(a);
            if a > limit {
                a = limit.saturating_add(1);
            }
        }
    }

    if negative {
        if acc >= limit {
            i32::MIN
        } else {
            -(acc as i32)
        }
    } else {
        if acc > i32::MAX as i64 {
            i32::MAX
        } else {
            acc as i32
        }
    }
}

/// Cl(0,8) Endomorphism Operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndomorphismOperator {
    pub left_map: CayleyDicksonVector,
    pub right_map: CayleyDicksonVector,
}

impl EndomorphismOperator {
    pub fn from_token_transition(prev: u32, next: u32) -> Self {
        Self {
            left_map: CayleyDicksonVector::from_u32(prev),
            right_map: CayleyDicksonVector::from_u32(next),
        }
    }

    /// Centralizer volume element projection score
    pub fn centralizer_score(&self, state: &CayleyDicksonVector) -> i32 {
        let left_score = self.left_map.endomorphism_scalar_product(state);
        let right_score = self.right_map.endomorphism_scalar_product(state);
        left_score.saturating_add(right_score)
    }
}
