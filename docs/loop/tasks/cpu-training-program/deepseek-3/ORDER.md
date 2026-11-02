# ORDER: CPU training program status refresh, deepseek-3

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3501 program registration). Base: `7b7534567` from fresh `origin/main`.

## Goal

Align the durable CPU training program and child-item status text with the merged and fresh-main-verified evidence from #3518, #3522 and #3523. Record what is now delivered, what remains unavailable, and the exact reason #3503 and #3506 remain downstream.

## Scope

- `docs/loop/tasks/cpu-training-program/deepseek-3/**`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-program.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No code, test, serving, corpus, GPU, model, bar, hold or owner-path change.

## Cancel probe

If `origin/main` no longer contains #3522/#3523 or the linked evidence reports do not support exact parameter-group and panel controls, stop rather than restating the work.

## Done when

- The shared-vector family row names the delivered tied-table, parameter-group and panel controls with direct REPORT links.
- Child state distinguishes completed component work from the still-missing pre-registered pilot and private trained-artifact controls.
- The quantization row records that the exact CPU/serving export-gap evaluator now exists while quantization-aware updates and float-shadow comparison remain open.
- The pilot item states the current readiness matrix without claiming a run or changing its dependencies.
- Local PR checks, formatting and full loop gates pass; a fresh exact-head review approves the documentation.
