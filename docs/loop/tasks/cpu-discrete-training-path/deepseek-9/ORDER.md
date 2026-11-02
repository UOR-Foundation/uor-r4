# ORDER: exact single-coordinate proposal apply, deepseek-9

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4fdec6487`.

## Goal

Apply one exact, validated proposal to in-memory discrete state with CUDA-matched epoch, mode, inventory and strict-improvement checks. Keep BVG-K joint application and deterministic restart/resume disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-9/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, corpus/GPU run or language claim.

## Done when

- Apply refuses stale epochs (3), mode mismatches (9), inventory/lane errors (4) and non-improving proposals (5).
- Lane 0 is a no-op that leaves state and epoch unchanged.
- Route, Gate and Table lane moves mutate exactly the validated coordinate and increment epoch once.
- Applies use the same generator, composition, root-transform and sibling-gate mappings as scoring.
- Receipts expose kind, index, lane, before/after loss and epoch.
- Full release tests, clippy, formatting, ratchets and loop gates pass.
- REPORT/notebook classify this as atomic-free single-coordinate apply; BVG-K and resume remain open.
