# ORDER: repair apply/chooser state coherence, deepseek-10

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4fdec6487`. Repair of [deepseek-9](../deepseek-9/ORDER.md).

## Finished before this repair

- `f02142609` added the apply ORDER alone.
- `61e97fd00` opened the REPORT.
- `b694540ce` and `d317f3676` added and verified the first apply slice.
- `8ae12651d` recorded the full suite, ratchets and gates.
- A fresh review found that Route/Table apply updated the scoring model but left the chooser's route/codebook copies stale.

## Repair scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-10/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

## Done when

- Successful Route/Table apply refreshes the chooser route/codebook state.
- Sequential Route and Table applies keep scoring and chooser state coherent.
- Route and Table mutation tests pin exact composition/root-transform results.
- Refused applies leave model and epoch unchanged.
- Epoch `u64::MAX` refuses an otherwise improving move with status 5.
- Full release tests, clippy, formatting, ratchets and loop gates pass.
- A fresh adversarial review approves the exact repaired head.
