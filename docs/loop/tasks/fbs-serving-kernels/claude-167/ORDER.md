# ORDER: fbs-serving-kernels, claude-167 (table builds from four guarded doublings)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `1e852ebd3`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-166 brought decode on the M1 to 1,406–1,410 tok/s. In its profile (1,575 samples), `stack_hamilton` holds 530 (34%), mostly table building and digit reads, and `stack_snap_select` holds 517 (33%).

**The table builders.** Every multiplier-free product in serving reads a table of the sixteen multiples `k a`. `multiples_u128` and `multiples_u64` build it by fifteen chained additions. Each passes through `black_box` and an XOR with an opaque zero, so the compiler cannot fold the chain into a multiply, but every step goes through memory. Their callers:
- `stack_mul_u128` and `stack_mul_u64`;
- `stack_hamilton`;
- the pointer mixture;
- two more `kernels.rs` sites.

**This unit builds each table from four guarded values.** `a`, `2a`, `4a` and `8a` are made by shifts, each behind `black_box`, and the eleven other multiples are sums of those (`3a = 2a + a`, `9a = 8a + a`, and so on).
- **No fold:** the four guarded values are opaque to the optimizer, so no sum of them can be recognized as a multiply.
- **Same values:** each entry is still `k a` modulo `2^128` or `2^64`.
- `stack_hamilton` also stops zeroing its four tables before filling them. The fresh code review of claude-166 found that `multiples_u128` overwrites every entry anyway.

This is another step toward done-when 2 and 3. The item stays `ready`.

## Cancel probe (run first)

Run at `1e852ebd3`:
- **No open PR** touches `stack/kernels.rs`.
- **Both builders still chain** fifteen guarded additions (`kernels.rs` lines 144 and 157).

Both held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **Exactness.** Each builder's every entry equals `k a` (by `wrapping_mul`, in the test) for 1,000,000 random values across the whole range, the edges (`0`, `1`, the maximum) and every power of two. The existing product tests still pass.
2. **Nothing else moves.** The whole `uor-r4-integer` library suite passes. `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
3. **Speed, on this M1, on claude-126's seed-1 export.** The new `fbs-toks --parts` runs alternately with claude-166's binary (sha256 `be139a40…ac64`), two runs each. Replies are identical to claude-163's to claude-166's. Measured: decode tok/s, the parts and a `sample` profile.
4. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **The audit refuses the new builders.** For example, a packed vector add of the sums would need a general-to-vector transfer (`fmov`). Then guard the outputs too, or write them one by one, and measure again.
- **The speed does not move.** Then the cost is in the digit reads, not the builds. Record that as the finding and keep the change only if it is no slower.

No step waits on Casey.
