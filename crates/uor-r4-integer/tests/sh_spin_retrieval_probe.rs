//! Probe: can the spin-1 pair score retrieve a key seen in another frame, where raw-coordinate
//! scorers cannot?
//!
//! The arithmetic is copied from `sh_spin_probe.rs` (integration tests cannot share code): exact
//! `Z[φ]` with `φ² = φ + 1`, the 120 binary-icosahedral quaternions as numerators over 2, the
//! sandwich action `q v q⁻¹`, and the pair score `⟨a_i, q_ij a_j q_ij⁻¹⟩` with `q_ij = q_i⁻¹ q_j`.
//!
//! Each trial plants a query `(a_q, q_q)` and K candidates. The target is the query's feature seen
//! in another frame: pose `q_q g`, feature `D(g)⁻¹ a_q`, so the pair score equals the query's
//! self-pair exactly. Distractors are random small `Z[φ]` features with random poses. In the planted
//! case one decoy copies `a_q` coordinate for coordinate under a pose that is not a frame-consistent
//! copy, so the raw-coordinate scorers rank it first.
//!
//! Scorers, all exact:
//! - `spin`: the pair score made scale-free, `s·|s| / (|a_q|² |a_j|²)` (signed squared cosine; the
//!   key's norm is its own self-pair, so this is still frame invariant); higher wins.
//! - `spin-raw`: the bare pair score `s`; higher wins. Reported, not asserted: a larger key wins.
//! - `l1`: the sum of absolute differences of the raw coordinates, each `a + bφ` encoded as the
//!   integer pair `(a, b)` over a common denominator of 4; lower wins.
//! - `popcount`: the Hamming distance over the six sign bits of those `(a, b)` pairs; lower wins.
//!
//! A tie at the top is never counted as correct. Assertions are on mechanics only.
//!
//! `cargo test -p uor-r4-integer --test sh_spin_retrieval_probe -- --nocapture`

use std::cmp::Ordering;
use std::collections::BTreeSet;

/// `a + b·φ`, with `φ² = φ + 1`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Phi {
    a: i128,
    b: i128,
}

const PHI_ZERO: Phi = Phi { a: 0, b: 0 };
const PHI_ONE: Phi = Phi { a: 1, b: 0 };

fn phi() -> Phi {
    Phi { a: 0, b: 1 }
}

fn int(value: i128) -> Phi {
    Phi { a: value, b: 0 }
}

fn gcd(x: i128, y: i128) -> i128 {
    let (mut x, mut y) = (x.abs(), y.abs());
    while y != 0 {
        let r = x % y;
        x = y;
        y = r;
    }
    x
}

impl Phi {
    fn add(self, other: Self) -> Self {
        Phi {
            a: self.a + other.a,
            b: self.b + other.b,
        }
    }

    fn sub(self, other: Self) -> Self {
        Phi {
            a: self.a - other.a,
            b: self.b - other.b,
        }
    }

    /// `(a + bφ)(c + dφ) = (ac + bd) + (ad + bc + bd)φ`.
    fn mul(self, other: Self) -> Self {
        Phi {
            a: self.a * other.a + self.b * other.b,
            b: self.a * other.b + self.b * other.a + self.b * other.b,
        }
    }

    fn neg(self) -> Self {
        Phi {
            a: -self.a,
            b: -self.b,
        }
    }

    fn is_zero(self) -> bool {
        self.a == 0 && self.b == 0
    }

    fn scale(self, k: i128) -> Self {
        Phi {
            a: self.a * k,
            b: self.b * k,
        }
    }

    /// The exact sign of the real number `a + bφ = (2a + b + b√5) / 2`.
    fn sign(self) -> i128 {
        let s = 2 * self.a + self.b;
        let b = self.b;
        if s == 0 && b == 0 {
            return 0;
        }
        if s >= 0 && b >= 0 {
            return 1;
        }
        if s <= 0 && b <= 0 {
            return -1;
        }
        // Opposite signs: compare s² with 5b², never equal since √5 is irrational.
        let s_wins = s * s > 5 * b * b;
        match (s > 0, s_wins) {
            (true, true) | (false, false) => 1,
            _ => -1,
        }
    }

    fn abs(self) -> Self {
        if self.sign() < 0 {
            self.neg()
        } else {
            self
        }
    }
}

/// An exact rational with a positive integer denominator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rat {
    num: Phi,
    den: i128,
}

