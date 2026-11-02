# ORDER: repair CPU Flock header validation and evidence, deepseek-3

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `3863dd75f`. Repair of [deepseek-2](../deepseek-2/ORDER.md).

## Finished before this repair

- `f88ff5470` added the deepseek-2 ORDER alone.
- `569c791d5` and `33a27b933` opened the REPORT with a valid shape.
- `a66b60fd2` added the Flock slice, tests, measurements and evidence.
- `54aa328ce` recorded the checks, ratchets and local gates.
- A fresh exact-head review at `54aa328ce` found no arithmetic divergence, but requested two evidence/validation repairs: enforce the remaining host header contract and stop describing the generic fixture as the published C2 copy-successor fixture.

## Repair scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-3/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, training update, corpus/GPU run or language claim.

## Done when

- The parser rejects nonzero header-alignment padding, nonempty matrices, wrong source snapshot/operator/composition identities and missing or unknown objective policies, matching `IntegerDiscreteModel` parsing.
- Serialized controls mutate each rejected header field independently.
- REPORT/notebook identify the first fixture as a generic serialized two-level parity fixture unless an actual C2 fixture is used.
- Release tests, clippy, formatting, ratchets and loop gates pass at the repaired head.
- A new fresh-context adversarial review approves the exact repaired head.
