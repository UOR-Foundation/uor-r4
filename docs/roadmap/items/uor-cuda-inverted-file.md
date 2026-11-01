---
id: uor-cuda-inverted-file
title: Package a standalone UOR inverted-file CUDA example
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [projects/uor-cuda-inverted-file/**, docs/labs/uor-cuda-inverted-file/**, docs/loop/tasks/uor-cuda-inverted-file/kimi-1/**, docs/superpowers/specs/2026-10-31-cuda-inverted-file-package-design.md, docs/superpowers/plans/2026-10-31-cuda-inverted-file-package.md, docs/roadmap/items/uor-cuda-inverted-file.md]
---

## Why it matters

The CSR bucketed writer and PTX admission gate are the most transferable parts of UOR-R4's custom CUDA work. A small independent package lets others evaluate an exact integer inverted-file index and a mechanical instruction-budget check without adopting the model or its artifact schemas.

## What is missing

**Host package and clean-copy verification are complete.** The CPU oracle, four CUDA operations, admission and negative controls, provenance, and clean-clone evidence exist. AOT compilation and GPU parity remain **NOT_RUN** because no compatible `nvcc`/GPU is available on this machine. A later pre-registered hardware run is still required before release or upstream submission.

## Done when

- Admission rejects forbidden instructions and inventory drift, with comment handling and negative controls.
- CPU and CUDA behavior match for CSR build/update/probe/full-scan where a compatible GPU is available; otherwise CUDA compilation and execution are explicitly NOT_RUN.
- The 64-move incremental update matches a fresh build, and non-root endpoints, stale/corrupt CSR, over-pruning, and tie cases are covered.
- The standalone package builds from a clean copy with no UOR-R4 dependency, and README, provenance, evidence, and NOT_RUN scopes are accurate.
- A fresh exact-head review approves protected delivery. No release or NVIDIA submission occurs in this unit.

## How to start

- Implement `projects/uor-cuda-inverted-file` from the approved design and plan.
- Keep `crates/**`, model artifacts, API/worker/engine code, `.github/**`, and Claude paths out of scope.
- Run host tests and a clean-clone build before review. Additional hardware benchmarking is a separate pre-registered unit.

## Rules that apply

- **Preservation:** no model, artifact, Claude, API/worker/engine, generated binary, or production deployment changes.
- **Evidence:** no general exact-kNN, performance, portability, energy, Faiss-comparison, or language-capability claim.
- **Rust first/CUDA C++:** no Python or new third-party dependency; CUDA C++17 and Make are sufficient.

## Sources

- Issue #3475 and `docs/labs/uor-cuda-inverted-file/`
- `crates/uor-r4-discrete/src/bucket_source.rs` and `build.rs`
- `docs/labs/m1-faster-writer/README.md`
- `docs/superpowers/specs/2026-10-31-cuda-inverted-file-package-design.md`
- `docs/superpowers/plans/2026-10-31-cuda-inverted-file-package.md`
