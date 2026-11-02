# REPORT: exact single-coordinate proposal apply, deepseek-9

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4fdec6487`.

## What did not work

- **This is single-coordinate atomic-free apply.** BVG-K joint application and deterministic restart/resume remain disabled.
- **No corpus, training run or language result occurred.**

## What changed

- Added `CpuDiscreteProposalApplier` with epoch, mode, inventory and strict-improvement checks plus typed apply receipts.
- Lane 0 is a state-preserving no-op; Route/Gate/Table lane moves mutate one coordinate and increment epoch.

## Measurements

- Focused apply tests: 2 passed. Exact Gate/Table/Route mutation and status order/no-op controls are pinned.
- Full `uor-r4-cpu-train` release suite: 80 passed and four ignored release benchmarks.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-apply-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 80 passed and four ignored.
- `CARGO_TARGET_DIR=target/deepseek-apply-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The executed apply semantics are being read directly from `discrete.cu`.

## Still open

- BVG-K joint application.
- Deterministic cursor, checkpoint and restart/resume.

## Plain summary

This branch is the isolated single-coordinate apply slice for #3501.
