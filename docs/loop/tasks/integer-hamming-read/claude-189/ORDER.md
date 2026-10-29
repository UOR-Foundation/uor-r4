# ORDER: integer-hamming-read, claude-189 (the copy heads' integer read, as a kernel)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `15c474a2d`.

## Goal

This is the first half of step 2 of the item's design: the copy heads' read, in integers, as a kernel in `stack/hamming.rs`. The session hooks are the second half (the next unit). The read follows `flock_read_hamming.rs`'s `read`, for the copy heads, at each row `t` of a token sequence:
1. **Codes.** A copy head's query is the sign code of `x_t`'s table point through the key rotation. The key at `s` is the code of `x_{s−1}`'s point through the same rotation. So every code is a vocabulary entry's, built once:
   - the rotation `stack_rotation(raw, 2^31, GATE_EPSILON)` (`(g/|g|)`, as `hamilton_gate`);
   - then `stack_hamilton` on the Q12 `writer.table` row;
   - a bit is 1 at or above 0, as `pack_signs`.
2. **Choice.** The `min(16, t − 1)` positions `1 ≤ s < t` nearest by `popcount(q XOR k_s)`, in ascending distance, then ascending `s`: the design's recorded tie rule.
3. **Weights.** `max(0, m − d) / max(m, 1)`, with `m` the head's margin (Q16) and `d` in bits, as a Q31 integer quotient (D81). The `max(m, 1)` is the float's `margin.maximum(1.0)`.
4. **The head's mean.** `Σ w_s · table[x_s] / max(1, #{w_s > 0})`.
5. **The copy mean.** The mean of the head means over the copy heads that found a candidate (`w > 0`). There is none when no head found one.

## Cancel probe (run first)

Run at `15c474a2d`:
- **The artifact carries the read** (claude-188), and the session refuses it.
- **No integer copy-head read exists.**
- **No open PR touches** `stack/hamming.rs` or `stack/session.rs`.

**Go. Class: mechanism.**

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/session.rs` (`stack_rotation` made `pub(super)`)
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The kernel holds against an f64 reference of the same quantized steps** (float rotation and signs, distances, the same tie rule, weights and means), on the synthetic trunk and read, over seeded sequences. The bar is fixed here before the test runs:
   - every vocabulary code equal;
   - every kept set equal, with its distances;
   - every copy mean within `2^−23` of the reference's. That bound is below a Q16 step of the stream by 2^7.
2. **A recall control.** Wherever `x_t` repeated an earlier `x_j` with `j + 1 < t`, every copy head keeps `s = j + 1` at distance 0, and its payload is `x_{j+1}`'s point.
3. **The weight's denominator:** a margin below one bit divides by 1, not by the margin.
4. **Negative gates:** the tie rule reversed, and `max(m, 1)` dropped, each turn a test red.
5. **The checks pass:**
   - `cargo test -p uor-r4-integer --features serving-wasm` shows no failure beyond `main`'s missing-input set;
   - the serving audit is FULL PASS;
   - fmt is clean.
6. **A REPORT and one notebook row.**

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **a code or kept set differs from the reference:** classify it. Is it a near-zero sign or a defect? Fix a defect. A real near-zero flip is reported, not hidden.
