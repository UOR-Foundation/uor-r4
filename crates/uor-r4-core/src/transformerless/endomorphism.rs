//! Endomorphic multiplication algebra End_R(V) ≅ Cl(0,8) operator store.
//!
//! Based on N. Furey (2026), "Standard Model Symmetries and the Nested Embeddings
//! of R ⊂ C ⊂ H ⊂ O" (arXiv:2607.18450v1).
//!
//! Implements 256D real matrix operators M_16(R), left/right multiplication maps
//! L_x and R_x, Clifford generators \gamma_j ≅ Cl(0,6), and volume element centralizers.

use super::cd_space::{CayleyDicksonVector, Octonion, Quaternion};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod array_256_serde {
    use super::*;

    pub fn serialize<S>(arr: &[f32; 256], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arr.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; 256], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<f32>::deserialize(deserializer)?;
        let mut arr = [0.0; 256];
        if vec.len() == 256 {
            arr.copy_from_slice(&vec);
            Ok(arr)
        } else {
            Err(serde::de::Error::custom("expected 256 floats"))
        }
    }
}

/// 256-dimensional real endomorphism matrix operator in M_16(R) ≅ Cl(0,8).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EndomorphismAlgebra {
    #[serde(with = "array_256_serde")]
    pub matrix: [f32; 256],
}

impl Default for EndomorphismAlgebra {
    fn default() -> Self {
        Self::zero()
    }
}

impl EndomorphismAlgebra {
    /// Zero matrix operator.
    pub fn zero() -> Self {
        Self { matrix: [0.0; 256] }
    }

    /// 16x16 Identity matrix operator I_16.
    pub fn identity() -> Self {
        let mut mat = [0.0; 256];
        for i in 0..16 {
            mat[i * 16 + i] = 1.0;
        }
        Self { matrix: mat }
    }

    /// Construct matrix operator from 16x16 row-major slice.
    pub fn from_matrix(matrix: [f32; 256]) -> Self {
        Self { matrix }
    }

    /// Matrix multiplication C = A * B.
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = [0.0; 256];
        for i in 0..16 {
            for k in 0..16 {
                let a_ik = self.matrix[i * 16 + k];
                if a_ik != 0.0 {
                    for j in 0..16 {
                        out[i * 16 + j] += a_ik * other.matrix[k * 16 + j];
                    }
                }
            }
        }
        Self { matrix: out }
    }

    /// Matrix transpose / adjoint A^\dagger.
    pub fn transpose(&self) -> Self {
        let mut out = [0.0; 256];
        for i in 0..16 {
            for j in 0..16 {
                out[j * 16 + i] = self.matrix[i * 16 + j];
            }
        }
        Self { matrix: out }
    }

    /// Commutator [A, B] = A*B - B*A.
    pub fn commutator(&self, other: &Self) -> Self {
        let ab = self.mul(other);
        let ba = other.mul(self);
        let mut out = [0.0; 256];
        for ((o, a_val), b_val) in out.iter_mut().zip(&ab.matrix).zip(&ba.matrix) {
            *o = a_val - b_val;
        }
        Self { matrix: out }
    }

    /// Anticommutator {A, B} = A*B + B*A.
    pub fn anticommutator(&self, other: &Self) -> Self {
        let ab = self.mul(other);
        let ba = other.mul(self);
        let mut out = [0.0; 256];
        for ((o, a_val), b_val) in out.iter_mut().zip(&ab.matrix).zip(&ba.matrix) {
            *o = a_val + b_val;
        }
        Self { matrix: out }
    }

    /// Apply 16x16 operator onto Cayley-Dickson vector v.
    pub fn apply(&self, vector: &CayleyDicksonVector) -> CayleyDicksonVector {
        let mut out = [0.0; 16];
        for (i, row_out) in out.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, &v_j) in vector.components.iter().enumerate() {
                sum += self.matrix[i * 16 + j] * v_j;
            }
            *row_out = sum;
        }
        CayleyDicksonVector::new(out)
    }

    /// Build octonionic left multiplication map L_x as 8x8 block embedded into 16x16.
    pub fn left_octonion(oct: &Octonion) -> Self {
        let mut mat = [0.0; 256];
        for col in 0..8 {
            let mut basis = Octonion::default();
            basis.r[col] = 1.0;
            let res = oct.mul(&basis);
            for row in 0..8 {
                mat[row * 16 + col] = res.r[row];
            }
        }
        for i in 8..16 {
            mat[i * 16 + i] = 1.0;
        }
        Self { matrix: mat }
    }

    /// Build octonionic right multiplication map R_x as 8x8 block embedded into 16x16.
    pub fn right_octonion(oct: &Octonion) -> Self {
        let mut mat = [0.0; 256];
        for col in 0..8 {
            let mut basis = Octonion::default();
            basis.r[col] = 1.0;
            let res = basis.mul(oct);
            for row in 0..8 {
                mat[row * 16 + col] = res.r[row];
            }
        }
        for i in 8..16 {
            mat[i * 16 + i] = 1.0;
        }
        Self { matrix: mat }
    }

    /// Build quaternionic left multiplication map L_h in 4x4 block (indices 8..12).
    pub fn left_quaternion(quat: &Quaternion) -> Self {
        let mut mat = [0.0; 256];
        for i in 0..8 {
            mat[i * 16 + i] = 1.0;
        }
        for col in 0..4 {
            let mut basis = Quaternion::default();
            basis.r[col] = 1.0;
            let res = quat.mul(&basis);
            for row in 0..4 {
                mat[(8 + row) * 16 + (8 + col)] = res.r[row];
            }
        }
        for i in 12..16 {
            mat[i * 16 + i] = 1.0;
        }
        Self { matrix: mat }
    }

    /// Build quaternionic right multiplication map R_h in 4x4 block (indices 8..12).
    pub fn right_quaternion(quat: &Quaternion) -> Self {
        let mut mat = [0.0; 256];
        for i in 0..8 {
            mat[i * 16 + i] = 1.0;
        }
        for col in 0..4 {
            let mut basis = Quaternion::default();
            basis.r[col] = 1.0;
            let res = basis.mul(quat);
            for row in 0..4 {
                mat[(8 + row) * 16 + (8 + col)] = res.r[row];
            }
        }
        for i in 12..16 {
            mat[i * 16 + i] = 1.0;
        }
        Self { matrix: mat }
    }

    /// Clifford algebra Cl(0,6) generator \gamma_j = L_{e_j} for j ∈ 1..=6.
    pub fn clifford_generator(j: usize) -> Self {
        assert!(
            (1..=6).contains(&j),
            "Clifford generator index must be in 1..=6"
        );
        let e_j = Octonion::imaginary(j);
        Self::left_octonion(&e_j)
    }

    /// Clifford imaginary volume element \omega = \gamma_1 \gamma_2 \gamma_3 \gamma_4 \gamma_5 \gamma_6 = L_{e_7}.
    pub fn volume_element() -> Self {
        let e_7 = Octonion::imaginary(7);
        Self::left_octonion(&e_7)
    }

    /// Check if operator commutes with the imaginary volume element [M, \omega] == 0.
    pub fn commutes_with_volume(&self) -> bool {
        let vol = Self::volume_element();
        let comm = self.commutator(&vol);
        comm.matrix.iter().all(|&x| x.abs() < 1e-5)
    }
}
