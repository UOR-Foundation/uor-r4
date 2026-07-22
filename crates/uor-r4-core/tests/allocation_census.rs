//! Allocation census for the transformerless runtime — Phase-0 baseline.
//!
//! A counting global allocator measures what the runtime APIs actually
//! allocate. One single `#[test]` by design: the allocator's gate and
//! counters are process-wide, and libtest runs tests in parallel threads by
//! default, so several measured tests could let one test's bookkeeping
//! allocations land in another test's census — fatal to the zero-allocation
//! assertion. One test, sequential phases, no cross-thread noise.
//!
//! Measured sections run inside [`measure`], which resets the counters,
//! opens the gate, runs the closure, closes the gate, and snapshots. Every
//! `println!` happens with the gate closed, so reporting never pollutes
//! the numbers. Fixed seed, no wall-clock assertions: the census is
//! deterministic for a given artifact set.
//!
//! Run with: `cargo test -p uor-r4-core --test allocation_census -- --nocapture`

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use uor_r4_core::transformerless::compiler::{self, Compiled, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime::{self, OpKernel, Prediction, Store};

// --------------------------------------------------- counting allocator --

/// Global allocator that counts allocation events and gross bytes requested
/// while the gate is open. Buffer growth goes through the default
/// `GlobalAlloc::realloc`, which calls `alloc` for the new layout, so a
/// grown buffer shows up as one fresh allocation of the new size.
/// Deallocations are not counted: the census answers "how much did the
/// measured section ask for", not "what was live at the end".
struct CountingAlloc;

static GATE: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && GATE.load(Ordering::SeqCst) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Census {
    allocations: usize,
    bytes: usize,
}

const ZERO: Census = Census {
    allocations: 0,
    bytes: 0,
};

fn census() -> Census {
    Census {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        bytes: BYTES.load(Ordering::Relaxed),
    }
}

/// Run `f` with the counting gate open; return its output and the census of
/// what it allocated.
fn measure<T>(f: impl FnOnce() -> T) -> (T, Census) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    GATE.store(true, Ordering::SeqCst);
    let out = f();
    GATE.store(false, Ordering::SeqCst);
    (out, census())
}

// ------------------------------------------------------------ op census --

/// Snapshot of the runtime's `OpKernel` counters (the kernel itself is not
/// `Clone`, so copy the public fields).
#[derive(Clone, Copy)]
struct Ops {
    adds: u64,
    xors: u64,
    shifts: u64,
    compares: u64,
    table_reads: u64,
    candidate_scans: u64,
}

impl Ops {
    fn of(k: &OpKernel) -> Self {
        Ops {
            adds: k.adds,
            xors: k.xors,
            shifts: k.shifts,
            compares: k.compares,
            table_reads: k.table_reads,
            candidate_scans: k.candidate_scans,
        }
    }
    fn since(self, before: Ops) -> Ops {
        Ops {
            adds: self.adds - before.adds,
            xors: self.xors - before.xors,
            shifts: self.shifts - before.shifts,
            compares: self.compares - before.compares,
            table_reads: self.table_reads - before.table_reads,
            candidate_scans: self.candidate_scans - before.candidate_scans,
        }
    }
}

// ---------------------------------------------------------------- census --

/// Fixed number of tokens to generate per census run.
const GEN_TOKENS: usize = 32;
/// Fixed seed window: deterministic, and every id is below the smallest
/// supported vocabulary (TLA3, V = 32000), so the same seed drives both the
/// fixture and the real smollm2 artifacts.
const SEED: [u32; WINDOW] = [1, 40, 416, 1024, 2048, 4096, 8192, 16384];

fn real_path(name: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    format!("{dir}/../../.uor-models/compiled/smollm2-135m-instruct/{name}")
}

fn fixture_path(name: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    format!("{dir}/tests/fixtures/{name}")
}

/// Parse the artifact container, measuring the parse. Prefers the real
/// compiled smollm2 artifacts; falls back to the repo fixture so the census
/// still runs on a bare checkout. Returns the artifacts and the path used.
fn parse_artifacts_measured() -> (Compiled, String) {
    for path in [
        real_path("tless_artifacts.bin"),
        fixture_path("tless_artifacts.bin"),
    ] {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let size = bytes.len();
        let (parsed, cen) = measure(|| compiler::parse_artifacts(&bytes));
        if let Some(art) = parsed {
            let vocab = art.token_codes.len() / STAGES;
            println!(
                "[parse] artifacts: {path} ({size} bytes, vocab {vocab}) \
                 → {} allocations, {} bytes",
                cen.allocations, cen.bytes
            );
            return (art, path);
        }
        println!("[parse] artifacts: {path} did not parse, trying next candidate");
    }
    panic!("no parsable TLA3/TLA4/TLA5 artifact container found");
}

