# REPORT: repair proposal-scoring status and Route header controls, deepseek-8

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `9c314cc8c`. Repair of [deepseek-7](../deepseek-7/ORDER.md).

## What did not work

- **The first exact-head review requested changes.** Arithmetic matched, but scorer errors did not carry numeric statuses, Route scorer construction accepted header forms the host rejects, and some failure controls were absent.

## What changed

- Added `CpuDiscreteScoreError` with numeric status and message; scorer now returns status 2, 4, 8, 9 or 12 by decision path.
- Route scorer construction now enforces empty matrices, source identities, admitted objective policy and header-padding rules before state loading.
- Tests pin all-ineligible status 8, mode mismatch status 9, inventory status 4 and malformed Route-header refusal.

## Measurements

- Full `uor-r4-cpu-train` release suite: 77 passed and four ignored release benchmarks.
- Status controls now pin 8 for empty/all-ineligible input, 9 for mode/kind mismatch and 4 for inventory refusal.
- Route scorer construction refuses a nonempty matrices header.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-scoring-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 77 passed and four ignored.
- `CARGO_TARGET_DIR=target/deepseek-scoring-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The repair follows the reviewer's source-cited findings.

## Still open

- Atomic-free apply.
- BVG-K and deterministic restart/resume.

## Plain summary

This repair closes the scorer status/header findings without changing the scoring arithmetic or claiming update capability.
