use core::time::Duration;

use crate::{
    helpers::{resized_view, to_dense},
    lu::{lu_factorize, LUFactors, ScratchSpace},
    sparse::{ScatteredVec, SparseMat, SparseVec},
    ComparisonOp, CsVec, Error, StopReason, VarDomain,
};
use sprs::CompressedStorage;

use web_time::Instant;

pub(crate) type Deadline = Option<Instant>;

type CsMat = sprs::CsMatI<f64, usize>;

/// The simplex engine's working tolerance: pivot eligibility, ratio-test
/// steps, reduced-cost optimality checks, bound-violation candidacy, and
/// `float_eq`.
///
/// Deliberately tight because the big-M correctness models rely on node LPs
/// resolving basic integer values sharply onto their bounds. Loosening it
/// (globally or just for bound-violation candidacy) lets basic values sit
/// about `1e-8` away from their bounds, which 1e9-scale big-M rows amplify
/// past the MIP layer's
/// rounded-incumbent feasibility guard and the branch-and-bound tree
/// explodes. The flip side of running this tight — round-off noise being
/// promoted into phantom infeasibilities — is handled where it bites, by
/// the refresh valve in [`Solver::restore_feasibility`].
pub const EPS: f64 = 1e-10;

/// How often (in simplex iterations) the primal/dual loops in `optimize` and
/// `restore_feasibility` check the deadline and emit a progress `debug!` log.
/// Checking every iteration would make the deadline check itself a
/// significant fraction of the per-iteration cost on easy problems; checking
/// too rarely would make a time limit overshoot by a visible amount on hard
/// ones. 1000 keeps the check overhead negligible while still bounding the
/// worst-case overshoot to about a thousand pivots.
pub(crate) const DEADLINE_CHECK_INTERVAL: u64 = 1000;

fn numerical_progress(event: &str, basis: &[usize], iterations: u64) {
    if std::env::var_os("UOR_MICROLP_PROGRESS").is_some() {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        basis.hash(&mut h);
        let s = crate::repair::stats();
        eprintln!("d22-numerics event={event} basis_tag={:016x} rows={} iterations={iterations} fresh={} exact={} singular={} refreshes={}",
            h.finish(), basis.len(), s.fresh_factors, s.exact_fallbacks,
            s.exact_singular, s.old_basis_refreshes);
    }
}

/// Compensate product rounding and accumulation in deterministic input order.
/// Offline arithmetic only; callers still certify the original system/phase.
fn compensated_products(terms: impl IntoIterator<Item = (f64, f64)>) -> Result<f64, Error> {
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for (a, b) in terms {
        let a = crate::repair::finite(a)?;
        let b = crate::repair::finite(b)?;
        let product = crate::repair::finite(a * b)?;
        let product_error = crate::repair::finite(a.mul_add(b, -product))?;
        let next = crate::repair::finite(sum + product)?;
        let sum_error = if sum.abs() >= product.abs() {
            (sum - next) + product
        } else {
            (product - next) + sum
        };
        correction =
            crate::repair::finite(correction + crate::repair::finite(sum_error + product_error)?)?;
        sum = next;
    }
    Ok(crate::repair::finite(sum + correction)?)
}

/// Threshold-pivoting stability coefficient passed to [`lu_factorize`] for
/// every LU (re)factorization the simplex performs: a candidate pivot is
/// accepted only if its magnitude is at least this fraction of the column's
/// largest eligible entry. 0.1 is the standard textbook default for
/// Gilbert-Peierls sparse LU (see `lu_factorize`'s doc reference) — it
/// balances numerical stability (higher would refuse more marginal pivots,
/// at the cost of extra fill-in) against sparsity (lower risks amplifying
/// rounding error through a poorly-conditioned pivot).
pub(crate) const LU_STABILITY_THRESHOLD: f64 = 0.1;

/// A variable bound whose magnitude is at least this large is treated as
/// infinite when choosing a non-basic variable's INITIAL value (see
/// [`initial_nonbasic_value`]). Seeding a non-basic variable *at* such a bound
/// floods the tableau with a value that swamps the actual problem data — the
/// rhs and structural coefficients lose all significance against it and the
/// solve converges to a wrong vertex or NaN. This is issue #3: `f64::MAX`,
/// `f32::MAX` and `i64::MAX` upper bounds produced non-optimal answers where
/// `f64::INFINITY` did not, because only the latter skipped the seed-at-bound
/// step. 2^52 is the largest f64 whose unit (`1.0`) is still exactly
/// representable; beyond it a finite bound is numerically a stand-in for
/// infinity, so we seed as if it were infinite. The true bound is left
/// untouched in `orig_var_mins`/`orig_var_maxs`, so the ratio test still
/// honours it exactly — only the starting vertex changes.
const SEED_AS_INFINITE: f64 = 4_503_599_627_370_496.0; // 2^52

/// Power-of-two row equilibration keeps the largest structural coefficient
/// near one without rounding its mantissa. If scaling would overflow the RHS,
/// leave the row unchanged and let the solver report any resulting numerical
/// failure explicitly.
fn equilibration_scale(coeffs: &CsVec, rhs: f64) -> f64 {
    let max_coeff = coeffs
        .data()
        .iter()
        .map(|coeff| coeff.abs())
        .fold(0.0, f64::max);
    if max_coeff == 0.0 || !max_coeff.is_finite() {
        return 1.0;
    }

    let exponent = (max_coeff.log2().floor() as i32).clamp(-1023, 1023);
    let scale = 2.0_f64.powi(-exponent);
    if scale.is_finite() && (rhs * scale).is_finite() {
        scale
    } else {
        1.0
    }
}

/// A non-empty constraint row in the exact representation consumed by the
/// simplex engine. Structural coefficients and the right-hand side share the
/// same power-of-two scale; the slack coefficient remains one.
struct PreparedRow {
    coeffs: CsVec,
    rhs: f64,
    row_scale: f64,
    slack_var_min: f64,
    slack_var_max: f64,
}

/// Validate empty-row semantics and prepare one retained row for storage.
/// `None` denotes a tautology that does not need a slack variable.
fn prepare_row(
    mut coeffs: CsVec,
    cmp_op: ComparisonOp,
    rhs: f64,
) -> Result<Option<PreparedRow>, Error> {
    if coeffs.indices().is_empty() {
        let tautological = match cmp_op {
            ComparisonOp::Eq => float_eq(rhs, 0.0),
            ComparisonOp::Le => 0.0 <= rhs,
            ComparisonOp::Ge => 0.0 >= rhs,
        };
        return if tautological {
            Ok(None)
        } else {
            Err(Error::Infeasible)
        };
    }

    let row_scale = equilibration_scale(&coeffs, rhs);
    if row_scale != 1.0 {
        coeffs.map_inplace(|coeff| coeff * row_scale);
    }
    let (slack_var_min, slack_var_max) = match cmp_op {
        ComparisonOp::Le => (0.0, f64::INFINITY),
        ComparisonOp::Ge => (f64::NEG_INFINITY, 0.0),
        ComparisonOp::Eq => (0.0, 0.0),
    };
    Ok(Some(PreparedRow {
        coeffs,
        rhs: rhs * row_scale,
        row_scale,
        slack_var_min,
        slack_var_max,
    }))
}

pub(crate) fn float_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

/// Initial value for a non-basic structural variable, preferring the bound that
/// keeps its reduced cost dual-feasible. Returns `(value, dual_feasible)`, where
/// `dual_feasible` is false when no finite bound can satisfy dual feasibility (a
/// free variable, or a variable unbounded on the side its objective coefficient
/// pushes toward). Callers handle a fixed variable (`min == max`) separately.
///
/// A bound at or beyond [`SEED_AS_INFINITE`] is treated as infinite here so that
/// a huge finite bound is never used as the seed value (issue #3); the caller's
/// stored bounds are left untouched, so the ratio test still honours them.
fn initial_nonbasic_value(obj_coeff: f64, min: f64, max: f64) -> (f64, bool) {
    let min = if min <= -SEED_AS_INFINITE {
        f64::NEG_INFINITY
    } else {
        min
    };
    let max = if max >= SEED_AS_INFINITE {
        f64::INFINITY
    } else {
        max
    };

    if min.is_infinite() && max.is_infinite() {
        // Free variable: dual-feasible only if the objective coefficient is zero.
        (0.0, float_eq(obj_coeff, 0.0))
    } else if obj_coeff > 0.0 {
        // Prefer the lower bound; fall back to the upper if the lower is infinite.
        if min.is_finite() {
            (min, true)
        } else {
            (max, false)
        }
    } else if obj_coeff < 0.0 {
        // Prefer the upper bound; fall back to the lower if the upper is infinite.
        if max.is_finite() {
            (max, true)
        } else {
            (min, false)
        }
    } else if min.is_finite() {
        // Zero objective coefficient: any finite bound is dual-feasible.
        (min, true)
    } else {
        (max, true)
    }
}

#[inline]
pub(crate) fn check_deadline(deadline: &Deadline) -> StopReason {
    if let Some(dl) = deadline {
        if Instant::now() >= *dl {
            return StopReason::Limit;
        }
    }
    StopReason::Finished
}

#[derive(Clone)]
pub(crate) struct Solver {
    pub(crate) num_vars: usize,
    pub(crate) deadline: Deadline,
    /// Duration granted to each subsequent public pure-LP operation.
    pub(crate) operation_time_limit: Option<Duration>,
    /// Total number of simplex pivots performed across all solves/reoptimizes on this instance.
    pub(crate) lp_iterations: u64,
    /// Wall-clock time accumulated by public pure-LP operations.
    pub(crate) elapsed: Duration,

    orig_obj_coeffs: Vec<f64>,
    working_obj_coeffs: Vec<f64>,
    // Persists through deadline interruption until the original objective is
    // restored and certified; never confuse zero-cost feasibility with optimum.
    numerical_feasibility_restart_active: bool,
    orig_var_mins: Vec<f64>,
    orig_var_maxs: Vec<f64>,
    pub(crate) orig_var_domains: Vec<VarDomain>,
    orig_constraints: CsMat, // excluding rhs
    orig_constraints_csc: CsMat,
    orig_rhs: Vec<f64>,
    /// Positive per-row equilibration factors. Every row is multiplied by
    /// these internally; validation multiplies its absolute user tolerance by
    /// the same factor so the public feasibility contract stays unscaled.
    row_scales: Vec<f64>,

    enable_primal_steepest_edge: bool,
    enable_dual_steepest_edge: bool,

    is_primal_feasible: bool,
    is_dual_feasible: bool,

    // Updated on each pivot
    /// For each var: whether it is basic/non-basic and the corresponding index.
    var_states: Vec<VarState>,
    basis_solver: BasisSolver,

    /// For each constraint the corresponding basic var.
    basic_vars: Vec<usize>,
    basic_var_vals: Vec<f64>,
    basic_var_mins: Vec<f64>,
    basic_var_maxs: Vec<f64>,
    dual_edge_sq_norms: Vec<f64>,

    /// Remaining variables. (idx -> var), 'nb' means 'non-basic'
    nb_vars: Vec<usize>,
    nb_var_obj_coeffs: Vec<f64>,
    nb_var_vals: Vec<f64>,
    nb_var_states: Vec<NonBasicVarState>,
    nb_var_is_fixed: Vec<bool>,
    primal_edge_sq_norms: Vec<f64>,

    pub(crate) cur_obj_val: f64,

    // Recomputed on each pivot
    col_coeffs: SparseVec,
    sq_norms_update_helper: Vec<f64>,
    inv_basis_row_coeffs: SparseVec,
    row_coeffs: ScatteredVec,
}

#[derive(Clone, Debug)]
enum VarState {
    Basic(usize),
    NonBasic(usize),
}

#[derive(Clone, Debug)]
struct NonBasicVarState {
    at_min: bool,
    at_max: bool,
}

/// Status of one variable in a simplex basis snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VarStatus {
    Basic,
    AtLower,
    AtUpper,
    /// Non-basic free variable (both bounds infinite), pinned at 0.
    Free,
}

/// A compact simplex basis: one status per total var (structural + slack).
/// Together with the current variable bounds it fully determines a vertex.
#[derive(Clone, Debug)]
pub(crate) struct Basis(pub(crate) Vec<VarStatus>);

impl std::fmt::Debug for Solver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Solver")?;
        writeln!(
            f,
            "num_vars: {}, num_constraints: {}, is_primal_feasible: {}, is_dual_feasible: {}",
            self.num_vars,
            self.num_constraints(),
            self.is_primal_feasible,
            self.is_dual_feasible,
        )?;
        writeln!(
            f,
            "numerical_feasibility_restart_active: {}",
            self.numerical_feasibility_restart_active
        )?;
        writeln!(f, "working_obj_coeffs:\n{:?}", self.working_obj_coeffs)?;
        writeln!(f, "orig_obj_coeffs:\n{:?}", self.orig_obj_coeffs)?;
        writeln!(f, "orig_var_mins:\n{:?}", self.orig_var_mins)?;
        writeln!(f, "orig_var_maxs:\n{:?}", self.orig_var_maxs)?;
        writeln!(f, "orig_constraints:")?;
        for row in self.orig_constraints.outer_iterator() {
            writeln!(f, "{:?}", to_dense(&row))?;
        }
        writeln!(f, "orig_rhs:\n{:?}", self.orig_rhs)?;
        writeln!(f, "basic_vars:\n{:?}", self.basic_vars)?;
        writeln!(f, "basic_var_vals:\n{:?}", self.basic_var_vals)?;
        writeln!(f, "dual_edge_sq_norms:\n{:?}", self.dual_edge_sq_norms)?;
        writeln!(f, "nb_vars:\n{:?}", self.nb_vars)?;
        writeln!(f, "nb_var_vals:\n{:?}", self.nb_var_vals)?;
        writeln!(f, "nb_var_obj_coeffs:\n{:?}", self.nb_var_obj_coeffs)?;
        writeln!(f, "primal_edge_sq_norms:\n{:?}", self.primal_edge_sq_norms)?;
        writeln!(f, "cur_obj_val: {:?}", self.cur_obj_val)?;
        Ok(())
    }
}

