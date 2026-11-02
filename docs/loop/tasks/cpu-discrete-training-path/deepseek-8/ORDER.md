# ORDER: repair proposal-scoring status and Route header controls, deepseek-8

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `9c314cc8c`. Repair of [deepseek-7](../deepseek-7/ORDER.md).

## Finished before this repair

- `6a80b7ba9` added the proposal-scoring ORDER alone.
- `9b4aefbde` added exact baseline/virtual scoring and tests.
- `39dffe87c` recorded the full suite, benchmark and local gates.
- A fresh exact-head review found no arithmetic divergence, but requested numeric status preservation, the complete Route header contract and missing writer/refusal controls.

## Repair scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-8/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, model mutation, training update, corpus/GPU run or language claim.

## Done when

- Scorer errors carry numeric statuses 2, 4, 8, 9 or 12 in the correct decision order.
- Route scorer construction rejects nonempty matrices, wrong source identities, unsupported objectives and nonzero header padding.
- Controls pin baseline-all-ineligible status 8, mode mismatch status 9, inventory refusal status 4 and the zero-candidate sentinel path.
- Full release tests, clippy, formatting, ratchets and loop gates pass at the repaired head.
- A new fresh-context adversarial review approves the exact repaired head.
