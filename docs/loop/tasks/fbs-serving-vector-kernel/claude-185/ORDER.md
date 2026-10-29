# ORDER: fbs-serving-vector-kernel, claude-185

Lab: claude. Item: [fbs-serving-vector-kernel](../../../../roadmap/items/fbs-serving-vector-kernel.md). Base: `c95dfaea6`.

## Goal

The item's done-when reads:

> 1. **The map.** Its gap 3 and R3 record that R3 was met with exact structured kernels (claude-164 to claude-169, about 5.2 ns a value). The per-value claim is restated as an open question.
> 2. **A faster exact change** to the snap or the Hamilton product: fewer golden candidates, or a vector kernel the audit allows.
>    - A host test holds the changed kernel equal to the old one.
>    - On claude-126's export, every reply is byte-identical.
>    - `scripts/audit_zero_matmul_serving.py --stack` passes on the release binary.
> 3. **Speed:** `fbs-toks --parts` against the previous unit's sealed binary, run alternately, on this M1.
> 4. One notebook row and a REPORT for each unit. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/fbs_simd.rs`
- `crates/uor-r4-integer/src/stack/simd.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `scripts/audit_zero_matmul_serving.py`
- `docs/model/pipeline-map.md`
- `docs/loop/tasks/fbs-serving-vector-kernel/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-vector-kernel.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
