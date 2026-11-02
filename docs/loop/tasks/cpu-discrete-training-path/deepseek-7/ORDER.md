# ORDER: exact discrete proposal scoring, deepseek-7

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `9c314cc8c`.

## Goal

Score all 16 discrete proposal lanes from immutable baseline state using the exact read and writer paths, for both admitted objectives, and feed the merged chooser. Keep model mutation/apply, BVG-K and resume disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-7/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, model mutation, training update, corpus/GPU run or language claim.

## Done when

- The scorer reproduces baseline writer loss and token-rival eligibility against `IntegerDiscreteModel::observe_target` for nearest-address and occupied-rank objectives.
- Route action, Flock/Relation Gate trit and Table block moves each reproduce the equivalent serialized host state.
- Table moves affect both codebook and matching input-token blocks, and admission cells are recomputed.
- Baseline-ineligible rows are skipped; all-ineligible, candidate sentinel, read failure and checked arithmetic failure are explicit.
- Candidate losses feed the exact chooser, including Gate active-lane and Route/Table tie behavior.
- A release benchmark records proposal-scoring throughput on the Ryzen 9 9950X.
- REPORT/notebook place this as read-only proposal scoring; apply, BVG-K and resume remain open.
