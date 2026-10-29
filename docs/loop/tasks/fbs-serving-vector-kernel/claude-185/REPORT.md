# REPORT: fbs-serving-vector-kernel, claude-185 (where a served token's time goes now; one reciprocal in place of four divides)

Lab: claude. Order: [ORDER.md](ORDER.md). The item is not done; its done-when 2 and 3 still stand.

**Speed.** `fbs-toks` decode on this M1, AC power, at `c95dfaea6`:
- **Snapped** (claude-126 s1): 4,832 tok/s.
- **Unsnapped `raw_both`** (dual-c131 s1): 4,236 tok/s.

This unit's change was 5% slower on `raw_both`, so it is reverted.

## What did not work, first

- **One reciprocal in place of `stack_rotation`'s four divides was exact but slower.**
  - **The change:** `m = ⌊2^78 / norm⌋`, then `q = (|v| m) >> 32` per coordinate, which is the quotient or one less, plus one remainder correction.
  - **Exact:** it equalled the four-divide form on all 1,384,160 cases. Those are 14⁴ quaternions of edge values (zero, ±1, ±2, 3, ±46,341, ±2^15, `i32::MAX`, `i32::MIN` and their neighbours), times both epsilons and five `lambda`, plus a million seeded quaternions at every scale. Every reply was byte-identical on both artifacts.
  - **Slower:** alternating with the base binary, `raw_both` decode fell from 3,982 / 3,992 to 3,775 / 3,787 tok/s (−5%). Snapped fell from 4,573 / 4,601 to 4,414 / 4,546.
  - **A likely reason, not measured:** the old numerators `|v| << 46` often fit in 64 bits, which is `__udivti3`'s fast path. The reciprocal's `2^78 / norm` never does, and the reciprocal adds two out-of-line `u128` multiplies per coordinate.
  - By the ORDER's stop rule, the four divides stay. The patch is kept here (`reciprocal.patch`) and in the store.
- **The `raw_both` reply cannot show a difference,** which the review found.
  - Under `fbs-toks`'s default prompt, dual-c131's reply is token 1, 65 times over.
  - So "every reply byte-identical" on that artifact shows almost nothing. This unit's exactness rests on the host test, 1,384,160 cases.
  - The item's done-when 2 now asks for a check on that artifact that can catch a difference: the session's final state, the per-step query codes, or a prompt whose reply varies.
- **The machine ran slower during the timing than during the probe:** 4,832 tok/s snapped at the probe, and 4,573–4,601 for the same base binary later. Only the alternating pairs compare like with like, and the REPORT gives no cross-run comparison.
- **The heredoc I first used to add the test** was refused by the session guard: it read `>> shift` inside the code as a redirect to a file named `shift`. The Edit tool wrote it instead.

## What changed

