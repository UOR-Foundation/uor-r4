# ORDER: exact CPU discrete writer core, deepseek-1

Lab: deepseek. Item: [cpu-discrete-writer-core](../../../../roadmap/items/cpu-discrete-writer-core.md) (#3502). Base: `03800ffd2` from fresh `origin/main`.

## Goal

Create the isolated CPU training crate and its first independently testable unit: exact codebook decoding, cell admission, full-vector token ranking, top-1/top-8, target-admission reporting and a deterministic throughput benchmark.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`, `Cargo.lock`
- `docs/loop/tasks/cpu-discrete-writer-core/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`
- `docs/start/FILEMAP.md`
- `docs/roadmap/items/cpu-discrete-writer-core.md`

No source edit to `crates/uor-r4-discrete`, `crates/uor-r4-float-train`, or `crates/uor-r4-integer`; those are reference inputs only. No corpus training, GPU run, model artifact, deployment, or generated binary.

## Cancel probe

Stop if an existing committed crate already provides the same exact CPU rank and admission API. The current repository has host oracle functions but no CPU training crate, so the unit stands.

## Done when

- The new crate builds CPU-only and is absent from the workspace default-members list.
- Checked codebook decoding, malformed-input refusal, cell admission, lower-token tie-breaking and full top-8 are covered by finite tests.
- Results match `writer_score_full_outer_l1` on deterministic queries and include target-admission/top-1/top-8 reporting.
- A release benchmark reports rows/s, elapsed time, thread count and a stable checksum on the Ryzen 9 9950X.
- The item and notebook record what is proven and what remains open.
