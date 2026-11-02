# ORDER: repair proposal-chooser tie evidence, deepseek-6

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `d65054c4d`. Repair of [deepseek-5](../deepseek-5/ORDER.md).

## Finished before this repair

- `8495d5d94` added the chooser ORDER alone.
- `270fa968f` opened the REPORT.
- `178553524` added the chooser and typed proposal decisions.
- `d6a493c6c` recorded full tests, ratchets and local gates.
- A fresh exact-head review found no arithmetic divergence, but requested independent Route/Table port vectors and controls for lane-0 ties and lower-loss selection.

## Repair scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-6/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, model mutation, scoring update, corpus/GPU run or language claim.

## Done when

- Route resulting-port ties compare against an independently pinned 16-lane vector rather than the implementation under test.
- Table resulting-port ties compare against an independently pinned 16-lane root vector.
- Equal Route/Table current/candidate losses keep lane 0 even when a candidate has a lower resulting port.
- A strictly lower-loss candidate wins independently of the better-port tie among other lanes.
- Full release tests, clippy, formatting, ratchets and loop gates pass at the repaired head.
- A new fresh-context adversarial review approves the exact repaired head.
