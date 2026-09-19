//! Exact `2I` (binary icosahedral) group composition by table lookup.
//!
//! # Why this exists
//!
//! `context_hopf_fiber_q30` accumulated the context state as
//! `s3 = s3.mul_q30(&q_root)` over up to 64 tokens, and `mul_q30` is a 16-multiply Hamilton
//! product. That is on the order of **1,000 multiplies per generated token** — roughly 40x the
//! next largest multiplier site in the serving path, and outside the arithmetic scan that the
//! multiplier-free invariant actually covered (see
//! `docs/evidence/native_geometric_d0a_multiplier_census_2026-09-19.txt`).
//!
//! # Why a table is the *correct* fix, not a workaround
//!
//! The 120 canonical H4 roots are exactly the 120 elements of `2I`, and `2I` is closed under
//! quaternion multiplication. So the accumulated state is always a group element and the
//! accumulation *is* exact group composition — which is what this project already claims as its
//! mechanism. Composing two group elements is therefore a lookup, not an arithmetic operation.
//!
//! The table is also **more exact than what it replaces**. The current path multiplies Q1.30
//! *quantized* roots, so the product drifts off the group and the code calls `normalized()` every
//! 8 steps to contain the drift. The table stores the *exact* product of the exact root elements,
//! so no drift accumulates and no renormalization is needed.
//!
//! # Cost and verification
//!
//! The table is 120x120 bytes (14,400) plus a 120-byte inverse row — about 14.2 KB, built once
//! lazily. Building it uses floating point for the exact product and nearest-element
//! classification, which is **initialization, not the serving path**: the served accumulation is
//! table reads and integer index arithmetic only, with zero multiplies and zero floats.
//!
//! Correctness is not assumed. The tests establish that the table is a **group**:
//! each row is a permutation of the elements (left multiplication is a bijection), composition is
//! associative for all 1,728,000 triples, the identity is two-sided, and every element has an
//! inverse in the table. A quasigroup with those properties *is* the group.

use std::sync::OnceLock;

use super::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};

/// Number of group elements: the 120 canonical H4 roots.
pub const GROUP_ORDER: usize = H4_ROOT_COUNT;

/// Exact `2I` composition tables.
pub struct GroupTable {
    /// Flat `GROUP_ORDER * GROUP_ORDER` table: `product[a * N + b]` is the index of
    /// `root_a * root_b` (left factor `a`, matching `s3.mul_q30(root)` order).
    pub product: Box<[u8]>,
    /// Index of the identity element `(1, 0, 0, 0)`.
    pub identity: u8,
    /// `inverse[a]` is the index of `root_a^-1`.
    pub inverse: Box<[u8]>,
    /// Worst-case distance from an exact product to its nearest group element, as
    /// `1 - |<product, nearest>|`. A large value would mean the classification is ambiguous.
    pub max_closure_residual: f64,
}

static TABLE: OnceLock<GroupTable> = OnceLock::new();

/// The process-wide exact composition table, built on first use.
pub fn group_table() -> &'static GroupTable {
    TABLE.get_or_init(GroupTable::build)
}

fn normalize(v: [f64; 4]) -> [f64; 4] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3]).sqrt();
    if n == 0.0 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n, v[3] / n]
    }
}

