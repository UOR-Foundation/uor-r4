# REPORT: fbs-serving-kernels, claude-166 (the Hamilton product by shared tables)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. Done-when 2's vector kernel and its switch stay open.

**Speed.** On this M1, the served FBS model now decodes at **1,406–1,410 tok/s**: 0.71 ms a token on claude-126's seed-1 export at a 192-token context.
- In the same session, claude-165's binary decodes at 1,142–1,225 tok/s.
- claude-163's binary decoded at 270.
- Every reply is byte-identical.

## What did not work, first

- **The gain is smaller than the profile suggested.** claude-165's profile put about 44% of a token under `stack_hamilton`, nearly all of it in 16 table products a call. Building 4 tables instead of 16 made the step only 1.16× faster (0.82 → 0.71 ms).
  - The new profile (1,575 samples) shows why. `stack_hamilton` itself now holds 530 (34%), because the table building and the digit reads are inlined into it. `stack_mul_u128` under `stack_hamilton` is gone: its 74 remaining samples are all under the lanes.
  - Each table is 15 additions, and each passes through `black_box`, as in `multiples_u128`. That keeps the compiler from folding the chain into a multiply, but forces every step through memory.
  - The 16 digit reads per product remain.
  - `_platform_memset` holds 35 samples, from zeroing the 4 tables and the 16 products on every call.
- **Decode tok/s varies more than the step between runs.** claude-165's binary read 1,142 and 1,225 tok/s at almost the same step (819,521 and 819,167 ns). The plain protocol's decode median includes the loop and the copies outside the step. The step ratio (1.16×) is the steadier reading.
- **The first negative test was an equivalent mutant.** I changed the sign rule to `(y <= 0)`, and the test still passed. That mutant flips the sign only of products with `y = 0`, which are zero either way, so it changes no value. The real negative, the inverted rule `(x < 0) == (y < 0)`, fails the test.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`, `stack_hamilton(a, b)`:**
  - It builds one table of multiples per `|b_j|` with `multiples_u128`, so 4 tables in place of 16.
  - For each nonzero `a_i`, it reads each table at `|a_i|`'s radix-16 digits with `mul_by_u64`, and negates when the signs differ, as `mul_i128` does. A zero `a_i` is skipped, since its products are zero.
  - The four sums are unchanged.
  - Explicit loops, not `array::from_fn` or `map`, keep closures out of the audited call graph.
- **Why it is exact.** Each product is `|a_i| |b_j| mod 2^128`. That value is the same whichever operand is tabulated: the product of two values below `2^64` is below `2^128`, so it is exact.
- **The orientation.** All three callers (`session.rs` lines 2845, 3349 and 3434) pass the snapped quaternion as `a`: its Q31 coordinates have few digits and a zero for every golden or axis root.
- **The new test `the_shared_table_hamilton_product_is_the_sixteen_products`** checks the new product against the old body, kept as `hamilton_by_products`:
  - 1,000,000 random pairs over the whole `i64` range, drawn from edges (`i64::MIN`, `i64::MIN + 1`, `±1`, `0`, `i64::MAX − 1`, `i64::MAX`), zeros, and magnitudes of every size;
  - 24,000 snapped transitions: every root, scaled by 200 random Q31 decays through `stack_snap_rotation`, against random states.

  With the sign rule inverted, it fails (`0 passed; 1 failed`). It passes once restored.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and a re-ranked next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-165's binary, sha256 `03c65108618ac005753ef083a154213c7eecf9b150c0482d09c633e0c554a161`.
- **New:** this unit's binary at `3a0dc4844`, sha256 `be139a40c29c0e82040bc1669e714530b73016994e79735729ded88cabfcac64`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|
| old 1 | 1,142 | 819,521 | 426,603 | 269,624 | yes |
| new 1 | **1,406** | 706,854 | 357,021 | 226,001 | yes |
| old 2 | 1,225 | 819,167 | 427,041 | 270,105 | yes |
| new 2 | **1,410** | 709,271 | 358,228 | 226,418 | yes |

- **The step:** 1.16× faster.
- **The core and the product:** 1.19× each.
- **Shares of the new step:** the core 50.5%, the product 31.9–32.0%.
- **Each block's parts** lie at 990–1,002 per mille of the block in the new runs.

**Profile of the new binary** (`/usr/bin/sample <pid> 8 -mayDie` on a plain `fbs-toks` run; 1,575 top-of-stack samples):

| Function | Samples |
|---|---|
| `stack_hamilton` | 530 (34%) |
| `stack_snap_select` | 517 (33%) |
| `stack_square` | 108 |
| `stack_mul_u128` | 74, all under `stack_lanes` |
| `stack_native_writer` | 63 |
| `stack_recurrence_core` | 42 |
| `stack_mul_u64`, `stack_rms_norm_fine` | 40 each |
| `_platform_memset`, `stack_snap_rotation` | 35 each |

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 | `314c3909…fdb305`, as claude-158 to claude-165 |
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, as claude-163 to claude-165 |
| `replies_match`, every run | true |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `3cec0718…07b2`) | FULL PASS: 0 forbidden instructions over 44 symbol ranges and 93 reachable functions |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-hamilton-2026-10-28/` holds the four runs, `sample-new.txt`, `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `336 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines (one `unused_mut` in the new test was fixed).
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The profile chose the target. The profile after the change shows the gain is smaller than projected, and why.
- **writing-meaningful-tests.** The first mutant was equivalent, so I replaced it with one that changes values. The test fails on that one.

## Still open

- **The table builds inside `stack_hamilton`** (now 34%). Each of the 60 guarded additions passes through `black_box`. A build with fewer guarded steps may cost less, as long as the audit still refuses no instruction.
- **The snap's 17-candidate scan** (now 33%). The 12 golden roots can be cut by the query's magnitude order, exactly.
- **Done-when 2's vector kernel,** measured against this unit's 1,406–1,410 tok/s.

## Plain summary

The model on the laptop now writes about 1,400 words a second, up from 1,200, with exactly the same output. The step that combines two four-number rotations had been rebuilding the same multiplication helpers sixteen times; it now builds four and reuses them. The saving was smaller than expected, because building those helpers safely is itself slow, and that is the next thing to improve.
