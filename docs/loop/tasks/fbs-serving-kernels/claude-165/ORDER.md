# ORDER: fbs-serving-kernels, claude-165 (the snap's exact comparison without squares)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `1766883f4`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-164 brought decode on the M1 to 749–752 tok/s. A `sample` profile of that binary's plain decode (8 s, 3,158 samples at the top of the stack) shows where the rest goes:
- `stack_mul_u128`: 1,919 samples, about 61%. The call tree shows it inside `stack_snap_select`, through `snap_difference_positive`.
- `stack_snap_select` itself: 668.
- `stack_square`: 131.
- `stack_hamilton`: 81.
- the writer: 69.

So the icosian snap's exact `ℤ[φ]` comparison is now most of a served token.

**The comparison.** `snap_difference_positive(da, db)` decides the sign of `P + D √5`, with `P = 2 da + db` and `D = db`.
- When `P` and `D` have the same sign, it answers at once.
- When they differ, it squares both, each through a radix-16 table product.

**This unit decides most opposite-sign cases without the squares.**
- Two rational bounds, `2289/1024 < √5 < 1145/512`, decide whether `|P| < |D| √5`. The bound products are computed by shifts and additions.
- Only when `|P| / |D|` falls between the bounds, a band 0.04% wide, does it fall back to the squares.
- The answer is exact. The bounds are proved, √5 is irrational so `P² = 5 D²` never holds, and the fallback is the existing exact rule.

This is another step toward done-when 2 and 3, not their completion. The item stays `ready`.

## Cancel probe (run first)

Run at `1766883f4`:
- **No open PR** touches `stack/kernels.rs`.
- **Only `stack_snap_select` calls `snap_difference_positive`**, at `kernels.rs:1077`. The test module also calls it.
- **Both bounds hold:**
  - `2289² = 5,239,521 < 5 × 1024² = 5,242,880`, so `2289/1024 < √5`;
  - `1145² = 1,311,025 > 5 × 512² = 1,310,720`, so `1145/512 > √5`.

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **The comparison is exact.** The new `snap_difference_positive` equals the old square rule, kept in the tests, on:
   - every `(da, db)` in `[-300, 300]²`;
   - 1,000,000 random pairs at the full bounds (`|da| <= 2^35`, `|db| <= 2^34`);
   - pairs at and around √5's continued-fraction convergents, at many scales, where `|P| / |D|` lies inside the band.
2. **The snap is exact.** `stack_snap_select` equals a full 120-root scan under the old square rule, on claude-164's query sets.
3. **Nothing else moves.** The whole `uor-r4-integer` library suite passes. `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
4. **Speed, on this M1, on claude-126's seed-1 export.** The new `fbs-toks --parts` runs alternately with claude-164's binary (sha256 `b0ed9537…8270`), two runs each. Replies are identical to claude-163's and claude-164's. Measured: decode tok/s, the parts, and a `sample` profile of the new binary beside claude-164's.
5. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **Any pair in done-when 1 differs.** The unit then stops and finds why, and never ships an approximate comparison.
- **The audit refuses the shift-and-add bounds** (an LLVM fold into `mul`). Then the terms go behind `black_box` or a call boundary, as `snap_add_square_terms` already does.

No step waits on Casey.
