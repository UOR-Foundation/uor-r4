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
    for (k, pool) in [(500usize, 1_000usize), (500, 250), (2_000, 4_000), (2_000, 1_000)] {
        let (ok, shared) = gcd_arm(k, pool, 0x6cd ^ k as u64);
        println!(
            "gcd K={k:5} pool={pool:5}  exact {ok}/{k} ({:.3})  other matches (mean) {shared:.1}",
            ok as f64 / k as f64
        );
    }
}
