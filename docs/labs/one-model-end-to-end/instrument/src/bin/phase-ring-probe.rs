//! The planted phase-ring probe: does a Z/256 ring address route, and does the exact
//! prime-product (gcd) read route on the same task?
//!
//! Owner's claim (16 Nov): "zeta ring does work, gcd is very fast". The archive's negatives were
//! taken on tests that could not exercise routing (a token-fixed cell; arms that differed by
//! retention rather than by ring adjacency). This probe isolates the ring with the one control that
//! cannot be explained by injectivity: **the same table with every address through a fixed random
//! bijection** — identical occupancy and value distribution, ring adjacency destroyed.
//!
//! Task. K slots, each an address `(u8, u8)` on the 256×256 torus and a value byte. A query steps a
//! stored address by δ on one channel, so it is never seen verbatim. The read returns the value of
//! the nearest stored address by circular distance (ties to the later slot). Arms:
//!   1. `ring`     — the ring address and the circular distance;
//!   2. `bij`      — the control: every byte through a fixed random bijection, same distance;
//!   3. `ring-bij` — the paired difference, the reading that matters.
//! A gcd arm on the same slots uses prime products: an entity is the product of two primes, a query
//! is a product sharing one factor, and the read returns the value with the greatest gcd.
//!
//! `cargo run --release --bin phase-ring-probe` (no arguments; deterministic).

use std::collections::BTreeMap;

/// A deterministic LCG, so runs repeat exactly.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Circular distance on one byte channel.
fn ring_step(a: u8, b: u8) -> u16 {
    let d = (i32::from(a) - i32::from(b)).unsigned_abs() as u16;
    d.min(256 - d)
}

/// Two-channel circular distance.
fn torus(a: (u8, u8), b: (u8, u8)) -> u16 {
    ring_step(a.0, b.0) + ring_step(a.1, b.1)
}

/// The nearest slot among `slots` (ties to the later one), and its distance.
fn nearest(slots: &[(u8, u8)], q: (u8, u8)) -> (usize, u16) {
    let mut best = 0usize;
    let mut best_d = u16::MAX;
    for (i, &a) in slots.iter().enumerate() {
        let d = torus(a, q);
        if d <= best_d {
            best_d = d;
            best = i;
        }
    }
    (best, best_d)
}

/// A fixed random bijection on 0..=255, from the probe's seed.
fn bijection(seed: u64) -> [u8; 256] {
    let mut order: Vec<u8> = (0..=255u8).collect();
    let mut rng = Lcg(seed);
    for i in (1..256).rev() {
        let j = rng.below(i + 1);
        order.swap(i, j);
    }
    let mut map = [0u8; 256];
    for (i, &v) in order.iter().enumerate() {
        map[i] = v;
    }
    map
}

fn primes(n: usize) -> Vec<u64> {
    let mut out = Vec::new();
    let mut candidate = 2u64;
    while out.len() < n {
        if (2..=candidate / 2).all(|d| candidate % d != 0) {
            out.push(candidate);
        }
        candidate += 1;
    }
    out
}

fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// One run of the ring task at K slots and one step δ: ring, bijection, and their paired counts.
fn ring_arm(k: usize, delta: u8, seed: u64) -> (u64, u64, u64, u64) {
    let mut rng = Lcg(seed);
    let map = bijection(seed ^ 0x5eed);
    let mut slots: Vec<(u8, u8)> = Vec::with_capacity(k);
    let mut seen = BTreeMap::new();
    while slots.len() < k {
        let a = (rng.below(256) as u8, rng.below(256) as u8);
        if seen.insert(a, ()).is_none() {
            slots.push(a);
        }
    }
    let (mut ring_ok, mut bij_ok, mut both_ok, mut ring_only, mut bij_only) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mapped: Vec<(u8, u8)> = slots
        .iter()
        .map(|&(x, y)| (map[x as usize], map[y as usize]))
        .collect();
    for (j, &a) in slots.iter().enumerate() {
        let q = (a.0.wrapping_add(delta), a.1);
        let (ring_pick, _) = nearest(&slots, q);
        let (bij_pick, _) = nearest(&mapped, (map[q.0 as usize], map[q.1 as usize]));
        let r = ring_pick == j;
        let b = bij_pick == j;
        ring_ok += u64::from(r);
        bij_ok += u64::from(b);
        both_ok += u64::from(r && b);
        ring_only += u64::from(r && !b);
        bij_only += u64::from(!r && b);
    }
    (ring_ok, bij_ok, ring_only, bij_only)
}

