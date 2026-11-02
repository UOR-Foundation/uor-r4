# ORDER: exact CPU Hamming reads, deepseek-3

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `c411f64e4` from fresh `origin/main`.

## Goal

Extend the independently implemented CPU FBS forward to execute the artifact's Hamming copy and type reads with complete Q12 query, token and kept-cell parity against `IntegerStackSession::step_native`.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-forward/deepseek-3/**`
- `docs/research/notebook/cpu-training.md`

No serving-source edits, trained-artifact publication, corpus run, GPU run, training, or language claim.

## Cancel probe

Stop if the public artifact reader does not expose complete Hamming tables and per-depth read records. `read_hamming_uorlut01` exposes both the record and reads, so the unit stands. Do not copy private serving implementation or infer a read from token output.

## Done when

- The CPU forward loads and validates the Hamming record, copy heads, type heads and per-depth read records from artifact bytes.
- Stream reads after matching named layers and the readout read use the serving hook order, exact arithmetic, type-cache state and final Q12 conversion.
- Deterministic fixtures compare the complete Q12 query, token and kept cells against `IntegerStackSession::step_native` for no-read, copy-only, type-only and full-read models as supported by the current artifact writer.
- Negative controls distinguish the read paths and malformed unsupported read modes are refused.
- A release benchmark records the Hamming-read forward throughput on the Ryzen 9 9950X.
- Exact turn memory remains explicitly open for the next isolated slice.
