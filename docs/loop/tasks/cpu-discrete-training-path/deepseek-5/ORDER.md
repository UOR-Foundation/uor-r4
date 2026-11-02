# ORDER: exact discrete proposal chooser, deepseek-5

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `d65054c4d`.

## Goal

Port the exact read-only `discrete_choose` decision surface: Gate strict improvement and ordinal ties, Route/Table resulting-port ties, objective sensitivity, proposal words and input refusal. Keep model mutation, full virtual writer scoring, BVG-K apply and resume disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-5/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, model mutation, training update, corpus/GPU run or language claim.

## Done when

- The 16-lane chooser reproduces `discrete_choose` words for Gate, Route and Table proposal kinds.
- Gate choice uses strict improvement only and keeps the lower ordinal on equal losses; inactive Gate lanes never affect sensitivity.
- Route/Table equal-loss ties choose the lower resulting port, not merely the lower lane.
- All-worse proposals remain sensitive and all-tied proposals are insensitive.
- Invalid lane inventories, Gate counts, Route actions, Table roots, proposal kinds/indices and mode/kind mismatches fail closed.
- A release benchmark records chooser decisions/s on the Ryzen 9 9950X.
- REPORT/notebook place this as proposal-selection parity only; scoring, apply and resume remain open.
