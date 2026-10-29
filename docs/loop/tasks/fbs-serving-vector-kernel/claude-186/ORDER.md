# ORDER: fbs-serving-vector-kernel, claude-186

Lab: claude. Item: [fbs-serving-vector-kernel](../../../../roadmap/items/fbs-serving-vector-kernel.md). Base: `c5805400a`.

## Goal

The item's done-when reads:

> 1. **The map.** Its gap 3 and R3 record that R3 was met with exact structured kernels (claude-164 to claude-169, about 5.2 ns a value). The per-value claim is restated as an open question.
> 2. **A faster exact change** to the snap site, in either form, or to the Hamilton product. Either form means the snapped selection or the unsnapped `stack_rotation` (claude-185). The change can be fewer golden candidates, a faster exact square root or quotient, or a vector kernel the audit allows.
>    - A host test holds the changed kernel equal to the old one.
>    - Every reply is byte-identical, on claude-126's export and on dual-c131's `raw_both` artifact (claude-182).
>      - dual-c131's reply under `fbs-toks`'s default prompt is one token repeated (claude-185), so it cannot show a difference.
>      - On that artifact the check also compares something that can differ: the session's final state, the per-step query codes, or a prompt whose reply varies.
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
