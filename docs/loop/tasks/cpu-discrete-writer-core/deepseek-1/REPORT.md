# REPORT: exact CPU discrete writer core, deepseek-1

Lab: deepseek. Item: [cpu-discrete-writer-core](../../../../roadmap/items/cpu-discrete-writer-core.md) (#3502). Base: `03800ffd2`.

## What did not work

- **The stored-GPU-observation parity gate is still open.** The repository contains aggregate Stage T/M1 GPU receipts (for example `t16-frozen-zero-seed1-trunk-dev-score.json`), but it does not contain that model's exact codebook table and query vectors. A synthetic or aggregate-only comparison would not establish row parity, so this PR does not claim the stored-observation gate.
- **The first target-reading test used `i32::MAX` to create a far query and overflowed while summing 32 absolute differences.** The test now uses a bounded real token vector and checks the reading's internal consistency instead.
- **Repo-wide `cargo fmt --all -- --check` is already red outside this branch.** The new crate was formatted with `cargo fmt --package uor-r4-cpu-train`.
- **Repo-wide Clippy with dependencies is already red in `uor-r4-integer`.** The new crate passes `clippy --no-deps -D warnings`; dependency warnings are reported but not changed here.

## What changed

- Added the non-default workspace crate `crates/uor-r4-cpu-train`.
- Added checked codebook decoding from the canonical 16-root orbit into 4,096 vectors and 256 writer cells.
- Added exact outer-cell L1 admission with `(distance, cell id)` ordering.
- Added full-vector L1 ranking inside admitted cells with `(distance, token id)` ordering.
- Added top-8, target rank/admission readings, deterministic lower-token tie handling and malformed-input refusal.
- Added a release benchmark for 1, 8, 16 and 32 threads.
- Added a test that keeps the crate out of the workspace default-member set.

## Measurements

- `cargo test -p uor-r4-cpu-train`: 7 passed, 0 failed.
- Existing host-oracle parity: 256 / 256 deterministic queries matched `writer_score_full_outer_l1` exactly for top-8 and admitted-cell order.
- Benchmark, 32,768 timed rows after 128 warm-up rows:
  - 1 thread: 221,990 rows/s, 147.611 ms;
  - 8 threads: 1,523,446 rows/s, 21.509 ms;
  - 16 threads: 2,392,580 rows/s, 13.696 ms;
  - 32 threads: 3,292,424 rows/s, 9.953 ms.
- Every thread count produced checksum `4667609706970220960`.

This benchmark measures component decisions on synthetic canonical vectors. It is not training throughput, a model result or evidence that the synthetic distribution matches a trained checkpoint.

## Checks run at the head

- `cargo test -p uor-r4-cpu-train`: PASS.
- `cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench`: PASS; readings above.
- `cargo tree -p uor-r4-cpu-train -e normal`: no `uor-r4-training`, `uor-r4-float-train` or Candle edge.
- `uor-harness ratchet d23`: PASS, baseline unchanged.
- `uor-harness ratchet d25`: PASS, baseline unchanged.
- `uor-harness ratchet d32`: PASS, baseline unchanged.
- `cargo fmt --package uor-r4-cpu-train`: PASS.

## Skills used

No repository skill was invoked for this code unit.

## Still open

- Bind an exact checkpoint/table/query fixture from a stored GPU evaluation and reproduce its per-row top-8 and admitted cells.
- Add snapshot/model parsing and token encoding after that fixture is available.
- The item remains `ready`; this PR references #3502 and does not close it.

## Plain summary

The CPU training program now has an independent exact writer core and a fast benchmark. It matches the current host writer on deterministic inputs, but it has not yet reproduced a stored trained-model observation, so it is not marked complete.
