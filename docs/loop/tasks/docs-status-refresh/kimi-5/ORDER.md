# ORDER: correct the current CUDA module inventory, kimi-5

Lab: kimi. Item: docs-status-refresh (#3380). Base: `7a1cbcffc80cb5840a28c9c909262486bfdbbf9c`.

## Goal

Correct `docs/model/gpu-and-cuda.md` so its ordinary and specialized CUDA kernel inventories match the current source and build script. No model, kernel, API, worker, deployment, or Claude-owned changes.

## Scope

- `docs/model/gpu-and-cuda.md`
- `docs/loop/tasks/docs-status-refresh/kimi-5/ORDER.md`
- `docs/loop/tasks/docs-status-refresh/kimi-5/REPORT.md`

No other file may change. No model/GPU run, paid compute, generated artifact, acceptance bar, technical rule, or production publication.

## Cancel probe

`discrete.cu` must contain 718 lines and 13 `extern "C" __global__` entries. `build.rs` must assemble nine modules: ordinary, decomposition, baseline-reuse, rows, rowsk, bvgk, bucket, wider-carrier, and within-cell-writer. Their declared entry counts must sum to 42. Stop if source or build inventory changes before the edit is reviewed; merge main forward instead.

## Done when

The guide lists all 13 ordinary entries including `discrete_carrier_select`, names all nine modules with their correct counts, and says nine cubins. `cargo test --locked -p xtask`, claim-wording, mdBook build/book-check, and `git diff --check` pass. A fresh exact-head documentation review approves before the protected merge.

## Stop and report

Stop after the corrected guide, task records, checks, review, and merge verification. Do not begin kernel cleanup, upstream implementation, or GPU benchmarking in this unit.
