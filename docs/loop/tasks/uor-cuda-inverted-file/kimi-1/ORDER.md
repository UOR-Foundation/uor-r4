# ORDER: standalone UOR inverted-file CUDA package, kimi-1

Lab: kimi. Item: uor-cuda-inverted-file (#3475). Base: `3d39b0c720dbe95693efe8aea93b3873247a7ba1`.

## Goal

Create an independently buildable CUDA C++17 CSR inverted-file example and PTX admission gate for later community review, without touching model, API, worker, engine, or Claude-owned work.

## Scope

- `projects/uor-cuda-inverted-file/**`
- `docs/labs/uor-cuda-inverted-file/**`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-1/**`
- `docs/superpowers/specs/2026-10-31-cuda-inverted-file-package-design.md`
- `docs/superpowers/plans/2026-10-31-cuda-inverted-file-package.md`
- `docs/roadmap/items/uor-cuda-inverted-file.md`

No other paths. No model training, GPU evaluation, paid compute, production deployment, generated binary, or NVIDIA/upstream submission.

## Cancel probe

The fixed codebook must have 16 roots and 256 cells. The public CUDA inventory is exactly four entries: `uor_if_build`, `uor_if_update`, `uor_if_probe`, `uor_if_full_scan`. If `main` advances, merge it forward and re-check affected behavior. Stop on any requirement to edit model, API, worker, engine, Claude, or `.github/**` files.

## Done when

- Admission and CPU parity/refusal/fault tests pass, including 64 incremental updates and removal/reinsertion.
- Four CUDA operations and the Driver API wrapper are present; unavailable nvcc/GPU is reported as NOT_RUN, not a pass.
- Provenance, README, CITATION, CODE_OF_CONDUCT, RESULTS, and the canonical lab record distinguish host evidence from unrun GPU work.
- A clean copy passes host tests without a UOR-R4 checkout, followed by fresh exact-head review and protected merge verification.

## Stop and report

Stop after package verification, review, and merge. Do not start public benchmarking, an NVIDIA issue, a Foundation Sandbox application, or an OpenTelemetry implementation in this unit.
