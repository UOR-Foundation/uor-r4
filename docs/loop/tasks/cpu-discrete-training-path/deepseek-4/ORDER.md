# ORDER: independent CPU Relation read, deepseek-4

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4706fb2ee`.

## Goal

Add the final read slice required by #3501: parse a lawful four-table Relation artifact and reproduce the exact active-leaf, multi-sweep relation read on deterministic fixtures. Keep proposals, apply, BVG-K and checkpoint resume disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-4/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, training update, corpus/GPU run or language claim.

## Done when

- CPU parsing binds `read_mode=relation`, `read_kept=1..=8`, `read_sweeps=1..=5`, `read_initialization=active_leaf_v1`, `blocks*stages` level gates plus five stages of relation gates.
- The read reproduces all five relation edge types and the per-sweep score update before relation-aware node ranking.
- A serialized multi-level fixture matches `IntegerDiscreteModel::evaluate_prefix` with nonzero relation gates, multiple sweeps and `read_kept>1`.
- Zero relation gates equal the corresponding Flock read when level gates and codebooks are equal.
- Route and Flock refusal, bad counts/trits/parameters, malformed spans/maps/codebooks/headers and prefix bounds are controlled.
- A release benchmark records Relation-read rows/s on the Ryzen 9 9950X.
- REPORT/notebook place the slice as component parity only; proposals, apply, BVG-K and resume remain open.