impl Solver {
    pub(crate) fn try_new(
        obj_coeffs: &[f64],
        var_mins: &[f64],
        var_maxs: &[f64],
        constraints: &[(CsVec, ComparisonOp, f64)],
        var_domains: &[VarDomain],
        deadline: Deadline,
    ) -> Result<Self, Error> {
        let enable_steepest_edge = true; // TODO: make user-settable.

        let num_vars = obj_coeffs.len();

        assert_eq!(num_vars, var_mins.len());
        assert_eq!(num_vars, var_maxs.len());
        let mut orig_var_mins = var_mins.to_vec();
        let mut orig_var_maxs = var_maxs.to_vec();

        let mut var_states = vec![];

        let mut nb_vars = vec![];
        let mut nb_var_vals = vec![];
        let mut nb_var_states = vec![];

        let mut obj_val = 0.0;

        let mut is_dual_feasible = true;

        for v in 0..num_vars {
            // choose initial variable values

            let min = orig_var_mins[v];
            let max = orig_var_maxs[v];
            if min.is_nan() || max.is_nan() || min > max {
                return Err(Error::Infeasible);
            }

            // initially all user-created variables are non-basic
            var_states.push(VarState::NonBasic(nb_vars.len()));
            nb_vars.push(v);

            // Choose an initial value, preferring a bound that keeps this
            // variable's reduced cost dual-feasible.
            let (init_val, var_dual_feasible) = if float_eq(min, max) {
                // Fixed variable: the obj. coeff doesn't matter.
                (min, true)
            } else {
                initial_nonbasic_value(obj_coeffs[v], min, max)
            };
            if !var_dual_feasible {
                is_dual_feasible = false;
            }

            nb_var_vals.push(init_val);
            obj_val += init_val * obj_coeffs[v];

            nb_var_states.push(NonBasicVarState {
                at_min: float_eq(init_val, min),
                at_max: float_eq(init_val, max),
            });
        }

        let mut constraint_coeffs = vec![];
        let mut orig_rhs = vec![];
        let mut row_scales = vec![];

        // Initially, all slack vars are basic.
        let mut basic_vars = vec![];
        let mut basic_var_vals = vec![];
        let mut basic_var_mins = vec![];
        let mut basic_var_maxs = vec![];

        for (coeffs, cmp_op, rhs) in constraints {
            let Some(PreparedRow {
                coeffs,
                rhs,
                row_scale,
                slack_var_min,
                slack_var_max,
            }) = prepare_row(coeffs.clone(), *cmp_op, *rhs)?
            else {
                continue;
            };

            constraint_coeffs.push(coeffs.clone());
            orig_rhs.push(rhs);
            row_scales.push(row_scale);

            orig_var_mins.push(slack_var_min);
            orig_var_maxs.push(slack_var_max);

            basic_var_mins.push(slack_var_min);
            basic_var_maxs.push(slack_var_max);

            let cur_slack_var = var_states.len();
            var_states.push(VarState::Basic(basic_vars.len()));
            basic_vars.push(cur_slack_var);

            let mut lhs_val = 0.0;
            for (var, &coeff) in coeffs.iter() {
                lhs_val += coeff * nb_var_vals[var];
            }
            basic_var_vals.push(rhs - lhs_val);
        }

        let num_constraints = constraint_coeffs.len();
        let num_total_vars = num_vars + num_constraints;

        let mut orig_obj_coeffs = obj_coeffs.to_vec();
        orig_obj_coeffs.resize(num_total_vars, 0.0);

        let mut orig_constraints = CsMat::empty(CompressedStorage::CSR, num_total_vars);
        for (cur_slack_var, coeffs) in constraint_coeffs.into_iter().enumerate() {
            let mut coeffs = into_resized(coeffs, num_total_vars);
            coeffs.append(num_vars + cur_slack_var, 1.0);
            orig_constraints = orig_constraints.append_outer_csvec(coeffs.view());
        }
        let orig_constraints_csc = orig_constraints.to_csc();

        let is_primal_feasible = basic_var_vals
            .iter()
            .zip(&basic_var_mins)
            .zip(&basic_var_maxs)
            .all(|((&val, &min), &max)| val >= min && val <= max);

        let need_artificial_obj = !is_primal_feasible && !is_dual_feasible;

        let enable_dual_steepest_edge = enable_steepest_edge;
        let dual_edge_sq_norms = if enable_dual_steepest_edge {
            vec![1.0; basic_vars.len()]
        } else {
            vec![]
        };

        // If is dual feasible at start, we don't need lengthy primal phase2.
        // Thus we can skip expensive calculations for primal sq. norms.
        let enable_primal_steepest_edge = enable_steepest_edge && !is_dual_feasible;
        let sq_norms_update_helper = if enable_primal_steepest_edge {
            vec![0.0; num_total_vars - num_constraints]
        } else {
            vec![]
        };

        let mut nb_var_obj_coeffs = vec![];
        let mut primal_edge_sq_norms = vec![];
        for (&var, state) in nb_vars.iter().zip(&nb_var_states) {
            //guaranteed to be a valid index
            let col = orig_constraints_csc.outer_view(var).unwrap();

            if need_artificial_obj {
                let coeff = if state.at_min && !state.at_max {
                    1.0
                } else if state.at_max && !state.at_min {
                    -1.0
                } else {
                    0.0
                };
                nb_var_obj_coeffs.push(coeff);
            } else {
                nb_var_obj_coeffs.push(orig_obj_coeffs[var]);
            }

            if enable_primal_steepest_edge {
                primal_edge_sq_norms.push(col.squared_l2_norm() + 1.0);
            }
        }

        let cur_obj_val = if need_artificial_obj { 0.0 } else { obj_val };

        let mut scratch = ScratchSpace::with_capacity(num_constraints);
        let lu_factors = lu_factorize(
            basic_vars.len(),
            |c| {
                orig_constraints_csc
                    .outer_view(basic_vars[c])
                    //guaranteed to be a valid index
                    .unwrap()
                    .into_raw_storage()
            },
            LU_STABILITY_THRESHOLD,
            &mut scratch,
        )?;
        let lu_factors_transp = lu_factors.transpose();

        let nb_var_is_fixed = vec![false; nb_vars.len()];

        let current_basis = crate::repair::Basis::new(basic_vars.len(), |c| {
            orig_constraints_csc
                .outer_view(basic_vars[c])
                .unwrap()
                .into_raw_storage()
        })?;
        let working_obj_coeffs = if need_artificial_obj {
            let mut cost = vec![0.0; num_total_vars];
            for (&v, &c) in nb_vars.iter().zip(&nb_var_obj_coeffs) {
                cost[v] = c;
            }
            cost
        } else {
            orig_obj_coeffs.clone()
        };
        let res = Self {
            num_vars,
            orig_obj_coeffs,
            working_obj_coeffs,
            numerical_feasibility_restart_active: false,
            orig_var_mins,
            orig_var_maxs,
            orig_constraints,
            orig_constraints_csc,
            orig_rhs,
            row_scales,
            deadline,
            operation_time_limit: None,
            lp_iterations: 0,
            elapsed: Duration::ZERO,
            orig_var_domains: var_domains.to_vec(),
            enable_primal_steepest_edge,
            enable_dual_steepest_edge,
            is_primal_feasible,
            is_dual_feasible,
            var_states,
            basis_solver: BasisSolver {
                current_basis,
                lu_factors,
                lu_factors_transp,
                scratch,
                eta_matrices: EtaMatrices::new(num_constraints),
                rhs: ScatteredVec::empty(num_constraints),
            },
            basic_vars,
            basic_var_vals,
            basic_var_mins,
            basic_var_maxs,
            dual_edge_sq_norms,
            nb_vars,
            nb_var_obj_coeffs,
            nb_var_vals,
            nb_var_states,
            nb_var_is_fixed,
            primal_edge_sq_norms,
            cur_obj_val,
            col_coeffs: SparseVec::new(),
            sq_norms_update_helper,
            inv_basis_row_coeffs: SparseVec::new(),
            row_coeffs: ScatteredVec::empty(num_total_vars - num_constraints),
        };

        debug!(
            "initialized solver: vars: {}, constraints: {}, primal feasible: {}, dual feasible: {}, nnz: {}",
            res.num_vars,
            res.orig_constraints.rows(),
            res.is_primal_feasible,
            res.is_dual_feasible,
            res.orig_constraints.nnz(),
        );

        Ok(res)
    }

    pub(crate) fn get_value(&self, var: usize) -> &f64 {
        match self.var_states[var] {
            VarState::Basic(idx) => &self.basic_var_vals[idx],
            VarState::NonBasic(idx) => &self.nb_var_vals[idx],
        }
    }

    /// Check `values` (one entry per structural var) against every ORIGINAL
    /// constraint row, within the ABSOLUTE tolerance `tol`. Bounds are not
    /// checked here. Each row's sense is encoded by its slack var's bounds
    /// (lhs + s = rhs with s in [smin, smax]  ⇔  rhs - smax ≤ lhs ≤ rhs - smin);
    /// slack bounds are never touched by branching, so this always reflects the
    /// user's original rows.
    ///
    /// Rows carry an internal power-of-two equilibration factor; the
    /// tolerance is multiplied by that same factor, which is algebraically
    /// equivalent to applying `tol` to the unscaled user row. It is deliberately
    /// NOT scaled by the row's magnitude: this check exists for the big-M trap,
    /// where a violation that is tiny
    /// RELATIVE to huge row coefficients (e.g. 5.0 on a 1e9-scale row) is
    /// decisive in absolute terms. Any row-scale-relative tolerance would be
    /// blind to exactly the violations this guard is for.
    pub(crate) fn check_constraints(&self, values: &[f64], tol: f64) -> bool {
        for (r, row) in self.orig_constraints.outer_iterator().enumerate() {
            let rhs = self.orig_rhs[r];
            let mut lhs = 0.0;
            for (v, &coeff) in row.iter() {
                if v < self.num_vars {
                    lhs += coeff * values[v];
                }
            }
            if !lhs.is_finite() {
                return false;
            }
            let slack = self.num_vars + r;
            let (smin, smax) = (self.orig_var_mins[slack], self.orig_var_maxs[slack]);
            let lo = if smax.is_finite() {
                rhs - smax
            } else {
                f64::NEG_INFINITY
            };
            let hi = if smin.is_finite() {
                rhs - smin
            } else {
                f64::INFINITY
            };
            let scaled_tol = tol * self.row_scales[r];
            if lhs < lo - scaled_tol || lhs > hi + scaled_tol {
                return false;
            }
        }
        true
    }

    /// Objective value (internal minimize space) of an explicit structural-var
    /// value vector.
    pub(crate) fn objective_of(&self, values: &[f64]) -> f64 {
        values
            .iter()
            .enumerate()
            .map(|(v, &x)| self.orig_obj_coeffs[v] * x)
            .sum()
    }

    pub(crate) fn get_var_bounds(&self, var: usize) -> (f64, f64) {
        (self.orig_var_mins[var], self.orig_var_maxs[var])
    }

    /// Change a variable's bounds in place. Records the new bounds and repairs the
    /// invariants that depend on them; does NOT run simplex — call [`Self::reoptimize`]
    /// afterwards. Returns `Err(Infeasible)` with state untouched if either bound
    /// is NaN or `min > max`.
    pub(crate) fn set_var_bounds(&mut self, var: usize, min: f64, max: f64) -> Result<(), Error> {
        if min.is_nan() || max.is_nan() || min > max {
            return Err(Error::Infeasible);
        }
        self.orig_var_mins[var] = min;
        self.orig_var_maxs[var] = max;
        match self.var_states[var] {
            VarState::Basic(row) => {
                self.basic_var_mins[row] = min;
                self.basic_var_maxs[row] = max;
                let val = self.basic_var_vals[row];
                if val < min - EPS || val > max + EPS {
                    self.is_primal_feasible = false;
                }
            }
            VarState::NonBasic(col) => {
                let cur = self.nb_var_vals[col];
                let new_val = cur.clamp(min, max);
                if new_val != cur {
                    // Shift the non-basic var to the nearest bound and propagate the
                    // delta into basic values (same mechanism as fix_var's non-basic arm).
                    self.calc_col_coeffs(col)?;
                    let diff = new_val - cur;
                    for (r, coeff) in self.col_coeffs.iter() {
                        self.basic_var_vals[r] -= diff * coeff;
                    }
                    self.cur_obj_val += diff * self.nb_var_obj_coeffs[col];
                    self.nb_var_vals[col] = new_val;
                    self.is_primal_feasible = false;
                }
                self.nb_var_states[col] = NonBasicVarState {
                    at_min: float_eq(new_val, min),
                    at_max: float_eq(new_val, max),
                };
                // A var at a loosened bound may no longer justify its reduced cost.
                self.is_dual_feasible = self.is_dual_feasible
                    && (self.nb_var_states[col].at_min && self.nb_var_obj_coeffs[col] > -EPS
                        || self.nb_var_states[col].at_max && self.nb_var_obj_coeffs[col] < EPS
                        || self.nb_var_obj_coeffs[col].abs() < EPS);
            }
        }
        Ok(())
    }

    /// Re-solve after bound changes or a basis load: dual simplex to restore primal
    /// feasibility, then primal simplex if reduced costs became dual-infeasible
    /// (only happens after loosening bounds or a numerically imperfect basis load).
    pub(crate) fn reoptimize(&mut self) -> Result<StopReason, Error> {
        if !self.is_primal_feasible && self.restore_feasibility()? == StopReason::Limit {
            return Ok(StopReason::Limit);
        }
        if !self.is_dual_feasible {
            self.recalc_obj_coeffs()?;
            if self.optimize()? == StopReason::Limit {
                return Ok(StopReason::Limit);
            }
            // Primal simplex may have moved through vertices; make sure primal holds too.
            if !self.is_primal_feasible && self.restore_feasibility()? == StopReason::Limit {
                return Ok(StopReason::Limit);
            }
        }
        self.finish_numerical_feasibility_restart()
    }

    pub(crate) fn snapshot_basis(&self) -> Basis {
        let mut statuses = Vec::with_capacity(self.num_total_vars());
        for var in 0..self.num_total_vars() {
            statuses.push(match self.var_states[var] {
                VarState::Basic(_) => VarStatus::Basic,
                VarState::NonBasic(col) => {
                    let s = &self.nb_var_states[col];
                    if s.at_min {
                        VarStatus::AtLower
                    } else if s.at_max {
                        VarStatus::AtUpper
                    } else {
                        VarStatus::Free
                    }
                }
            });
        }
        Basis(statuses)
    }

    /// The all-slack basis (identity basis matrix). Loading it cannot fail with a
    /// singular factorization, so it is the universal fallback.
    pub(crate) fn slack_basis(&self) -> Basis {
        let mut statuses = Vec::with_capacity(self.num_total_vars());
        for var in 0..self.num_vars {
            let min = self.orig_var_mins[var];
            let max = self.orig_var_maxs[var];
            statuses.push(if min.is_finite() {
                VarStatus::AtLower
            } else if max.is_finite() {
                VarStatus::AtUpper
            } else {
                VarStatus::Free
            });
        }
        for _ in 0..self.num_constraints() {
            statuses.push(VarStatus::Basic);
        }
        Basis(statuses)
    }

    /// Rebuild the solver state from a basis snapshot and the CURRENT variable bounds:
    /// non-basic values come from statuses + bounds, basic values and reduced costs are
    /// recomputed from scratch, and the LU factorization is rebuilt. Feasibility flags
    /// are recomputed honestly, so any partially rebuilt pre-load state is discarded.
    ///
    /// Statuses are interpreted against the CURRENT bounds: a status referring to a
    /// bound that has since moved or become infinite is remapped to the nearest finite
    /// bound (else 0) rather than rejected — the branch & bound driver relies on this
    /// when loading a parent basis after changing variable bounds.
    ///
    /// # Errors
    ///
    /// If this returns `Err`, the solver's internal state is unspecified and must
    /// not be used for solving until a subsequent successful `load_basis` restores
    /// it (the all-slack basis from [`Self::slack_basis`] always loads
    /// successfully and is the designated recovery path).
    pub(crate) fn load_basis(&mut self, basis: &Basis) -> Result<(), Error> {
        let n = self.num_total_vars();
        let m = self.num_constraints();
        if basis.0.len() != n || basis.0.iter().filter(|s| **s == VarStatus::Basic).count() != m {
            return Err(Error::InternalError("basis shape mismatch".to_string()));
        }

        self.basic_vars.clear();
        self.basic_var_mins.clear();
        self.basic_var_maxs.clear();
        self.nb_vars.clear();
        self.nb_var_vals.clear();
        self.nb_var_states.clear();
        self.nb_var_is_fixed.clear();

        for var in 0..n {
            match basis.0[var] {
                VarStatus::Basic => {
                    self.var_states[var] = VarState::Basic(self.basic_vars.len());
                    self.basic_vars.push(var);
                    self.basic_var_mins.push(self.orig_var_mins[var]);
                    self.basic_var_maxs.push(self.orig_var_maxs[var]);
                }
                ref status => {
                    let min = self.orig_var_mins[var];
                    let max = self.orig_var_maxs[var];
                    let val = match status {
                        VarStatus::AtLower => {
                            if min.is_finite() {
                                min
                            } else if max.is_finite() {
                                max
                            } else {
                                0.0
                            }
                        }
                        VarStatus::AtUpper => {
                            if max.is_finite() {
                                max
                            } else if min.is_finite() {
                                min
                            } else {
                                0.0
                            }
                        }
                        VarStatus::Free => {
                            if min.is_finite() {
                                min
                            } else if max.is_finite() {
                                max
                            } else {
                                0.0
                            }
                        }
                        VarStatus::Basic => unreachable!(),
                    };
                    self.var_states[var] = VarState::NonBasic(self.nb_vars.len());
                    self.nb_vars.push(var);
                    self.nb_var_vals.push(val);
                    self.nb_var_states.push(NonBasicVarState {
                        at_min: float_eq(val, min),
                        at_max: float_eq(val, max),
                    });
                    self.nb_var_is_fixed.push(false);
                }
            }
        }

        self.basis_solver
            .reset(&self.orig_constraints_csc, &self.basic_vars)?;

        // Steepest-edge reference reset (standard practice after a warm-start load;
        // only affects pivot ordering quality, not correctness).
        if self.enable_dual_steepest_edge {
            self.dual_edge_sq_norms = vec![1.0; self.basic_vars.len()];
        }

        self.recalc_basic_var_vals()?;
        self.recalc_obj_coeffs()?;

        self.is_primal_feasible = self.calc_primal_infeasibility().0 == 0;
        self.is_dual_feasible = self.calc_dual_infeasibility().0 == 0;
        Ok(())
    }

