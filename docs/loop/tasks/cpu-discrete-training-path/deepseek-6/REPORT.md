# REPORT: repair proposal-chooser tie evidence, deepseek-6

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `d65054c4d`. Repair of [deepseek-5](../deepseek-5/ORDER.md).

## What did not work

- **The first exact-head review requested changes.** The chooser behavior matched, but Route/Table tie tests derived their expected winners from the same resulting-port function under test and did not pin the lane-0 tie guard.

## What changed

- Route and Table tests now compare all 15 nonzero resulting ports against independently pinned numeric vectors.
- A separate test oracle recomputes those vectors from `NormFrameTable` generator frames, the finite composition table and the root transform, without calling the chooser's resulting-port implementation.
- Equal current/candidate losses keep lane 0 even when nonzero candidates have lower ports.
- A strictly lower-loss candidate still wins over the better-port tie among other lanes.

## Measurements

- Route resulting ports are pinned to `[16, 27, 34, 41, 47, 2, 3, 4, 5, 21, 22, 23, 24, 25, 26]`; Table ports are pinned to `[0, 10, 2, 5, 6, 5, 13, 7, 9, 11, 0, 2, 8, 7, 4]`.
- Equal current/candidate losses keep lane 0 for both Route and Table, despite lower candidate ports.
- A strictly lower-loss lane still wins over the better-port tie among other candidates.
- Full `uor-r4-cpu-train` release suite: 74 passed and three ignored release benchmarks.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-proposals-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 74 passed and three ignored.
- `CARGO_TARGET_DIR=target/deepseek-proposals-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, 11 gates green.

## Skills used

No repository skill was invoked. The repair follows the reviewer's source-cited findings.

## Still open

- Virtual writer scoring for the 16 proposal lanes.
- Atomic-free apply, BVG-K and deterministic resume.

## Plain summary

This repair closes the chooser review findings without changing the product code or claiming update capability.
