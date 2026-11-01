# REPORT: CPU shared-vector forward parity, deepseek-1

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `d8f4282cf`.

## What did not work

- **Hamming reads and exact turn memory are not in this unit.** `FbsImage` intentionally omits models carrying either facility, so this PR proves the no-read trunk only. It is a substantial serving-aligned forward slice, not the complete current FBS model.
- **The first compile exposed type mistakes in the new port:** residual scratch is `i64` while route gather outputs are `i32`, and 16-bit activations need a generic gather input. The code now uses one checked gather path for `i32`, `i64` and `i16`.
- **Clippy initially caught two edge issues:** a manual multiple check and a redundant conversion. Both are fixed.

## What changed

- Added `CpuFbsForward`, which copies the no-read FBS tables exposed by `FbsImage` into the isolated CPU crate.
- Ported exact Q12-to-residual conversion, grid-coded gains, FBS route gather/sign, fine RMS normalization, 16-bit requantization, quaternion recurrence, convolution history, icosian snapping, Hamilton recurrence gate, Hamilton feed-forward gate and final Q12 normalization.
- Added both serving readouts: whole-table nearest and the FBS native cell writer.
- Added synthetic artifact parity over one, three and six recurrence layers.
- Added a route-sign corruption test that changes the complete query vector.
- Added a six-layer width-256 throughput benchmark.

## Measurements

- `cargo test -p uor-r4-cpu-train`: 17 passed, 0 failed; 7 are FBS-forward tests.
- One-layer, three-layer and six-layer tests compare every step's complete `query_q12`, token and kept-cell vector against `IntegerStackSession::step_native` across eight tokens.
- Writer-head parity passes on a two-layer model; nearest-head parity passes on one-, three- and six-layer models.
- Six-layer shape (`vocab=4096`, `width=256`, `mlp=384`, `context=192`, nearest head): 64 timed steps in 19.811 ms, **3,230.533 tokens/s**, checksum `11743905789448238222`.

This is exact synthetic-forward throughput. It does not include Hamming reads, turn memory, training, language quality or a trained checkpoint.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train`: PASS, 17 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench`: PASS; reading above.
- `cargo fmt --package uor-r4-cpu-train`: PASS.

## Skills used

No repository skill was invoked. Subagents independently audited the serving forward and fixture boundary before implementation.

## Still open

- Add Hamming copy/type reads and exact turn memory on top of the verified trunk.
- Compare a trained artifact, not only deterministic synthetic artifacts.
- Add training gradients and checkpoints in `cpu-shared-vector-train`.
- The item remains `ready`; this PR references #3504 and does not close it.

## Plain summary

The CPU crate can now run the current no-read FBS trunk and produce the same query vectors and token decisions as serving on finite synthetic models, including a six-layer case and both output heads. Hamming and memory remain open, so this is verified forward infrastructure rather than a complete CPU model.
