# REPORT: fbs-serving-kernels, claude-168 (the snap's scores from |r| by additions)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. Done-when 2's vector kernel and its switch stay open.

**Speed.** On this M1, the served FBS model now decodes at **2,448–2,449 tok/s**: 0.41 ms a token on claude-126's seed-1 export at a 192-token context.
- In the same session, claude-167's binary decodes at 1,878–1,880 tok/s.
- claude-163's binary decoded at 270.
- Every reply is byte-identical.

## What did not work, first

- **The first two builds failed the serving audit, each on one `fmov`.** Those two audit outputs, and the disassembly quoted below, were read in the session and not kept. The sealed log is the passing one.
  - **The first:** with the four magnitudes `|r_c|` computed beside the sign pattern, the compiler packed them into a NEON register and moved a result back to a general register. Guarding each magnitude with `black_box` did not remove it.
  - **The second build** showed where it came from: the four sign tests of the pattern. The disassembly reads `ushl.4h`, `xtn.4h`, then `fmov x12, d0`. The sign tests had not been packed before, when `raw` was read only coefficient by coefficient.
  - **The fix:** guarding each lane's result with `black_box` as well makes the audit pass, with 0 forbidden instructions.
  - **The convention this supports:** values computed side by side from a query's four coordinates in an audited kernel must each pass through `black_box`, or the compiler may pack them. That joins claude-164's power-of-two row stride.
- **The Hamilton product leads again.** In the new profile (453 samples on the thread), `stack_hamilton` holds 173 (38%) and `stack_snap_select` 115 (25%). This unit does not touch the Hamilton product.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - **`snap_compatible_score(j, s, total)`** gives root `j`'s score against a sign-compatible query from `s = |raw|`:
    - an axis root (`j < 8`, on axis `j >> 1`): `(2 s_i, 0)`;
    - a half-root (`8 <= j < 24`): `(s_0 + s_1 + s_2 + s_3, 0)`;
    - a golden root (`j >= 24`, block `(j − 24) >> 3`): `(s_q − s_t, s_p + s_t)`, where the block's even permutation puts `φ/2` on axis `p`, `1/2` on `q` and `(φ−1)/2` on `t`.

    Index masks (`& 3`, `.min(11)`) let the compiler prove every index in range.
  - **`GOLDEN_AXES`** holds the twelve blocks' `[p, q, t]`, read at compile time from `H4_ROOT_COEFFICIENTS`. Each block's first root is checked: `[0, ±1]` is `p`, `[±1, 0]` is `q`, and a pair with both nonzero is `t`.
  - **`stack_snap_select`** computes `s` and the sum once per query, each magnitude and each sign lane behind `black_box`. It scores each listed candidate by `snap_compatible_score`, in index order, with the same strict comparison as before.
- **Why it is exact.** On a compatible root, each nonzero coordinate's sign equals the query's, so `raw_c (a_c, b_c)` is `|r_c|` times the root's coefficients made positive by that sign. That gives `(2, 0)` for an axis coordinate, `(1, 0)` for `1/2`, `(0, 1)` for `φ/2` and `(−1, 1)` for `(φ−1)/2`. A zero query coordinate contributes 0 either way. The sums are integers below `2^34`, exactly the old `small_mul` sums.
- **Tests in `kernels.rs`:**
  - **`the_root_table_has_the_layout_the_scores_read`** checks the axis, half and golden index ranges against the table. For every golden root it checks that its block's `[p, q, t]` are distinct and hold coefficients of `φ/2`, `1/2` and `(φ−1)/2`, with `a = −b` at `t`.
  - **`the_compatible_score_is_the_coefficient_sum`** checks, for every root and 10,000 sign-compatible queries each (zeros and free signs where the root is zero), that the structured score equals the `small_mul` sums.
  - **The existing snap tests** still hold: the full 120-root scan under the old square rule and the independent oracle.
  - **The negative:** with the golden `A` built as `s_q + s_t`, two tests fail (`0 passed; 2 failed`). They pass once restored.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and a re-ranked next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-167's binary, sha256 `215b7adb6d4be522cbbe79179031885192f9bbe2dd763a0e3e72ca0289a2c0bd`.
- **New:** this unit's binary at `24ac384a8`, sha256 `b0106169c76fabdea92db643c8675424d571e53c896a8b957e0177807f874780`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|
| old 1 | 1,878 | 543,125 | 280,793 | 180,873 | yes |
| new 1 | **2,448** | 409,834 | 203,959 | 126,416 | yes |
| old 2 | 1,880 | 534,208 | 276,126 | 178,124 | yes |
| new 2 | **2,449** | 410,354 | 204,166 | 126,666 | yes |

- **The step:** 1.30–1.33× faster.
- **The core:** 1.35–1.38×.
- **The product:** 1.41–1.43×.
- **Shares of the new step:** the core 49.8%, the product 30.8–30.9%.
- **Each block's parts** lie at 998–1,004 per mille of the block in the new runs.

**Profile of the new binary** (`/usr/bin/sample <pid> 8 -mayDie` on a plain `fbs-toks` run): 453 samples on the thread. The run is shorter than the 8 s window.

| Function | Samples |
|---|---|
| `stack_hamilton` | 173 (38%) |
| `stack_snap_select` | 115 (25%) |
| `stack_native_writer` | 21 |
| `stack_mul_u128`, `stack_snap_rotation` | 17 each |
| `stack_fbs_gather` (i32), `stack_recurrence_core` | 15 each |
| `stack_mul_u64` | 13 |
| `stack_square` | 12 |

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 | `314c3909…fdb305`, as claude-158 to claude-167 |
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, as claude-163 to claude-167 |
| `replies_match`, every run | true |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `07becbd3…edcc`) | FULL PASS: 0 forbidden instructions over 44 symbol ranges and 93 reachable functions |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-snap-terms-2026-10-28/` holds the four runs, `sample-new.txt`, `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `339 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines.
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The target came from claude-167's profile, and the audit's disassembly located each forbidden instruction before it was fixed.
- **writing-meaningful-tests.** The layout the scores rely on is itself a test, and the score test fails with one sign wrong.

## Still open

- **The Hamilton product's digit reads** (now 38%). Each product reads about 8 digits of a Q31 transition from a table of `|b_j|`.
- **Fewer golden candidates, and done-when 2's vector kernel,** measured against this unit's 2,448–2,449 tok/s.

## Plain summary

The model on the laptop now writes about 2,450 words a second, up from 1,850, and from 270 this morning, with exactly the same output. Snapping a direction to the nearest of 120 fixed points used to score each candidate with eight small multiplications. The candidates' shapes are known in advance, so each score is now two or three additions.
