---
id: integer-retention-rates
title: Serve the recurrence's retention reading in the integer artifact
milestone: research
area: serving
status: done
open_to: lab
needs_gpu: true
depends_on: [d11-exporter-float-trunk]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-compare.rs, docs/roadmap/items/integer-retention-rates.md, docs/harness/prep/integer-retention-rates*, docs/labs/integer-retention-rates/**, docs/loop/tasks/integer-retention-rates/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The project's first hurdle is one working model end to end, served from its integer artifact. The trained model and the served one must compute the same recurrence. Claude-226 built the retained reading and its artifact flag, and claude-227 proved their GPU parity on the trained checkpoint within the registered 3% bar.

- **The float trunk's retention reading:** with `retain_rates` on, a recurrence lane carries 1 − 2^−(k+1) of its state at each token, with the input share keep = √(1 − λ²). `geometric_stack_native.rs`: "On (`in_space_long`, 27 Oct 2026): a lane carries `1 - 2^-(k+1)`, so entry `k` holds about `2^(k+1)` tokens."
- **Every one-model-end-to-end checkpoint since then trained with it.** (r-drop), which claude-217 to claude-220 scored, has `"retain_rates": true` in its run's `config.json`, and claude-220 scored it with `--retain-rates 1`.
- **Before claude-226, the integer artifact served only the off reading,** λ = 2^−(k+1), and the exporter carried no flag. Claude-225 found that mismatch by reading the code, before any run.
- **Claude-226 built the flag and retained recurrence** ([REPORT](../../loop/tasks/integer-retention-rates/claude-226/REPORT.md), #3457): 386 integer-library tests, 186 float-library tests, nine negative gates and a full serving audit. Earlier snapshots still export byte for byte, and an older engine refuses the new `geometric-retention` reading.
- **Claude-227 passed GPU parity** ([REPORT](../../loop/tasks/integer-retention-rates/claude-227/REPORT.md), #3458): the retained artifact agrees on 1,531 / 1,536 sequences (0.33% residual) and 4,023 / 4,096 single tokens (1.78%), both inside the 3% bar. The decay control agrees on only 17 and 84. The item is done. [`point-key-integer`](point-key-integer.md)'s real run then found row 0's readout instrument fault, and claude-231 built the nearest readout. [Claude-232](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun.

## What is missing

- **No remaining gate in this item.** Parity is measured at the registered seed-1 checkpoint, with a known residual: the 1.78% single-token gap is inside the bar but its cause is not located. No throughput is measured here. Three full snapshots are stored for the point-key run.
- **The completed integer form, for reference:** `rec_gates = "geometric-retention"`; λ_i = 1 − 2^−(i+1) at Q31 and keep_i = isqrt(2^62 − λ_i²), using precomputed tables with a test-side f64 reference. A run with `fold_scores` or `reflect_floor` on is refused; both are off in (r-drop), so this item serves `retain_rates` alone.

## Done when

1. **The snapshot and the artifact carry the reading.**
   - An artifact without it decodes exactly as today, and an engine without the retained form refuses one with it.
   - A run config with `fold_scores` or `reflect_floor` on is refused by the snapshot.
2. **The retained recurrence matches a test-side f64 reference** of the float rule at every rate index, within one unit of its rounding, with a negative gate.
3. **Parity, under a prep sheet with a fresh FIT and a pre-run review:** `fbs-compare` on (r-drop) s1, the integer readout against the float readout, reported as earlier exporter units reported it. The bar is registered in the sheet.
4. **The serving audit** (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
5. **One notebook row, and a REPORT with the negatives first.**

## How to start

- **Read:**
  - `geometric_stack_native.rs`: `set_retention_rates`, and the recurrence's `(lambda, keep)` with `retention_rates()`;
  - the integer recurrence in `stack/session.rs` (`rec_geometric`, `GEOMETRIC_KEEP_Q31`);
  - `fbs_export.rs` and `fbs-snapshot.rs`;
  - `fbs-compare`.
- **Before claiming:** check that no open order holds `stack/session.rs` (`point-key-integer` and `turn-memory-integer` also edit it).

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction. D32 applies to the recurrence.
- **Research compute is GPU-only for the float reference** (§3.5): the parity's float side runs on the GPU.
- **Deterministic exact checks only** (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- `crates/uor-r4-float-train/src/geometric_stack_native.rs`, the retention note and the recurrence.
- (r-drop)'s run config, `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s1/r-drop-s1/config.json` in the store.
- [point-key-integer](point-key-integer.md), whose row-0 run and subsequent nearest-readout build depended on this item.
