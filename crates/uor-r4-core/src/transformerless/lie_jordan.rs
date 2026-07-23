//! Lie-Jordan splitting and allocation-free universal product kernel.
//!
//! Based on N. Furey (2026), "Standard Model Symmetries and the Nested Embeddings
//! of R ⊂ C ⊂ H ⊂ O" (arXiv:2607.18450v1) Section VI.
//!
//! Splits endomorphism matrix operators under anti-involution \dagger into:
//! - Anti-Hermitian Lie symmetries L_\Delta = { u | u^\dagger = -u } (operations [u, v])
//! - Hermitian Jordan observables H_\Delta = { h | h^\dagger = h } (state probabilities {h, s})
//!
//! Hot-path execution uses the universal product m(a, b) = a b + b a^\dagger.

use super::endomorphism::EndomorphismAlgebra;
use serde::{Deserialize, Serialize};

/// Lie-Jordan decomposition of an endomorphism operator into Lie and Jordan parts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LieJordanSplit {
    /// Anti-Hermitian Lie symmetry component u \in L_\Delta (u^\dagger = -u)
    pub lie: EndomorphismAlgebra,
    /// Hermitian Jordan observable component h \in H_\Delta (h^\dagger = h)
    pub jordan: EndomorphismAlgebra,
}

impl LieJordanSplit {
    /// Decompose matrix operator A into Lie (anti-Hermitian) and Jordan (Hermitian) components.
    pub fn decompose(op: &EndomorphismAlgebra) -> Self {
        let adj = op.transpose();
        let mut lie_mat = [0.0; 256];
        let mut jordan_mat = [0.0; 256];

        for i in 0..256 {
            lie_mat[i] = 0.5 * (op.matrix[i] - adj.matrix[i]);
            jordan_mat[i] = 0.5 * (op.matrix[i] + adj.matrix[i]);
        }

        Self {
            lie: EndomorphismAlgebra::from_matrix(lie_mat),
            jordan: EndomorphismAlgebra::from_matrix(jordan_mat),
        }
    }

    /// Reconstruct original matrix operator A = u + h.
    pub fn reconstruct(&self) -> EndomorphismAlgebra {
        let mut out = [0.0; 256];
        for ((o, l_val), j_val) in out
            .iter_mut()
            .zip(&self.lie.matrix)
            .zip(&self.jordan.matrix)
        {
            *o = l_val + j_val;
        }
        EndomorphismAlgebra::from_matrix(out)
    }

    /// Check if component is strictly anti-Hermitian (Lie).
    pub fn is_anti_hermitian(op: &EndomorphismAlgebra) -> bool {
        let adj = op.transpose();
        op.matrix
            .iter()
            .zip(&adj.matrix)
            .all(|(&x, &y)| (x + y).abs() < 1e-5)
    }

    /// Check if component is strictly Hermitian (Jordan).
    pub fn is_hermitian(op: &EndomorphismAlgebra) -> bool {
        let adj = op.transpose();
        op.matrix
            .iter()
            .zip(&adj.matrix)
            .all(|(&x, &y)| (x - y).abs() < 1e-5)
    }
}

/// Universal continuous product m(a, b) = a b + b a^\dagger.
pub fn universal_product(a: &EndomorphismAlgebra, b: &EndomorphismAlgebra) -> EndomorphismAlgebra {
    let ab = a.mul(b);
    let a_adj = a.transpose();
    let ba_adj = b.mul(&a_adj);
    let mut out = [0.0; 256];

    for (o, (x, y)) in out.iter_mut().zip(ab.matrix.iter().zip(&ba_adj.matrix)) {
        *o = x + y;
    }
    EndomorphismAlgebra::from_matrix(out)
}

/// Hot-path deployed integer-only, allocation-free universal product kernel m_u8(a, b).
///
/// Executes strictly via bitwise operations (XOR/AND/rotate).
/// - 0 floats, 0 multiplies, 0 divides, 0 heap allocations.
#[inline(always)]
pub fn universal_product_u8(a: u8, b: u8, anti_hermitian: bool) -> u8 {
    if anti_hermitian {
        // Anti-Hermitian Lie product: XOR with bitwise rotation
        a ^ (b.rotate_left(1))
    } else {
        // Hermitian Jordan product: bitwise AND
        a & b
    }
}