/// The gcd arm: K entities, each a product of two primes drawn from a pool of `pool` primes. A
/// query is the entity's first prime times a fresh prime, so it shares exactly that factor with its
/// entity. The read ranks by gcd (ties to the later entity). `pool < 2K` reuses primes, which is the
/// realistic case; the number that matters is the **match set**: how many other entities share a
/// factor with the query, i.e. how ambiguous the address is.
fn gcd_arm(k: usize, pool: usize, seed: u64) -> (u64, f64) {
    let pool = pool.max(4);
    let primes = primes(pool + 1);
    let mut rng = Lcg(seed ^ 0x9cd);
    let mut entities: Vec<u64> = Vec::with_capacity(k);
    for _ in 0..k {
        let a = primes[rng.below(pool)];
        let mut b = primes[rng.below(pool)];
        while b == a {
            b = primes[rng.below(pool)];
        }
        entities.push(a * b);
    }
    let fresh = primes[pool]; // outside the pool, so it never matches
    let (mut ok, mut shared_total) = (0u64, 0usize);
    for &e in &entities {
        // The entity's own first factor is the smaller of its two primes.
        let a = {
            let mut f = 2u64;
            while e % f != 0 {
                f += 1;
            }
            f
        };
        let query = a * fresh;
        let mut best = 0usize;
        let mut best_g = 0u64;
        for (i, &c) in entities.iter().enumerate() {
            let g = gcd(query, c);
            if g >= best_g {
                best_g = g;
                best = i;
            }
        }
        let exact = entities.iter().position(|&c| c == e).unwrap_or(usize::MAX);
        ok += u64::from(best == exact);
        shared_total += entities
            .iter()
            .filter(|&&c| gcd(query, c) > 1 && c != e)
            .count();
    }
    (ok, shared_total as f64 / k as f64)
}


/// The first sixteen zeta zeros, from `crates/uor-r4-core/src/zeta_zeros.rs` (Odlyzko's table).
const ZEROS: [f64; 16] = [
    14.134725142, 21.022039639, 25.010857580, 30.424876126,
    32.935061588, 37.586178159, 40.918719012, 43.327073281,
    48.005150881, 49.773832478, 52.970321478, 56.446247697,
    59.347044003, 60.831778525, 65.112544048, 67.079810529,
];

/// One prime's phase vector over `channels` zeros, quantized to a byte per channel.
fn phase_code(p: u64, channels: usize) -> Vec<u8> {
    (0..channels)
        .map(|k| {
            let angle = ZEROS[k] * (p as f64).ln();
            let turns = angle / std::f64::consts::TAU;
            (turns.rem_euclid(1.0) * 256.0).floor() as u8
        })
        .collect()
}

/// Circular distance between two phase codes, summed over channels.
fn code_distance(a: &[u8], b: &[u8]) -> u32 {
    a.iter().zip(b).map(|(&x, &y)| u32::from(ring_step(x, y))).sum()
}