    pub(crate) fn fix_var(&mut self, var: usize, val: f64) -> Result<StopReason, Error> {
        if val < self.orig_var_mins[var] || val > self.orig_var_maxs[var] {
            return Err(Error::Infeasible);
        }

        let col = match self.var_states[var] {
            VarState::Basic(row) => {
                // if var was basic, remove it.
                self.calc_row_coeffs(row)?;
                let pivot_info = self.choose_entering_col_dual(row, val)?;
                self.calc_col_coeffs(pivot_info.col)?;
                self.pivot(&pivot_info, NumericalPhase::FixVariable)?;
                pivot_info.col
            }

            VarState::NonBasic(col) => {
                self.calc_col_coeffs(col)?;

                let diff = val - self.nb_var_vals[col];
                for (r, coeff) in self.col_coeffs.iter() {
                    self.basic_var_vals[r] -= diff * coeff;
                }
                self.cur_obj_val += diff * self.nb_var_obj_coeffs[col];
                self.nb_var_vals[col] = val;

                col
            }
        };

        self.nb_var_states[col] = NonBasicVarState {
            at_min: true,
            at_max: true,
        };
        self.nb_var_is_fixed[col] = true;

        self.is_primal_feasible = false;
        self.restore_feasibility()
    }

    /// Return whether the var was really unset and whether reoptimization
    /// finished within the active deadline.
    pub(crate) fn unfix_var(&mut self, var: usize) -> Result<(bool, StopReason), Error> {
        if let VarState::NonBasic(col) = self.var_states[var] {
            if !std::mem::replace(&mut self.nb_var_is_fixed[col], false) {
                return Ok((false, StopReason::Finished));
            }

            let cur_val = self.nb_var_vals[col];
            self.nb_var_states[col] = NonBasicVarState {
                at_min: float_eq(cur_val, self.orig_var_mins[var]),
                at_max: float_eq(cur_val, self.orig_var_maxs[var]),
            };

            self.is_dual_feasible = false;
            let stop = self.optimize()?;
            Ok((true, stop))
        } else {
            Ok((false, StopReason::Finished))
        }
    }

    pub(crate) fn num_constraints(&self) -> usize {
        self.orig_constraints.rows()
    }

    fn num_total_vars(&self) -> usize {
        self.num_vars + self.num_constraints()
    }

    pub(crate) fn initial_solve(&mut self) -> Result<StopReason, Error> {
        if check_deadline(&self.deadline) == StopReason::Limit {
            return Ok(StopReason::Limit);
        }

        if !self.is_primal_feasible && self.restore_feasibility()? == StopReason::Limit {
            return Ok(StopReason::Limit);
        }

        if !self.is_dual_feasible {
            self.recalc_obj_coeffs()?;
            if self.optimize()? == StopReason::Limit {
                return Ok(StopReason::Limit);
            }
        }

        // Disable updates of primal sq. norms, because lengthy primal simplex runs
        // are unlikely after the initial solve.
        self.enable_primal_steepest_edge = false;

        self.finish_numerical_feasibility_restart()
    }

    fn require_numerical_phase(&self, phase: NumericalPhase) -> Result<(), Error> {
        let violation = match phase {
            NumericalPhase::Primal => self.basic_vars.iter().enumerate().find_map(|(row, &var)| {
                let value = self.basic_var_vals[row];
                let min = self.basic_var_mins[row];
                let max = self.basic_var_maxs[row];
                (value < min - EPS || value > max + EPS).then(|| format!(
                    "row={row} var={var} value={value:.17e} bounds=[{min:.17e},{max:.17e}]"))
            }),
            NumericalPhase::Dual | NumericalPhase::FixVariable => self.nb_vars.iter()
                .zip(&self.nb_var_obj_coeffs).zip(&self.nb_var_states).enumerate()
                .find_map(|(col, ((&var, &cost), state))| {
                    let min = self.orig_var_mins[var];
                    let max = self.orig_var_maxs[var];
                    let valid = state.at_min && cost > -EPS || state.at_max && cost < EPS
                        || (!state.at_min && !state.at_max && min == f64::NEG_INFINITY
                            && max == f64::INFINITY && cost.abs() <= EPS);
                    (!valid).then(|| format!(
                        "col={col} var={var} cost={cost:.17e} value={:.17e} bounds=[{min:.17e},{max:.17e}] at_min={} at_max={} fixed={}",
                        self.nb_var_vals[col], state.at_min, state.at_max, self.nb_var_is_fixed[col]))
                }),
        };
        if let Some(detail) = violation {
            Err(Error::InternalError(format!(
                "recomputed state violates required {phase:?} phase: {detail}; EPS={EPS:.17e}; iterations={}", self.lp_iterations
            )))
        } else {
            Ok(())
        }
    }

    fn certify_phase_with_refinement(&mut self, phase: NumericalPhase) -> Result<(), Error> {
        match self.require_numerical_phase(phase) {
            Ok(()) => Ok(()),
            Err(before) if matches!(phase, NumericalPhase::Dual | NumericalPhase::FixVariable) => {
                if std::env::var_os("UOR_MICROLP_PROGRESS").is_some() {
                    eprintln!("d22-numerics reduced-cost-refine-before: {before}");
                }
                self.recalc_working_obj_coeffs_impl(true)?;
                let result = self.require_numerical_phase(phase);
                if std::env::var_os("UOR_MICROLP_PROGRESS").is_some() {
                    eprintln!("d22-numerics reduced-cost-refine-after: {result:?}");
                }
                result
            }
            Err(error) => Err(error),
        }
    }

    fn restart_zero_objective_feasibility(&mut self) -> Result<(), Error> {
        if self.numerical_feasibility_restart_active {
            return Err(Error::InternalError(
                "numerical feasibility restart already active".into(),
            ));
        }
        let mut candidate = self.clone();
        candidate
            .basis_solver
            .reset(&candidate.orig_constraints_csc, &candidate.basic_vars)?;
        candidate.recalc_basic_var_vals()?;
        candidate.working_obj_coeffs.fill(0.0);
        candidate.recalc_working_obj_coeffs()?;
        candidate.require_numerical_phase(NumericalPhase::Dual)?;
        candidate.is_primal_feasible = candidate.calc_primal_infeasibility().0 == 0;
        candidate.is_dual_feasible = false; // original objective is not yet optimized
        candidate.numerical_feasibility_restart_active = true;
        if candidate.enable_dual_steepest_edge {
            candidate.dual_edge_sq_norms.fill(1.0);
        }
        if candidate.enable_primal_steepest_edge {
            candidate.recalc_primal_sq_norms()?;
        }
        *self = candidate;
        numerical_progress(
            "zero-objective-feasibility-restart",
            &self.basic_vars,
            self.lp_iterations,
        );
        Ok(())
    }

    fn finish_numerical_feasibility_restart(&mut self) -> Result<StopReason, Error> {
        if !self.numerical_feasibility_restart_active {
            return Ok(StopReason::Finished);
        }
        self.recalc_basic_var_vals()?;
        self.require_numerical_phase(NumericalPhase::Primal)?;
        self.recalc_obj_coeffs()?;
        self.is_dual_feasible = false;
        if self.require_numerical_phase(NumericalPhase::Dual).is_err()
            && self.optimize()? == StopReason::Limit
        {
            return Ok(StopReason::Limit);
        }
        self.recalc_basic_var_vals()?;
        self.recalc_obj_coeffs()?;
        self.require_numerical_phase(NumericalPhase::Primal)?;
        self.certify_phase_with_refinement(NumericalPhase::Dual)?;
        let values: Vec<f64> = (0..self.num_vars).map(|v| *self.get_value(v)).collect();
        if values.iter().enumerate().any(|(var, &x)| {
            !x.is_finite() || x < self.orig_var_mins[var] - EPS || x > self.orig_var_maxs[var] + EPS
        }) || !self.check_constraints(&values, EPS)
        {
            return Err(Error::InternalError(
                "restored original objective failed original bounds/constraints".into(),
            ));
        }
        self.is_primal_feasible = true;
        self.is_dual_feasible = true;
        self.numerical_feasibility_restart_active = false;
        numerical_progress(
            "original-objective-restored",
            &self.basic_vars,
            self.lp_iterations,
        );
        Ok(StopReason::Finished)
    }

