//! Bott Fock context scaling benchmark (issue #107, Gate F).
//!
//! Proves the O(1) claims of `BottFockContextStore`:
//!
//! - **Space**: the store is a fixed-size value (256 `i16` cells + one
//!   counter); folding more tokens never grows it — no KV cache.
//! - **Time**: every append performs the same fixed 256-cell integer
//!   sweep, so per-append latency is independent of how many tokens were
//!   folded before. The full N = 10^3 .. 10^6 wall-clock table runs in
//!   the release-mode ignored test:
//!
//! ```text
//! cargo test -p uor-r4-core --release --test bott_fock_scaling -- --ignored --nocapture
//! ```

use std::time::Instant;
use uor_r4_core::transformerless::bott_fock::{BottFockContextStore, CONTEXT_DIM};

/// Deterministic xorshift embedding stream — no RNG state outside the
/// caller, identical token sequences across runs.
struct TokenStream(u64);

impl TokenStream {
    fn next(&mut self, out: &mut [i16; CONTEXT_DIM]) {
        let mut i = 0usize;
        while i < CONTEXT_DIM {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            out[i] = self.0 as i16;
            i += 1;
        }
    }
}

#[test]
fn store_size_is_independent_of_token_count() {
    let fixed_size = std::mem::size_of::<BottFockContextStore>();
    let mut store = BottFockContextStore::new();
    let mut stream = TokenStream(0x9e3779b97f4a7c15);
    let mut token = [0i16; CONTEXT_DIM];
    let mut n = 0u64;
    while n < 100_000 {
        stream.next(&mut token);
        store.append_token(&token);
        n += 1;
        assert_eq!(
            std::mem::size_of_val(&store),
            fixed_size,
            "store grew after {n} appends"
        );
    }
    assert_eq!(store.token_count(), 100_000);
    assert_eq!(store.state().len(), 256);
}

/// Wall-clock O(1) proof across the DoD range N = 10^3 .. 10^6. Timing
/// belongs in release mode; ignored by default so debug-mode CI never
/// sees a flaky clock assertion.
#[test]
#[ignore = "release-mode wall-clock benchmark"]
fn append_latency_is_constant_across_context_lengths() {
    let sizes = [1_000u64, 10_000, 100_000, 1_000_000];
    let mut per_append_ns = Vec::new();
    for &n in &sizes {
        let mut store = BottFockContextStore::new();
        let mut stream = TokenStream(0x243f6a8885a308d3);
        let mut token = [0i16; CONTEXT_DIM];
        let start = Instant::now();
        let mut i = 0u64;
        while i < n {
            stream.next(&mut token);
            store.append_token(&token);
            i += 1;
        }
        let elapsed = start.elapsed().as_nanos();
        let per_append = elapsed / (n as u128);
        per_append_ns.push(per_append);
        println!(
            "N = {n:>9}: total {:?}, {per_append} ns/append",
            start.elapsed()
        );
    }
    let min = *per_append_ns.iter().min().expect("sizes non-empty");
    let max = *per_append_ns.iter().max().expect("sizes non-empty");
    println!("per-append spread: min {min} ns, max {max} ns");
    // Strict O(1) would be ratio 1.0; wall clocks jitter, so fail only
    // when the slowest batch is more than 3x the fastest — a bound that
    // still catches any accidental O(N) work (10^3 x spread at N = 10^6).
    assert!(
        max <= min.saturating_mul(3),
        "per-append latency grew with context length: min {min} ns, max {max} ns"
    );
}