impl Rat {
    fn integer(value: i128) -> Self {
        Rat {
            num: int(value),
            den: 1,
        }
    }

    fn reduce(self) -> Self {
        let g = gcd(gcd(self.num.a, self.num.b), self.den);
        if g <= 1 {
            return self;
        }
        Rat {
            num: Phi {
                a: self.num.a / g,
                b: self.num.b / g,
            },
            den: self.den / g,
        }
    }

    /// Cross-multiplied equality, exact for positive denominators.
    fn same(self, other: Self) -> bool {
        self.num.scale(other.den) == other.num.scale(self.den)
    }
}

/// An exact real quotient `num / den` in `Q(φ)`, with `den > 0` as a real number.
#[derive(Clone, Copy, Debug)]
struct Frac {
    num: Phi,
    den: Phi,
}

impl Frac {
    fn new(num: Phi, den: Phi) -> Self {
        assert_eq!(den.sign(), 1, "a Frac needs a positive denominator");
        let g = gcd(gcd(num.a, num.b), gcd(den.a, den.b));
        if g <= 1 {
            return Frac { num, den };
        }
        Frac {
            num: Phi {
                a: num.a / g,
                b: num.b / g,
            },
            den: Phi {
                a: den.a / g,
                b: den.b / g,
            },
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        match self.num.mul(other.den).sub(other.num.mul(self.den)).sign() {
            1 => Ordering::Greater,
            -1 => Ordering::Less,
            _ => Ordering::Equal,
        }
    }

    /// `floor(scale · num / den)`, by bisection on exact signs; `|num / den| ≤ 1` is required.
    fn floor_scaled(&self, scale: i128) -> i128 {
        let fits = |m: i128| self.num.scale(scale).sub(self.den.scale(m)).sign() >= 0;
        let (mut lo, mut hi) = (-scale, scale + 1);
        assert!(fits(lo) && !fits(hi), "value outside [-1, 1]");
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if fits(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }
}

type Quat = [Phi; 4];

fn qmul(a: &Quat, b: &Quat) -> Quat {
    let [aw, ax, ay, az] = *a;
    let [bw, bx, by, bz] = *b;
    [
        aw.mul(bw).sub(ax.mul(bx)).sub(ay.mul(by)).sub(az.mul(bz)),
        aw.mul(bx).add(ax.mul(bw)).add(ay.mul(bz)).sub(az.mul(by)),
        aw.mul(by).sub(ax.mul(bz)).add(ay.mul(bw)).add(az.mul(bx)),
        aw.mul(bz).add(ax.mul(by)).sub(ay.mul(bx)).add(az.mul(bw)),
    ]
}

fn qconj(q: &Quat) -> Quat {
    [q[0], q[1].neg(), q[2].neg(), q[3].neg()]
}

fn qnorm2(q: &Quat) -> Phi {
    q.iter().fold(PHI_ZERO, |acc, c| acc.add(c.mul(*c)))
}

/// A unit quaternion carried as an integer numerator over `den`; `|n|² = den²`.
#[derive(Clone, Copy, Debug)]
struct Unit {
    n: Quat,
    den: i128,
}

impl Unit {
    /// The product, with common factors of two removed so group elements stay over 2.
    fn mul(self, other: Self) -> Self {
        let mut out = Unit {
            n: qmul(&self.n, &other.n),
            den: self.den * other.den,
        };
        while out.den > 1 && out.den % 2 == 0 && out.n.iter().all(|c| c.a % 2 == 0 && c.b % 2 == 0)
        {
            out.n = out.n.map(|c| Phi {
                a: c.a / 2,
                b: c.b / 2,
            });
            out.den /= 2;
        }
        out
    }

    fn conj(self) -> Self {
        Unit {
            n: qconj(&self.n),
            den: self.den,
        }
    }

    /// `±1`, the two elements that act as the identity rotation.
    fn is_central(&self) -> bool {
        self.n[1].is_zero() && self.n[2].is_zero() && self.n[3].is_zero()
    }
}

/// `q · v · q⁻¹` for a unit quaternion `q = n/den` and `v` a spin-1 vector.
fn rotate(q: Unit, v: [Rat; 3]) -> [Rat; 3] {
    assert_eq!(qnorm2(&q.n), int(q.den * q.den));
    let common = v.iter().map(|r| r.den).max().expect("three coordinates");
    let right = qconj(&q.n);
    let mut out = [PHI_ZERO; 3];
    for (coordinate, value) in v.iter().enumerate() {
        let scaled = value.num.scale(common / value.den);
        let mut basis = [PHI_ZERO; 4];
        basis[coordinate + 1] = PHI_ONE;
        let term = qmul(&qmul(&q.n, &basis), &right);
        for (slot, out_slot) in out.iter_mut().enumerate() {
            *out_slot = out_slot.add(term[slot + 1].mul(scaled));
        }
    }
    let den = q.den * q.den * common;
    [
        Rat { num: out[0], den },
        Rat { num: out[1], den },
        Rat { num: out[2], den },
    ]
}

/// The dot product of two vectors whose coordinates each share one denominator.
fn dot(a: [Rat; 3], b: [Rat; 3]) -> Rat {
    assert!(a.iter().all(|r| r.den == a[0].den));
    assert!(b.iter().all(|r| r.den == b[0].den));
    let num = a
        .iter()
        .zip(b.iter())
        .fold(PHI_ZERO, |acc, (x, y)| acc.add(x.num.mul(y.num)));
    Rat {
        num,
        den: a[0].den * b[0].den,
    }
    .reduce()
}

/// The pair score `⟨a_i, q_ij · a_j · q_ij⁻¹⟩`, with `q_ij = q_i⁻¹ q_j`.
fn score(qi: Unit, qj: Unit, ai: [Rat; 3], aj: [Rat; 3]) -> Rat {
    let relative = qi.conj().mul(qj);
    dot(ai, rotate(relative, aj))
}

fn same_vector(x: [Rat; 3], y: [Rat; 3]) -> bool {
    x.iter().zip(y.iter()).all(|(p, q)| p.same(*q))
}

/// The 120 binary-icosahedral quaternions, as integer numerators over 2.
fn group_2i() -> Vec<Unit> {
    let mut set: BTreeSet<[i128; 8]> = BTreeSet::new();
    let mut push = |n: Quat| {
        set.insert([
            n[0].a, n[0].b, n[1].a, n[1].b, n[2].a, n[2].b, n[3].a, n[3].b,
        ]);
    };

    // 8: (±1, 0, 0, 0) and its permutations, as numerators ±2 over 2.
    for position in 0..4 {
        for sign in [-2, 2] {
            let mut n = [PHI_ZERO; 4];
            n[position] = int(sign);
            push(n);
        }
    }

    // 16: (±1, ±1, ±1, ±1) over 2.
    for bits in 0..16u32 {
        let sign = |index: usize| int(if bits & (1 << index) == 0 { 1 } else { -1 });
        push([sign(0), sign(1), sign(2), sign(3)]);
    }

    // 96: the even permutations of (±1, ±φ, ±1/φ, 0) over 2, with the eight sign patterns.
    let inv_phi = Phi { a: -1, b: 1 }; // φ - 1 = 1/φ
    let values = [PHI_ONE, phi(), inv_phi, PHI_ZERO];
    let even = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    for perm in even {
        let base = perm.map(|index| values[index]);
        for bits in 0..8u32 {
            let mut nonzero = 0;
            let mut signed = [PHI_ZERO; 4];
            for (slot, value) in base.iter().enumerate() {
                if !value.is_zero() {
                    let flip = bits & (1 << nonzero) != 0;
                    signed[slot] = if flip { value.neg() } else { *value };
                    nonzero += 1;
                }
            }
            push(signed);
        }
    }

    set.into_iter()
        .map(|k| Unit {
            n: [
                Phi { a: k[0], b: k[1] },
                Phi { a: k[2], b: k[3] },
                Phi { a: k[4], b: k[5] },
                Phi { a: k[6], b: k[7] },
            ],
            den: 2,
        })
        .collect()
}

/// SplitMix64: a small deterministic generator, no external crate.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// A nonzero feature with each coordinate `a + bφ`, `a, b ∈ [-3, 3]`.
    fn feature(&mut self) -> [Rat; 3] {
        loop {
            let mut part = || self.below(7) as i128 - 3;
            let v = [
                Phi {
                    a: part(),
                    b: part(),
                },
                Phi {
                    a: part(),
                    b: part(),
                },
                Phi {
                    a: part(),
                    b: part(),
                },
            ];
            if v.iter().any(|c| !c.is_zero()) {
                return v.map(|num| Rat { num, den: 1 });
            }
        }
    }
}

/// The raw-coordinate encoding: each `a + bφ` as the integer pair `(a, b)` over 4.
fn encode(v: [Rat; 3]) -> [i128; 6] {
    let mut out = [0; 6];
    for (index, r) in v.iter().enumerate() {
        assert_eq!(
            4 % r.den,
            0,
            "raw coordinates live over a denominator dividing 4"
        );
        let k = 4 / r.den;
        out[2 * index] = r.num.a * k;
        out[2 * index + 1] = r.num.b * k;
    }
    out
}

fn l1(x: &[i128; 6], y: &[i128; 6]) -> i128 {
    x.iter().zip(y.iter()).map(|(p, q)| (p - q).abs()).sum()
}

fn popcount(x: &[i128; 6], y: &[i128; 6]) -> i128 {
    x.iter()
        .zip(y.iter())
        .filter(|(p, q)| (**p < 0) != (**q < 0))
        .count() as i128
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    Correct,
    TargetTied,
    Wrong,
}

/// Where the target lands. `better(x, y) == Greater` means `x` ranks above `y`.
fn outcome<T>(values: &[T], target: usize, better: impl Fn(&T, &T) -> Ordering) -> (Outcome, bool) {
    let mut best = 0;
    for index in 1..values.len() {
        if better(&values[index], &values[best]) == Ordering::Greater {
            best = index;
        }
    }
    let at_top = values
        .iter()
        .filter(|v| better(v, &values[best]) == Ordering::Equal)
        .count();
    let result = match better(&values[target], &values[best]) {
        Ordering::Less => Outcome::Wrong,
        _ if at_top > 1 => Outcome::TargetTied,
        _ => Outcome::Correct,
    };
    (result, at_top > 1)
}

#[derive(Default, Clone, Copy)]
struct Tally {
    correct: usize,
    target_tied: usize,
    wrong: usize,
    tie_at_top: usize,
}

impl Tally {
    fn record(&mut self, (result, tie): (Outcome, bool)) {
        match result {
            Outcome::Correct => self.correct += 1,
            Outcome::TargetTied => self.target_tied += 1,
            Outcome::Wrong => self.wrong += 1,
        }
        if tie {
            self.tie_at_top += 1;
        }
    }
}

const SCORERS: [&str; 4] = ["spin", "spin-raw", "l1", "popcount"];
const MILLI: i128 = 1000;

struct Run {
    planted: bool,
    k: usize,
    trials: usize,
    tallies: [Tally; 4],
    /// Spin margin, target minus the best other candidate, in thousandths of signed cos², ceiling.
    margins: Vec<i128>,
}

fn run(planted: bool, k: usize, trials: usize, seed: u64) -> Run {
    let group = group_2i();
    let identity = *group
        .iter()
        .find(|u| u.is_central() && u.n[0].sign() > 0)
        .expect("the identity is in 2I");
    let frames: Vec<Unit> = group.iter().copied().filter(|u| !u.is_central()).collect();
    let mut rng = SplitMix(seed);
    let mut tallies = [Tally::default(); 4];
    let mut margins = Vec::with_capacity(trials);

    for _ in 0..trials {
        let a_q = rng.feature();
        let q_q = group[rng.below(group.len())];
        let g = if planted {
            frames[rng.below(frames.len())]
        } else {
            identity
        };

        // The target: the query's feature seen in the frame `g`.
        let mut poses = vec![q_q.mul(g)];
        let mut features = vec![rotate(g.conj(), a_q)];
        let self_pair = score(q_q, q_q, a_q, a_q);
        let target_pair = score(q_q, poses[0], a_q, features[0]);
        assert!(
            target_pair.same(self_pair),
            "the target's pair score {target_pair:?} differs from the self-pair {self_pair:?}"
        );

        // The decoy: a_q's raw coordinates under a pose that does not carry a_q to itself.
        if planted {
            loop {
                let h = group[rng.below(group.len())];
                if !same_vector(rotate(h, a_q), a_q) {
                    poses.push(q_q.mul(h));
                    features.push(a_q);
                    break;
                }
            }
        }
        while poses.len() < k {
            poses.push(group[rng.below(group.len())]);
            features.push(rng.feature());
        }

        let norm_q = dot(a_q, a_q);
        let raw_q = encode(a_q);
        let mut spin = Vec::with_capacity(k);
        let mut spin_raw = Vec::with_capacity(k);
        let mut l1s = Vec::with_capacity(k);
        let mut pops = Vec::with_capacity(k);
        for (pose, feature) in poses.iter().zip(features.iter()) {
            let s = score(q_q, *pose, a_q, *feature);
            let norm_j = dot(*feature, *feature);
            spin.push(Frac::new(
                s.num.mul(s.num.abs()).scale(norm_q.den * norm_j.den),
                norm_q.num.mul(norm_j.num).scale(s.den * s.den),
            ));
            spin_raw.push(Frac::new(s.num, int(s.den)));
            let raw = encode(*feature);
            l1s.push(l1(&raw_q, &raw));
            pops.push(popcount(&raw_q, &raw));
        }

        let one = Frac::new(PHI_ONE, PHI_ONE);
        assert_eq!(spin[0].cmp(&one), Ordering::Equal, "target spin is not 1");
        if planted {
            assert_eq!(l1s[1], 0, "the decoy copies the raw coordinates");
            assert_eq!(pops[1], 0, "the decoy copies the sign bits");
            assert_eq!(
                spin[1].cmp(&one),
                Ordering::Less,
                "the decoy is not frame consistent"
            );
        }

        tallies[0].record(outcome(&spin, 0, |x, y| x.cmp(y)));
        tallies[1].record(outcome(&spin_raw, 0, |x, y| x.cmp(y)));
        tallies[2].record(outcome(&l1s, 0, |x, y| y.cmp(x)));
        tallies[3].record(outcome(&pops, 0, |x, y| y.cmp(x)));

        let best_other = spin[1..]
            .iter()
            .max_by(|x, y| x.cmp(y))
            .expect("at least one other candidate");
        margins.push(MILLI - best_other.floor_scaled(MILLI));
    }

    margins.sort_unstable();
    for tally in &tallies {
        assert_eq!(tally.correct + tally.target_tied + tally.wrong, trials);
        assert!(tally.tie_at_top >= tally.target_tied);
    }
    Run {
        planted,
        k,
        trials,
        tallies,
        margins,
    }
}

fn table(runs: &[Run]) -> String {
    let mut out = String::from(
        "case     K   N    scorer    correct  target-tied  wrong  tie@top  spin-margin(min/median, 1/1000)\n",
    );
    for r in runs {
        let case = if r.planted { "planted" } else { "null" };
        for (index, name) in SCORERS.iter().enumerate() {
            let t = r.tallies[index];
            let margin = if index == 0 {
                format!("{}/{}", r.margins[0], r.margins[r.margins.len() / 2])
            } else {
                String::from("-")
            };
            out.push_str(&format!(
                "{case:<8} {:<3} {:<4} {name:<9} {:>7}  {:>11}  {:>5}  {:>7}  {margin}\n",
                r.k, r.trials, t.correct, t.target_tied, t.wrong, t.tie_at_top
            ));
        }
    }
    out
}

const TRIALS: usize = 200;

#[test]
fn the_group_has_120_units_and_two_central_elements() {
    let group = group_2i();
    assert_eq!(group.len(), 120);
    for u in &group {
        assert_eq!(qnorm2(&u.n), int(4));
    }
    assert_eq!(group.iter().filter(|u| u.is_central()).count(), 2);
    // The two central elements fix every vector.
    let probe = [Rat::integer(1), Rat::integer(2), Rat::integer(3)];
    for u in group.iter().filter(|u| u.is_central()) {
        assert!(same_vector(rotate(*u, probe), probe));
    }
    // Closure on a sample: a product of two icosians reduces back to a numerator over 2.
    assert_eq!(group[7].mul(group[42]).den, 2);
}

#[test]
fn the_null_control_finds_the_target_for_every_asserted_scorer() {
    let runs = [
        run(false, 8, TRIALS, 396_008),
        run(false, 32, TRIALS, 396_032),
    ];
    println!("{}", table(&runs));
    for r in &runs {
        // spin, l1 and popcount: nothing strictly beats an exact copy. spin-raw is not asserted.
        for index in [0, 2, 3] {
            assert_eq!(
                r.tallies[index].wrong, 0,
                "{} lost the target in the null control at K = {}",
                SCORERS[index], r.k
            );
        }
    }
}

#[test]
fn the_planted_frames_keep_the_self_pair_score_and_the_counts_add_up() {
    let runs = [
        run(true, 8, TRIALS, 396_108),
        run(true, 32, TRIALS, 396_132),
    ];
    println!("{}", table(&runs));
    for r in &runs {
        assert_eq!(r.margins.len(), r.trials);
        for t in &r.tallies {
            assert_eq!(t.correct + t.target_tied + t.wrong, r.trials);
        }
    }
}
