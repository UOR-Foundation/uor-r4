//! `search::solve_search_certify` on every control and random LP: the
//! certificate, not the bounded-precision search, decides. At `P = 120`
//! every optimal LP is certified with `blp`'s exact objective; infeasible
//! LPs are errors; at `P = 12` an `Optimal` must still be exact.
//! The generators are copied from `certify_basis.rs`.

use legal_exact_solver::lp::{Lp, LpResult};
use legal_exact_solver::{blp, search};
use legal_exact_solver::{bb, problem::Problem};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

const MAX_PIVOTS: usize = 10_000;

fn q(n: i64, d: i64) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn rv(v: &[(i64, i64)]) -> Vec<BigRational> {
    v.iter().map(|&(n, d)| q(n, d)).collect()
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % (hi - lo + 1) as u64) as i64
    }
}

/// Same generator as `random.rs`.
fn instance(seed: u64) -> Problem {
    let mut rng = Rng(seed);
    let n = rng.range(1, 5) as usize;
    let rows = rng.range(0, 4) as usize;
    let masters: Vec<BigRational> = (0..n)
        .map(|_| {
            if rng.range(0, 1) == 0 {
                q(rng.range(-7, 7), 4)
            } else {
                q(rng.range(-30, 30), 16)
            }
        })
        .collect();
    let g: Vec<BigRational> = (0..n).map(|_| q(rng.range(-8, 8), 4)).collect();
    let probe = Problem::new(masters.clone(), g.clone(), vec![]);
    let planted: Vec<BigRational> = probe
        .coeffs()
        .iter()
        .map(|c| {
            let (lo, hi) = (-7 - c.q0, 7 - c.q0);
            if !c.fractional {
                return q(rng.range(lo, hi), 4);
            }
            match rng.range(0, 2) {
                1 if lo <= -1 => q(rng.range(lo, -1), 4) + &c.shift,
                2 if hi >= 1 => q(rng.range(1, hi), 4) + &c.shift,
                _ => BigRational::zero(),
            }
        })
        .collect();
    let guards = (0..rows)
        .map(|_| {
            let row: Vec<BigRational> = (0..n).map(|_| q(rng.range(-4, 4), 2)).collect();
            let v = row
                .iter()
                .zip(&planted)
                .fold(BigRational::zero(), |s, (a, b)| s + a * b);
            if v.is_negative() {
                row.into_iter().map(|a| -a).collect()
            } else {
                row
            }
        })
        .collect();
    Problem::new(masters, g, guards)
}

/// The instances of `controls.rs`.
fn controls() -> Vec<Problem> {
    let mut forbid = Vec::new();
    for i in 0..3 {
        for s in [1, -1] {
            let mut row = rv(&[(0, 1), (0, 1), (0, 1)]);
            row[i] = q(s, 1);
            forbid.push(row);
        }
    }
    let geq = rv(&[(1, 2), (-3, 4), (1, 1)]);
    vec![
        Problem::new(
            rv(&[(0, 1), (1, 4)]),
            rv(&[(1, 1), (-2, 1)]),
            vec![rv(&[(1, 1), (-1, 1)])],
        ),
        Problem::new(
            rv(&[(3, 8), (0, 1)]),
            rv(&[(1, 1), (1, 1)]),
            vec![rv(&[(1, 1), (2, 1)])],
        ),
        Problem::new(rv(&[(5, 16)]), rv(&[(-1, 1)]), vec![]),
        Problem::new(rv(&[(7, 4), (-7, 4)]), rv(&[(1, 1), (-1, 1)]), vec![]),
        Problem::new(rv(&[(7, 4)]), rv(&[(-1, 1)]), vec![]),
        Problem::new(
            rv(&[(5, 16), (0, 1)]),
            rv(&[(1, 1), (1, 1)]),
            vec![rv(&[(1, 1), (0, 1)]), rv(&[(-1, 1), (0, 1)])],
        ),
        Problem::new(
            rv(&[(0, 1), (0, 1)]),
            rv(&[(1, 1), (-1, 1)]),
            vec![rv(&[(0, 1), (0, 1)]), rv(&[(0, 1), (0, 1)])],
        ),
        Problem::new(
            rv(&[(3, 8), (1, 4), (-5, 16)]),
            rv(&[(1, 1), (-1, 1), (2, 1)]),
            forbid,
        ),
        Problem::new(rv(&[(3, 8), (0, 1), (-5, 16)]), geq.clone(), vec![geq]),
    ]
}

