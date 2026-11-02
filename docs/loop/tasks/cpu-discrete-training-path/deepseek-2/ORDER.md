# ORDER: independent CPU Flock read, deepseek-2

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `3863dd75f`.

## Goal

Add the next independent read slice required by #3501: parse a lawful four-table Flock artifact and reproduce the exact active-leaf, level-aware `IntegerDiscreteModel` read on deterministic fixtures. Keep Relation, proposals, apply, BVG-K and checkpoint resume disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, training update, corpus/GPU run or language claim.

## Done when

- CPU parsing binds schema, shape, table kinds and lengths, and exactly `read_mode=flock`, `read_kept=1..=8`, `read_sweeps=0`, `read_initialization=active_leaf_v1`.
- The Flock binder plan retains each node's level, first and last source-token positions, including unequal-level frontier ties.
- The independent read matches `IntegerDiscreteModel::evaluate_prefix` on serialized multi-level Flock artifacts with nonzero level gates and multiple output blocks.
- Zero-gate fallback and nonzero-gate transport are separately pinned, as are Route and Relation refusal.
- Malformed gate count/trits, `read_kept`, sweeps, spans, alignment, padding, maps and codebooks are refused.
- A release benchmark records Flock-read rows/s on the Ryzen 9 9950X.
- REPORT/notebook place the slice as component parity only; Relation, proposals, apply, BVG-K and resume remain open.
