# REPORT: independent CPU Relation read, deepseek-4

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4706fb2ee`.

## What did not work

- **This is a read slice, not #3501 completion.** Counterfactual scoring, BVG-K apply and checkpoint resume remain disabled.
- **The benchmark uses an in-memory synthetic Relation model.** It excludes artifact parsing, training proposals and apply.
- **CUDA-only position transport and raw/unit-snap controls remain outside this artifact slice.** The four-table host path uses endpoint BFS and scale-matched snap.
- **No corpus or training run occurred.**
- **The known duplicate `uor-r4-integer` artifact build collision intermittently surfaced as unresolved imports.** The identical release command passed on rerun at the same source; the passing result is counted below.

## What changed

- Added `CpuDiscreteRelation` for four-table Relation artifacts with one to five sweeps.
- Flock and Relation now share one active-leaf parser, binder consensus, transport, snap and ranking implementation; Relation adds the exact five edge types and per-sweep score recurrence.
- Relation requires `read_mode=relation`, `read_kept=1..=8`, `read_sweeps=1..=5` and `read_initialization=active_leaf_v1`.
- Parsing requires exactly `blocks*stages` level gates plus `5*stages` relation gates in the source order.
- Serialized multi-level fixtures match `IntegerDiscreteModel::evaluate_prefix` with all five edge kinds, `read_kept=3` and sweeps 1, 2 and 5.
- Relation with all relation gates zero matches the corresponding Flock read over the same codebook and level gates.
- Route and Flock modes, bad parameters/counts/trits/maps and prefix bounds fail closed.

## Measurements

- Focused Relation tests: 4 passed, with the release benchmark intentionally ignored.
- Full `uor-r4-cpu-train` release suite: 71 passed and two ignored release benchmarks.
- Every relation edge kind matched the host; at least one single-edge fixture changed the read from the zero-relation fallback.
- Release benchmark on the Ryzen 9 9950X: 1,024 prefix reads, width 32, four blocks, five stages, `read_kept=3`, two sweeps and a 64-token prefix in 82.587 ms (`12,398.992` rows/s), checksum `c9f78ff7c7f78ff7`.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-relation-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after one known duplicate-artifact collision retry, 71 passed and two ignored.
- `CARGO_TARGET_DIR=target/deepseek-relation-serial cargo test --release --locked -p uor-r4-cpu-train discrete_route::tests::release_relation_read_benchmark -- --ignored --exact --nocapture`: PASS.
- `CARGO_TARGET_DIR=target/deepseek-relation-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, 11 gates green.

## Skills used

No repository skill was invoked. The executed Relation semantics and finite SpiralCore tables are being read directly from the repository.

## Still open

- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

The CPU crate now reproduces all three supported four-table reads exactly on deterministic artifacts. No training capability is claimed.
