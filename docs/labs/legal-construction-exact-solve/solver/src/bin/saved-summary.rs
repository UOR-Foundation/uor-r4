//! `saved-summary <dir>`: load a saved legal-construction case exactly and
//! print a JSON summary. It does not solve anything.
//!
//! `binaries` counts the two 0/1 variables (`l_i`, `r_i`) of each fractional
//! coefficient. An exponent is the 2-adic one: a nonzero dyadic `x` is
//! `odd · 2^e`; zeros are skipped.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use legal_exact_solver::saved;
use num_rational::BigRational;
use num_traits::Zero;
use serde_json::json;

fn exponent(x: &BigRational) -> i64 {
    let tz = |v: &num_bigint::BigInt| v.trailing_zeros().expect("nonzero") as i64;
    if x.denom() == &num_bigint::BigInt::from(1) {
        tz(x.numer())
    } else {
        -tz(x.denom())
    }
}

fn exp_range<'a>(xs: impl Iterator<Item = &'a BigRational>) -> (Option<i64>, Option<i64>) {
    let mut lo = None;
    let mut hi = None;
    for x in xs.filter(|x| !x.is_zero()) {
        let e = exponent(x);
        lo = Some(lo.map_or(e, |l: i64| l.min(e)));
        hi = Some(hi.map_or(e, |h: i64| h.max(e)));
    }
    (lo, hi)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: saved-summary <dir>");
        return ExitCode::from(2);
    }
    let (p, verified) = match saved::load(Path::new(&args[1])) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let coeffs = p.coeffs();
    let fractional = coeffs.iter().filter(|c| c.fractional).count();
    let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
    for c in &coeffs {
        *hist.entry(c.q0).or_default() += 1;
    }
    let zero_guards = p.guards.iter().filter(|row| row.iter().all(Zero::is_zero)).count();
    let (g_lo, g_hi) = exp_range(p.g.iter());
    let (j_lo, j_hi) = exp_range(p.guards.iter().flatten());
    let out = json!({
        "n": p.n(),
        "fractional": fractional,
        "binaries": 2 * fractional,
        "guards": p.guards.len(),
        "zero_guards": zero_guards,
        "q0_histogram": hist,
        "g_exp_min": g_lo,
        "g_exp_max": g_hi,
        "j_exp_min": j_lo,
        "j_exp_max": j_hi,
        "files_verified": verified.len(),
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("serialisable"));
    ExitCode::SUCCESS
}
