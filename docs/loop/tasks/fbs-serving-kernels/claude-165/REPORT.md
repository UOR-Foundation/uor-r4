# REPORT: fbs-serving-kernels, claude-165 (the snap's exact comparison without squares)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. Done-when 2's vector kernel and its switch stay open.

**Speed.** On this M1, the served FBS model now decodes at **1,169–1,217 tok/s**: 0.82 ms a token on claude-126's seed-1 export at a 192-token context.
- In the same session, claude-164's binary decodes at 742–744 tok/s (1.6× slower).
- claude-163's binary decoded at 270 (4.3–4.5× slower).
- Every reply is byte-identical.

## What did not work, first

- **claude-164's REPORT named the wrong next target.** It proposed fewer golden candidates, and then a vector kernel for the Hamilton products and lanes.
  - A `sample` profile of claude-164's binary shows otherwise. 1,919 of 3,158 top-of-stack samples (61%) sit in `stack_mul_u128`, called by `stack_snap_select` through its sign test `snap_difference_positive`.
  - That test squared both sides by radix-16 table products whenever `P` and `D` differ in sign. That happens on most comparisons in a scan.
  - The candidates themselves were a smaller cost.
- **The cost now sits in the Hamilton product.** In the new profile, `stack_mul_u128` still holds 487 of 1,132 samples (43%). Of those, 444 come from `stack_hamilton` and 43 from `stack_lanes`, none from the snap.
  - `stack_snap_select` itself holds 326 (29%). `stack_square` holds 78, and `stack_hamilton` itself 59.
  - The profile counts samples, not time per call. The ranking is the claim, not a timing.
- **The two runs of each binary differ by up to 4%** (new: 1,169 and 1,217 tok/s; old: 742 and 744, with steps of 1.38 and 1.33 ms). The machine's state varies between runs. Every comparison here is run against run, alternated.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - **`below_d_sqrt5(q, d)`** decides `q < d √5` for `0 < q < 2^37` and `0 < d <= 2^34` when the bounds allow it:
    - `true` when `1024 q <= 2289 d`;
    - `false` when `512 q >= 1145 d`;
    - `None` between.

    Its products by the two constants are four shifted terms each (`2289 = 2048 + 256 − 16 + 1`, `1145 = 1024 + 128 − 8 + 1`), each behind `black_box` so the compiler cannot fold them into a multiply.
  - **The bounds are proved:** `2289² = 5,239,521 < 5 × 1024² = 5,242,880`, and `1145² = 1,311,025 > 5 × 512² = 1,310,720`.
  - **`snap_difference_positive`** calls it in the two opposite-sign branches, and falls back to the existing square rule only on `None`. The same-sign branches, which never squared, are unchanged.
- **Tests in `kernels.rs`:**
  - **`the_bounded_comparison_is_the_square_rule`** checks the new comparison against the old square rule, written independently in plain `i128` (`snap_difference_positive_by_squares`). It covers:
    - all 361,201 pairs in `[-300, 300]²`;
    - 1,000,000 random pairs at the full bounds;
    - pairs at and around √5's continued-fraction convergents (`h² − 5k² = ±1`), at eight scales, offsets −3 to 3 and all four sign pairs.

    It asserts that more than 100 opposite-sign pairs fall in the band and more than 100 are decided by the bounds. With the lower bound wrongly raised to 2297/1024, it fails (`0 passed; 1 failed`), and it passes once restored.
  - **`the_sign_compatible_snap_is_the_full_scan`** now compares against a full 120-root scan under the old square rule, so it checks the pruning and the comparison together.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and a re-ranked next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-164's binary, sha256 `b0ed9537917f00fc41a89f1b44166c44b49da9533657dbc7be0400f5f5848270`.
- **New:** this unit's binary at `5bbb29398`, sha256 `03c65108618ac005753ef083a154213c7eecf9b150c0482d09c633e0c554a161`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|
| old 1 | 744 | 1,381,916 | 746,791 | 492,561 | yes |
| new 1 | **1,169** | 816,521 | 425,167 | 269,166 | yes |
| old 2 | 742 | 1,332,499 | 723,918 | 478,958 | yes |
| new 2 | **1,217** | 821,020 | 428,312 | 270,709 | yes |

- **The core:** 1.7–1.8× faster.
- **The product:** 1.8× faster.
- **Shares of the new step:** the core 52.1–52.2%, the product 33.0%. Each norm is 4.6% (37.5 µs over six layers, unchanged).
- **Each block's parts** lie at 983–1,009 per mille of the block in the new runs.

**Profiles** (`/usr/bin/sample <pid> 8 -mayDie` on a plain `fbs-toks` run; top-of-stack counts):

| Function | claude-164 (3,158 samples) | This unit (1,132 samples) |
|---|---|---|
| `stack_mul_u128` | 1,919 (61%), from the snap | 487 (43%): 444 from `stack_hamilton`, 43 from `stack_lanes`, 0 from the snap |
| `stack_snap_select` | 668 | 326 |
| `stack_square` | 131 | 78 |
| `stack_hamilton` | 81 | 59 |

The new profile is shorter because the run finished inside the 8 s window. The counts compare shares, not durations.

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 against claude-158's to claude-164's | match |
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, claude-163's and claude-164's |
| `replies_match`, every run | true |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `7f15ec70…398a`) | FULL PASS: 0 forbidden instructions over 44 symbol ranges and 93 reachable functions, on the first build |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-compare-2026-10-28/` holds the four runs, both profiles (`sample-c164.txt`, `sample-new.txt`), `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `335 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines.
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** A profile, not claude-164's guess, chose the target. The same profile, rerun, names the next one: the Hamilton product.
- **writing-meaningful-tests.** The comparison test asserts that both of its regimes are exercised, and fails with a wrong bound.

## Still open

- **The Hamilton product** (`stack_hamilton`, 16 table products of `i64` values, now the largest cost). A snapped root's coordinates take at most three distinct nonzero magnitudes, and every golden root has a zero coordinate. So the exact products can be shared and the zero ones skipped, with no change to any value.
- **Fewer golden candidates, and done-when 2's vector kernel,** measured against this unit's 1,169–1,217 tok/s.

## Plain summary

The model on the laptop now writes about 1,200 words a second, up from 750 this morning and 270 before that, with exactly the same output. Measuring showed the biggest cost was an exact check that compared two numbers by squaring them. Two simple fractions on either side of √5 now settle that check almost every time without squaring.
