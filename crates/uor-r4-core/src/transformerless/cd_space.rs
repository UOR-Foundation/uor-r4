//! Cayley-Dickson 16D vector space substrate and nested division algebraic inclusions.
//!
//! Based on N. Furey (2026), "Standard Model Symmetries and the Nested Embeddings
//! of R ⊂ C ⊂ H ⊂ O" (arXiv:2607.18450v1).
//!
//! Provides the 16D real / 8D complex vector space V = e_i O ⊕ e_5 H ⊕ e_6 C ⊕ e_7 R ⊕ R
//! and the nested division algebraic inclusion chain R ⊂ C ⊂ H ⊂ O ⊂ V.

use serde::{Deserialize, Serialize};

/// Oriented Fano plane triples for octonion multiplication (f_ijk = +1).
pub const FANO_TRIPLES: [(usize, usize, usize); 7] = [
    (1, 2, 4),
    (2, 3, 5),
    (3, 4, 6),
    (4, 5, 7),
    (5, 6, 1),
    (6, 7, 2),
    (7, 1, 3),
];

/// A real octonion o = r_0 + \sum_{j=1}^7 r_j e_j.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Octonion {
    pub r: [f32; 8],
}

impl Default for Octonion {
    fn default() -> Self {
        Self { r: [0.0; 8] }
    }
}

impl Octonion {
    /// Construct a new octonion from real components [r0, r1..r7].
    pub fn new(r: [f32; 8]) -> Self {
        Self { r }
    }

    /// Real identity octonion 1.
    pub fn identity() -> Self {
        let mut r = [0.0; 8];
        r[0] = 1.0;
        Self { r }
    }

    /// Pure imaginary basis octonion e_k for k ∈ 1..=7.
    pub fn imaginary(k: usize) -> Self {
        assert!(
            (1..=7).contains(&k),
            "octonion imaginary unit must be in 1..=7"
        );
        let mut r = [0.0; 8];
        r[k] = 1.0;
        Self { r }
    }

    /// Octonion conjugate o* = r_0 - \sum_{j=1}^7 r_j e_j.
    pub fn conjugate(&self) -> Self {
        let mut conj = self.r;
        for c in conj.iter_mut().skip(1) {
            *c = -*c;
        }
        Self { r: conj }
    }

    /// Squared norm ||o||^2 = \sum r_i^2.
    pub fn norm_squared(&self) -> f32 {
        self.r.iter().map(|&x| x * x).sum()
    }

    /// Norm ||o||.
    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }

    /// Octonion non-associative multiplication o1 * o2.
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = [0.0; 8];

        // Real scalar contributions
        out[0] = self.r[0] * other.r[0];
        for i in 1..=7 {
            out[0] -= self.r[i] * other.r[i];
            out[i] += self.r[0] * other.r[i] + self.r[i] * other.r[0];
        }

        // Fano plane imaginary cross-contributions
        for &(i, j, k) in &FANO_TRIPLES {
            // e_i e_j = e_k
            out[k] += self.r[i] * other.r[j] - self.r[j] * other.r[i];
            // e_j e_k = e_i
            out[i] += self.r[j] * other.r[k] - self.r[k] * other.r[j];
            // e_k e_i = e_j
            out[j] += self.r[k] * other.r[i] - self.r[i] * other.r[k];
        }

        Self { r: out }
    }
}

/// A real quaternion h = r_0 + \sum_{m=1}^3 r_m \epsilon_m.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub r: [f32; 4],
}

impl Default for Quaternion {
    fn default() -> Self {
        Self { r: [0.0; 4] }
    }
}

impl Quaternion {
    pub fn new(r: [f32; 4]) -> Self {
        Self { r }
    }

    pub fn identity() -> Self {
        Self {
            r: [1.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            r: [self.r[0], -self.r[1], -self.r[2], -self.r[3]],
        }
    }

    pub fn norm_squared(&self) -> f32 {
        self.r.iter().map(|&x| x * x).sum()
    }

    pub fn mul(&self, other: &Self) -> Self {
        let a = self.r;
        let b = other.r;
        Self {
            r: [
                a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
                a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
                a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
                a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
            ],
        }
    }
}

/// A complex number c = r_0 + r_1 i.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComplexNumber {
    pub r: [f32; 2],
}

impl Default for ComplexNumber {
    fn default() -> Self {
        Self { r: [0.0; 2] }
    }
}

impl ComplexNumber {
    pub fn new(r: [f32; 2]) -> Self {
        Self { r }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            r: [self.r[0], -self.r[1]],
        }
    }

    pub fn norm_squared(&self) -> f32 {
        self.r[0] * self.r[0] + self.r[1] * self.r[1]
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self {
            r: [
                self.r[0] * other.r[0] - self.r[1] * other.r[1],
                self.r[0] * other.r[1] + self.r[1] * other.r[0],
            ],
        }
    }
}

/// The 16R-dimensional Cayley-Dickson vector space V = e_i O ⊕ e_5 H ⊕ e_6 C ⊕ e_7 R ⊕ R.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CayleyDicksonVector {
    pub components: [f32; 16],
}

impl Default for CayleyDicksonVector {
    fn default() -> Self {
        Self {
            components: [0.0; 16],
        }
    }
}

impl CayleyDicksonVector {
    pub fn new(components: [f32; 16]) -> Self {
        Self { components }
    }

    /// Linear embedding \iota_V(o, h, c, r) into 16R-dimensional V.
    pub fn embed(oct: &Octonion, quat: &Quaternion, comp: &ComplexNumber, real: f32) -> Self {
        let mut v = [0.0; 16];
        // e_i O (8 components: 0..8)
        v[0..8].copy_from_slice(&oct.r);
        // e_5 H (4 components: 8..12)
        v[8..12].copy_from_slice(&quat.r);
        // e_6 C (2 components: 12..14)
        v[12..14].copy_from_slice(&comp.r);
        // e_7 R (1 component: 14)
        v[14] = real;
        // R (1 component: 15)
        v[15] = 0.0;
        Self { components: v }
    }

    /// Extract octonionic projection o ∈ O.
    pub fn project_octonion(&self) -> Octonion {
        let mut r = [0.0; 8];
        r.copy_from_slice(&self.components[0..8]);
        Octonion { r }
    }

    /// Extract quaternionic projection h ∈ H.
    pub fn project_quaternion(&self) -> Quaternion {
        let mut r = [0.0; 4];
        r.copy_from_slice(&self.components[8..12]);
        Quaternion { r }
    }

    /// Extract complex projection c ∈ C.
    pub fn project_complex(&self) -> ComplexNumber {
        ComplexNumber {
            r: [self.components[12], self.components[13]],
        }
    }

    /// Extract real scalar projection r ∈ R.
    pub fn project_real(&self) -> f32 {
        self.components[14]
    }

    /// Squared norm of the 16D vector.
    pub fn norm_squared(&self) -> f32 {
        self.components.iter().map(|&x| x * x).sum()
    }
}