    fn refresh_numerics(&mut self, phase: NumericalPhase) -> Result<(), Error> {
        numerical_progress("refresh-begin", &self.basic_vars, self.lp_iterations);
        let mut refreshed = self.clone();
        refreshed
            .basis_solver
            .reset(&refreshed.orig_constraints_csc, &refreshed.basic_vars)?;
        refreshed.recalc_basic_var_vals()?;
        refreshed.recalc_working_obj_coeffs()?;
        // Feasibility flags describe the original objective, while an
        // artificial phase can have a different working objective. Validate
        // what the caller needs without silently changing either flag.
        refreshed.certify_phase_with_refinement(phase)?;
        if refreshed.enable_primal_steepest_edge {
            refreshed.recalc_primal_sq_norms()?;
        }
        if refreshed.enable_dual_steepest_edge {
            refreshed.dual_edge_sq_norms.fill(1.0);
        }
        *self = refreshed;
        crate::repair::count(|s| s.old_basis_refreshes += 1);
        numerical_progress("refresh-commit", &self.basic_vars, self.lp_iterations);
        Ok(())
    }
    fn optimize(&mut self) -> Result<StopReason, Error> {
        for iter in 0.. {
            self.lp_iterations += 1;
            if iter % DEADLINE_CHECK_INTERVAL == 0 {
                numerical_progress("primal-loop", &self.basic_vars, self.lp_iterations);
                if check_deadline(&self.deadline) == StopReason::Limit {
                    return Ok(StopReason::Limit);
                }

                let (num_vars, infeasibility) = self.calc_dual_infeasibility();
                debug!(
                    "optimize iter {}: obj.: {}, non-optimal coeffs: {} ({})",
                    iter, self.cur_obj_val, num_vars, infeasibility,
                );
            }

            let mut attempt = 0;
            let moved = loop {
                let result = (|| -> Result<bool, Error> {
                    if let Some(p) = self.choose_pivot()? {
                        self.pivot(&p, NumericalPhase::Primal)?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                })();
                match result {
                    Err(Error::InternalError(_)) if attempt == 0 => {
                        self.refresh_numerics(NumericalPhase::Primal)?;
                        attempt += 1;
                    }
                    other => break other?,
                }
            };
            if !moved {
                debug!(
                    "found optimum in {} iterations, obj.: {}",
                    iter + 1,
                    self.cur_obj_val,
                );
                break;
            }
        }

        self.is_dual_feasible = true;
        Ok(StopReason::Finished)
    }

    fn restore_feasibility(&mut self) -> Result<StopReason, Error> {
        let obj_str = if self.is_dual_feasible {
            "obj."
        } else {
            "artificial obj."
        };

        // Numerics valve, armed once per stall: before an infeasibility
        // declaration is allowed to stand, the basis gets refactorized and
        // the basic values recomputed from the original data. See below.
        let mut refreshed_since_pivot = false;

        for iter in 0.. {
            self.lp_iterations += 1;
            if iter % DEADLINE_CHECK_INTERVAL == 0 {
                numerical_progress("dual-loop", &self.basic_vars, self.lp_iterations);
                if check_deadline(&self.deadline) == StopReason::Limit {
                    return Ok(StopReason::Limit);
                }

                let (num_vars, infeasibility) = self.calc_primal_infeasibility();
                debug!(
                    "restore feasibility iter {}: {}: {}, infeas. vars: {} ({})",
                    iter, obj_str, self.cur_obj_val, num_vars, infeasibility,
                );
            }

            let mut attempt = 0;
            let keep_going = loop {
                let result = (|| -> Result<bool, Error> {
                    if let Some((row, leaving_new_val)) = self.choose_pivot_row_dual() {
                        self.calc_row_coeffs(row)?;
                        let pivot_info = match self.choose_entering_col_dual_policy(
                            row,
                            leaving_new_val,
                            attempt == 2,
                        ) {
                            Err(error) if attempt == 2 => {
                                return Err(Error::InternalError(format!(
                                    "strict numerical reselection failed: {error}"
                                )))
                            }
                            Ok(pivot_info) => pivot_info,
                            Err(Error::Infeasible) if !refreshed_since_pivot => {
                                // "No eligible entering column" is a proof of primal
                                // infeasibility only in exact arithmetic. This deep
                                // in an eta-file chain, the leaving row can be a
                                // *phantom* violation — basic values drifted by
                                // accumulated round-off — whose (equally drifted)
                                // pivot row then blocks every candidate; declaring
                                // infeasibility here is a wrong answer (netlib/brandy
                                // did exactly this). Rebuild the factorization and
                                // the basic values from the original data and
                                // re-examine: a phantom dissolves, a real
                                // infeasibility survives the refresh and the next
                                // declaration stands.
                                debug!(
                                    "restore feasibility iter {}: no entering column for row {}; \
                             refreshing basis before declaring infeasibility",
                                    iter, row,
                                );
                                self.refresh_numerics(NumericalPhase::Dual)?;
                                refreshed_since_pivot = true;
                                return Ok(true);
                            }
                            Err(e) => return Err(e),
                        };
                        self.calc_col_coeffs(pivot_info.col)?;
                        self.pivot(&pivot_info, NumericalPhase::Dual)?;
                        // Any successful pivot is progress: re-arm the valve.
                        refreshed_since_pivot = false;
                    } else {
                        debug!(
                            "restored feasibility in {} iterations, {}: {}",
                            iter + 1,
                            obj_str,
                            self.cur_obj_val,
                        );
                        return Ok(false);
                    }
                    Ok(true)
                })();
                match result {
                    Err(Error::InternalError(_)) if attempt == 0 => {
                        self.refresh_numerics(NumericalPhase::Dual)?;
                        attempt += 1;
                    }
                    Err(Error::InternalError(_)) if attempt == 1 => {
                        // A refreshed Harris proposal still failed the full
                        // certificate. Try one actual signed minimum ratio.
                        attempt = 2;
                    }
                    Err(Error::InternalError(_))
                        if attempt == 2 && !self.numerical_feasibility_restart_active =>
                    {
                        self.restart_zero_objective_feasibility()?;
                        refreshed_since_pivot = false;
                        attempt = 0;
                    }
                    other => break other?,
                }
            };
            if !keep_going {
                break;
            }
        }

        self.is_primal_feasible = true;
        self.finish_numerical_feasibility_restart()
    }

    pub(crate) fn add_constraint(
        &mut self,
        coeffs: CsVec,
        cmp_op: ComparisonOp,
        rhs: f64,
    ) -> Result<StopReason, Error> {
        assert!(self.is_primal_feasible);
        assert!(self.is_dual_feasible);

        let Some(PreparedRow {
            mut coeffs,
            rhs,
            row_scale,
            slack_var_min,
            slack_var_max,
        }) = prepare_row(coeffs, cmp_op, rhs)?
        else {
            return Ok(StopReason::Finished);
        };

        let slack_var = self.num_total_vars();

        self.orig_obj_coeffs.push(0.0);
        self.working_obj_coeffs.push(0.0);
        self.orig_var_mins.push(slack_var_min);
        self.orig_var_maxs.push(slack_var_max);
        self.var_states.push(VarState::Basic(self.basic_vars.len()));
        self.basic_vars.push(slack_var);
        self.basic_var_mins.push(slack_var_min);
        self.basic_var_maxs.push(slack_var_max);

        let mut lhs_val = 0.0;
        for (var, &coeff) in coeffs.iter() {
            let val = match self.var_states[var] {
                VarState::Basic(idx) => self.basic_var_vals[idx],
                VarState::NonBasic(idx) => self.nb_var_vals[idx],
            };
            lhs_val += val * coeff;
        }
        self.basic_var_vals.push(rhs - lhs_val);

        let new_num_total_vars = self.num_total_vars() + 1;
        let mut new_orig_constraints = CsMat::empty(CompressedStorage::CSR, new_num_total_vars);
        for row in self.orig_constraints.outer_iterator() {
            new_orig_constraints =
                new_orig_constraints.append_outer_csvec(resized_view(&row, new_num_total_vars));
        }
        coeffs = into_resized(coeffs, new_num_total_vars);
        coeffs.append(slack_var, 1.0);
        new_orig_constraints = new_orig_constraints.append_outer_csvec(coeffs.view());

        self.orig_rhs.push(rhs);
        self.row_scales.push(row_scale);

        self.orig_constraints = new_orig_constraints;
        self.orig_constraints_csc = self.orig_constraints.to_csc();

        self.basis_solver
            .reset(&self.orig_constraints_csc, &self.basic_vars)?;

        if self.enable_primal_steepest_edge || self.enable_dual_steepest_edge {
            // existing tableau rows didn't change, so we calc the last row
            // and add its contribution to the sq. norms.
            self.calc_row_coeffs(self.num_constraints() - 1)?;

            if self.enable_primal_steepest_edge {
                for (c, &coeff) in self.row_coeffs.iter() {
                    self.primal_edge_sq_norms[c] += coeff * coeff;
                }
            }

            if self.enable_dual_steepest_edge {
                self.dual_edge_sq_norms
                    .push(self.inv_basis_row_coeffs.sq_norm());
            }
        }

        self.is_primal_feasible = false;
        self.restore_feasibility()
    }

    /// Number of infeasible basic vars and sum of their infeasibilities.
    fn calc_primal_infeasibility(&self) -> (usize, f64) {
        let mut num_vars = 0;
        let mut infeasibility = 0.0;
        for ((&val, &min), &max) in self
            .basic_var_vals
            .iter()
            .zip(&self.basic_var_mins)
            .zip(&self.basic_var_maxs)
        {
            if val < min - EPS {
                num_vars += 1;
                infeasibility += min - val;
            } else if val > max + EPS {
                num_vars += 1;
                infeasibility += val - max;
            }
        }
        (num_vars, infeasibility)
    }

    /// Number of infeasible obj. coeffs and sum of their infeasibilities.
    fn calc_dual_infeasibility(&self) -> (usize, f64) {
        let mut num_vars = 0;
        let mut infeasibility = 0.0;
        for (&obj_coeff, var_state) in self.nb_var_obj_coeffs.iter().zip(&self.nb_var_states) {
            if !(var_state.at_min && obj_coeff > -EPS || var_state.at_max && obj_coeff < EPS) {
                num_vars += 1;
                infeasibility += obj_coeff.abs();
            }
        }
        (num_vars, infeasibility)
    }

    /// Calculate current coeffs column for a single non-basic variable.
    fn calc_col_coeffs(&mut self, c_var: usize) -> Result<(), Error> {
        let var = self.nb_vars[c_var];
        //guaranteed to be a valid index
        let orig_col = self.orig_constraints_csc.outer_view(var).unwrap();
        self.basis_solver
            .solve(orig_col.iter())?
            .to_sparse_vec(&mut self.col_coeffs);
        Ok(())
    }

    /// Calculate current coeffs row for a single constraint (permuted according to nb_vars).
    fn calc_row_coeffs(&mut self, r_constr: usize) -> Result<(), Error> {
        self.basis_solver
            .solve_transp(std::iter::once((r_constr, &1.0)))?
            .to_sparse_vec(&mut self.inv_basis_row_coeffs);

        self.row_coeffs.clear_and_resize(self.nb_vars.len());
        for (r, &coeff) in self.inv_basis_row_coeffs.iter() {
            //guaranteed to be a valid index
            for (v, &val) in self.orig_constraints.outer_view(r).unwrap().iter() {
                if let VarState::NonBasic(idx) = self.var_states[v] {
                    *self.row_coeffs.get_mut(idx) += val * coeff;
                }
            }
        }
        Ok(())
    }

    fn choose_pivot(&mut self) -> Result<Option<PivotInfo>, Error> {
        let entering_c = {
            let filtered_obj_coeffs = self
                .nb_var_obj_coeffs
                .iter()
                .zip(&self.nb_var_states)
                .enumerate()
                .filter_map(|(col, (&obj_coeff, var_state))| {
                    // Choose only among non-basic vars that can be changed
                    // with objective decreasing.
                    if (var_state.at_min && obj_coeff > -EPS)
                        || (var_state.at_max && obj_coeff < EPS)
                    {
                        None
                    } else {
                        Some((col, obj_coeff))
                    }
                });

            let mut best_col = None;
            let mut best_score = f64::NEG_INFINITY;
            if self.enable_primal_steepest_edge {
                for (col, obj_coeff) in filtered_obj_coeffs {
                    let score = obj_coeff * obj_coeff / self.primal_edge_sq_norms[col];
                    if score > best_score {
                        best_col = Some(col);
                        best_score = score;
                    }
                }
            } else {
                for (col, obj_coeff) in filtered_obj_coeffs {
                    let score = obj_coeff.abs();
                    if score > best_score {
                        best_col = Some(col);
                        best_score = score;
                    }
                }
            }

            if let Some(col) = best_col {
                col
            } else {
                return Ok(None);
            }
        };

        let entering_cur_val = self.nb_var_vals[entering_c];
        // If true, entering variable will increase (because the objective function must decrease).
        let entering_diff_sign = self.nb_var_obj_coeffs[entering_c] < 0.0;
        let entering_other_val = if entering_diff_sign {
            self.orig_var_maxs[self.nb_vars[entering_c]]
        } else {
            self.orig_var_mins[self.nb_vars[entering_c]]
        };

        self.calc_col_coeffs(entering_c)?;

        let get_leaving_var_step = |r: usize, coeff: f64| -> f64 {
            let val = self.basic_var_vals[r];
            // leaving_diff = -entering_diff * coeff. From this we can determine
            // in which direction this basic var will change and select appropriate bound.
            if (entering_diff_sign && coeff < 0.0) || (!entering_diff_sign && coeff > 0.0) {
                let max = self.basic_var_maxs[r];
                if val < max {
                    max - val
                } else {
                    0.0
                }
            } else {
                let min = self.basic_var_mins[r];
                if val > min {
                    val - min
                } else {
                    0.0
                }
            }
        };

        // Harris rule. See e.g.
        // Gill, P. E., Murray, W., Saunders, M. A., & Wright, M. H. (1989).
        // A practical anti-cycling procedure for linearly constrained optimization.
        // Mathematical Programming, 45(1-3), 437-474.
        //
        // https://link.springer.com/content/pdf/10.1007/BF01589114.pdf

        // First, we determine the max change in entering variable so that basic variables
        // remain feasible using relaxed bounds.
        let mut max_step = (entering_other_val - entering_cur_val).abs();
        for (r, &coeff) in self.col_coeffs.iter() {
            let coeff_abs = coeff.abs();
            if coeff_abs < EPS {
                continue;
            }

            // By which amount can we change the entering variable so that the limit on this
            // basic var is not violated. The var with the minimum such amount becomes leaving.
            let cur_step = (get_leaving_var_step(r, coeff) + EPS) / coeff_abs;
            if cur_step < max_step {
                max_step = cur_step;
            }
        }

        // Second, we choose among variables with steps less than max_step a variable with the biggest
        // abs. coefficient as the leaving variable. This means that we get numerically more stable
        // basis at the price of slight infeasibility of some basic variables.
        let mut leaving_r = None;
        let mut leaving_new_val = 0.0;
        let mut pivot_coeff_abs = f64::NEG_INFINITY;
        let mut pivot_coeff = 0.0;
        for (r, &coeff) in self.col_coeffs.iter() {
            let coeff_abs = coeff.abs();
            if coeff_abs < EPS {
                continue;
            }

            let cur_step = get_leaving_var_step(r, coeff) / coeff_abs;
            if cur_step <= max_step && coeff_abs > pivot_coeff_abs {
                leaving_r = Some(r);
                leaving_new_val = if (entering_diff_sign && coeff < 0.0)
                    || (!entering_diff_sign && coeff > 0.0)
                {
                    self.basic_var_maxs[r]
                } else {
                    self.basic_var_mins[r]
                };
                pivot_coeff = coeff;
                pivot_coeff_abs = coeff_abs;
            }
        }

        if let Some(row) = leaving_r {
            self.calc_row_coeffs(row)?;

            let entering_diff = (self.basic_var_vals[row] - leaving_new_val) / pivot_coeff;
            let entering_new_val = entering_cur_val + entering_diff;

            Ok(Some(PivotInfo {
                col: entering_c,
                entering_new_val,
                entering_diff,
                elem: Some(PivotElem {
                    row,
                    coeff: pivot_coeff,
                    leaving_new_val,
                }),
            }))
        } else {
            if entering_other_val.is_infinite() {
                return Err(Error::Unbounded);
            }

            Ok(Some(PivotInfo {
                col: entering_c,
                entering_new_val: entering_other_val,
                entering_diff: entering_other_val - entering_cur_val,
                elem: None,
            }))
        }
    }

    fn choose_pivot_row_dual(&self) -> Option<(usize, f64)> {
        let infeasibilities = self
            .basic_var_vals
            .iter()
            .zip(&self.basic_var_mins)
            .zip(&self.basic_var_maxs)
            .enumerate()
            .filter_map(|(r, ((&val, &min), &max))| {
                if val < min - EPS {
                    Some((r, min - val))
                } else if val > max + EPS {
                    Some((r, val - max))
                } else {
                    None
                }
            });

        let mut leaving_r = None;
        let mut max_score = f64::NEG_INFINITY;
        if self.enable_dual_steepest_edge {
            for (r, infeasibility) in infeasibilities {
                let sq_norm = self.dual_edge_sq_norms[r];
                let score = infeasibility * infeasibility / sq_norm;
                if score > max_score {
                    leaving_r = Some(r);
                    max_score = score;
                }
            }
        } else {
            for (r, infeasibility) in infeasibilities {
                if infeasibility > max_score {
                    leaving_r = Some(r);
                    max_score = infeasibility;
                }
            }
        }

        leaving_r.map(|r| {
            let val = self.basic_var_vals[r];
            let min = self.basic_var_mins[r];
            let max = self.basic_var_maxs[r];

            // If we choose this var as leaving, its new val will be at the boundary
            // which is violated.
            // Why is that? We must maintain primal optimality (a.k.a. dual feasibility) for
            // the leaving variable, thus new_obj_coeff must be >= 0 if new_val is min, and <= 0
            // if new_val is max. Sign of the leaving var obj coeff:
            // sign(new_obj_coeff) = -sign(old_obj_coeff) * sign(pivot_coeff).
            // Another constraint is that we must not decrease primal objective.
            // As sign(obj_val_diff) = -sign(old_obj_coeff) * sign(leaving_diff) * sign(pivot_coeff)
            // must be >= 0, we conclude that sign(new_obj_coeff) = sign(leaving_diff).
            // From this we see that if old val was < min, dual feasibility is maintained if the
            // new var is min (analogously for max).
            let new_val = if val < min {
                min
            } else if val > max {
                max
            } else {
                unreachable!();
            };
            (r, new_val)
        })
    }

    fn choose_entering_col_dual(
        &self,
        row: usize,
        leaving_new_val: f64,
    ) -> Result<PivotInfo, Error> {
        self.choose_entering_col_dual_policy(row, leaving_new_val, false)
    }

    /// Proposal interval over every actual column, including ineligible/small
    /// coefficients and the leaving variable. Final candidate certification is
    /// independent of this floating-point filter.
    fn dual_step_interval(&self, row: usize, leaving_new_val: f64) -> Result<(f64, f64), Error> {
        let direction = if leaving_new_val > self.basic_var_vals[row] {
            1.0
        } else {
            -1.0
        };
        let mut lower = f64::NEG_INFINITY;
        let mut upper = f64::INFINITY;
        let mut restrict =
            |cost: f64, slope: f64, state: &NonBasicVarState, free: bool| -> Result<(), Error> {
                crate::repair::finite(cost)?;
                crate::repair::finite(slope)?;
                if state.at_min && state.at_max {
                    return Ok(());
                }
                if !state.at_min && !state.at_max && !free {
                    return Err(Error::InternalError(
                        "nonbasic interior state has no certified step interval".into(),
                    ));
                }
                // Upper inequalities v+k*t<EPS, or <=EPS for truly free vars.
                for (is_upper, v, k) in [(true, cost, slope), (false, -cost, -slope)] {
                    let active = if is_upper {
                        state.at_max || free
                    } else {
                        state.at_min || free
                    };
                    if !active {
                        continue;
                    }
                    if k == 0.0 {
                        if v > EPS || (v == EPS && !free) {
                            return Err(Error::InternalError(
                                "constant reduced cost excludes every step".into(),
                            ));
                        }
                    } else {
                        let bound = crate::repair::finite((EPS - v) / k)?;
                        if k > 0.0 {
                            upper = upper.min(bound);
                        } else {
                            lower = lower.max(bound);
                        }
                    }
                }
                Ok(())
            };
        for (col, &var) in self.nb_vars.iter().enumerate() {
            let state = &self.nb_var_states[col];
            let free = !state.at_min
                && !state.at_max
                && self.orig_var_mins[var] == f64::NEG_INFINITY
                && self.orig_var_maxs[var] == f64::INFINITY;
            restrict(
                self.nb_var_obj_coeffs[col],
                direction * *self.row_coeffs.get(col),
                state,
                free,
            )?;
        }
        let var = self.basic_vars[row];
        let leaving = NonBasicVarState {
            at_min: float_eq(leaving_new_val, self.orig_var_mins[var]),
            at_max: float_eq(leaving_new_val, self.orig_var_maxs[var]),
        };
        restrict(0.0, direction, &leaving, false)?;
        if lower > upper {
            return Err(Error::InternalError(format!(
                "empty reduced-cost step interval [{lower:.17e},{upper:.17e}]"
            )));
        }
        Ok((lower, upper))
    }

    fn affine_dual_step_valid(
        &self,
        row: usize,
        leaving_new_val: f64,
        step: f64,
    ) -> Result<bool, Error> {
        let direction = if leaving_new_val > self.basic_var_vals[row] {
            1.0
        } else {
            -1.0
        };
        let valid = |var: usize, cost: f64, state: &NonBasicVarState| {
            state.at_min && cost > -EPS
                || state.at_max && cost < EPS
                || (!state.at_min
                    && !state.at_max
                    && self.orig_var_mins[var] == f64::NEG_INFINITY
                    && self.orig_var_maxs[var] == f64::INFINITY
                    && cost.abs() <= EPS)
        };
        for (col, &var) in self.nb_vars.iter().enumerate() {
            let cost = crate::repair::finite(
                (direction * *self.row_coeffs.get(col)).mul_add(step, self.nb_var_obj_coeffs[col]),
            )?;
            if !valid(var, cost, &self.nb_var_states[col]) {
                return Ok(false);
            }
        }
        let var = self.basic_vars[row];
        let state = NonBasicVarState {
            at_min: float_eq(leaving_new_val, self.orig_var_mins[var]),
            at_max: float_eq(leaving_new_val, self.orig_var_maxs[var]),
        };
        Ok(valid(var, crate::repair::finite(direction * step)?, &state))
    }

    fn choose_entering_col_dual_policy(
        &self,
        row: usize,
        leaving_new_val: f64,
        strict_signed_ratio: bool,
    ) -> Result<PivotInfo, Error> {
        // True if the new obj. coeff. must be nonnegative in a dual-feasible configuration.
        let leaving_diff_sign = leaving_new_val > self.basic_var_vals[row];
        let interval = if strict_signed_ratio {
            self.dual_step_interval(row, leaving_new_val)?
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        };

        fn clamp_obj_coeff(mut obj_coeff: f64, var_state: &NonBasicVarState) -> f64 {
            if var_state.at_min && obj_coeff < 0.0 {
                obj_coeff = 0.0;
            }
            if var_state.at_max && obj_coeff > 0.0 {
                obj_coeff = 0.0;
            }
            obj_coeff
        }

        let is_eligible_var = |coeff: f64, var_state: &NonBasicVarState| -> bool {
            let entering_diff_sign = if coeff >= EPS {
                !leaving_diff_sign
            } else if coeff <= -EPS {
                leaving_diff_sign
            } else {
                return false;
            };

            if entering_diff_sign {
                !var_state.at_max
            } else {
                !var_state.at_min
            }
        };

        // Harris rule. See e.g.
        // Gill, P. E., Murray, W., Saunders, M. A., & Wright, M. H. (1989).
        // A practical anti-cycling procedure for linearly constrained optimization.
        // Mathematical Programming, 45(1-3), 437-474.
        //
        // https://link.springer.com/content/pdf/10.1007/BF01589114.pdf

        // First, we determine the max step (change in the leaving variable obj. coeff that still
        // leaves us with a dual-feasible state) using relaxed bounds.
        let mut max_step = f64::INFINITY;
        for (c, &coeff) in self.row_coeffs.iter() {
            let var_state = &self.nb_var_states[c];
            if !is_eligible_var(coeff, var_state) {
                continue;
            }

            let obj_coeff = clamp_obj_coeff(self.nb_var_obj_coeffs[c], var_state);
            let cur_step = (obj_coeff.abs() + EPS) / coeff.abs();
            if cur_step < max_step {
                max_step = cur_step;
            }
        }

        // Second, we choose among the variables satisfying the relaxed step bound
        // the one with the biggest pivot coefficient. This allows for a much more
        // numerically stable basis at the price of slight infeasibility in dual variables.
        let mut entering_c = None;
        let mut pivot_coeff_abs = f64::NEG_INFINITY;
        let mut pivot_coeff = 0.0;
        let mut best_signed_ratio = f64::INFINITY;
        for (c, &coeff) in self.row_coeffs.iter() {
            let var_state = &self.nb_var_states[c];
            if !is_eligible_var(coeff, var_state) {
                continue;
            }

            let obj_coeff = clamp_obj_coeff(self.nb_var_obj_coeffs[c], var_state);

            // If we change obj. coeff of the leaving variable by this amount,
            // obj. coeff if the current variable will reach the bound of dual infeasibility.
            // Variable with the tightest such bound is the entering variable.
            let cur_step = obj_coeff.abs() / coeff.abs();
            let coeff_abs = coeff.abs();
            if strict_signed_ratio {
                // c'_j = c_j + d*t*a_j, d=sign(leaving_new-old).
                // The zero crossing is t=-d*c_j/a_j. Keep its actual
                // sign: a pre-existing within-EPS violation may yield a
                // negative tolerance-scale step. The complete recomputed
                // candidate must still pass the unchanged phase certificate.
                let direction = if leaving_diff_sign { 1.0 } else { -1.0 };
                let ratio = crate::repair::finite(-direction * self.nb_var_obj_coeffs[c] / coeff)?;
                if ratio < interval.0
                    || ratio > interval.1
                    || !self.affine_dual_step_valid(row, leaving_new_val, ratio)?
                {
                    continue;
                }
                if ratio < best_signed_ratio
                    || (ratio == best_signed_ratio
                        && (coeff_abs > pivot_coeff_abs
                            || (coeff_abs == pivot_coeff_abs
                                && entering_c.is_none_or(|old| c < old))))
                {
                    best_signed_ratio = ratio;
                    entering_c = Some(c);
                    pivot_coeff_abs = coeff_abs;
                    pivot_coeff = coeff;
                }
            } else if cur_step <= max_step && coeff_abs > pivot_coeff_abs {
                entering_c = Some(c);
                pivot_coeff_abs = coeff_abs;
                pivot_coeff = coeff;
            }
        }

        if let Some(col) = entering_c {
            if strict_signed_ratio && std::env::var_os("UOR_MICROLP_PROGRESS").is_some() {
                eprintln!("d22-numerics strict-signed-ratio row={row} col={col} ratio={best_signed_ratio:.17e} interval=[{:.17e},{:.17e}] tolerance_qualified_only=true", interval.0, interval.1);
            }
            let entering_diff = (self.basic_var_vals[row] - leaving_new_val) / pivot_coeff;
            let entering_new_val = self.nb_var_vals[col] + entering_diff;

            Ok(PivotInfo {
                col,
                entering_new_val,
                entering_diff,
                elem: Some(PivotElem {
                    row,
                    leaving_new_val,
                    coeff: pivot_coeff,
                }),
            })
        } else if strict_signed_ratio {
            Err(Error::InternalError(
                "no admissible signed zero crossing in reduced-cost interval".into(),
            ))
        } else {
            Err(Error::Infeasible)
        }
    }

    fn pivot(&mut self, pivot_info: &PivotInfo, phase: NumericalPhase) -> Result<(), Error> {
        if let Some(e) = &pivot_info.elem {
            let col = *self
                .col_coeffs
                .iter()
                .find(|(r, _)| *r == e.row)
                .map(|(_, v)| v)
                .ok_or_else(|| Error::InternalError("missing actual pivot".into()))?;
            let row = *self.row_coeffs.get(pivot_info.col);
            let scale = col.abs().max(row.abs()).max(e.coeff.abs());
            if !col.is_finite() || !row.is_finite() || !e.coeff.is_finite() || scale == 0.0 {
                return Err(Error::InternalError(
                    "nonfinite or zero pivot evidence".into(),
                ));
            }
            if (col - row).abs() > 128.0 * self.basic_vars.len() as f64 * f64::EPSILON * scale
                || (col - e.coeff).abs()
                    > 128.0 * self.basic_vars.len() as f64 * f64::EPSILON * scale
            {
                let boundary = format!(
                    "uncertified row/column pivot agreement: row={} col={} column={:.17e} row_value={:.17e} selected={:.17e} scale={:.17e} threshold={:.17e}",
                    e.row, pivot_info.col, col, row, e.coeff, scale,
                    128.0 * self.basic_vars.len() as f64 * f64::EPSILON * scale
                );
                return self
                    .pivot_by_original_basis(pivot_info, phase)
                    .map_err(|e| {
                        Error::InternalError(format!(
                            "{boundary}; proposed-basis rebuild rejected: {e}"
                        ))
                    });
            }
        }
        let mut candidate = self.clone();
        candidate.pivot_inner(pivot_info)?;
        // Both actual primal RHS and current objective RHS must survive the update.
        candidate.recalc_basic_var_vals()?;
        candidate.recalc_working_obj_coeffs()?;
        if matches!(phase, NumericalPhase::FixVariable) {
            candidate.nb_var_states[pivot_info.col] = NonBasicVarState {
                at_min: true,
                at_max: true,
            };
            candidate.nb_var_is_fixed[pivot_info.col] = true;
        }
        candidate.certify_phase_with_refinement(phase)?;
        *self = candidate;
        Ok(())
    }

    /// An ambiguous coefficient never enters an eta update. Treat the selected
    /// variable exchange as a proposal only, rebuild from original columns,
    /// recompute both actual RHS solutions, and check the feasibility required
    /// by the current simplex phase before committing the clone.
    fn pivot_by_original_basis(
        &mut self,
        info: &PivotInfo,
        phase: NumericalPhase,
    ) -> Result<(), Error> {
        numerical_progress(
            "ambiguous-exchange-rebuild",
            &self.basic_vars,
            self.lp_iterations,
        );
        let elem = info.elem.as_ref().ok_or_else(|| {
            Error::InternalError("proposed-basis rebuild requires an exchange".into())
        })?;
        let mut candidate = self.clone();
        let entering = candidate.nb_vars[info.col];
        let leaving = candidate.basic_vars[elem.row];
        let value = crate::repair::finite(elem.leaving_new_val)?;
        if value < candidate.orig_var_mins[leaving] - EPS
            || value > candidate.orig_var_maxs[leaving] + EPS
        {
            return Err(Error::InternalError(
                "proposed nonbasic value violates bounds".into(),
            ));
        }
        candidate.basic_vars[elem.row] = entering;
        candidate.var_states[entering] = VarState::Basic(elem.row);
        candidate.basic_var_mins[elem.row] = candidate.orig_var_mins[entering];
        candidate.basic_var_maxs[elem.row] = candidate.orig_var_maxs[entering];
        candidate.nb_vars[info.col] = leaving;
        candidate.var_states[leaving] = VarState::NonBasic(info.col);
        candidate.nb_var_vals[info.col] = value;
        candidate.nb_var_states[info.col] = NonBasicVarState {
            at_min: float_eq(value, candidate.orig_var_mins[leaving]),
            at_max: float_eq(value, candidate.orig_var_maxs[leaving]),
        };
        if matches!(phase, NumericalPhase::FixVariable) {
            // fix_var will mark this nonbasic value fixed immediately after
            // the exchange. Its requested value can be an interior point of
            // the original bounds and may require primal restoration.
            candidate.nb_var_states[info.col] = NonBasicVarState {
                at_min: true,
                at_max: true,
            };
            candidate.nb_var_is_fixed[info.col] = true;
        }
        candidate
            .basis_solver
            .reset(&candidate.orig_constraints_csc, &candidate.basic_vars)?;
        candidate.recalc_basic_var_vals()?;
        candidate.recalc_working_obj_coeffs()?;
        candidate.certify_phase_with_refinement(phase)?;
        if matches!(phase, NumericalPhase::Primal) && candidate.cur_obj_val > self.cur_obj_val + EPS
        {
            return Err(Error::InternalError(
                "proposed basis does not preserve phase feasibility/descent".into(),
            ));
        }
        if candidate.enable_primal_steepest_edge {
            candidate.recalc_primal_sq_norms()?;
        }
        if candidate.enable_dual_steepest_edge {
            candidate.dual_edge_sq_norms.fill(1.0);
        }
        *self = candidate;
        Ok(())
    }
    fn pivot_inner(&mut self, pivot_info: &PivotInfo) -> Result<(), Error> {
        // TODO: periodically (say, every 1000 pivots) recalc basic vars and object coeffs
        // from scratch for numerical stability.

        self.cur_obj_val += self.nb_var_obj_coeffs[pivot_info.col] * pivot_info.entering_diff;

        let entering_var = self.nb_vars[pivot_info.col];

        if pivot_info.elem.is_none() {
            // "entering" var is still non-basic, it just changes value from one limit
            // to the other.
            self.nb_var_vals[pivot_info.col] = pivot_info.entering_new_val;
            for (r, coeff) in self.col_coeffs.iter() {
                self.basic_var_vals[r] -= pivot_info.entering_diff * coeff;
            }
            let var_state = &mut self.nb_var_states[pivot_info.col];
            var_state.at_min = float_eq(
                pivot_info.entering_new_val,
                self.orig_var_mins[entering_var],
            );
            var_state.at_max = float_eq(
                pivot_info.entering_new_val,
                self.orig_var_maxs[entering_var],
            );
            return Ok(());
        }
        //guaranteed, none variant already handled
        let pivot_elem = pivot_info.elem.as_ref().unwrap();
        let pivot_coeff = pivot_elem.coeff;

        // Update basic vars stuff

        for (r, coeff) in self.col_coeffs.iter() {
            if r == pivot_elem.row {
                self.basic_var_vals[r] = pivot_info.entering_new_val;
            } else {
                self.basic_var_vals[r] -= pivot_info.entering_diff * coeff;
            }
        }

        self.basic_var_mins[pivot_elem.row] = self.orig_var_mins[entering_var];
        self.basic_var_maxs[pivot_elem.row] = self.orig_var_maxs[entering_var];

        if self.enable_dual_steepest_edge {
            self.update_dual_sq_norms(pivot_elem.row, pivot_coeff)?;
        }

        // Update non-basic vars stuff

        let leaving_var = self.basic_vars[pivot_elem.row];

        self.nb_var_vals[pivot_info.col] = pivot_elem.leaving_new_val;
        let leaving_var_state = &mut self.nb_var_states[pivot_info.col];
        leaving_var_state.at_min =
            float_eq(pivot_elem.leaving_new_val, self.orig_var_mins[leaving_var]);
        leaving_var_state.at_max =
            float_eq(pivot_elem.leaving_new_val, self.orig_var_maxs[leaving_var]);

        let pivot_obj = self.nb_var_obj_coeffs[pivot_info.col] / pivot_coeff;
        for (c, &coeff) in self.row_coeffs.iter() {
            if c == pivot_info.col {
                self.nb_var_obj_coeffs[c] = -pivot_obj;
            } else {
                self.nb_var_obj_coeffs[c] -= pivot_obj * coeff;
            }
        }

        if self.enable_primal_steepest_edge {
            self.update_primal_sq_norms(pivot_info.col, pivot_coeff)?;
        }

        // Update basis itself

        self.basic_vars[pivot_elem.row] = entering_var;
        self.var_states[entering_var] = VarState::Basic(pivot_elem.row);
        self.nb_vars[pivot_info.col] = leaving_var;
        self.var_states[leaving_var] = VarState::NonBasic(pivot_info.col);

        // A simple heuristic to choose when to recompute LU factorization.
        // Note: a possible failure mode is that the LU factorization accidentally
        // generates a lot of fill-in and doesn't get recomputed for a long time.
        let eta_matrices_nnz = self.basis_solver.eta_matrices.coeff_cols.nnz();
        if eta_matrices_nnz < self.basis_solver.lu_factors.nnz() {
            self.basis_solver
                .push_eta_matrix(&self.col_coeffs, pivot_elem.row, pivot_coeff);
            self.basis_solver.current_basis =
                crate::repair::Basis::new(self.basic_vars.len(), |c| {
                    self.orig_constraints_csc
                        .outer_view(self.basic_vars[c])
                        .unwrap()
                        .into_raw_storage()
                })?;
        } else {
            self.basis_solver
                .reset(&self.orig_constraints_csc, &self.basic_vars)?;
        }
        Ok(())
    }

    fn update_primal_sq_norms(
        &mut self,
        entering_col: usize,
        pivot_coeff: f64,
    ) -> Result<(), Error> {
        // Computations for the steepest edge pivoting rule. See
        // Forrest, J. J., & Goldfarb, D. (1992).
        // Steepest-edge simplex algorithms for linear programming.
        // Mathematical programming, 57(1-3), 341-374.
        //
        // https://link.springer.com/content/pdf/10.1007/BF01581089.pdf

        let tmp = self.basis_solver.solve_transp(self.col_coeffs.iter())?;
        // now tmp contains the v vector from the article.

        for &r in tmp.indices() {
            //guaranteed to be a valid index
            for &v in self.orig_constraints.outer_view(r).unwrap().indices() {
                if let VarState::NonBasic(idx) = self.var_states[v] {
                    self.sq_norms_update_helper[idx] = 0.0;
                }
            }
        }
        // now significant positions in sq_norms_update_helper are cleared.

        for (r, &coeff) in tmp.iter() {
            //guaranteed to be a valid index
            for (v, &val) in self.orig_constraints.outer_view(r).unwrap().iter() {
                if let VarState::NonBasic(idx) = self.var_states[v] {
                    self.sq_norms_update_helper[idx] += val * coeff;
                }
            }
        }
        // now sq_norms_update_helper contains transp(N) * v vector.

        // Calculate pivot_sq_norm directly to avoid loss of precision.
        let pivot_sq_norm = self.col_coeffs.sq_norm() + 1.0;
        // assert!((self.primal_edge_sq_norms[entering_col] - pivot_sq_norm).abs() < 0.1);

        let pivot_coeff_sq = pivot_coeff * pivot_coeff;
        for (c, &r_coeff) in self.row_coeffs.iter() {
            if c == entering_col {
                self.primal_edge_sq_norms[c] = pivot_sq_norm / pivot_coeff_sq;
            } else {
                self.primal_edge_sq_norms[c] += -2.0 * r_coeff * self.sq_norms_update_helper[c]
                    / pivot_coeff
                    + pivot_sq_norm * r_coeff * r_coeff / pivot_coeff_sq;
            }

            crate::repair::finite(self.primal_edge_sq_norms[c])?;
        }
        Ok(())
    }

    fn update_dual_sq_norms(&mut self, leaving_row: usize, pivot_coeff: f64) -> Result<(), Error> {
        // Computations for the dual steepest edge pivoting rule.
        // See the same reference (Forrest, Goldfarb).

        let tau = self.basis_solver.solve(self.inv_basis_row_coeffs.iter())?;

        // Calculate pivot_sq_norm directly to avoid loss of precision.
        let pivot_sq_norm = self.inv_basis_row_coeffs.sq_norm();
        // assert!((self.dual_edge_sq_norms[leaving_row] - pivot_sq_norm).abs() < 0.1);

        let pivot_coeff_sq = pivot_coeff * pivot_coeff;
        for (r, &col_coeff) in self.col_coeffs.iter() {
            if r == leaving_row {
                self.dual_edge_sq_norms[r] = pivot_sq_norm / pivot_coeff_sq;
            } else {
                self.dual_edge_sq_norms[r] += -2.0 * col_coeff * tau.get(r) / pivot_coeff
                    + pivot_sq_norm * col_coeff * col_coeff / pivot_coeff_sq;
            }

            crate::repair::finite(self.dual_edge_sq_norms[r])?;
        }
        Ok(())
    }

    fn recalc_basic_var_vals(&mut self) -> Result<(), Error> {
        let mut cur_vals = self.orig_rhs.clone();
        for (i, var) in self.nb_vars.iter().enumerate() {
            let val = self.nb_var_vals[i];
            if val != 0.0 {
                //guaranteed to be a valid index
                for (r, &coeff) in self.orig_constraints_csc.outer_view(*var).unwrap().iter() {
                    cur_vals[r] =
                        crate::repair::finite(cur_vals[r] - crate::repair::finite(val * coeff)?)?;
                }
            }
        }

        // Etas are applied to the dense solve directly; a pending eta file
        // does not require a full basis refactorization.
        self.basis_solver.solve_dense_with_etas(&mut cur_vals)?;
        self.basic_var_vals = cur_vals;
        Ok(())
    }

    fn recalc_obj_coeffs(&mut self) -> Result<(), Error> {
        self.working_obj_coeffs = self.orig_obj_coeffs.clone();
        self.recalc_working_obj_coeffs()
    }
    fn recalc_working_obj_coeffs(&mut self) -> Result<(), Error> {
        self.recalc_working_obj_coeffs_impl(false)
    }

    fn refine_transpose_multipliers(
        &mut self,
        rhs: &[f64],
        values: &mut Vec<f64>,
    ) -> Result<(), Error> {
        fn residual(
            basis: &crate::repair::Basis,
            rhs: &[f64],
            values: &[f64],
        ) -> Result<Vec<f64>, Error> {
            basis
                .columns
                .iter()
                .zip(rhs)
                .map(|((rows, coefficients), &b)| {
                    compensated_products(
                        std::iter::once((b, 1.0)).chain(
                            rows.iter()
                                .zip(coefficients)
                                .map(|(&row, &a)| (-a, values[row])),
                        ),
                    )
                })
                .collect()
        }
        self.basis_solver.current_basis.check(rhs, values, true)?;
        let mut r = residual(&self.basis_solver.current_basis, rhs, values)?;
        let mut norm = r.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
        let before = norm;
        let mut accepted = 0;
        for _ in 0..3 {
            if norm == 0.0 {
                break;
            }
            let mut delta = r.clone();
            self.basis_solver.solve_transp_dense_with_etas(&mut delta)?;
            let proposed: Vec<f64> = values
                .iter()
                .zip(delta)
                .map(|(&x, dx)| crate::repair::finite(x + dx))
                .collect::<Result<_, _>>()?;
            self.basis_solver
                .current_basis
                .check(rhs, &proposed, true)?;
            let next = residual(&self.basis_solver.current_basis, rhs, &proposed)?;
            let next_norm = next.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
            if next_norm >= norm {
                break;
            }
            *values = proposed;
            r = next;
            norm = next_norm;
            accepted += 1;
        }
        if std::env::var_os("UOR_MICROLP_PROGRESS").is_some() {
            eprintln!("d22-numerics transpose-refine residual_before={before:.17e} residual_after={norm:.17e} accepted={accepted}");
        }
        Ok(())
    }

    fn recalc_working_obj_coeffs_impl(&mut self, refine: bool) -> Result<(), Error> {
        // Same as recalc_basic_var_vals: pending etas participate in the
        // (transposed) dense solve instead of forcing a refactorization.
        let multipliers = {
            let mut rhs = vec![0.0; self.num_constraints()];
            for (c, &var) in self.basic_vars.iter().enumerate() {
                rhs[c] = self.working_obj_coeffs[var];
            }
            let original_rhs = rhs.clone();
            self.basis_solver.solve_transp_dense_with_etas(&mut rhs)?;
            if refine {
                self.refine_transpose_multipliers(&original_rhs, &mut rhs)?;
            }
            rhs
        };

        self.nb_var_obj_coeffs.clear();
        for &var in &self.nb_vars {
            //guaranteed to be a valid index
            let col = self.orig_constraints_csc.outer_view(var).unwrap();
            let cost = if refine {
                compensated_products(
                    std::iter::once((self.working_obj_coeffs[var], 1.0))
                        .chain(col.iter().map(|(r, &val)| (-val, multipliers[r]))),
                )?
            } else {
                let mut dot_prod = 0.0;
                for (r, &val) in col.iter() {
                    dot_prod = crate::repair::finite(
                        dot_prod + crate::repair::finite(val * multipliers[r])?,
                    )?;
                }
                crate::repair::finite(self.working_obj_coeffs[var] - dot_prod)?
            };
            self.nb_var_obj_coeffs.push(cost);
        }

        self.cur_obj_val = 0.0;
        for (r, &var) in self.basic_vars.iter().enumerate() {
            self.cur_obj_val = crate::repair::finite(
                self.cur_obj_val
                    + crate::repair::finite(self.working_obj_coeffs[var] * self.basic_var_vals[r])?,
            )?;
        }
        for (c, &var) in self.nb_vars.iter().enumerate() {
            self.cur_obj_val = crate::repair::finite(
                self.cur_obj_val
                    + crate::repair::finite(self.working_obj_coeffs[var] * self.nb_var_vals[c])?,
            )?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn recalc_primal_sq_norms(&mut self) -> Result<(), Error> {
        self.primal_edge_sq_norms.clear();
        for &var in &self.nb_vars {
            //guaranteed to be a valid index
            let col = self.orig_constraints_csc.outer_view(var).unwrap();
            let sq_norm = self.basis_solver.solve(col.iter())?.sq_norm() + 1.0;
            self.primal_edge_sq_norms.push(sq_norm);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct PivotInfo {
    col: usize,
    entering_new_val: f64,
    entering_diff: f64,

    /// Contains info about the intersection between pivot row and column.
    /// If it is None, objective can be decreased without changing the basis
    /// (simply by changing the value of non-basic variable chosen as entering)
    elem: Option<PivotElem>,
}

#[derive(Clone, Copy, Debug)]
enum NumericalPhase {
    Primal,
    Dual,
    FixVariable,
}

#[derive(Debug)]
struct PivotElem {
    row: usize,
    coeff: f64,
    leaving_new_val: f64,
}

/// Stuff related to inversion of the basis matrix
#[derive(Clone)]
struct BasisSolver {
    current_basis: crate::repair::Basis,
    lu_factors: LUFactors,
    lu_factors_transp: LUFactors,
    scratch: ScratchSpace,
    eta_matrices: EtaMatrices,
    rhs: ScatteredVec,
}

impl BasisSolver {
    fn push_eta_matrix(&mut self, col_coeffs: &SparseVec, r_leaving: usize, pivot_coeff: f64) {
        let coeffs = col_coeffs.iter().map(|(r, &coeff)| {
            let val = if r == r_leaving {
                1.0 - 1.0 / pivot_coeff
            } else {
                coeff / pivot_coeff
            };
            (r, val)
        });
        self.eta_matrices.push(r_leaving, coeffs);
    }

    fn reset(&mut self, orig_constraints_csc: &CsMat, basic_vars: &[usize]) -> Result<(), Error> {
        numerical_progress("factor-begin", basic_vars, 0);
        // No old factor, eta chain or scratch state changes on a failed refactor.
        let n = basic_vars.len();
        let current_basis = crate::repair::Basis::new(n, |c| {
            orig_constraints_csc
                .outer_view(basic_vars[c])
                .unwrap()
                .into_raw_storage()
        })?;
        let mut scratch = ScratchSpace::with_capacity(n);
        let factors = lu_factorize(
            n,
            |c| {
                orig_constraints_csc
                    .outer_view(basic_vars[c])
                    .unwrap()
                    .into_raw_storage()
            },
            LU_STABILITY_THRESHOLD,
            &mut scratch,
        )?;
        self.lu_factors_transp = factors.transpose();
        self.lu_factors = factors;
        self.current_basis = current_basis;
        self.scratch = scratch;
        self.eta_matrices.clear_and_resize(n);
        self.rhs.clear_and_resize(n);
        numerical_progress("factor-commit", basic_vars, 0);
        Ok(())
    }

    fn solve<'a>(
        &mut self,
        rhs: impl Iterator<Item = (usize, &'a f64)>,
    ) -> Result<&ScatteredVec, Error> {
        self.rhs.set(rhs);
        let original_rhs = self.rhs.values.clone();
        self.lu_factors.solve(&mut self.rhs, &mut self.scratch);

        // apply eta matrices (Vanderbei p.139)
        for idx in 0..self.eta_matrices.len() {
            let r_leaving = self.eta_matrices.leaving_rows[idx];
            let coeff = *self.rhs.get(r_leaving);
            for (r, &val) in self.eta_matrices.coeff_cols.col_iter(idx) {
                *self.rhs.get_mut(r) -= coeff * val;
            }
        }

        self.current_basis
            .check(&original_rhs, &self.rhs.values, false)?;
        Ok(&self.rhs)
    }

    /// Dense counterpart of [`Self::solve`]: LU solve plus the forward eta
    /// application, so callers with dense right-hand sides (the recalcs) no
    /// longer need a full refactorization just because etas are pending.
    fn solve_dense_with_etas(&mut self, rhs: &mut [f64]) -> Result<(), Error> {
        let original_rhs = rhs.to_vec();
        self.lu_factors.solve_dense(rhs, &mut self.scratch);
        for idx in 0..self.eta_matrices.len() {
            let coeff = rhs[self.eta_matrices.leaving_rows[idx]];
            if coeff != 0.0 {
                for (r, &val) in self.eta_matrices.coeff_cols.col_iter(idx) {
                    rhs[r] -= coeff * val;
                }
            }
        }
        self.current_basis.check(&original_rhs, rhs, false)?;
        Ok(())
    }

    /// Dense counterpart of [`Self::solve_transp`]: the reverse eta
    /// application, then the transposed LU solve.
    fn solve_transp_dense_with_etas(&mut self, rhs: &mut [f64]) -> Result<(), Error> {
        let original_rhs = rhs.to_vec();
        for idx in (0..self.eta_matrices.len()).rev() {
            let mut coeff = 0.0;
            for (i, &val) in self.eta_matrices.coeff_cols.col_iter(idx) {
                coeff += val * rhs[i];
            }
            rhs[self.eta_matrices.leaving_rows[idx]] -= coeff;
        }
        self.lu_factors_transp.solve_dense(rhs, &mut self.scratch);
        self.current_basis.check(&original_rhs, rhs, true)?;
        Ok(())
    }

    /// Pass right-hand side via self.rhs
    fn solve_transp<'a>(
        &mut self,
        rhs: impl Iterator<Item = (usize, &'a f64)>,
    ) -> Result<&ScatteredVec, Error> {
        self.rhs.set(rhs);
        let original_rhs = self.rhs.values.clone();
        // apply eta matrices in reverse (Vanderbei p.139)
        for idx in (0..self.eta_matrices.len()).rev() {
            let mut coeff = 0.0;
            // eta col `dot` rhs_transp
            for (i, &val) in self.eta_matrices.coeff_cols.col_iter(idx) {
                coeff += val * self.rhs.get(i);
            }
            let r_leaving = self.eta_matrices.leaving_rows[idx];
            *self.rhs.get_mut(r_leaving) -= coeff;
        }

        self.lu_factors_transp
            .solve(&mut self.rhs, &mut self.scratch);
        self.current_basis
            .check(&original_rhs, &self.rhs.values, true)?;
        Ok(&self.rhs)
    }
}

#[derive(Clone, Debug)]
struct EtaMatrices {
    leaving_rows: Vec<usize>,
    coeff_cols: SparseMat,
}

impl EtaMatrices {
    fn new(n_rows: usize) -> EtaMatrices {
        EtaMatrices {
            leaving_rows: vec![],
            coeff_cols: SparseMat::new(n_rows),
        }
    }

    fn len(&self) -> usize {
        self.leaving_rows.len()
    }

    fn clear_and_resize(&mut self, n_rows: usize) {
        self.leaving_rows.clear();
        self.coeff_cols.clear_and_resize(n_rows);
    }

    fn push(&mut self, leaving_row: usize, coeffs: impl Iterator<Item = (usize, f64)>) {
        self.leaving_rows.push(leaving_row);
        self.coeff_cols.append_col(coeffs);
    }
}

fn into_resized(vec: CsVec, len: usize) -> CsVec {
    let (mut indices, mut data) = vec.into_raw_storage();

    while let Some(&i) = indices.last() {
        if i < len {
            // TODO: binary search
            break;
        }

        indices.pop();
        data.pop();
    }

    CsVec::new(len, indices, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::{assert_matrix_eq, to_sparse};
    use crate::{OptimizationDirection, Problem};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn initialize() {
        init();
        let sol = Solver::try_new(
            &[2.0, 1.0],
            &[f64::NEG_INFINITY, 5.0],
            &[0.0, f64::INFINITY],
            &[
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Le, 6.0),
                (to_sparse(&[1.0, 2.0]), ComparisonOp::Le, 8.0),
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Ge, 2.0),
                (to_sparse(&[0.0, 1.0]), ComparisonOp::Eq, 3.0),
            ],
            &[VarDomain::Real, VarDomain::Real],
            Default::default(),
        )
        .unwrap();

        assert_eq!(sol.num_vars, 2);
        assert!(!sol.is_primal_feasible);
        assert!(!sol.is_dual_feasible);

        assert_eq!(&sol.orig_obj_coeffs, &[2.0, 1.0, 0.0, 0.0, 0.0, 0.0]);

        assert_eq!(
            &sol.orig_var_mins,
            &[f64::NEG_INFINITY, 5.0, 0.0, 0.0, f64::NEG_INFINITY, 0.0,]
        );
        assert_eq!(
            &sol.orig_var_maxs,
            &[0.0, f64::INFINITY, f64::INFINITY, f64::INFINITY, 0.0, 0.0]
        );

        // Equilibration scales the second constraint (max structural
        // coefficient 2) and its rhs by 1/2; the slack column stays 1. The
        // unit-coefficient rows are unchanged.
        let orig_constraints_ref = vec![
            vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
            vec![0.5, 1.0, 0.0, 1.0, 0.0, 0.0],
            vec![1.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        ];
        assert_matrix_eq(&sol.orig_constraints, &orig_constraints_ref);

        assert_eq!(&sol.orig_rhs, &[6.0, 4.0, 2.0, 3.0]);

        assert_eq!(&sol.basic_vars, &[2, 3, 4, 5]);
        assert_eq!(&sol.basic_var_vals, &[1.0, -1.0, -3.0, -2.0]);
        assert_eq!(&sol.dual_edge_sq_norms, &[1.0, 1.0, 1.0, 1.0]);

        assert_eq!(&sol.nb_vars, &[0, 1]);
        assert_eq!(&sol.nb_var_obj_coeffs, &[-1.0, 1.0]);
        assert_eq!(&sol.nb_var_vals, &[0.0, 5.0]);
        assert_eq!(&sol.primal_edge_sq_norms, &[3.25, 5.0]);

        assert_eq!(sol.cur_obj_val, 0.0);
    }

    #[test]
    fn try_new_rejects_nan_bound() {
        init();
        // A NaN bound used to slip past the `min > max` guard (every
        // comparison against NaN is false, including this one), so it was
        // accepted as an ordinary bound. Downstream, the simplex loop's own
        // bound comparisons against that NaN never resolve either, so the
        // solve hangs forever instead of reporting Infeasible up front (the
        // same thing set_var_bounds already guards against for edits).
        let res = Solver::try_new(
            &[1.0],
            &[f64::NAN],
            &[10.0],
            &[],
            &[VarDomain::Real],
            Default::default(),
        );
        assert_eq!(res.unwrap_err(), Error::Infeasible);
    }

    /// Dense recalculations with pending etas must match a fresh factorization
    /// of the same basis. Values are compared per variable because reloading
    /// may reorder basis positions.
    #[test]
    fn recalcs_with_pending_etas_match_a_fresh_factorization() {
        init();
        // minimize x + y + z, pairwise sums >= 2, boxes [0, 10]: optimum
        // x = y = z = 1. A bound tightening then forces dual pivots, which
        // push etas.
        let mut solver = Solver::try_new(
            &[1.0, 1.0, 1.0],
            &[0.0, 0.0, 0.0],
            &[10.0, 10.0, 10.0],
            &[
                (to_sparse(&[1.0, 1.0, 0.0]), ComparisonOp::Ge, 2.0),
                (to_sparse(&[0.0, 1.0, 1.0]), ComparisonOp::Ge, 2.0),
                (to_sparse(&[1.0, 0.0, 1.0]), ComparisonOp::Ge, 2.0),
            ],
            &[VarDomain::Real, VarDomain::Real, VarDomain::Real],
            None,
        )
        .unwrap();
        assert_eq!(solver.initial_solve().unwrap(), StopReason::Finished);
        solver.set_var_bounds(2, 0.0, 0.25).unwrap();
        assert_eq!(solver.reoptimize().unwrap(), StopReason::Finished);
        assert!(
            solver.basis_solver.eta_matrices.len() > 0,
            "fixture must leave etas pending to exercise eta-aware recalculation"
        );

        // Recalculate through the eta-aware dense solves.
        solver.recalc_basic_var_vals().unwrap();
        solver.recalc_obj_coeffs().unwrap();
        let by_var = |s: &Solver| -> Vec<(usize, f64)> {
            let mut v: Vec<(usize, f64)> = s
                .basic_vars
                .iter()
                .zip(&s.basic_var_vals)
                .map(|(&var, &val)| (var, val))
                .collect();
            v.sort_by_key(|&(var, _)| var);
            v
        };
        let rc_by_var = |s: &Solver| -> Vec<(usize, f64)> {
            let mut v: Vec<(usize, f64)> = s
                .nb_vars
                .iter()
                .zip(&s.nb_var_obj_coeffs)
                .map(|(&var, &rc)| (var, rc))
                .collect();
            v.sort_by_key(|&(var, _)| var);
            v
        };
        let eta_vals = by_var(&solver);
        let eta_rcs = rc_by_var(&solver);
        let eta_obj = solver.cur_obj_val;

        // Reloading the solver's own snapshot refactorizes from scratch and
        // reruns the recalcs eta-free — the ground truth.
        let basis = solver.snapshot_basis();
        solver.load_basis(&basis).unwrap();
        assert_eq!(solver.basis_solver.eta_matrices.len(), 0);
        for ((va, a), (vb, b)) in eta_vals.iter().zip(by_var(&solver).iter()) {
            assert_eq!(va, vb);
            assert!((a - b).abs() < 1e-9, "basic val of var {va}: {a} vs {b}");
        }
        for ((va, a), (vb, b)) in eta_rcs.iter().zip(rc_by_var(&solver).iter()) {
            assert_eq!(va, vb);
            assert!((a - b).abs() < 1e-9, "reduced cost of var {va}: {a} vs {b}");
        }
        assert!((eta_obj - solver.cur_obj_val).abs() < 1e-9);
    }

    #[test]
    fn solve_integer_singular_var() {
        init();
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        let x = problem.add_integer_var(1.0, (0, 10));
        problem.add_constraint([(x, 30.0)], ComparisonOp::Ge, 90.0);
        assert!(
            (problem
                .solve()
                .unwrap()
                .into_solution()
                .unwrap()
                .objective()
                - 3.0)
                .abs()
                < EPS
        );

        let mut problem = Problem::new(OptimizationDirection::Minimize);
        let x = problem.add_integer_var(1.0, (0, 10));
        problem.add_constraint([(x, 30.0)], ComparisonOp::Ge, 91.0);
        assert!(
            (problem
                .solve()
                .unwrap()
                .into_solution()
                .unwrap()
                .objective()
                - 4.0)
                .abs()
                < EPS
        );

        let mut problem = Problem::new(OptimizationDirection::Maximize);
        let x = problem.add_integer_var(1.0, (0, 10));
        problem.add_constraint([(x, 30.0)], ComparisonOp::Le, 90.0);
        assert!(
            (problem
                .solve()
                .unwrap()
                .into_solution()
                .unwrap()
                .objective()
                - 3.0)
                .abs()
                < EPS
        );

        let mut problem = Problem::new(OptimizationDirection::Maximize);
        let x = problem.add_integer_var(1.0, (0, 10));
        problem.add_constraint([(x, 30.0)], ComparisonOp::Le, 91.0);
        assert!(
            (problem
                .solve()
                .unwrap()
                .into_solution()
                .unwrap()
                .objective()
                - 3.0)
                .abs()
                < EPS
        );
    }

    #[test]
    fn solve_powers_integer() {
        init();
        let n = 15626;
        // return (a,b,c) such that 2^a * 3^b * 5^c >= n and is minimized given a,b,c € N
        let logn = (n as f64).log2();
        let log2 = 2_f64.log2();
        let log3 = 3_f64.log2();
        let log5 = 5_f64.log2();
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        let p2 = problem.add_integer_var(log2, (0, 100));
        let p3 = problem.add_integer_var(log3, (0, 100));
        let p5 = problem.add_integer_var(log5, (0, 100));
        problem.add_constraint(
            &[(p2, log2), (p3, log3), (p5, log5)],
            ComparisonOp::Ge,
            logn,
        );
        let sol = problem.solve().unwrap().into_solution().unwrap();
        assert_eq!(sol.objective().round() as i64, 14);
    }

    #[test]
    fn initial_solve() {
        init();
        let mut sol = Solver::try_new(
            &[-3.0, -4.0],
            &[f64::NEG_INFINITY, 5.0],
            &[20.0, f64::INFINITY],
            &[
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Le, 20.0),
                (to_sparse(&[-1.0, 4.0]), ComparisonOp::Le, 20.0),
            ],
            &[VarDomain::Real, VarDomain::Real],
            Default::default(),
        )
        .unwrap();
        sol.initial_solve().unwrap();

        assert!(sol.is_primal_feasible);
        assert!(sol.is_dual_feasible);

        assert_eq!(&sol.basic_vars, &[0, 1]);
        assert_eq!(&sol.basic_var_vals, &[12.0, 8.0]);
        assert_eq!(&sol.nb_vars, &[2, 3]);
        assert_eq!(&sol.nb_var_vals, &[0.0, 0.0]);
        // The optimum (x=12, y=8, obj -68) is unchanged by equilibration; only
        // the second constraint's slack reduced cost is scaled: that row
        // (-x+4y<=20, max coeff 4) is equilibrated by 1/4, so its dual scales
        // up 4x, 0.2 -> 0.8.
        assert_eq!(&sol.nb_var_obj_coeffs, &[3.2, 0.8]);
        assert_eq!(sol.cur_obj_val, -68.0);

        let infeasible = Solver::try_new(
            &[1.0, 1.0],
            &[0.0, 0.0],
            &[f64::INFINITY, f64::INFINITY],
            &[
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Ge, 10.0),
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Le, 5.0),
            ],
            &[VarDomain::Real, VarDomain::Real],
            Default::default(),
        )
        .unwrap()
        .initial_solve();
        assert_eq!(infeasible.unwrap_err(), Error::Infeasible);
    }

    #[test]
    fn set_var_bounds_tighten_matches_fresh_solve() {
        init();
        // minimize 2x + 3y s.t. x + y >= 4, 0 <= x,y <= 10. Optimum: x=4, y=0, obj 8.
        let coeffs = [2.0, 3.0];
        let mins = [0.0, 0.0];
        let maxs = [10.0, 10.0];
        let cons = [(to_sparse(&[1.0, 1.0]), ComparisonOp::Ge, 4.0)];
        let domains = [VarDomain::Real, VarDomain::Real];

        let mut warm = Solver::try_new(&coeffs, &mins, &maxs, &cons, &domains, None).unwrap();
        warm.initial_solve().unwrap();
        assert!(float_eq(warm.cur_obj_val, 8.0));

        // Tighten x to [0, 2] and re-solve warm: optimum becomes x=2, y=2, obj 10.
        warm.set_var_bounds(0, 0.0, 2.0).unwrap();
        assert_eq!(warm.reoptimize().unwrap(), StopReason::Finished);
        assert!(warm.is_primal_feasible && warm.is_dual_feasible);
        assert!(float_eq(warm.cur_obj_val, 10.0));
        assert!(float_eq(*warm.get_value(0), 2.0));
        assert!(float_eq(*warm.get_value(1), 2.0));

        // Fresh solve of the tightened problem must agree.
        let mut fresh =
            Solver::try_new(&coeffs, &mins, &[2.0, 10.0], &cons, &domains, None).unwrap();
        fresh.initial_solve().unwrap();
        assert!(float_eq(fresh.cur_obj_val, warm.cur_obj_val));
    }

    #[test]
    fn set_var_bounds_loosen_and_retighten() {
        init();
        // maximize x + y (internally minimize -x - y) s.t. x + y <= 4, 0 <= x,y <= 3.
        let mut solver = Solver::try_new(
            &[-1.0, -1.0],
            &[0.0, 0.0],
            &[3.0, 3.0],
            &[(to_sparse(&[1.0, 1.0]), ComparisonOp::Le, 4.0)],
            &[VarDomain::Real, VarDomain::Real],
            None,
        )
        .unwrap();
        solver.initial_solve().unwrap();
        assert!(float_eq(solver.cur_obj_val, -4.0));

        // Tighten x to [0, 0.5]: optimum x=0.5, y=3, obj -3.5.
        solver.set_var_bounds(0, 0.0, 0.5).unwrap();
        assert_eq!(solver.reoptimize().unwrap(), StopReason::Finished);
        assert!(float_eq(solver.cur_obj_val, -3.5));

        // Loosen x back to [0, 3]: optimum returns to -4.
        solver.set_var_bounds(0, 0.0, 3.0).unwrap();
        assert_eq!(solver.reoptimize().unwrap(), StopReason::Finished);
        assert!(float_eq(solver.cur_obj_val, -4.0));

        assert!(solver.lp_iterations > 0);
    }

    #[test]
    fn set_var_bounds_crossing_is_infeasible_and_leaves_state_untouched() {
        init();
        let mut solver = Solver::try_new(
            &[1.0],
            &[0.0],
            &[10.0],
            &[(to_sparse(&[1.0]), ComparisonOp::Ge, 1.0)],
            &[VarDomain::Real],
            None,
        )
        .unwrap();
        solver.initial_solve().unwrap();
        let obj_before = solver.cur_obj_val;
        assert_eq!(
            solver.set_var_bounds(0, 2.0, 1.0).unwrap_err(),
            Error::Infeasible
        );
        assert_eq!(solver.get_var_bounds(0), (0.0, 10.0)); // untouched
        assert!(float_eq(solver.cur_obj_val, obj_before));
    }

    #[test]
    fn set_var_bounds_nan_is_infeasible_and_leaves_state_untouched() {
        let mut original =
            Solver::try_new(&[1.0], &[0.0], &[10.0], &[], &[VarDomain::Real], None).unwrap();
        assert_eq!(original.initial_solve().unwrap(), StopReason::Finished);

        for (min, max) in [(f64::NAN, 10.0), (0.0, f64::NAN)] {
            let mut solver = original.clone();
            let bounds_before = solver.get_var_bounds(0);
            let value_before = *solver.get_value(0);
            let objective_before = solver.cur_obj_val;
            let primal_before = solver.is_primal_feasible;
            let dual_before = solver.is_dual_feasible;

            assert_eq!(solver.set_var_bounds(0, min, max), Err(Error::Infeasible));
            assert_eq!(solver.get_var_bounds(0), bounds_before);
            assert_eq!(*solver.get_value(0), value_before);
            assert_eq!(solver.cur_obj_val, objective_before);
            assert_eq!(solver.is_primal_feasible, primal_before);
            assert_eq!(solver.is_dual_feasible, dual_before);
        }

        let mut solver = original;
        assert_eq!(
            solver.set_var_bounds(0, f64::NEG_INFINITY, f64::INFINITY),
            Ok(())
        );
        assert_eq!(solver.get_var_bounds(0), (f64::NEG_INFINITY, f64::INFINITY));
    }

    #[test]
    fn check_constraints_rejects_non_finite_activity() {
        let solver = Solver::try_new(
            &[0.0],
            &[0.0],
            &[f64::INFINITY],
            &[(to_sparse(&[1.0e308]), ComparisonOp::Eq, f64::INFINITY)],
            &[VarDomain::Real],
            None,
        )
        .unwrap();

        assert!(!solver.check_constraints(&[1.0e308], 1.0e-7));
    }

    #[test]
    fn basis_snapshot_load_roundtrip() {
        init();
        // This bounded fixture has objective -68 at (12, 8), with both
        // structural variables basic and both slacks non-basic. Its basis is
        // therefore non-trivial and differs from the slack basis.
        let mut solver = Solver::try_new(
            &[-3.0, -4.0],
            &[f64::NEG_INFINITY, 5.0],
            &[20.0, f64::INFINITY],
            &[
                (to_sparse(&[1.0, 1.0]), ComparisonOp::Le, 20.0),
                (to_sparse(&[-1.0, 4.0]), ComparisonOp::Le, 20.0),
            ],
            &[VarDomain::Real, VarDomain::Real],
            None,
        )
        .unwrap();
        solver.initial_solve().unwrap();
        let obj = solver.cur_obj_val;
        let vals: Vec<f64> = (0..2).map(|v| *solver.get_value(v)).collect();
        let basis = solver.snapshot_basis();

        // Wreck the state by loading the all-slack basis…
        let slack = solver.slack_basis();
        solver.load_basis(&slack).unwrap();

        // …then reload the optimal basis: objective and values must round-trip.
        solver.load_basis(&basis).unwrap();
        assert!(solver.is_primal_feasible && solver.is_dual_feasible);
        assert!(float_eq(solver.cur_obj_val, obj));
        for v in 0..2 {
            assert!(float_eq(*solver.get_value(v), vals[v]));
        }
    }

    #[test]
    fn slack_basis_load_then_reoptimize_reaches_optimum() {
        init();
        // minimize 2x + 3y s.t. x + y >= 4, 0 <= x,y <= 10 → obj 8.
        let mut solver = Solver::try_new(
            &[2.0, 3.0],
            &[0.0, 0.0],
            &[10.0, 10.0],
            &[(to_sparse(&[1.0, 1.0]), ComparisonOp::Ge, 4.0)],
            &[VarDomain::Real, VarDomain::Real],
            None,
        )
        .unwrap();
        solver.initial_solve().unwrap();
        assert!(float_eq(solver.cur_obj_val, 8.0));

        let slack = solver.slack_basis();
        solver.load_basis(&slack).unwrap();
        assert_eq!(solver.reoptimize().unwrap(), StopReason::Finished);
        assert!(float_eq(solver.cur_obj_val, 8.0));
    }

    #[test]
    fn load_basis_rejects_wrong_shape() {
        init();
        let mut solver = Solver::try_new(
            &[1.0],
            &[0.0],
            &[1.0],
            &[(to_sparse(&[1.0]), ComparisonOp::Le, 1.0)],
            &[VarDomain::Real],
            None,
        )
        .unwrap();
        solver.initial_solve().unwrap();
        // 2 total vars (1 structural + 1 slack); a basis with zero Basic entries is invalid.
        let bad = Basis(vec![VarStatus::AtLower, VarStatus::AtLower]);
        assert!(solver.load_basis(&bad).is_err());
        // Solver must still be usable via the slack-basis fallback path.
        let slack = solver.slack_basis();
        solver.load_basis(&slack).unwrap();
        assert_eq!(solver.reoptimize().unwrap(), StopReason::Finished);
    }
}

// These fixtures are available only to the dedicated source-owned numerical
// test harness. They exercise the production mutation and checked-solve paths.
#[cfg(feature = "numerical-fixtures")]
pub(crate) fn d22_fixture(kind: &str) -> Result<(), Error> {
    fn require(ok: bool, message: &str) -> Result<(), Error> {
        if ok {
            Ok(())
        } else {
            Err(Error::InternalError(message.into()))
        }
    }
    let mut solver = Solver::try_new(
        &[-1.0],
        &[0.0],
        &[10.0],
        &[
            (CsVec::new(1, vec![0], vec![1.0]), ComparisonOp::Le, 1.0),
            (CsVec::new(1, vec![0], vec![1.0]), ComparisonOp::Le, 2.0),
        ],
        &[VarDomain::Real],
        None,
    )?;
    match kind {
        "failed_reset_is_transactional" => {
            let before = solver.basis_solver.clone();
            let singular = CsMat::new_csc((2, 2), vec![0, 1, 2], vec![0, 0], vec![1.0, 1.0]);
            require(
                solver.basis_solver.reset(&singular, &[0, 1]).is_err(),
                "singular reset accepted",
            )?;
            require(
                format!("{:?}", before.lu_factors)
                    == format!("{:?}", solver.basis_solver.lu_factors),
                "factor changed after rejected reset",
            )?;
            require(
                before.current_basis.columns == solver.basis_solver.current_basis.columns,
                "original basis changed after rejected reset",
            )?;
            require(
                before.eta_matrices.len() == solver.basis_solver.eta_matrices.len(),
                "etas changed after rejected reset",
            )?;
            let b = [1.0, 2.0];
            let mut x = b;
            solver.basis_solver.solve_dense_with_etas(&mut x)?;
            require(x == b, "old identity solve lost after rejected reset")
        }
        "corrupt_eta_refreshes_and_reselects" => {
            crate::repair::reset_stats();
            solver
                .basis_solver
                .eta_matrices
                .push(0, [(0, 0.5)].into_iter());
            require(
                solver.initial_solve()? == StopReason::Finished,
                "fixture did not finish",
            )?;
            require(
                crate::repair::stats().old_basis_refreshes > 0,
                "bad eta did not trigger original-basis refresh",
            )?;
            require(
                (*solver.get_value(0) - 1.0).abs() < 1e-12,
                "refreshed selection chose wrong solution",
            )
        }
        "failed_pivot_is_transactional" => {
            let (row, value) = solver
                .choose_pivot_row_dual()
                .ok_or_else(|| Error::InternalError("fixture has no infeasible row".into()))?;
            solver.calc_row_coeffs(row)?;
            let info = solver.choose_entering_col_dual(row, value)?;
            solver.calc_col_coeffs(info.col)?;
            // Failure occurs during candidate recalculation, after inner mutation.
            solver.working_obj_coeffs.fill(f64::NAN);
            let before = format!("{:?}", solver);
            let basis_before = solver.basis_solver.current_basis.columns.clone();
            let eta_before = solver.basis_solver.eta_matrices.len();
            require(
                solver.pivot(&info, NumericalPhase::Dual).is_err(),
                "nonfinite candidate accepted",
            )?;
            require(
                before == format!("{:?}", solver),
                "failed pivot changed solver state",
            )?;
            require(
                basis_before == solver.basis_solver.current_basis.columns
                    && eta_before == solver.basis_solver.eta_matrices.len(),
                "failed pivot changed numerical basis",
            )
        }
        "disagreeing_pivot_rebuilds_original_basis" | "disagreeing_pivot_rollback" => {
            let (row, value) = solver
                .choose_pivot_row_dual()
                .ok_or_else(|| Error::InternalError("fixture has no infeasible row".into()))?;
            solver.calc_row_coeffs(row)?;
            let info = solver.choose_entering_col_dual(row, value)?;
            solver.calc_col_coeffs(info.col)?;
            // The candidate exchange remains meaningful, but this coefficient
            // must never enter an eta or reduced-cost update.
            *solver.row_coeffs.get_mut(info.col) += 1e-6;
            let rollback = kind == "disagreeing_pivot_rollback";
            if rollback {
                solver.working_obj_coeffs.fill(f64::NAN);
            }
            let before = format!("{:?}", solver);
            let working = solver.working_obj_coeffs.clone();
            let factors = crate::repair::stats().fresh_factors;
            let result = solver.pivot(&info, NumericalPhase::Dual);
            require(
                crate::repair::stats().fresh_factors > factors,
                "ambiguous pivot did not rebuild original columns",
            )?;
            if rollback {
                require(result.is_err(), "nonfinite rebuilt objective admitted")?;
                require(
                    before == format!("{:?}", solver),
                    "rejected rebuilt pivot changed old state",
                )
            } else {
                result?;
                require(
                    solver.basis_solver.eta_matrices.len() == 0,
                    "ambiguous pivot retained eta arithmetic",
                )?;
                require(
                    solver.working_obj_coeffs == working,
                    "rebuilt pivot changed phase objective",
                )?;
                require(
                    solver.calc_dual_infeasibility().0 == 0,
                    "rebuilt pivot lost required dual feasibility",
                )?;
                require(
                    (*solver.get_value(0) - 1.0).abs() < 1e-12,
                    "rebuilt pivot did not solve original constraint",
                )
            }
        }
        "refresh_rejects_lost_primal" | "refresh_rejects_lost_working_dual" => {
            let phase = if kind == "refresh_rejects_lost_primal" {
                solver.basic_var_vals.fill(0.0);
                solver.is_primal_feasible = true;
                NumericalPhase::Primal
            } else {
                solver.working_obj_coeffs[0] = 1.0;
                NumericalPhase::Dual
            };
            let before = format!("{:?}", solver);
            require(
                solver.refresh_numerics(phase).is_err(),
                "refresh resumed with lost phase invariant",
            )?;
            require(
                before == format!("{:?}", solver),
                "phase-invalid refresh changed old state",
            )
        }
        "fix_variable_ambiguous_exchange" => {
            require(
                solver.initial_solve()? == StopReason::Finished,
                "fix-variable fixture did not optimize",
            )?;
            let row = match solver.var_states[0] {
                VarState::Basic(row) => row,
                _ => return Err(Error::InternalError("fixture variable is not basic".into())),
            };
            solver.calc_row_coeffs(row)?;
            let info = solver.choose_entering_col_dual(row, 0.5)?;
            solver.calc_col_coeffs(info.col)?;
            *solver.row_coeffs.get_mut(info.col) += 1e-6;
            let old_objective = solver.cur_obj_val;
            solver.pivot(&info, NumericalPhase::FixVariable)?;
            require(
                solver.cur_obj_val > old_objective,
                "fixture did not exercise permitted fixing cost increase",
            )?;
            require(
                solver.nb_var_states[info.col].at_min
                    && solver.nb_var_states[info.col].at_max
                    && solver.nb_var_is_fixed[info.col],
                "interior fixed value lost fixed nonbasic semantics",
            )?;
            solver.is_primal_feasible = false;
            require(
                solver.restore_feasibility()? == StopReason::Finished,
                "fixed exchange could not restore primal state",
            )?;
            require(
                (*solver.get_value(0) - 0.5).abs() < 1e-12,
                "fixed value was not retained",
            )
        }
        "zero_phase_direct_restore" => {
            let mut case = Solver::try_new(
                &[-0.4 * EPS, 1.5 * EPS, -EPS + 1e-18],
                &[0.0; 3],
                &[1.0; 3],
                &[(
                    CsVec::new(3, vec![0, 1, 2], vec![-0.5, -1.0, 1e-6]),
                    ComparisonOp::Le,
                    -0.5,
                )],
                &vec![VarDomain::Real; 3],
                None,
            )?;
            case.nb_var_vals.fill(0.0);
            case.nb_var_states.fill(NonBasicVarState {
                at_min: true,
                at_max: false,
            });
            case.recalc_basic_var_vals()?;
            case.recalc_working_obj_coeffs()?;
            case.is_primal_feasible = false;
            case.is_dual_feasible = true;
            case.require_numerical_phase(NumericalPhase::Dual)?;
            require(
                case.restore_feasibility()? == StopReason::Finished,
                "direct restore did not finish",
            )?;
            require(
                !case.numerical_feasibility_restart_active
                    && case.working_obj_coeffs == case.orig_obj_coeffs,
                "direct restore left artificial objective active",
            )?;
            case.require_numerical_phase(NumericalPhase::Primal)?;
            case.require_numerical_phase(NumericalPhase::Dual)?;
            let values: Vec<f64> = (0..3).map(|v| *case.get_value(v)).collect();
            require(
                case.check_constraints(&values, EPS)
                    && (case.cur_obj_val - case.objective_of(&values)).abs() < 1e-20,
                "direct restore returned wrong original objective or rows",
            )
        }
        "zero_phase_limit_before_feasibility" => {
            solver.restart_zero_objective_feasibility()?;
            solver.deadline = Some(Instant::now());
            require(
                solver.restore_feasibility()? == StopReason::Limit,
                "deadline did not interrupt zero phase",
            )?;
            require(
                solver.numerical_feasibility_restart_active
                    && solver.working_obj_coeffs.iter().all(|&x| x == 0.0),
                "interruption lost pending original objective",
            )?;
            let mut resumed = solver.clone();
            resumed.deadline = None;
            require(
                resumed.initial_solve()? == StopReason::Finished,
                "zero phase resume failed",
            )?;
            require(
                !resumed.numerical_feasibility_restart_active
                    && resumed.working_obj_coeffs == resumed.orig_obj_coeffs
                    && *resumed.get_value(0) == 1.0
                    && resumed.cur_obj_val == -1.0,
                "resume did not restore original optimum",
            )
        }
        "zero_phase_limit_during_original_optimize" => {
            solver.nb_var_vals.fill(0.0);
            solver.nb_var_states.fill(NonBasicVarState {
                at_min: true,
                at_max: false,
            });
            solver.recalc_basic_var_vals()?;
            solver.restart_zero_objective_feasibility()?;
            solver.deadline = Some(Instant::now());
            require(
                solver.finish_numerical_feasibility_restart()? == StopReason::Limit,
                "deadline did not interrupt restored-original optimization",
            )?;
            require(
                solver.numerical_feasibility_restart_active
                    && solver.working_obj_coeffs == solver.orig_obj_coeffs
                    && !solver.is_dual_feasible,
                "interrupted original optimization lost phase state",
            )?;
            solver.deadline = None;
            require(
                solver.initial_solve()? == StopReason::Finished,
                "original optimize resume failed",
            )?;
            require(
                !solver.numerical_feasibility_restart_active
                    && *solver.get_value(0) == 1.0
                    && solver.cur_obj_val == -1.0,
                "original optimize resume returned zero objective solution",
            )
        }
        "zero_phase_preserves_fixed_interior" => {
            require(
                solver.initial_solve()? == StopReason::Finished,
                "fixed fixture initial solve failed",
            )?;
            require(
                solver.fix_var(0, 0.5)? == StopReason::Finished,
                "fixed fixture fixing failed",
            )?;
            solver.restart_zero_objective_feasibility()?;
            require(
                solver.restore_feasibility()? == StopReason::Finished,
                "fixed zero phase restore failed",
            )?;
            let col = match solver.var_states[0] {
                VarState::NonBasic(col) => col,
                _ => return Err(Error::InternalError("fixed variable became basic".into())),
            };
            require(
                *solver.get_value(0) == 0.5
                    && solver.nb_var_is_fixed[col]
                    && solver.nb_var_states[col].at_min
                    && solver.nb_var_states[col].at_max
                    && !solver.numerical_feasibility_restart_active
                    && solver.working_obj_coeffs == solver.orig_obj_coeffs,
                "zero restart discarded fixed interior value or original objective",
            )
        }
        "step_interval_covers_all_columns" => {
            let mut case = Solver::try_new(
                &[-0.4 * EPS, 1.5 * EPS, -0.3 * EPS, 0.0, 0.0],
                &[0.0, 0.0, 0.0, 0.0, f64::NEG_INFINITY],
                &[1.0, 1.0, 1.0, 1.0, f64::INFINITY],
                &[(
                    CsVec::new(5, vec![0, 1, 2, 3, 4], vec![-0.5, -1.0, -1e-6, 1e-6, 1e-12]),
                    ComparisonOp::Le,
                    -0.5,
                )],
                &vec![VarDomain::Real; 5],
                None,
            )?;
            case.nb_var_vals.fill(0.0);
            case.nb_var_states.fill(NonBasicVarState {
                at_min: true,
                at_max: false,
            });
            case.nb_var_states[4] = NonBasicVarState {
                at_min: false,
                at_max: false,
            };
            case.recalc_basic_var_vals()?;
            case.recalc_working_obj_coeffs()?;
            case.require_numerical_phase(NumericalPhase::Dual)?;
            let (row, value) = case
                .choose_pivot_row_dual()
                .ok_or_else(|| Error::InternalError("missing interval fixture row".into()))?;
            case.calc_row_coeffs(row)?;
            let interval = case.dual_step_interval(row, value)?;
            require(
                interval.0 >= -EPS,
                "leaving reduced cost did not constrain negative step",
            )?;
            let selected = case.choose_entering_col_dual_policy(row, value, true)?;
            require(
                selected.col == 0,
                "small coefficient amplified negative ratio despite leaving bound",
            )?;
            let mut accepted = case.clone();
            accepted.calc_col_coeffs(selected.col)?;
            accepted.pivot(&selected, NumericalPhase::Dual)?;
            accepted.require_numerical_phase(NumericalPhase::Dual)?;
            for (var, cost) in [(3, -EPS + 1e-18), (4, -EPS)] {
                let mut blocked = case.clone();
                blocked.working_obj_coeffs[var] = cost;
                blocked.recalc_working_obj_coeffs()?;
                blocked.require_numerical_phase(NumericalPhase::Dual)?;
                // col3 is opposite-direction/ineligible; col4 is truly free
                // and below EPS pivot eligibility. Both still constrain t.
                require(
                    blocked.dual_step_interval(row, value)?.0 > -0.8 * EPS,
                    "ineligible/free coefficient omitted from step interval",
                )?;
                require(
                    matches!(
                        blocked.choose_entering_col_dual_policy(row, value, true),
                        Err(Error::InternalError(_))
                    ),
                    "no crossing was admitted or misclassified infeasible",
                )?;
            }
            let mut empty = case;
            empty.working_obj_coeffs[0] = -2.0 * EPS;
            empty.recalc_working_obj_coeffs()?;
            require(
                matches!(
                    empty.dual_step_interval(row, value),
                    Err(Error::InternalError(_))
                ),
                "empty interval was admitted or misclassified infeasible",
            )
        }
        "signed_ratio_avoids_double_allowance" => {
            let mut case = Solver::try_new(
                &[-0.4 * EPS, 1.5 * EPS],
                &[0.0, 0.0],
                &[1.0, 1.0],
                &[(
                    CsVec::new(2, vec![0, 1], vec![-0.5, -1.0]),
                    ComparisonOp::Le,
                    -0.5,
                )],
                &[VarDomain::Real, VarDomain::Real],
                None,
            )?;
            // A real feasible-within-EPS dual state with both variables at
            // lower bounds, but primal slack=-0.5 requiring restoration.
            case.nb_var_vals.fill(0.0);
            case.nb_var_states.fill(NonBasicVarState {
                at_min: true,
                at_max: false,
            });
            case.recalc_basic_var_vals()?;
            case.recalc_working_obj_coeffs()?;
            case.require_numerical_phase(NumericalPhase::Dual)?;
            let (row, value) = case
                .choose_pivot_row_dual()
                .ok_or_else(|| Error::InternalError("no fixture leaving row".into()))?;
            case.calc_row_coeffs(row)?;
            let relaxed = case.choose_entering_col_dual(row, value)?;
            require(
                relaxed.col == 1,
                "fixture did not choose larger Harris coefficient",
            )?;
            case.calc_col_coeffs(relaxed.col)?;
            let before = format!("{case:?}");
            require(
                case.pivot(&relaxed, NumericalPhase::Dual).is_err(),
                "double-allowance Harris pivot passed",
            )?;
            require(
                format!("{case:?}") == before,
                "rejected Harris pivot mutated state",
            )?;
            let strict = case.choose_entering_col_dual_policy(row, value, true)?;
            require(
                strict.col == 0,
                "strict policy clamped/absolutized signed ratio",
            )?;
            case.calc_col_coeffs(strict.col)?;
            case.pivot(&strict, NumericalPhase::Dual)?;
            case.require_numerical_phase(NumericalPhase::Dual)?;
            require(
                *case.get_value(0) == 1.0 && *case.get_value(1) == 0.0,
                "strict certified candidate violates known primal solution",
            )
        }
        "compensated_cost_and_refinement" => {
            require(
                compensated_products([(1e16, 1.0), (1.0, 1.0), (-1e16, 1.0)])? == 1.0,
                "compensated cost lost cancellation residual",
            )?;
            require(
                compensated_products([(f64::INFINITY, 1.0)]).is_err(),
                "nonfinite product admitted",
            )?;
            // Nonsymmetric B=[[2,1],[0,3]], B^T*[1,2]=[2,7]. A
            // certified but inaccurate multiplier must use the transpose,
            // and the residual correction must have the right sign.
            let matrix = CsMat::new_csc((2, 2), vec![0, 1, 3], vec![0, 0, 1], vec![2.0, 1.0, 3.0]);
            solver.basis_solver.reset(&matrix, &[0, 1])?;
            let rhs = vec![2.0, 7.0];
            let mut values = vec![1.0 + 1e-14, 2.0 - 1e-14];
            solver.refine_transpose_multipliers(&rhs, &mut values)?;
            require(
                values == vec![1.0, 2.0],
                "transpose refinement did not correct known residual",
            )?;
            let mut nonfinite = vec![f64::NAN, 2.0];
            require(
                solver
                    .refine_transpose_multipliers(&rhs, &mut nonfinite)
                    .is_err(),
                "nonfinite multiplier admitted",
            )
        }
        "regular_pivot_phase_rollback" => {
            let (row, value) = solver
                .choose_pivot_row_dual()
                .ok_or_else(|| Error::InternalError("fixture has no infeasible row".into()))?;
            solver.calc_row_coeffs(row)?;
            let info = solver.choose_entering_col_dual(row, value)?;
            solver.calc_col_coeffs(info.col)?;
            // Selection still uses the original valid costs; alter only the
            // actual working objective to make the recomputed candidate invalid.
            solver.working_obj_coeffs[0] = 1.0;
            let before = format!("{solver:?}");
            let error = solver
                .pivot(&info, NumericalPhase::Dual)
                .err()
                .ok_or_else(|| {
                    Error::InternalError("regular pivot lost phase but committed".into())
                })?;
            require(
                error.to_string().contains("required Dual phase"),
                "wrong rejection boundary",
            )?;
            require(
                format!("{solver:?}") == before,
                "regular phase rejection mutated solver",
            )
        }
        "refresh_preserves_artificial_objective" => {
            let mut artificial = Solver::try_new(
                &[1.0],
                &[f64::NEG_INFINITY],
                &[f64::INFINITY],
                &[(CsVec::new(1, vec![0], vec![1.0]), ComparisonOp::Ge, 1.0)],
                &[VarDomain::Real],
                None,
            )?;
            let working = artificial.working_obj_coeffs.clone();
            require(
                working != artificial.orig_obj_coeffs,
                "fixture has no artificial objective",
            )?;
            artificial.refresh_numerics(NumericalPhase::Dual)?;
            require(
                artificial.working_obj_coeffs == working,
                "refresh changed phase objective",
            )?;
            for cost in [-1.0, 1.0] {
                let mut invalid = artificial.clone();
                invalid.working_obj_coeffs[0] = cost;
                let before = format!("{invalid:?}");
                require(
                    invalid.refresh_numerics(NumericalPhase::Dual).is_err(),
                    "free nonbasic nonzero reduced cost was admitted",
                )?;
                require(
                    format!("{invalid:?}") == before,
                    "rejected free-variable refresh mutated solver",
                )?;
            }
            Ok(())
        }
        _ => Err(Error::InternalError("unknown numerical fixture".into())),
    }
}
