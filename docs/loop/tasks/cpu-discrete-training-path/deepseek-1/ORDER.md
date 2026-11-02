# ORDER: independent CPU Route read, deepseek-1

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f7f8773f8`.

## Goal

Add the first independent read slice required by #3501: parse a lawful four-table discrete artifact, construct its codebook and Route maps, and reproduce `IntegerDiscreteModel` Route query vectors on deterministic fixtures. Keep Flock, Relation, proposals and apply disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, training update, corpus/GPU run or language claim.

## Done when

- CPU parsing binds schema, shape, table kinds/lengths and Route-only initialization.
- The independent Route read matches `IntegerDiscreteModel::evaluate_prefix` on one- and multi-stage deterministic Route artifacts.
- Relation/Flock artifacts are explicitly refused by this Route-only slice.
- Malformed map lengths, route actions and non-orbit codebooks are refused.
- A release benchmark records Route-read rows/s on the Ryzen 9 9950X.
- REPORT/notebook place the slice as component parity only; proposals/apply remain open.
