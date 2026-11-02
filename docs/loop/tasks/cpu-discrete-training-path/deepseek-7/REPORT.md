# REPORT: exact discrete proposal scoring, deepseek-7

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `9c314cc8c`.

## What did not work

- **This is read-only scoring.** Model mutation/apply, BVG-K and deterministic resume remain disabled.
- **The virtual scorer clones state once per active proposal lane.** Correctness comes first; no optimized zero-copy lane engine is claimed.
- **No corpus, training run or language result occurred.**

## What changed

- Added `CpuDiscreteProposalScorer` over immutable baseline state with `CpuDiscreteProposalSample` rows and summed 16-lane losses.
- Baseline uses the exact read and writer and skips rows whose target has no admitted token rival.
- Route proposals compose one virtual route action; Gate proposals apply the exact sibling-trit virtual move, including Relation tails; Table proposals move one codebook/input block.
- The writer reproduces occupied-cell admission, global token ranking, token/address hinges, occupied-cell rank hinge, objective selection and checked accumulation.
- Candidate losses feed the merged chooser, preserving Gate three-lane inventory and Route/Table tie behavior.
- Baseline output matches `IntegerDiscreteModel::observe_target` for both objective policies; Route, Gate and Table virtual states match equivalent serialized host artifacts.

## Measurements

- Focused proposal-scoring tests: 3 passed, with the release benchmark intentionally ignored.
- Full `uor-r4-cpu-train` release suite: 77 passed and four ignored release benchmarks.
- Baseline nearest and occupied-rank losses match `IntegerDiscreteModel::observe_target`.
- Route, Gate, Table and rank-Gate virtual states match equivalent serialized host artifacts.
- All-ineligible rows and invalid proposal requests fail closed.
- Release scoring benchmark on the Ryzen 9 9950X: 512 full 16-lane scores over one sample in 4.628 ms (`110,638.329` scores/s), checksum `0e0a0e0a0e0a0e0a`.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-scoring-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 77 passed and four ignored.
- `CARGO_TARGET_DIR=target/deepseek-scoring-serial cargo test --release --locked -p uor-r4-cpu-train discrete_route::tests::release_proposal_scoring_benchmark -- --ignored --exact --nocapture`: PASS.
- `CARGO_TARGET_DIR=target/deepseek-scoring-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, 11 gates green.

## Skills used

No repository skill was invoked. The exact writer and virtual move semantics were derived directly from `discrete.cu` and the D11 host implementation.

## Still open

- Atomic-free apply.
- BVG-K and deterministic restart/resume.

## Plain summary

This branch turns the exact read and chooser layers into read-only proposal scoring over deterministic states. It does not mutate a model.
