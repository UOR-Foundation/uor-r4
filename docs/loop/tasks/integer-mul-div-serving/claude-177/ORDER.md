# ORDER: integer-mul-div-serving, claude-177 (the kernels use the hardware multiply and divide)

Lab: claude. Item: [integer-mul-div-serving](../../../../roadmap/items/integer-mul-div-serving.md). Base: `7120c4351`.

## Goal

The item's done-when 2 and 3: the FBS stack's products and quotients use the hardware integer multiply and divide (D80, D81), exactly, and the speed is measured against claude-169's 3,042–3,044 tok/s.

- **Bodies only.** `stack_mul_u128`, `stack_mul_u64`, `stack_square`, `stack_div_u64`, `stack_div_u128` and `mul_by_u64`'s callers (the norm, the pointer mixture) get their hardware form; `stack_hamilton` takes its 16 products as widening multiplies. Every caller (the norm, pointer, flock read, snap squares) follows. `mul_by_i32` and the dense read's query tables stay as they are: their `[i64; 16]` type runs through `session.rs`, and an FBS step does not run that read.
- **Names kept.** The `#[inline(never)]` kernels stay symbols, as the audit's mandatory list requires.
- **The old table code becomes the test reference.** Each new kernel is tested equal to it, on edge and random inputs.

## Cancel probe (run first)

Run at `7120c4351`:
- `kernels.rs` still builds `multiples_u128` tables in `stack_mul_u128` and `stack_hamilton`.
- claude-175's audits (merged) allow scalar integer multiply and divide in `--stack` mode and on the WASM serving surface.
- The baseline exists: claude-169's `fbs-toks` (sha256 `8a7a83c7…805a`) at `local/claude/claude-169/fbs-toks`, and claude-126's seed-1 artifact (sha256 `314c3909…fdb305`) at `local/claude/claude-164/.../seed-1/artifact.uorlut`.

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs` (bodies and tests)
- `crates/uor-r4-integer/src/stack/tests.rs`, if a session-level check needs it
- `docs/research/notebook/m4-serving.md` (one row)
- `docs/loop/tasks/integer-mul-div-serving/claude-177/**`
- The measurement lines claude-176 listed, only where this unit's audit result makes them false

## Done when

1. Every new kernel equals the old one on edge values and at least 100,000 random inputs. Each test fails with a deliberately wrong kernel.
2. Replies on claude-126's export are byte-identical to claude-169's (SHA-256 `97c8ad41…`).
3. `audit_zero_matmul_serving.py --stack` passes on the release `uor-r4-stack` and reports the multiplies and divides it allows. The WASM serving audit passes 7 of 7. `uor-r4-stack`'s own objdump check passes.
4. Speed: `fbs-toks --parts`, claude-169's binary against this unit's, run alternately, two runs each. Report decode tok/s, the parts, and a `sample` profile with its total read from the thread line. Sealed to the HF store.
5. m5 and `uor-r4-integer` stack tests pass with `--no-fail-fast`; fmt; `loop gates --full`; a notebook row and a REPORT.

## Stop and ask if

Stop and decide, never wait:
- A reply changes: the kernel is wrong. Fix it; never accept it.
- The audit refuses an instruction the compiler emits for these bodies (a vector form, `fmov`): keep that kernel's old body and record why.
