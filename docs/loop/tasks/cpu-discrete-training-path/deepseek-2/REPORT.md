# REPORT: independent CPU Flock read, deepseek-2

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `3863dd75f`.

## What did not work

- **This is a read slice, not #3501 completion.** Relation reads, counterfactual scoring, BVG-K apply and checkpoint resume remain disabled.
- **The benchmark is an in-memory synthetic Flock model.** It does not include artifact parsing, training proposals or apply.
- **The device-only controls are not ported.** The four-table host path uses legacy endpoint BFS and scale-matched snap; CUDA-only position transport and raw/unit-snap controls remain outside this artifact slice.
- **No corpus or training run occurred.**
- **Cargo intermittently reported unresolved imports while linking two same-named `uor-r4-integer` artifacts.** The identical command passed on rerun at the same source; the passing outputs below are the counted results. The benchmark is an ignored release test because the package's build-only targets hit that collision reproducibly.

## What changed

- Added `CpuDiscreteFlock`, an independent active-leaf reader for four-table `uor-r4.lut-discrete/1` Flock artifacts.
- Binder nodes now retain `level`, `first` and `last`; Flock ranks only closed subtrees and preserves their source spans across unequal frontier levels.
- Flock reconstructs each output block from its query payload, exact frame/payload consensus, exact per-level transport, signed gate trits and final scale-matched snap.
- Parsing requires `read_mode=flock`, `read_kept=1..=8`, `read_sweeps=0` and `read_initialization=active_leaf_v1`.
- Route, Relation, bad initialization and parameters, malformed trit/count/maps, non-orbit codebooks, bad cells, wrong kinds/lengths/alignment, duplicate/missing tables, malformed spans, nonempty matrices, source-geometry mismatches, unknown objectives and nonzero header/data padding fail closed.
- A serialized generic two-level fixture and a three-level fixture match `IntegerDiscreteModel::evaluate_prefix`; these are direct host-oracle comparisons, not copied queries.

## Measurements

- Focused Flock tests: 5 passed.
- Full `uor-r4-cpu-train` release suite: 67 passed.
- The serialized generic two-level shape and a context-9, three-level synthetic shape matched the host read on every tested prefix.
- Zero gates returned the last token's payload blocks exactly. Nonzero gates matched the host path, and a positive-gate fixture changed the read from the zero-gate fallback.
- Release benchmark on the Ryzen 9 9950X: 1,024 prefix reads, width 32, four blocks, five stages, `read_kept=3` and a 64-token prefix in 45.368 ms (`22,570.818` rows/s), checksum `f0078bc7f0078bc7`.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-flock-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS, 67 tests passed and one ignored benchmark.
- `CARGO_TARGET_DIR=target/deepseek-flock-serial cargo test --release --locked -p uor-r4-cpu-train discrete_route::tests::release_flock_read_benchmark -- --ignored --exact --nocapture`: PASS.
- `CARGO_TARGET_DIR=target/deepseek-flock-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, 11 gates green.

## Skills used

No repository skill was invoked. The executed Flock semantics and finite SpiralCore tables are being read directly from the repository.

## Still open

- Relation reads.
- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

The CPU crate now reproduces the supported four-table Flock read exactly on deterministic artifacts. No training capability is claimed.
