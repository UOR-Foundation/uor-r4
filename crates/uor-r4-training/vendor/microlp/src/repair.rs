//! D22 offline numerical boundary. Exact fallback operates on the stored f64
//! dyadic coefficients, not on an idealized exact model/Jacobian. No LP tolerance
//! or model constraint is relaxed. A numerical error is never infeasibility.
use crate::lu::{LUFactors, ScratchSpace};
use crate::sparse::{Error, ScatteredVec};
use std::cell::RefCell;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub fresh_factors: u64,
    pub exact_fallbacks: u64,
    pub exact_singular: u64,
    pub certified_factors: u64,
    pub certified_solves: u64,
    pub rejected_solves: u64,
    pub old_basis_refreshes: u64,
}
thread_local! { static STATS: RefCell<Stats> = RefCell::new(Stats::default()); }
pub fn stats() -> Stats {
    STATS.with(|s| s.borrow().clone())
}
pub fn reset_stats() {
    STATS.with(|s| *s.borrow_mut() = Stats::default());
}
pub(crate) fn count(f: impl FnOnce(&mut Stats)) {
    STATS.with(|s| f(&mut s.borrow_mut()));
}
pub(crate) fn finite(x: f64) -> Result<f64, Error> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(Error::NonFiniteFactorization)
    }
}
pub(crate) fn pivot_threshold(n: usize, original: f64, residual: f64) -> Result<f64, Error> {
    finite(original)?;
    finite(residual)?;
    if original < 0.0 || residual < 0.0 {
        return Err(Error::NonFiniteFactorization);
    }
    let scale = original.max(residual);
    if scale == 0.0 {
        return Err(Error::SingularMatrix);
    }
    finite(32.0 * n as f64 * f64::EPSILON * scale)
}

/// Original matrix columns, retained separately from rounded factors and etas.
#[derive(Clone, Debug)]
pub(crate) struct Basis {
    pub columns: Vec<(Vec<usize>, Vec<f64>)>,
    normal_norm: f64,
    transpose_norm: f64,
}
impl Basis {
    pub fn new<'a>(
        n: usize,
        get: impl Fn(usize) -> (&'a [usize], &'a [f64]),
    ) -> Result<Self, Error> {
        let mut columns = Vec::with_capacity(n);
        let mut rows = vec![0.0; n];
        let mut cn: f64 = 0.0;
        for c in 0..n {
            let (rs, vs) = get(c);
            if rs.len() != vs.len() || rs.windows(2).any(|w| w[0] >= w[1]) {
                return Err(Error::UnreliableFactorization);
            }
            let mut sum = 0.0;
            for (&r, &v) in rs.iter().zip(vs) {
                if r >= n {
                    return Err(Error::UnreliableFactorization);
                }
                let a = finite(v)?.abs();
                rows[r] = finite(rows[r] + a)?;
                sum = finite(sum + a)?;
            }
            cn = cn.max(sum);
            columns.push((rs.to_vec(), vs.to_vec()));
        }
        Ok(Self {
            columns,
            normal_norm: rows.into_iter().fold(0.0, f64::max),
            transpose_norm: cn,
        })
    }
    pub fn error(&self, b: &[f64], x: &[f64], transpose: bool) -> Result<f64, Error> {
        let n = self.columns.len();
        if b.len() != n || x.len() != n {
            return Err(Error::UnreliableFactorization);
        }
        let mut ax = vec![0.0; n];
        let mut xn: f64 = 0.0;
        let mut bn: f64 = 0.0;
        for (&b, &x) in b.iter().zip(x) {
            bn = bn.max(finite(b)?.abs());
            xn = xn.max(finite(x)?.abs());
        }
        for (c, (rs, vs)) in self.columns.iter().enumerate() {
            for (&r, &v) in rs.iter().zip(vs) {
                let (i, xi) = if transpose { (c, x[r]) } else { (r, x[c]) };
                ax[i] = finite(ax[i] + finite(v * xi)?)?;
            }
        }
        let mut residual: f64 = 0.0;
        for (&b, &a) in b.iter().zip(&ax) {
            residual = residual.max(finite(b - a)?.abs());
        }
        let norm = if transpose {
            self.transpose_norm
        } else {
            self.normal_norm
        };
        let denom = finite(finite(norm * xn)? + bn)?;
        if denom == 0.0 {
            return if residual == 0.0 {
                Ok(0.0)
            } else {
                Err(Error::UnreliableFactorization)
            };
        }
        finite(residual / denom)
    }
    pub fn check(&self, b: &[f64], x: &[f64], transpose: bool) -> Result<(), Error> {
        let error = self.error(b, x, transpose);
        match error {
            Ok(e) if e <= 128.0 * self.columns.len() as f64 * f64::EPSILON => {
                count(|s| s.certified_solves += 1);
                Ok(())
            }
            Err(e) => {
                count(|s| s.rejected_solves += 1);
                Err(e)
            }
            _ => {
                count(|s| s.rejected_solves += 1);
                Err(Error::UnreliableFactorization)
            }
        }
    }
}

pub(crate) fn verify(basis: &Basis, factors: &LUFactors) -> Result<(), Error> {
    let n = basis.columns.len();
    let transposed = factors.transpose();
    let mut scratch = ScratchSpace::with_capacity(n);
    for (transpose, f) in [(false, factors), (true, &transposed)] {
        for i in 0..n {
            let mut b = vec![0.0; n];
            b[i] = 1.0;
            let mut x = b.clone();
            f.solve_dense(&mut x, &mut scratch);
            basis.check(&b, &x, transpose)?;
            let mut sparse = ScatteredVec::empty(n);
            sparse.set(std::iter::once((i, &1.0)));
            f.solve(&mut sparse, &mut scratch);
            x.fill(0.0);
            for (r, &v) in sparse.iter() {
                x[r] = v;
            }
            basis.check(&b, &x, transpose)?;
        }
    }
    count(|s| s.certified_factors += 1);
    Ok(())
}

/// Source-independent fixture boundary used by the owned numerical test crate.
/// Performs no simplex, gradient, candidate selection or model evaluation.
pub fn qualify(columns: Vec<(Vec<usize>, Vec<f64>)>) -> Result<(), String> {
    let n = columns.len();
    let mut scratch = ScratchSpace::with_capacity(n);
    crate::lu::lu_factorize(n, |c| (&columns[c].0, &columns[c].1), 0.1, &mut scratch)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(feature = "numerical-fixtures")]
pub fn transaction_fixture(kind: &str) -> Result<(), String> {
    crate::solver::d22_fixture(kind).map_err(|e| e.to_string())
}
