# REPORT: fbs-serving-kernels, claude-167 (table builds from four guarded doublings)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. Done-when 2's vector kernel and its switch stay open.

**Speed.** On this M1, the served FBS model now decodes at **1,829–1,861 tok/s**: 0.54 ms a token on claude-126's seed-1 export at a 192-token context.
- In the same session, claude-166's binary decodes at 1,384–1,418 tok/s.
- claude-163's binary decoded at 270.
- Every reply is byte-identical.

## What did not work, first

- **The gain did not stay inside the Hamilton product.** The ORDER aimed at `stack_hamilton`'s table builds, but `multiples_u128` and `multiples_u64` serve every multiplier-free product in serving. The core and the product are 1.26–1.29× faster, and each norm is 2× faster (37.5 → 18.8 µs over six layers), because its squares read the same tables. That is a measured effect, not one this unit set out to claim.
- **The snap's scan is now the largest cost.** In the new profile (941 samples), `stack_snap_select` holds 400 (42%) and `stack_hamilton` 284 (30%). Nothing in this unit touches the scan.
- **Decode tok/s still varies between runs at nearly the same step**, by up to 2.5% here: claude-166's binary read 1,384 and 1,418. The step ratio (1.31–1.33×) is the steadier reading.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - **`multiples_u128` and `multiples_u64`** used to build `k a` for `k = 0..16` by fifteen chained additions, each behind `black_box` and an XOR with an opaque zero. Now they guard only four values, `a`, `2a`, `4a` and `8a`, made by shifts. The other eleven are sums of those:
    - `3a = 2a + a`, `5a = 4a + a`, `6a = 4a + 2a`, `7a = 4a + 3a`;
    - `9a` to `15a` are `8a` plus `a` to `7a`.
  - **Why no multiply can reappear:** the four guarded values are opaque to the optimizer, so no sum of them can be recognized as a multiply.
  - **Same values:** each entry is still `k a` modulo `2^128` or `2^64`, since the shifts and wrapping additions are exact modulo the width.
  - **`stack_hamilton`** builds its four tables as one array of builder calls. It no longer zeroes a `[[0u128; 16]; 4]` and overwrites it, a waste the fresh code review of claude-166 found.
- **The new test `every_table_entry_is_its_multiple`** checks every entry of both builders against `a.wrapping_mul(k)` on:
  - the edges (`0`, `1`, the maximum, the maximum − 1, half the maximum);
  - every power of two and its predecessor (128 widths for `u128`, 64 for `u64`);
  - 1,000,000 random values of each width, at every bit length.

  With `7a` built wrongly as `4a + 2a`, it fails (`0 passed; 1 failed`). It passes once restored. The existing products and Hamilton tests still pass.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and a re-ranked next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-166's binary, sha256 `be139a40c29c0e82040bc1669e714530b73016994e79735729ded88cabfcac64`.
- **New:** this unit's binary at `5447b38ca`, sha256 `215b7adb6d4be522cbbe79179031885192f9bbe2dd763a0e3e72ca0289a2c0bd`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | One norm, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|---|
| old 1 | 1,384 | 704,917 | 355,480 | 225,312 | 37,541 | yes |
| new 1 | **1,861** | 537,229 | 278,333 | 179,166 | 18,750 | yes |
| old 2 | 1,418 | 717,125 | 361,083 | 228,335 | 37,751 | yes |
| new 2 | **1,829** | 540,416 | 279,916 | 180,165 | 18,791 | yes |

The norm column is `mlp.norm`, the feed-forward's. The recurrence's norm reads the same: 18,791 ns in new 2.

- **The step:** 1.31–1.33× faster.
- **The core:** 1.28–1.29×.
- **The product:** 1.26–1.27×.
- **Shares of the new step:** the core 51.8%, the product 33.3%, each norm 3.5%.
- **Each block's parts** lie at 984–1,007 per mille of the block in the new runs.

**Profile of the new binary** (`/usr/bin/sample <pid> 8 -mayDie` on a plain `fbs-toks` run; 941 top-of-stack samples):

| Function | Samples |
|---|---|
| `stack_snap_select` | 400 (42%) |
| `stack_hamilton` | 284 (30%) |
| `stack_native_writer` | 37 |
| `stack_mul_u128` | 32 |
| `stack_snap_rotation` | 28 |
| `stack_rms_norm_fine` | 27 |
| `stack_recurrence_core` | 26 |
| `stack_mul_u64` | 21 |

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 | `314c3909…fdb305`, as claude-158 to claude-166 |
| Replies of all four runs | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'`, as claude-163 to claude-166 |
| `replies_match`, every run | true |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `0e65fee2…7d3e`) | FULL PASS: 0 forbidden instructions over 44 symbol ranges and 93 reachable functions |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-tables-2026-10-28/` holds the four runs, `sample-new.txt`, `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `337 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines.
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The target came from claude-166's profile. The A/B measured the change's whole effect, the norms included, not only the projected one.
- **writing-meaningful-tests.** The table test fails with one entry built wrong.

## Still open

- **The snap's 17-candidate scan** (now 42%). The 12 golden roots can be cut by the query's magnitude order, exactly, lowest-index tie included.
- **Done-when 2's vector kernel,** measured against this unit's 1,829–1,861 tok/s.

## Plain summary

The model on the laptop now writes about 1,850 words a second, up from 1,400, and from 270 this morning, with exactly the same output. Every multiplication in the served model reads a small table of multiples. Building those tables took fifteen protected steps; it now takes four, with the rest added from those.
