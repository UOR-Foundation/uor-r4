# REPORT: integer-hamming-read, claude-189 (the copy heads' integer read, as a kernel)

Lab: claude. Order: [ORDER.md](ORDER.md). The item is not done: this is the first half of step 2 of its six. It has no GitHub issue; claims and merges are noted on #2028.

**Speed.** No serving change. The kernel is not yet on the served path, since the session still refuses a `/6` artifact; the next unit puts it there.

## What did not work, first

- **Step 2 was too large for one unit.** The copy heads served end to end needs both the read and the session's hooks: the stream after a depth, the readout move, and the float's Q16 rounding of the stream read. This unit is the read as a kernel; the hooks are the next unit.
- **The first form of the reference test was too easy.**
  - Its margins (1 to 9 bits, the synthetic read's) on 64-bit codes let only exact repeats count. Random distinct tokens sit about 32 bits apart, so every weight was exactly 1, and the Q31 quotient and the truncating means were barely exercised.
  - Two seeds now run with margins of 33 to 41 bits plus a fraction. There most candidates fall inside the margin, with fractional weights.

## What changed

- **`crates/uor-r4-integer/src/stack/hamming.rs`:**
  - **`CopyHeads::new`** builds every vocabulary entry's code for each copy head.
    - The rotation is `stack_rotation(raw, 2^31, GATE_EPSILON)`, the unit `g / |g|` as the unsnapped gates normalize it. It is applied on the left by `stack_hamilton` to the Q12 `writer.table` row, as the float `hamilton_gate` is.
    - A bit is 1 at or above 0, 64 coordinates to a word in coordinate order, as `pack_signs`.
  - **`choose`:** the `min(16, t − 1)` positions `1 ≤ s < t` nearest by `popcount(q XOR k_s)`. The key at `s` is `x_{s−1}`'s code. They come in ascending distance, then ascending `s`: the design's recorded tie rule.
  - **`copy_weight_q31`:** `max(0, m − d) / max(m, 1)` at Q31, by `stack_div_u64` (D81). The `max(m, 1)` is the float's `margin.maximum(1.0)`.
  - **`mean`:** at Q43, each head's `Σ w_s · table[x_s] / max(1, #{w_s > 0})`, then the mean over the heads that found a candidate, or `None`. Both quotients truncate toward zero.
  - The kernel uses explicit arrays, not `array::map`, so no `core::array` closure symbol joins the served path when the next unit calls it.
- **`crates/uor-r4-integer/src/stack/session.rs`:** `stack_rotation` and `GATE_EPSILON` are `pub(super)`, for the kernel.
- **Three tests and two helpers** (`crates/uor-r4-integer/src/stack/tests.rs`):
  - **`copy_control`:** a Q12 table and a read, quantized as the exporter writes them.
  - **`reference_copy`:** the f64 reference. It runs the float read's steps on the quantized values: `(g / |g|) ⊗ p` in f64 and its signs, the same distances and tie rule, and f64 weights and means.
  - `copy_heads_match_the_f64_reference`;
  - `copy_heads_recall_the_token_after_an_earlier_copy`;
  - `copy_weights_divide_by_at_least_one_bit`.

## Measurements

**Against the f64 reference** (`copy_heads_match_the_f64_reference`): a 256-row, 256-wide table and a 4-head read (2 copy heads, 64 coordinates each), over 5 seeds, every row of a 48-token sequence. Three seeds use the synthetic margins and two the wide margins. The bars were fixed in the ORDER before the test ran, and all hold:
- **Every vocabulary code equal:** 256 × 2 per seed. There was no near-zero sign flip.
- **Every kept set equal,** in order, with its distances.
- **Every copy mean within `2^−23`** of the reference's, and the same rows found and not found.

**The recall control** (`copy_heads_recall_the_token_after_an_earlier_copy`):
- **One earlier copy, one-bit margins.** In 20 distinct tokens, then a repeat of position 5, both copy heads keep `s = 6` at distance 0. The copy mean is exactly `table[x_6] << 31`, the point of the token that followed the copy.
- **Many repeats.** In a 40-token sequence from 6 tokens, every earlier copy is kept at distance 0 whenever the zero-distance candidates fit the kept count of 16.

**The denominator** (`copy_weights_divide_by_at_least_one_bit`):
- `w(0.5 bits, d = 0) = 2^30`, that is 0.5, not 1;
- `w(1, 0) = 2^31`;
- `w(4, 1) = 3 · 2^29`;
- `w(4, 4) = w(4, 9) = 0`.

**Negative gates.** Each set-aside built cleanly and then failed its test:
- **Ties in descending `s`:** `copy_heads_match_the_f64_reference` fails (`neg-tie.out`).
- **Dividing by `m`, not `max(m, 1)`:** `copy_weights_divide_by_at_least_one_bit` fails (`neg-den.out`).
- **Restored:** all three pass (`neg-restore.out`).

## Checks run at the head

At `7021f2fc7`:
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`:** 560 passed, 35 failed, 4 ignored. The 560 are `main`'s 557 and this unit's 3. All 35 failures panic on a missing local input, as on `main`: 32 a bundle and 3 the tokenizer (`tokenizer.json`, or `tok_path.exists()`).
- **`cargo test --locked -p uor-r4-float-train --no-fail-fast`:** 122 passed.
- **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`): FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 95 reachable functions. The kernel is not on the served path yet, so the audit does not reach it.
- **The GPU evaluator's host tests** (`docs/labs/d11-exporter-float-trunk/instrument`, which links this crate; stub nvcc, `--target aarch64-apple-darwin`): 5 passed.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- **holding-the-goal.** The bar was fixed before the test ran. When the first reference test proved too easy to fail on the parts that matter, it was strengthened rather than kept, and the two negative gates show that it can fail.

## Still open

- **The second half of step 2:**
  - the session's hooks: the stream read after its depth, rounded to Q16 as the float's `q16(read.apply(...))`, and the readout move `n + w_c ⊙ (c − n)` where a copy head matched;
  - the session keeping each position's token;
  - the trunk-level controls;
  - lifting the `/6` refusal for a read whose type weights are all zero.

  claude-188's review asks that unit to match the session's error, not only `is_err()`, once `/6` is served.
- **Steps 3 to 6** of the design.

## Plain summary

This unit builds, in whole-number arithmetic, the part of the memory lookup that finds an earlier copy of the current token and brings back what followed it.

It checks the new code against a plain floating-point version of the same steps. On every test case they choose exactly the same earlier positions, and they produce the same result to within about one part in eight million. When a token repeats, the lookup finds the earlier copy and returns exactly the word that came after it. Two deliberately broken versions show the tests catch mistakes. The model doesn't use this code yet; wiring it into the model is the next step.
