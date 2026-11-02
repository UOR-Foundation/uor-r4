# REPORT: independent CPU Route read, deepseek-1

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f7f8773f8`.

## What did not work

- **This is a first slice, not #3501 completion.** It parses and evaluates Route reads, but Flock, Relation, proposals, BVG-K, apply and resume remain disabled.
- **The benchmark is an in-memory synthetic Route model.** It does not measure artifact parsing, training proposals or apply.
- **No corpus or training run occurred.**

## What changed

- Added `CpuDiscreteRoute`, an independent reader for four-table `uor-r4.lut-discrete/1` Route artifacts.
- Validates artifact schema, Route-only initialization and shape, including `kept_cells`, zero `read_kept`/`read_sweeps`, codebook orbit roots, map lengths and action bounds.
- Validates all four required tables, kinds, lengths, duplicate/missing names, zero Route gates, expected stage counts and checked section bounds.
- Reimplements the ternary binder plan and Route traversal using the exact finite SpiralCore composition table.
- A serialized two-block, two-stage artifact with nonzero Route actions matches `IntegerDiscreteModel` exactly on one- and multi-token prefixes.
- Explicitly refuses non-Route artifacts; Flock and Relation cannot enter this path.

## Measurements

- Focused tests: 4 passed.
- Full `uor-r4-cpu-train` release suite: 62 passed.
- Serialized nonzero-action Route reads match `IntegerDiscreteModel` on four prefixes.
- Identity Route output is deterministic; empty and out-of-vocabulary prefixes are refused.
- Serialized Flock, Relation, bad-action, bad-map, short-span, nonzero Route read parameters, invalid `kept_cells`, malformed artifact and non-orbit-codebook controls pass through `from_bytes`.
- Release benchmark on the Ryzen 9 9950X: 1,024 prefix reads, width 32, four blocks, five stages and 64-token prefix in 1.019 ms (`1,005,271.787` rows/s), checksum `401ffb34401ffb34`, peak RSS 3,316 KiB.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train`: PASS, 62 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, 11 gates green.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The executed Route semantics in `discrete_d11.rs` and the finite SpiralCore table were read directly.

## Still open

- Flock and Relation reads.
- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

The CPU crate now has a guarded independent Route-read skeleton, but #3501 remains open and no training capability is claimed.
