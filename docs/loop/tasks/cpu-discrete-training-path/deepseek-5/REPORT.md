# REPORT: exact discrete proposal chooser, deepseek-5

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `d65054c4d`.

## What did not work

- **This is only the chooser slice.** Virtual writer scoring, model mutation/apply, BVG-K and deterministic resume remain disabled.
- **The chooser consumes externally supplied 16-lane losses.** It does not yet prove those losses by rerunning the read and writer for a proposed move.
- **The benchmark is a synthetic selector microbenchmark.** It is not training throughput.
- **The known duplicate `uor-r4-integer` artifact collision intermittently surfaced as unresolved imports.** The identical release command passed on rerun at the same source.
- **No corpus or training run occurred.**

## What changed

- Added `CpuDiscreteProposalChooser` and typed Route/Table/Gate proposal decisions with the five source proposal words.
- Gate selection uses strict loss improvement only and retains the lower active ordinal on ties; inactive lanes 3..15 never affect sensitivity.
- Route and Table selection compares resulting ports when candidate losses tie.
- Resulting Route ports use exact finite-group composition; Table ports use the exact generator action and root identity.
- Chooser construction reuses the validated Route, Flock and Relation artifact parsers and records the exact gate inventory for each read mode.
- Invalid proposal mode/kind pairs and Route, Table or Gate indices fail closed.

## Measurements

- Focused chooser tests: 3 passed, with the release benchmark intentionally ignored.
- Full `uor-r4-cpu-train` release suite: 74 passed and three ignored release benchmarks.
- Gate cases pin strict improvement, equal-loss ordinal retention, all-worse sensitivity and inactive-lane exclusion.
- Route and Table cases pin equal-loss selection by lower resulting port.
- Release selector benchmark on the Ryzen 9 9950X: 1,000,000 decisions in 6.024 ms (`165,993,535.216` decisions/s), checksum `0351a0204080e342`.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-proposals-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 74 passed and three ignored.
- `CARGO_TARGET_DIR=target/deepseek-proposals-serial cargo test --release --locked -p uor-r4-cpu-train discrete_route::tests::release_proposal_chooser_benchmark -- --ignored --exact --nocapture`: PASS.
- `CARGO_TARGET_DIR=target/deepseek-proposals-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The executed chooser semantics are being read directly from `discrete.cu`.

## Still open

- Virtual writer scoring for the 16 proposal lanes.
- Atomic-free apply, BVG-K and deterministic resume.

## Plain summary

The CPU crate now reproduces the discrete chooser's decision semantics over supplied losses. No update capability is claimed.
