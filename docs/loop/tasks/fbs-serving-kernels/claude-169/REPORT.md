# REPORT: fbs-serving-kernels, claude-169 (the Hamilton product by shifts where it can)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. Done-when 2's vector kernel and its switch stay open.

**Speed.** On this M1, the served FBS model now decodes at **3,042–3,044 tok/s**: 0.33 ms a token on claude-126's seed-1 export at a 192-token context.
- In the same session, claude-168's binary decodes at 2,428–2,435 tok/s.
- claude-163's binary decoded at 270.
- Every reply is byte-identical.

## What did not work, first

- **The first two profiles were the wrong instrument.**
  - **The first:** sampling started 1 s into a plain run. The run now lasts about 1.3 s, so it caught only 110 samples on the thread.
  - **The second:** run on a `--parts` run, it credited 3,059 of 6,007 samples to `stack_step_fbs_timed` itself. A plain run's profile has no such frame, so its ranking cannot be compared with the earlier units'.
  - **The sealed profile** is the third: a plain run, sampled from 0.1 s, with 940 samples on the thread. Neither earlier profile is kept.
- **The snap's scan leads again.** `stack_snap_select` holds 300 (32%) and `stack_hamilton` 193 (21%). The product part got the larger gain (1.50–1.52×): its Hamilton product is a gate snapped at `λ = 2^31`, and every one of its zero, axis, half and `1/2` coordinates is now a shift or skipped.
- **Why only the product part gained so much is not measured here.** The core's lanes use a smaller `λ = 2^(30−idx)` and the same golden coordinates. Which roots the model picks, and so how many coordinates take the shift path, is not counted by `fbs-toks`.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`, `stack_hamilton(a, b)`,** now goes row by row:
  - a zero `a_i` is skipped;
  - `|a_i| = 2^k` (`m & (m − 1) == 0`) gives `|b_j| << k` for each `j`;
  - any other `|a_i|` gets one table of its own multiples (`multiples_u128`), read at each `|b_j|`'s digits (`mul_by_u64`).

  The sign is `mul_i128`'s throughout. There are no shared `b` tables anymore.
- **Why it is exact.** Each product is still `|a_i| |b_j| mod 2^128`. A shift by `k <= 63` of a value below `2^64` is exact in `u128`, and the table read gives the same product, whichever operand is tabulated.
- **Why the shift path is taken.** All three callers pass `stack_snap_rotation(raw, λ)` with `λ` a power of two: `1u64 << 31` at `session.rs` lines 2835 and 3339, and `1u64 << (30 − idx)` in the geometric lanes at line 3422. `shift(x, 1) = (x + 1) >> 1` is exact on even `x`, so:
  - an axis coordinate is `±λ`;
  - a half or `1/2` coordinate is `±λ/2`.

  On generic input, every row takes the table path: at most 4 tables and 16 digit reads, as in claude-166.
- **The extended test `the_shared_table_hamilton_product_is_the_sixteen_products`** now also covers:
  - every root snapped at every power-of-two `λ` from `2^15` to `2^31`, against 20 random states each. It asserts that more than 2,040 of those coordinates take the shift path;
  - 100,000 inputs whose rows mix zero, `±2^k` and other values.

  With the shift off by one (`<< k − 1`), it fails (`0 passed; 1 failed`). It passes once restored.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and what is next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-168's binary, sha256 `b0106169c76fabdea92db643c8675424d571e53c896a8b957e0177807f874780`.
- **New:** this unit's binary at `52333e824`, sha256 `8a7a83c76ef83e4302a0fbfb211061c113795fbdb35761bc2e8822123207805a`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|
| old 1 | 2,435 | 416,958 | 207,917 | 128,479 | yes |
| new 1 | **3,044** | 331,750 | 165,792 | 84,334 | yes |
| old 2 | 2,428 | 411,583 | 204,895 | 126,833 | yes |
| new 2 | **3,042** | 331,937 | 165,520 | 84,417 | yes |

- **The step:** 1.24–1.26× faster.
- **The core:** 1.24–1.25×.
- **The product:** 1.50–1.52×.
- **Shares of the new step:** the core 49.9–50.0%, the product 25.4%.
- **Each block's parts** lie at 1,000–1,005 per mille of the block in the new runs.

**Profile of the new binary** (`/usr/bin/sample <pid> 8 -mayDie`, started 0.1 s into a plain `fbs-toks` run; 940 samples on the thread):

| Function | Samples |
|---|---|
| `stack_snap_select` | 300 (32%) |
| `stack_hamilton` | 193 (21%) |
| `stack_native_writer` | 64 |
| `stack_mul_u128` | 53 |
| `stack_recurrence_core` | 49 |
| `stack_fbs_gather` (i32), `stack_rms_norm_fine` | 43 each |
| `stack_snap_rotation` | 40 |
| `stack_square` | 25 |
| `stack_mul_u64` | 23 |

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 | `314c3909…fdb305`, as claude-158 to claude-168 |
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, as claude-163 to claude-168 |
| `replies_match`, every run | true |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `ec46df10…9658`) | FULL PASS: 0 forbidden instructions over 44 symbol ranges and 93 reachable functions, on the first build |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-hamilton-shifts-2026-10-28/` holds the four runs, `sample-new.txt`, `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `339 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines (one `manual_is_multiple_of` in the new test was fixed).
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The target came from claude-168's profile. The structure the change relies on (`λ` a power of two in every caller) was read from the callers before the code was written.
- **writing-meaningful-tests.** The test asserts that the shift path is taken, and fails with the shift off by one.

## Still open

- **The snap's scan** (now 32%). The 12 golden candidates can be cut by the query's magnitude order, exactly.
- **A count of the roots the model picks,** which would explain the core's smaller gain.
- **Done-when 2's vector kernel,** measured against this unit's 3,042–3,044 tok/s.

## Plain summary

The model on the laptop now writes about 3,000 words a second, up from 2,450, and from 270 this morning, with exactly the same output. Many of the numbers it multiplies are powers of two, and multiplying by a power of two is just shifting the digits, so the model now does that instead of a full multiplication whenever it can.