/// Parse the TLS1 store, measuring the parse. Falls back to a small
/// synthetic store (level 0 populated, so prediction terminates) when the
/// real store file is absent or unparsable. Returns the store and an origin
/// label.
fn parse_store_measured() -> (Store, String) {
    let path = real_path("tless_store.bin");
    if let Ok(bytes) = std::fs::read(&path) {
        let size = bytes.len();
        let (parsed, cen) = measure(|| runtime::parse_store(&bytes));
        if let Some(store) = parsed {
            let keys: usize = store.iter().map(|level| level.len()).sum();
            println!(
                "[parse] store: {path} ({size} bytes TLS1, {keys} class keys) \
                 → {} allocations, {} bytes",
                cen.allocations, cen.bytes
            );
            return (store, path);
        }
        println!("[parse] store: {path} did not parse; using a synthetic store");
    } else {
        println!("[parse] store: {path} not present; using a synthetic store");
    }
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    for i in 0..64u16 {
        let code = [(i % 251) as u8, (i / 2) as u8, 0, 0];
        runtime::add_evidence(&mut store, &code, i as u32, 1);
    }
    (store, "synthetic (add_evidence, gate closed)".to_string())
}

#[test]
fn allocation_census() {
    println!("=== allocation census: uor-r4-core transformerless runtime (Phase-0 baseline) ===");

    // Phase 1 — container parsing (expected > 0; reported, not asserted).
    let (art, art_src) = parse_artifacts_measured();
    let (store, store_src) = parse_store_measured();

    // Phase 2 — Runtime construction (expected 0; reported, not asserted).
    let (mut rt, rt_cen) = measure(|| runtime::Runtime::new(&art));
    println!(
        "[runtime] Runtime::new → {} allocations, {} bytes (report only)",
        rt_cen.allocations, rt_cen.bytes
    );

    // Phase 3 — prediction and generation, every runtime API variant
    // exercised: assign_window, predict, predict_witness, and
    // generate_greedy_into with a caller-owned output buffer. Documented
    // allocation-free; asserted zero.
    let mut out = [Prediction::default(); GEN_TOKENS];
    let ((code, wit, n, gen_ops), gen_cen) = measure(|| {
        let code = rt.assign_window(&SEED);
        let tok = rt.predict(&store, &code);
        rt.state.clear_token_state();
        let wit = rt.predict_witness(&store, &code);
        assert_eq!(tok, wit.token, "predict and predict_witness agree");
        let before = Ops::of(&rt.kernel);
        let n = rt.generate_greedy_into(&store, &SEED, &mut out);
        let gen_ops = Ops::of(&rt.kernel).since(before);
        (code, wit, n, gen_ops)
    });
    assert_eq!(n, GEN_TOKENS, "every output slot filled");
    println!(
        "[runtime] assign_window + predict + predict_witness + \
         generate_greedy_into({GEN_TOKENS} tokens) → {} allocations, {} bytes",
        gen_cen.allocations, gen_cen.bytes
    );
    println!(
        "          seed code {code:?} → token {} (depth {}, evidence {})",
        wit.token, wit.depth, wit.count
    );
    assert_eq!(
        gen_cen, ZERO,
        "runtime prediction and generation must be allocation-free"
    );

    // Phase 4 — per-token op census from the Runtime's public OpKernel.
    let t = GEN_TOKENS as f64;
    println!(
        "[kernel] ops over {GEN_TOKENS} generated tokens: adds {} | xors {} | \
         shifts {} | compares {} | table-reads {} | candidate-scans {}",
        gen_ops.adds,
        gen_ops.xors,
        gen_ops.shifts,
        gen_ops.compares,
        gen_ops.table_reads,
        gen_ops.candidate_scans
    );
    println!(
        "[kernel] per generated token (avg): adds {:.1} | xors {:.1} | \
         shifts {:.1} | compares {:.1} | table-reads {:.1} | candidate-scans {:.1}",
        gen_ops.adds as f64 / t,
        gen_ops.xors as f64 / t,
        gen_ops.shifts as f64 / t,
        gen_ops.compares as f64 / t,
        gen_ops.table_reads as f64 / t,
        gen_ops.candidate_scans as f64 / t
    );
    println!("[kernel] cumulative {}", rt.kernel.report());
    let tokens: Vec<u32> = out.iter().map(|p| p.token).collect();
    let depths: Vec<usize> = out.iter().map(|p| p.depth).collect();
    println!("[gen] tokens {tokens:?}");
    println!("[gen] depths {depths:?}");

    // Phase 5 — the store write path, KNOWN to allocate (heap
    // Vec<BTreeMap>): measured on a scratch store, reported, never asserted.
    let mut scratch: Store = (0..=STAGES).map(|_| Default::default()).collect();
    let (_, ev_cen) = measure(|| {
        for i in 0..64u16 {
            let code = [(i % 251) as u8, ((i / 4) % 256) as u8, (i % 17) as u8, 0];
            runtime::add_evidence(&mut scratch, &code, (i.wrapping_mul(7)) as u32, 1);
        }
    });
    println!(
        "[store] add_evidence × 64 (known-allocating write path) \
         → {} allocations, {} bytes (report only)",
        ev_cen.allocations, ev_cen.bytes
    );

    println!("=== end census (artifacts: {art_src}; store: {store_src}) ===");
}
