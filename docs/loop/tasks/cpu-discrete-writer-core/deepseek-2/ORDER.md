# ORDER: public M1 discrete writer parity, deepseek-2

Lab: deepseek. Item: [cpu-discrete-writer-core](../../../../roadmap/items/cpu-discrete-writer-core.md) (#3502). Base: `707e5025d` from fresh `origin/main`.

## Goal

Close the stored trained-artifact observation gate using the public M1 release and its public DEV split. Extract the exact 19-table artifact's discrete codebook, rerun all 14,287 eligible DEV rows through the independent CPU writer, compare every row with `IntegerDiscreteModel`, and compare the aggregate to the published GPU observation. Repair the aggregate `top8_hits` double-count found during the probe.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`, `Cargo.lock`
- `docs/loop/tasks/cpu-discrete-writer-core/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-writer-core.md`

No serving-source edit, training update, model/corpus upload, pod, GPU run or language claim. Public artifacts remain outside git and are verified by SHA-256.

## Cancel probe

If the public seed-1 artifact does not hash to `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, or the public DEV tokens/mask do not reproduce 14,287 eligible rows, stop and record the identity failure. Do not substitute seed 2 or a different panel after parsing.

## Done when

- The tool binds the public seed-1 artifact, tokenizer, DEV token stream and mask by exact hashes.
- It extracts `discrete.table` from the artifact without editing serving source and constructs the independent `CpuWriterCore`.
- Every one of the 14,287 rows matches `IntegerDiscreteModel` exactly for read/query vector, admitted cells, top-8 and prediction.
- CPU aggregate target admission, top-1 and top-8 equal the published M1 seed-1 GPU observation `9,012`, `7,002` and `7,485`.
- `CpuWriterCore::summarize` counts each top-8 hit once; a regression test fails on the duplicated increment.
- A release benchmark records the full public-panel elapsed time, rows/s, thread count and stable checksum on the Ryzen 9 9950X.
- A notebook row and REPORT distinguish exact component parity from training or language evidence.