/// The first `n` zeros: from `UOR_ZETA_ZEROS` (a gz of Odlyzko's `zeros6.gz`) when set and
/// readable, else the sixteen embedded values. The receipt beside this binary names the source
/// and the gzip's sha256.
fn load_zeros(n: usize) -> Vec<f64> {
    if let Ok(path) = std::env::var("UOR_ZETA_ZEROS") {
        if let Ok(bytes) = std::fs::read(&path) {
            let out = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "gzip -dc '{}' | head -n {n}",
                    path.replace('\'', "'\\''")
                ))
                .output();
            if let Ok(out) = out {
                let text = String::from_utf8_lossy(&out.stdout);
                let zeros: Vec<f64> = text
                    .lines()
                    .filter_map(|line| line.trim().parse::<f64>().ok())
                    .collect();
                if !zeros.is_empty() {
                    return zeros;
                }
            }
        }
    }
    ZEROS.iter().copied().take(n).collect()
}

/// The zeta phase code at `channels` channels: two numbers, honestly measured.
///
/// (a) **Resolution**: how many of `k` primes share a code with another. The phase is
///     `θ_k(p) = γ_k ln p`; one byte resolves `1/256` of a turn, i.e. a factor `e^{2π/256} ≈ 1.0247`
///     in `p` per channel. One channel therefore buckets primes by magnitude; `J` incommensurate
///     channels fingerprint `ln p` far more finely.
/// (b) **Locality**: does the code of `p + 2` land on `p`'s own code, i.e. is closeness *magnitude*
///     or *meaning*? A smooth `ln p` code makes numerically adjacent primes near; it says nothing
///     about two words with the same meaning and unrelated primes.
fn zeta_resolution(k: usize, channels: usize) -> (usize, u64) {
    let pool = primes(k + 1);
    let codes: Vec<Vec<u8>> = pool[..k].iter().map(|&p| phase_code(p, channels)).collect();
    let distinct: std::collections::BTreeSet<&Vec<u8>> = codes.iter().collect();
    // Locality: the nearest code to phase(p+2) is p's own.
    let mut local = 0u64;
    for (i, &p) in pool[..k].iter().enumerate() {
        let q = phase_code(p + 2, channels);
        let (mut best, mut best_d) = (usize::MAX, u32::MAX);
        for (j, c) in codes.iter().enumerate() {
            let d = code_distance(c, &q);
            if d <= best_d { best_d = d; best = j; }
        }
        local += u64::from(best == i);
    }
    (distinct.len(), local)
}


/// The routed-zero read. Each **class** owns a sparse set of channels (the zeros it routes to);
/// stored surface forms carry the class's channels plus a per-form phase signature; a query is a NEW
/// surface form of the same class, so it shares the channels and none of the signature. The read is
/// the least energy over the union: a shared channel costs its circular phase distance, a channel on
/// one side only costs a fixed penalty.
///
/// `permute` rotates the class→channel assignment in the store (the control: identical occupancy,
/// broken routing). `swap_values` rotates the values across classes (the value-swap falsifier: if
/// accuracy survives, nothing reads the pairing).
fn routed_arm(
    j: usize,
    classes: usize,
    class_channels: usize,
    forms: usize,
    permute: bool,
    swap_values: bool,
    seed: u64,
) -> f64 {
    const PENALTY: u32 = 128;
    let mut rng = Lcg(seed);
    let mut sets: Vec<Vec<u16>> = Vec::with_capacity(classes);
    for _ in 0..classes {
        let mut set = Vec::with_capacity(class_channels);
        while set.len() < class_channels {
            let c = rng.below(j) as u16;
            if !set.contains(&c) {
                set.push(c);
            }
        }
        set.sort_unstable();
        sets.push(set);
    }
    // The values are the memory's content, bound to their class. `swap_values` changes only the
    // *stored* value, so a read that follows the channels returns the wrong one; the ground truth
    // stays each class's own value.
    let values: Vec<u8> = (0..classes).map(|_| rng.below(256) as u8).collect();
    // The store: for each class, `forms` entries on the class's channels (or the rotated set).
    let mut entries: Vec<(Vec<u16>, Vec<u8>, u8)> = Vec::new();
    for c in 0..classes {
        let set = if permute {
            sets[(c + 1) % classes].clone()
        } else {
            sets[c].clone()
        };
        let stored_value = if swap_values {
            values[(c + 1) % classes]
        } else {
            values[c]
        };
        for _ in 0..forms {
            let phases: Vec<u8> = (0..set.len()).map(|_| rng.below(256) as u8).collect();
            entries.push((set.clone(), phases, stored_value));
        }
    }
    let mut ok = 0usize;
    let queries = 3;
    for c in 0..classes {
        for _ in 0..queries {
            let phases: Vec<u8> = (0..sets[c].len()).map(|_| rng.below(256) as u8).collect();
            let mut best = 0usize;
            let mut best_e = u32::MAX;
            for (i, (set, stored, _)) in entries.iter().enumerate() {
                let mut energy = 0u32;
                for (k, &ch) in sets[c].iter().enumerate() {
                    match set.iter().position(|&s| s == ch) {
                        Some(at) => energy += u32::from(ring_step(phases[k], stored[at])),
                        None => energy += PENALTY,
                    }
                }
                for &ch in set.iter() {
                    if !sets[c].contains(&ch) {
                        energy += PENALTY;
                    }
                }
                if energy <= best_e {
                    best_e = energy;
                    best = i;
                }
            }
            ok += usize::from(entries[best].2 == values[c]);
        }
    }
    ok as f64 / (classes * queries) as f64
}

