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
        let hash = token.wrapping_mul(2654435761);
        for (i, coord) in coords.iter_mut().enumerate() {
            let bit = (hash >> (i * 2)) & 0x03;
            *coord = (bit as i32) - 1;
        }
        Self { coords }
    }

    /// Compute Lie product commutator [a, b] = ab - ba
    pub fn lie_product(&self, other: &Self) -> Self {
        let mut res = [0i32; 16];
        for (i, item) in res.iter_mut().enumerate() {
            *item = self.coords[i]
                .wrapping_mul(other.coords[(i + 1) % 16])
                .wrapping_sub(other.coords[i].wrapping_mul(self.coords[(i + 1) % 16]));
        }
        Self { coords: res }
    }

    /// Compute Jordan product {a, b} = ab + ba
    pub fn jordan_product(&self, other: &Self) -> Self {
        let mut res = [0i32; 16];
        for (i, item) in res.iter_mut().enumerate() {
            *item = self.coords[i]
                .wrapping_mul(other.coords[(i + 2) % 16])
                .wrapping_add(other.coords[i].wrapping_mul(self.coords[(i + 2) % 16]));
        }
        Self { coords: res }
    }

    /// Integer norm / trace product in Cl(0,8) endomorphism space
    pub fn endomorphism_scalar_product(&self, other: &Self) -> i32 {
        let mut sum = 0i32;
        for (a, b) in self.coords.iter().zip(other.coords.iter()) {
            sum = sum.saturating_add(a.saturating_mul(*b));
        }
        sum
    }
}

/// Cl(0,8) Endomorphism Operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndomorphismOperator {
    pub left_map: CayleyDicksonVector,
    pub right_map: CayleyDicksonVector,
}

impl EndomorphismOperator {
    pub fn identity() -> Self {
        Self {
            left_map: CayleyDicksonVector::ZERO,
            right_map: CayleyDicksonVector::ZERO,
        }
    }

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
