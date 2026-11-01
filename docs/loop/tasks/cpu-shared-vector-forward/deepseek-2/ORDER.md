# ORDER: harden CPU FBS forward admission, deepseek-2

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `070e35b59` from fresh `origin/main`. Repair after review at `cf8766ba4`.

## Goal

Close the reviewed admission gaps in the merged no-read CPU FBS forward: reject unexecuted acquisition modes, validate route bounds, enforce context exhaustion, and extend the test record.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-forward/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`

No serving-source edits, training run, GPU run, artifact change, or model-capability claim.

## Cancel probe

Stop if the current serving model cannot expose raw-gate, raw-transition or retention-rate shape flags. `StackShape` exposes all three, so the checks are possible.

## Done when

- `CpuFbsForward` accepts only snapped, rotated, decay-rate, recurrence-only FBS models.
- Route atom bounds and context exhaustion are checked before execution.
- Raw and retention-rate artifact controls are refused.
- The one-, three- and six-layer full-query parity suite still passes.
