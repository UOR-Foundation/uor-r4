# REPORT: CPU panel admission and exact-export gate, deepseek-3

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `ee534711f`.

## What did not work

- **This is finite component evidence.** The panel is 64 one-token synthetic rows on a width-64, one-layer, 4,096-token model. It does not establish corpus behavior, language quality, complete replies or fresh-phrasing transfer.
- **A zero CPU/serving export gap is not a float-export measurement.** The new check compares the CPU forward against the integer serving session on the same artifact and rows. A float-shadow comparison needs a separately bound trained float checkpoint and panel.
- **The evaluator is serial and allocates a fresh destination-side session per row.** Its row/s number is an untuned correctness path, not a corpus-training rate.
- **The gate checks target admission and exact export only.** It reports top-1, top-8 and margin deltas but does not define a language or capability pass bar.

## What changed

- Added `CpuPanelEvaluator`, which evaluates one immutable artifact over a finite `RankMarginSample` panel.
- Each row runs through the CPU rank-margin forward and a fresh `IntegerStackSession` stream. The reading records:
  - artifact BLAKE3 digest;
  - the complete sample-set fingerprint;
  - sample, target-admission, top-1 and top-8 counts;
  - summed exact rank margin and target rank;
  - checked and mismatching query coordinates;
  - checked and mismatching serving tokens;
  - checked and mismatching kept-cell entries.
- `PanelDelta` reports admitted, top-1, top-8, margin and exact-export differences only when the sample set and checked export shapes match.
- `panel_admission_gate` accepts only non-decreasing admission with an exact serving export. It separately refuses admission loss and export mismatch.
- A trained tied-table checkpoint and its resumed trainer produce byte-identical artifacts and identical panel readings, with zero export mismatch.
- Added `cpu-panel-export-bench` for release rows/s and process peak RSS.

## Measurements

- `cargo test --release --locked -p uor-r4-cpu-train`: 58 passed, 0 failed.
- Exact finite panel: 64/64 target-admitted, 64/64 top-8, zero query-coordinate, token and kept-cell mismatches.
- The no-change matched control returns all-zero deltas and accepts. A real patched artifact whose full panel evaluation loses target admission is refused, as are export mismatch, a different sample set and a valid nearest-readout artifact unsupported by this evaluator.
- Release benchmark on the Ryzen 9 9950X:

| Rows per run | Timed runs | Timed rows | Elapsed | Rows/s | Peak RSS |
|---:|---:|---:|---:|---:|---:|
| 64 | 16 | 1,024 | 94.790 ms | 10,802.874 | 6,320 KiB |

Artifact BLAKE3: `260b989b18ac3531b8b863cf72efe33c587d264f9b74145b19f0018ad3a3dd30`.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train`: PASS, 58 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-panel-export-bench`: PASS; readings above.
- D23, D25, D32, local PR checks and full loop gates are recorded after they run at the final exact head.

## Skills used

No repository skill was invoked. The prior CPU parameter-group review established the strict-margin and admission-control patterns reused here; a fresh exact-head reviewer audits this unit independently.

## Still open

- A trained-artifact, non-synthetic panel.
- Float-shadow versus integer-server export gap on that checkpoint.
- Corpus-scale admission, complete replies and fresh phrasing.
- The pre-registered bounded pilot in #3506.

## Plain summary

The CPU trainer can now grade a whole finite panel, report ranking and admission beside the exact serving gap, compare checkpoints against a matched control, and refuse a checkpoint that loses target admission. This closes the immediate #3505 checkpoint-control gap, but it is not yet a corpus pilot or a useful-learning result.