/// Root relaxation plus the node LPs of `blp_agrees.rs`.
fn lps(p: &Problem) -> Vec<Lp> {
    let root = bb::relaxation(p);
    let mut out = vec![root.clone()];
    for v in 0..p.n() {
        let mut up = root.clone();
        up.lower[v] = q(1, 1).max(up.lower[v].clone());
        out.push(up);
        let mut down = root.clone();
        down.upper[v] = q(-1, 1).min(down.upper[v].clone());
        out.push(down);
    }
    let mut all = root.clone();
    for v in 0..p.n() {
        all.lower[v] = q(1, 1).max(all.lower[v].clone());
    }
    out.push(all);
    out
}

fn suite() -> Vec<(String, Lp)> {
    let mut problems = controls();
    problems.extend((0..240).map(instance));
    let mut out = Vec::new();
    for (i, p) in problems.iter().enumerate() {
        for (k, lp) in lps(p).into_iter().enumerate() {
            out.push((format!("problem {i} lp {k}"), lp));
        }
    }
    out
}

/// (optimal LPs, certified, infeasible LPs). Panics on any wrong optimum.
fn run<const P: u32>() -> (usize, usize, usize) {
    let (mut optimal, mut certified, mut infeasible) = (0, 0, 0);
    for (what, lp) in suite() {
        let got = search::solve_search_certify::<P>(&lp, MAX_PIVOTS);
        match blp::solve(&lp) {
            LpResult::Optimal {
                objective: want, ..
            } => {
                optimal += 1;
                match got {
                    Ok(LpResult::Optimal { objective, .. }) => {
                        assert_eq!(objective, want, "P={P} {what}: wrong optimum");
                        certified += 1;
                    }
                    Ok(other) => panic!("P={P} {what}: unexpected {other:?}"),
                    Err(_) => {}
                }
            }
            LpResult::Infeasible => {
                infeasible += 1;
                assert!(got.is_err(), "P={P} {what}: infeasible LP gave {got:?}");
            }
        }
    }
    eprintln!("P={P}: certified {certified} of {optimal} optimal LPs; {infeasible} infeasible LPs all Err");
    (optimal, certified, infeasible)
}

#[test]
fn p120_certifies_every_optimal_lp_exactly() {
    let (optimal, certified, infeasible) = run::<120>();
    assert!(optimal > 500, "only {optimal} optimal LPs");
    assert!(infeasible > 0, "no infeasible LP in the suite");
    assert_eq!(certified, optimal, "certified {certified} of {optimal}");
}

#[test]
fn infeasible_lp_is_an_error() {
    // x <= -1 with 0 <= x <= 1.
    let lp = Lp {
        c: vec![q(1, 1)],
        rows: vec![(vec![q(1, 1)], q(-1, 1))],
        lower: vec![q(0, 1)],
        upper: vec![q(1, 1)],
    };
    assert!(search::search_basis::<120>(&lp, MAX_PIVOTS).is_none());
    assert!(search::solve_search_certify::<120>(&lp, MAX_PIVOTS).is_err());
}

/// Planted low precision: the search may fail, but never produce a wrong
/// optimum, because the certificate decides.
#[test]
fn p12_never_returns_a_wrong_optimum() {
    let (optimal, _certified, _) = run::<12>();
    assert!(optimal > 500, "only {optimal} optimal LPs");
}
