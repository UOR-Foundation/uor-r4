# REPORT: repair CPU Flock header validation and evidence, deepseek-3

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `3863dd75f`. Repair of [deepseek-2](../deepseek-2/ORDER.md).

## What did not work

- **The first exact-head review requested changes.** The Flock arithmetic matched, but the parser accepted malformed headers that `IntegerDiscreteModel` rejects, and the evidence called a generic two-level fixture the published copy-successor fixture.

## What changed

- Flock parsing now rejects nonempty matrices, wrong source snapshot/operator/composition identities, missing or unknown objective policies, and nonzero bytes between the JSON header and the 64-byte data boundary.
- Independent serialized controls mutate each rejected header field.
- REPORT and notebook now call the first parity shape a generic two-level fixture.

## Measurements

- Focused Flock tests: 5 passed, with the release benchmark intentionally ignored.
- Full `uor-r4-cpu-train` release suite: 67 passed, one ignored benchmark.
- Independent controls now reject nonempty matrices, each wrong source identity, missing/unknown objective policy, nonzero header-alignment padding and nonzero trailing data padding.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-flock-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after the same known duplicate-artifact collision was retried once.
- `CARGO_TARGET_DIR=target/deepseek-flock-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The repair follows the fresh reviewer's source-cited findings.

## Still open

- Relation reads.
- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

This repair closes the exact-head review findings without changing scope or claiming training.