fn main() {
    println!("phase-ring-probe: planted ring vs a fixed-bijection control, and a prime-gcd arm");
    println!("(the bijection keeps occupancy and injectivity identical; only ring adjacency is destroyed)");
    println!();
    for k in [500usize, 2_000, 8_000] {
        for delta in [1u8, 2u8] {
            let (ring, bij, both, ring_only) = ring_arm(k, delta, 0x51ce ^ k as u64);
            let _ = both;
            println!(
                "K={k:5} δ={delta}  ring {:6.3}  bijection {:6.3}  ring−bij {:6.3}  (ring-only {ring_only})",
                ring as f64 / k as f64,
                bij as f64 / k as f64,
                (ring as f64 - bij as f64) / k as f64,
            );
        }
    }
    println!();
    let zeros = load_zeros(4_096);
    let j = zeros.len();
    println!("routed-zero read: each class owns a sparse channel set among the first {j} zeros;");
    println!("a query is a new surface form (same channels, new phases); least energy over the union.");
    for classes in [64usize, 256] {
        let (r, c, v) = (
            routed_arm(j.min(1024), classes, 8, 4, false, false, 0x701 ^ classes as u64),
            routed_arm(j.min(1024), classes, 8, 4, true, false, 0x701 ^ classes as u64),
            routed_arm(j.min(1024), classes, 8, 4, false, true, 0x701 ^ classes as u64),
        );
        println!(
            "classes={classes:4} channels={}  routed {r:.3}  permuted control {c:.3}  value-swapped {v:.3}",
            j.min(1024)
        );
    }
    println!();
    println!("zeta phase code: resolution (distinct codes of K primes) and locality (phase(p+2) finds p):");
    for k in [2_000usize, 8_000] {
        for channels in [1usize, 2, 4, 8, 16] {
            let (distinct, local) = zeta_resolution(k, channels);
            println!(
                "K={k:5} channels={channels:2}  distinct codes {distinct:5}/{k} ({:.4})  phase(p+2) finds p {local:5}/{k} ({:.3})",
                distinct as f64 / k as f64,
                local as f64 / k as f64,
            );
        }
    }
    println!();
    for (k, pool) in [(500usize, 1_000usize), (500, 250), (2_000, 4_000), (2_000, 1_000)] {
        let (ok, shared) = gcd_arm(k, pool, 0x6cd ^ k as u64);
        println!(
            "gcd K={k:5} pool={pool:5}  exact {ok}/{k} ({:.3})  other matches (mean) {shared:.1}",
            ok as f64 / k as f64
        );
    }
}
