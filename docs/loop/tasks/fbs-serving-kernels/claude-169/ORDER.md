# ORDER: fbs-serving-kernels, claude-169 (the Hamilton product by shifts where it can)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `86591e2f1`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-168 brought decode on the M1 to 2,448–2,449 tok/s. In its profile (453 samples on the thread), `stack_hamilton` holds 173 (38%).

**The structure.** `stack_hamilton(a, b)` builds 4 tables of multiples of `|b_j|` and reads them at the digits of `|a_i|`. Its `a` is always a snapped quaternion `stack_snap_rotation(raw, λ)`, and `λ` is a power of two in all three callers:
- `2^31` for the recurrence's output gate and the feed-forward's gated product (`session.rs` lines 2835 and 3339);
- `2^(30 − idx)`, with `idx` from 0 to 15, for the geometric lanes (line 3422).

A coordinate is `shift(λ a_c + λφ b_c, 1)`. Since `λ` is even, the rounding shift halves it exactly, so:
- an axis root's coordinate is `±λ`;
- a half-root's are `±λ/2`;
- a golden root's `1/2` coordinate is `±λ/2`.

Those are all powers of two, so their products with `|b_j|` are exact shifts. A golden root's other two coordinates, `±shift(λφ, 1)` and `±shift(λφ − λ, 1)`, are not.

**This unit computes each row by the cheapest exact means:**
- a zero `a_i` is skipped;
- `|a_i| = 2^k` gives `|b_j| << k` for every `j`;
- any other `|a_i|` gets one table of its own multiples, read at each `|b_j|`'s digits.

Each product is still `|a_i| |b_j| mod 2^128`, with the sign `mul_i128` gives.
- **Golden roots:** 2 tables, 8 digit reads and 4 shifts, in place of 4 tables and 12 reads.
- **Axis and half-roots:** no table at all.
- **Generic input:** at most 4 tables and 16 reads, as before.

This is another step toward done-when 2 and 3. The item stays `ready`.

## Cancel probe (run first)

Run at `86591e2f1`:
- **No open PR** touches `stack/kernels.rs`.
- **The three callers' `λ`** are as above: `1u64 << 31` twice, and `1u64 << (30 − idx)` in the geometric lane branch.
- **`shift(x, 1)` is `(x + 1) >> 1`** (`kernels.rs` line 56), so it is exact on even `x`.

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **Exactness.** The new `stack_hamilton` equals the old 16 products (`hamilton_by_products` in the tests) on:
   - claude-166's random pairs and snapped transitions;
   - snapped transitions of every root at every power-of-two `λ` from `2^15` to `2^31`, against random states;
   - inputs mixing power-of-two, zero and other coordinates.
2. **Nothing else moves.** The whole `uor-r4-integer` library suite passes. `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
3. **Speed, on this M1, on claude-126's seed-1 export.** The new `fbs-toks --parts` runs alternately with claude-168's binary (sha256 `b0106169…4780`), two runs each. Replies are identical to claude-163's to claude-168's. Measured: decode tok/s, the parts and a `sample` profile, with the total read from the thread line.
4. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **Any product differs.** Then stop and find why.
- **The audit refuses the new code.** Then keep the lanes scalar with `black_box`, as the existing kernels do.

No step waits on Casey.
