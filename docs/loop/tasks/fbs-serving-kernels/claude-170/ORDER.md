# ORDER: fbs-serving-kernels, claude-170

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `c910dd7e5`.

## Goal

The item's done-when reads:

> 1. **The split:** each layer's gathers, core, gated product and requantization timed on the M1, by extending `measure_fbs_step_stages` and `fbs-toks --stages`. The parts sum to the layer within 10%.
> 2. **A vector kernel for the largest part,** written for D11 (CHARTER §3.6: integer, no multiply instruction; products through its tables), in `stack/fbs_simd.rs`, behind a runtime switch (no new cargo feature), with the scalar path kept for WASM and other targets.
>    - A host test holds every token, kept cell, query and session state equal to the scalar `step_native` over a synthetic trunk and a stored artifact.
>    - The kernel is in the audit's kernel list, and `scripts/audit_zero_matmul_serving.py --stack` passes on the release binary.
> 3. **Speed:** `fbs-toks` decode tokens/s on the M1 with and without the kernel, on claude-126's export, with the same replies.
> 4. One notebook row and a REPORT. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/fbs_simd.rs`
- `crates/uor-r4-integer/src/stack/simd.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `scripts/audit_zero_matmul_serving.py`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