- **`docs/model/pipeline-map.md`:** gap 3 and R3 now record that R3 was met by exact structured kernels (claude-163 to claude-170, then claude-177's hardware multiply and divide). That is about 3.3 ns per learnable value over a whole token. The unsnapped contract's cost and this unit's negative are noted, and the per-value claim is an open question. This is the item's done-when 1.
- **`docs/roadmap/items/fbs-serving-vector-kernel.md`,** restated under the decision review:
  - **State:** a state note, with the profile and the negative.
  - **Done-when 2:** it now reads "the snap site, in either form", which covers the snapped selection and the unsnapped `stack_rotation`, or the Hamilton product. Its replies are checked on both artifacts.
  - **Baseline:** this head's sealed binary, in place of claude-169's.
  - **Candidates:** the remaining ones, with the integer square root added.
- **`reciprocal.patch`:** the reverted change and its test, kept as the negative's source.
- **One notebook row.**

No code change is merged: `stack_rotation` is as it was.

## Measurements

**The profile** (`fbs-toks --parts`, base binary `f7842898…e2a3`, M1, AC power):

| Artifact | Decode tok/s | Step | `recurrence.core` per layer | `mlp.product` per layer | Replies match |
|---|---|---|---|---|---|
| claude-126 s1, snapped (`314c3909…`) | 4,832 | 205 µs | 14.7–16.1 µs (mean 15.2) | 8.8–9.4 µs (mean 9.0) | yes |
| dual-c131 s1, `raw_both` (`a9df5263…`) | 4,236 | 235 µs | 17.8–18.4 µs (mean 18.0) | 11.3–11.4 µs (mean 11.4) | yes |

Ranges are over the six layers. The other parts (the norms, gathers and requantization) are the same on both artifacts, to within 42 ns.

**The reciprocal, alternating with the base** (`fbs-toks`, decode and context tok/s, median of 20 timed runs):

| Run | Snapped, base | Snapped, new | `raw_both`, base | `raw_both`, new |
|---|---|---|---|---|
| 1 | 4,573 / 4,565 | 4,414 / 4,424 | 3,982 / 3,939 | 3,775 / 3,756 |
| 2 | 4,601 / 4,578 | 4,546 / 4,568 | 3,992 / 3,918 | 3,787 / 3,740 |

- **Replies:** byte-identical in all four pairs.
- **The new build's serving audit** (`audit_zero_matmul_serving.py --stack`, on `uor-r4-stack` `4e4da61a…1344`): FULL PASS. The multiplies / divides are 47 / 1 in the symbol ranges and 65 / 9 in the call graph, unchanged.
- **Binaries:** base `fbs-toks` `f7842898…e2a3`, new `debe3564…ba7f`.
- **Sealed** to `caseyallard/uor-r4-store:results/claude/fbs-step-profile-2026-10-29/`: 16 files. The `SHA256SUMS` sha256 is `beefa4561ff5c178a0e6bca2a3b086bbaa63a8c87d2a1ccff2b2d94716d62d78`. Read back, every file verifies.

**Classification:** a measured negative for one configuration, the one-reciprocal quotient in `stack_rotation`. It is not a result about the mechanism or the serving path.

## Checks run at the head

At `d3714afde`. The code is unchanged from `main`, because the change is reverted.
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`:** 344 passed, 1 ignored.
- **The reciprocal's own test, during the attempt:** it passed. With the correction set to `+= 0`, it failed (at `[0, 0, 0, 3]`, epsilon 0: 2,147,483,646 against 2,147,483,648). Restored, it passed. Both builds recompiled the crate.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **The fresh review** (decision review and claims check, one reviewer, at `d3714afde`): APPROVE and SUPPORTED. Its Medium finding, the degenerate `raw_both` reply, is disclosed above and folded into the item. Its Low finding, that the map's 3.3 ns is derived, is marked in the map.

## Skills used

- **holding-the-goal.** The change was measured against the item's bar, faster and exact, and dropped when it missed. The four divides were not kept by way of an argument about divide counts.

## Still open

- **The item's done-when 2 and 3:** a faster exact change, timed against the sealed base.
  - **First candidate:** the rotation's integer square root. It is a digit-by-digit `u128` loop that first walks down from `2^126` to the value's top bit, which a leading-zero count gives at once. Its share of the rotation is not split yet.
  - **The others:** fewer golden candidates (snapped), and a vector kernel the audit allows.
- **The map's gap 5** still says the FBS export gap fails on all six receipts. R6 (claude-179 to claude-182) answers both of its parts. A map refresh should update it.

## Plain summary

This unit measured where the time goes when the integer model produces one token, then tried one speed-up.

The newer integer model, which keeps its rotations exact, is about 12% slower than the older one, and all of the difference is in the two steps that compute those rotations. I tried computing one shared reciprocal instead of four divisions. The answers were identical in every one of 1.4 million test cases, but it ran 5% slower, so it was taken back out.

The project's map now says the earlier speed target was reached by exact code rather than vector instructions. The task's description now points at the next candidate.
