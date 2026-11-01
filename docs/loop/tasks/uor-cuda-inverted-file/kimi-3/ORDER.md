# ORDER: matched CUDA performance and replication, kimi-3

Lab: kimi. Item: #3475. Base: current `origin/main` including #3483/#3485 correctness. Owner request: get the CUDA package ready for public presentation.

## Goal

Add a genuinely optimized, exact full-scan comparator and measure it against the bucket probe with separate timing scopes, then repeat correctness/performance on additional named hardware where available. No release or upstream submission without the declared gates and independent replication.

## Scope

- `projects/uor-cuda-inverted-file/**`
- `docs/labs/uor-cuda-inverted-file/**`
- `docs/harness/prep/uor-cuda-inverted-file-performance-2026-11-01.md`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-3/**`
- `docs/roadmap/items/uor-cuda-inverted-file.md`
- Canonical `docs/roadmap/README.md` and `docs/roadmap/issues.json`
- Task-local raw evidence under ignored `local/kimi/cuda-performance-*/`

No model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, production deployment, or upstream submission. Do not alter the four public CUDA entry names or their result semantics.

## Cancel probe

The optimized full scan must preserve all 504 correctness rows, with the same integer arithmetic, admission, candidate set, and output order. Any compile/admission/parity failure stops timing. Every measured row must be associated with source/module/hardware identity and a declared timing scope. No historical model-writer ratio is a package result.

## Done when

- Non-author FIT precedes each GPU measurement configuration.
- An optimized full-scan comparator and a raw benchmark exist, with exact parity on the guarded grid.
- Kernel-only, direct API, and complete-caller times, warmup/repetition counts, resource/occupancy data, and raw per-run JSON are retained for both operations across declared sizes and K.
- Additional architecture/driver results are recorded where actually run, with every remaining NOT_RUN explicit. A non-author replication and external-consumer check are separately identified.
- Fresh exact-head review and protected delivery precede any release/advertising decision.

## Stop and report

Stop after matched measurements, replication, and evidence delivery. Do not open an NVIDIA issue, post performance claims publicly, or submit upstream in this unit. Any unsupported matrix cell stays NOT_RUN, never inferred.
