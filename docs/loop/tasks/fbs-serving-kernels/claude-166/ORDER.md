# ORDER: fbs-serving-kernels, claude-166 (the Hamilton product by shared tables)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `acfb043a6`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-165 brought decode on the M1 to 1,169–1,217 tok/s. In its `sample` profile (1,132 top-of-stack samples), `stack_mul_u128` holds 487. Of those, 444 sit under `stack_hamilton` and 43 under the lanes, and `stack_hamilton` itself holds 59. So the Hamilton product is about 44% of a served token.

**`stack_hamilton(a, b)`** forms 16 products `a_i b_j` through `mul_i128`. Each product builds a 16-entry table of multiples of one operand (15 additions of 128-bit values), then reads it at the radix-16 digits of the other.

**This unit builds 4 tables, not 16.**
- **Shared tables:** each `|b_j|` gets one table, read at the digits of each `|a_i|` (`mul_by_u64`). The sign comes from the operands' signs, as `mul_i128` does.
- **Zero coordinates:** a zero `a_i` contributes four zero products, which are skipped. A snapped root has one zero coordinate (golden) or three (axis).
- **Same values:** each product is the same integer `|a_i| |b_j| mod 2^128`, so the four results are unchanged.
- **The orientation:** all three callers pass the snapped quaternion as `a`: the lanes' transition, the recurrence's output gate and the feed-forward's gated product. Its Q31 coordinates have few digits.

This is another step toward done-when 2 and 3. The item stays `ready`.

## Cancel probe (run first)

Run at `acfb043a6`:
- **No open PR** touches `stack/kernels.rs`.
- **The callers:** `stack_hamilton`'s three are `session.rs` lines 2845, 3349 and 3434, each with the snapped quaternion first.
- **The helpers exist:** `multiples_u128` and `mul_by_u64` already build a table and read it at a `u64`'s digits (`kernels.rs`, lines 144 and 1156).

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **Exactness.** The new `stack_hamilton` equals the old one (16 `mul_i128` products, kept in the tests) on:
   - 1,000,000 random pairs over the whole `i64` range, including `i64::MIN`, `0` and `±1`;
   - snapped-transition-shaped `a`, with zeros and few distinct magnitudes.
2. **Nothing else moves.** The whole `uor-r4-integer` library suite passes. `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
3. **Speed, on this M1, on claude-126's seed-1 export.** The new `fbs-toks --parts` runs alternately with claude-165's binary (sha256 `03c65108…a161`), two runs each. Replies are identical to claude-163's to claude-165's. Measured: decode tok/s, the parts, and a `sample` profile, with each `stack_mul_u128` sample tallied by caller.
4. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **Any pair in done-when 1 differs.** Then stop and find why.
- **The audit refuses the new code**, for example a `core::array` closure symbol, or an `fmov` from a packed table. Then rewrite it with explicit loops and `black_box`, as the existing kernels are written.

No step waits on Casey.