/// Hamilton product `a * b` in the same order as `UnitS3Q30::mul_q30`.
fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn dot(a: [f64; 4], b: [f64; 4]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

impl GroupTable {
    fn build() -> Self {
        let roots = canonical_h4_roots_q30();
        debug_assert_eq!(roots.len(), GROUP_ORDER);
        let exact: Vec<[f64; 4]> = roots
            .iter()
            .map(|q| normalize([q.0[0] as f64, q.0[1] as f64, q.0[2] as f64, q.0[3] as f64]))
            .collect();

        let n = GROUP_ORDER;
        let mut product = vec![0u8; n * n];
        let mut max_closure_residual = 0.0f64;

        for a in 0..n {
            for b in 0..n {
                let p = normalize(hamilton(exact[a], exact[b]));
                // Classify by the *signed* inner product so `q` and `-q` stay distinct: they are
                // different group elements even though they are the same rotation.
                let mut best = 0usize;
                let mut best_dot = -2.0f64;
                let mut second = -2.0f64;
                for (i, e) in exact.iter().enumerate() {
                    let d = dot(p, *e);
                    if d > best_dot {
                        second = best_dot;
                        best_dot = d;
                        best = i;
                    } else if d > second {
                        second = d;
                    }
                }
                product[a * n + b] = best as u8;
                let residual = 1.0 - best_dot;
                if residual > max_closure_residual {
                    max_closure_residual = residual;
                }
                debug_assert!(
                    best_dot - second > 1e-6,
                    "ambiguous classification for ({a}, {b}): {best_dot} vs {second}"
                );
            }
        }

        // Identity is the element with (1, 0, 0, 0) up to the exact representation.
        let mut identity = 0usize;
        let mut best_dot = -2.0f64;
        for (i, e) in exact.iter().enumerate() {
            let d = dot(*e, [1.0, 0.0, 0.0, 0.0]);
            if d > best_dot {
                best_dot = d;
                identity = i;
            }
        }

        let mut inverse = vec![0u8; n];
        for a in 0..n {
            let mut found = None;
            for b in 0..n {
                if product[a * n + b] as usize == identity {
                    found = Some(b as u8);
                    break;
                }
            }
            inverse[a] = found.unwrap_or(0);
        }

        Self {
            product: product.into_boxed_slice(),
            identity: identity as u8,
            inverse: inverse.into_boxed_slice(),
            max_closure_residual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_a_latin_square_so_left_multiplication_is_a_bijection() {
        let t = group_table();
        assert_eq!(t.product.len(), GROUP_ORDER * GROUP_ORDER);
        for a in 0..GROUP_ORDER {
            let mut seen = [false; GROUP_ORDER];
            for b in 0..GROUP_ORDER {
                let p = t.product[a * GROUP_ORDER + b] as usize;
                assert!(p < GROUP_ORDER);
                assert!(!seen[p], "row {a} repeats element {p}");
                seen[p] = true;
            }
        }
        for b in 0..GROUP_ORDER {
            let mut seen = [false; GROUP_ORDER];
            for a in 0..GROUP_ORDER {
                let p = t.product[a * GROUP_ORDER + b] as usize;
                assert!(!seen[p], "column {b} repeats element {p}");
                seen[p] = true;
            }
        }
    }

    #[test]
    fn composition_is_associative_for_every_triple() {
        let t = group_table();
        let n = GROUP_ORDER;
        for a in 0..n {
            for b in 0..n {
                let ab = t.product[a * n + b] as usize;
                for c in 0..n {
                    let left = t.product[ab * n + c] as usize;
                    let bc = t.product[b * n + c] as usize;
                    let right = t.product[a * n + bc] as usize;
                    assert_eq!(left, right, "associativity failed for ({a}, {b}, {c})");
                }
            }
        }
    }

    #[test]
    fn identity_is_two_sided_and_inverses_exist() {
        let t = group_table();
        let e = t.identity as usize;
        let n = GROUP_ORDER;
        for a in 0..n {
            assert_eq!(t.product[e * n + a] as usize, a, "e*a != a for {a}");
            assert_eq!(t.product[a * n + e] as usize, a, "a*e != a for {a}");
            let inv = t.inverse[a] as usize;
            assert_eq!(t.product[a * n + inv] as usize, e, "a*inv(a) != e for {a}");
        }
    }

    #[test]
    fn classification_is_unambiguous() {
        let t = group_table();
        assert!(
            t.max_closure_residual < 1e-6,
            "closure residual {:.3e} is too large: the product classification would be ambiguous",
            t.max_closure_residual
        );
    }
}
