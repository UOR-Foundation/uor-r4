# REPORT: integer-mul-div-serving, claude-177 (the kernels use the hardware multiply and divide)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays open: done-when 2's last bullet (refreshing the docs' audit counts) is left; done-when 3 is met here.

**Speed. Decode 3,029 / 3,041 → 4,829 / 4,827 tok/s on this M1 (1.59×), replies byte-identical.** The step went from 0.332 to 0.207 ms.

## What did not work, first

- **The docs' audit counts are not refreshed in this unit.** Done-when 2's last bullet asks for it in the same unit. Casey asked on 29 Oct whether the work had drifted from the mechanism into docs, so this unit lands the kernel change first.
  - The lines claude-176 listed now understate what the engine does: "7 of 7 … no multiply", "zero-multiply audits green".
  - They are left for one short follow-up unit, before the exporter work.
- **The dense read is unchanged.** `stack_query_tables`, `stack_dot` and `stack_mix_row` still use multiples tables, because their `[i64; 16]` table type runs through about eight places in `session.rs`. An FBS step does not run that read.
- **`stack_isqrt` is unchanged** (digit by digit). It is not in the profile's top entries.
- **34 suite failures predate this change.**
  - Seven test binaries hard-code `/Users/casey.allard/uor-r4-investigations/…` bundles, a folder removed in the 20 Oct workspace reset, and fail with `NotFound` on load.
  - `m7_parity_library_cli_browser` fails under Homebrew's `cargo` ("unknown command 'build'") and passes with `~/.cargo/bin` first on PATH.
- **The first library build compiled nothing of the stack.** `stack` exists only under the `serving-wasm` feature. Every later build and test here used that feature.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - `stack_mul_u128`, `stack_mul_u64` and `stack_square` are one hardware multiply each.
  - `stack_div_u64` and `stack_div_u128` are one hardware divide each (`__udivti3` for `u128`). A zero divisor still gives 0.
  - `stack_hamilton` takes its sixteen products as `i128` widening multiplies.
  - The RMS norm's product and the pointer mixture multiply directly, with no multiples table.
  - The names and `#[inline(never)]` stay, as the audit's mandatory symbol list needs them.
  - The multiplier-free forms move under `#[cfg(test)]` as references: `multiples_u128`, `multiples_u64` and `mul_by_u64`, plus `table_mul_u128`, `table_mul_u64`, `table_square`, `long_division_u64`, `long_division_u128` and `table_hamilton`.
  - New tests `hardware_kernels_equal_their_multiplier_free_forms`, and `the_hardware_norm_equals_its_multiplier_free_form`, which runs both norm variants against the old body (`table_rms_norm_into`) on 2,000 random vectors. The second was added after the review found that a wrong norm product passed every stack test; it fails with `r + 1` in place of `r`. The pointer mixture's existing test already caught a swap of its two coefficients.
- **Notebook:** one row in `docs/research/notebook/m4-serving.md`.

## Measurements

Run on this M1 on AC power, claude-126's seed-1 export (sha256 `314c3909…fdb305`). `fbs-toks --parts`, old and new alternately, each protocol 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps. Sealed to `caseyallard/uor-r4-store:results/claude/integer-mul-div-kernels-2026-10-29/`; `SHA256SUMS` was read back OK (10 files).

| Run | Binary | Decode tok/s (median) | p95 | Step (ns) |
|---|---|---|---|---|
| old 1 | claude-169 `8a7a83c7…805a` | 3,029 | 2,575 | 331,916 |
| new 1 | claude-177 `3b278e81…7d4b` | **4,829** | 4,750 | 207,250 |
| old 2 | claude-169 | 3,041 | 3,013 | 331,917 |
| new 2 | claude-177 | **4,827** | 4,782 | 207,542 |

**Parts, summed over six layers (old 1 → new 1, ns):**

| Part | Old | New | Speedup |
|---|---|---|---|
| Recurrence core | 165,103 | 92,291 | 1.79× |
| Gated product | 84,373 | 54,915 | 1.54× |
| The two norms | 37,459 | 19,999 | 1.87× |
| Final norm | 3,166 | 1,708 | 1.85× |
| Writer | 19,959 | 19,417 | unchanged |
| Gathers | 16,383 | 16,424 | unchanged |

The parts sum to 999 per mille of the step in new 1, and 990 in old 1.

**Profile of the new binary** (`/usr/bin/sample <pid> 8 -mayDie`, started 0.1 s into a plain run; 725 samples on the thread): `stack_snap_select` 333 (46%), `stack_recurrence_core` 81, `stack_native_writer` 64, `stack_snap_rotation` 42, `stack_fbs_gather` (i32) 38, `stack_rms_norm_fine` 35, `stack_lanes` 26, `stack_hamilton` 17 (was 193 of 940 in claude-169).

| Check | Result |
|---|---|
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, as claude-163 to claude-169 |
| New kernels against their old forms | equal on edge values and 100,000 random inputs |
| Negative gates | the test fails with `stack_mul_u64` wrong at 17, `stack_div_u128` wrong at divisor 16, or the Hamilton product wrong at `i64::MIN` |
| Native audit, release `uor-r4-stack` (`e6be19f1…6224`), `--stack` | FULL PASS: 44 symbol ranges, 93 call-graph functions; 47 / 1 scalar multiplies / divides allowed in the ranges, 65 / 9 in the call graph; no vector form, no `fmov` |
| `uor-r4-stack`'s own objdump check (m5, `serving-wasm`) | passes: 122 symbols, 16,571 instructions |
| WASM serving build (`260c9b82…`), `--serving --call-graph` | 7 of 7 PASS; `step` and `IntegerStackSession::step` at 14 multiplies and 9 divides, 0 float |
| Library, CLI and browser (m7 parity) | bit-identical tokens and snapshots on the frozen panel |
| `uor-r4-integer` stack tests | 84 passed |
| Whole `uor-r4-integer` suite, `serving-wasm` | 545 passed, 36 failed. 34 are the missing-bundle tests and 2 the m7 parity tests under Homebrew `cargo`, which pass with `~/.cargo/bin` first (above) |

## Checks run at the head

- The tests and audits above; `cargo fmt`.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The kernels use exactly what D80 and D81 allow: scalar integer products and quotients. No float, no vector form, no matrix product.

## Still open

- **A short follow-up unit:** refresh the docs' audit counts that claude-176 listed.
- **The next serving lever:** the icosian snap's scan, now 46% of a token (`fbs-serving-vector-kernel`, or fewer golden candidates).
- **The exporter (R6), per Casey's "fix the exporter".**

## Plain summary

The engine now uses the chip's own multiply and divide in place of the long workarounds it needed before. Every reply came out byte-for-byte the same, and the model now writes about 4,830 tokens (word pieces) per second on this laptop, up from about 3,040.
