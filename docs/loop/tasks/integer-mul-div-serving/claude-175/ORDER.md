# ORDER: integer-mul-div-serving, claude-175 (the audits allow integer multiply and divide)

Lab: claude. Item: [integer-mul-div-serving](../../../../roadmap/items/integer-mul-div-serving.md). Base: `48435fd53`.

## Goal

This unit is the **audit half** of the item's done-when 1. The docs half is the next unit (claude-176). The item's words:

> - The native and WASM audits allow integer multiply- and divide-class instructions, and keep refusing every float instruction.
> - The native audit keeps refusing `fmov` for now. Float and vector values share one register file, so it cannot tell an integer move from a float one. `fbs-serving-vector-kernel` may revisit this.
> - Host tests show a float instruction still fails each verifier, and an integer `mul` or `udiv` now passes. The Rust verifiers are tested in `tests/m5_full_path_cost_controls.rs`, and each audit script in its own `--self-test`: the WASM audit already has one, and the native audit gains one.

D80 and D81 amend CHARTER §3.6: the served artifact may use the hardware integer multiply and divide, and still has no float or transcendental instruction.

## Cancel probe (run first)

Run at `48435fd53`:
- **No other open PR** touches `scripts/audit_zero_matmul_serving.py`, `scripts/audit_zero_matmul_wasm.py`, `report_output.rs` or `tests/m5_full_path_cost_controls.rs`.
- **The audits still refuse what §3.6 now allows:**
  - `report_output.rs` lists `"udiv",` among its forbidden mnemonics;
  - the native audit's Class I pattern starts `mul|madd`;
  - the WASM audit fails a symbol on any multiply or divide (`elif result["mul"] or result["div"] or result["float"]`, line 597).
- **The `wasm32-unknown-unknown` target is installed**, so the browser build can be audited.

All held. **Not cancelled** (instrument: none needed).

## Decisions taken here (stop and decide)

1. **Only the FBS stack's serving step is relaxed.**
   - The Rust verifiers serve only the stack (their callers are `uor-r4-stack.rs`, the `stack` re-exports and the m5 tests).
   - The native script relaxes only in `--stack` mode.
   - The WASM script relaxes only the stack's serving surface (`SERVING_WASM_SYMBOLS`).
   - The retained `IntegerModel` and Hopf symbols stay strict. They need no multiply, and the docs report their zero counts.
2. **Only scalar, general-register forms are allowed:**
   - AArch64 `mul`, `madd`, `msub`, `mneg`, `smull`, `umull`, `smaddl`, `umaddl`, `smsubl`, `umsubl`, `smnegl`, `umnegl`, `smulh`, `umulh`, `sdiv` and `udiv`, with `x`/`w` operands;
   - the x86-64 general-register multiply and divide families;
   - the compiler-rt integer helpers (`__multi3`, `__muldi3`, `__mulodi4`, `__udivti3`, `__divti3`, `__umodti3`, `__modti3`, `__udivdi3`, `__divdi3`);
   - WASM `i32`/`i64` `mul`, `div_*` and `rem_*`.

   **Every vector form stays refused,** for two reasons:
   - vector registers are shared with float (the `fmov` reason);
   - dot-product and matrix multiply-accumulate instructions (`sdot`, `udot`, `usdot`, `sudot`, `smmla`, `ummla`, `usmmla`) are matrix-product forms under §3.3. The item's own rule says "single exact products, never a matrix product".

   `fbs-serving-vector-kernel` may revisit vector integer multiply.
3. **Counts are still reported.** The scripts report how many multiply and divide instructions the step uses. They are allowed, not hidden.
4. **`fmov` and float compiler-rt helpers stay refused,** as do indirect calls.
5. **The public name `verify_step_objdump_zero_mul_zero_float` is kept** (its callers and re-exports would all move); its doc comment says what it now checks.

## Files in scope

This unit touches:
- `crates/uor-r4-integer/src/report_output.rs`
- `crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`
- `scripts/audit_zero_matmul_serving.py`
- `scripts/audit_zero_matmul_wasm.py`
- `docs/research/notebook/m4-serving.md` (one row)
- `docs/loop/tasks/integer-mul-div-serving/claude-175/**`

Only if a caller's doc comment states the old rule: `crates/uor-r4-integer/src/bin/uor-r4-stack.rs`, `stack/mod.rs`, `stack/session.rs`.

## Done when

1. **Rust verifiers** (`verify_step_disassembly_text` and its objdump wrapper):
   - pass a scalar `mul`, `madd`, `umulh`, `sdiv` and `udiv` (both otool and GNU syntax) and a `bl ___udivti3`;
   - still fail a vector `mul.4s` / `mul v0.4s`, `udot`, `smmla`, `fadd`, `fmov` and a `bl` to a float compiler-rt helper.

   Tests in `tests/m5_full_path_cost_controls.rs`, each shown to fail with the change set aside.
2. **The native script, `--stack` mode:** the same allowed and refused sets. `--self-test` runs every sentinel, including the new stack-mode cases, and exits 0. The other modes' patterns are unchanged.
3. **The WASM script:**
   - a stack serving symbol with `i64.mul` and `i64.div_u` passes and reports both counts;
   - one with a float instruction fails;
   - a legacy symbol with `i64.mul` still fails.

   All three are in `--self-test`.
4. **The real builds still pass:**
   - the release `uor-r4-stack` binary under `audit_zero_matmul_serving.py --stack`;
   - its own cost check (the objdump verifier);
   - the WASM serving module under `audit_zero_matmul_wasm.py --call-graph`, if it builds here.
5. **Checks:**
   - the `uor-r4-integer` tests that cover the verifiers, run with `--no-fail-fast`;
   - clippy and fmt;
   - `loop gates --full`;
   - a notebook row and a REPORT.

## Stop and ask if

Stop and decide, never wait:
- If a real build fails an audit for a reason other than a now-allowed scalar multiply or divide, it is a finding: record it, and do not widen the allowed set to hide it.
- If telling scalar from vector needs more than the operand registers (`x`/`w` against `v`/`q`/`d`/`s`/`h`/`b`), keep the instruction refused and note it.
