# ORDER: standalone CUDA hardware correctness, kimi-2

Lab: kimi. Item: `uor-cuda-inverted-file` (#3475). Base: `7d8a0c264`. Owner request: get the CUDA package and OpenTelemetry integration ready for public presentation without touching Claude's model work.

## Goal

Establish actual AOT/CUDA correctness on one RTX 5090 before implementing matched performance evidence. Preserve a reproducible source/module/log lineage and every NOT_RUN.

## Scope

- `projects/uor-cuda-inverted-file/**`
- `docs/labs/uor-cuda-inverted-file/**`
- `docs/harness/prep/uor-cuda-inverted-file-sm120-2026-11-01.md`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-2/**`
- `docs/superpowers/plans/2026-11-01-cuda-public-readiness.md`
- `docs/roadmap/items/uor-cuda-inverted-file.md`
- Canonical `docs/roadmap/README.md` and `docs/roadmap/issues.json` when roadmap sync changes them
- Task-local pod logs under ignored `local/kimi/cuda-release-1/`

No model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, or production-deployment changes. No NVIDIA/upstream submission or public performance announcement in this unit.

## Cancel probe

Run `scripts/pod/uor-pod status`; lease only this session's GPU inside the shared caps. The pre-registration must receive non-author FIT before the lease. Exactly four CUDA entries are admitted. Stop before performance work on any compilation, admission, parity, refusal, or determinism failure. Preserve and classify every instrument failure separately from a source defect.

## Done when

- A non-author FIT record and the prep sheet precede GPU use.
- Clean AOT compilation/admission and three successful CUDA parity executions are sealed on one named RTX 5090/driver, or a concrete source defect is reported with host regression evidence and no pass claim.
- The actual source, PTX/cubin hashes, Toolkit/driver/GPU, commands, cost, and complete NOT_RUN scope are recorded and archived before pod teardown.
- Fresh exact-head review and protected delivery cover the resulting source/evidence. Performance, second architectures/drivers, external replication, and public release remain explicitly open.

## Stop and report

Stop after this correctness unit and its evidence are delivered. Do not benchmark, claim tuned performance, open an NVIDIA issue, or advertise the package in this unit. A passing correctness gate authorizes the separately planned comparator stage, not showtime by itself.
