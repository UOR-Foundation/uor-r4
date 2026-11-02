# REPORT: CPU tied-table rank margin, deepseek-1

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `6a2d26249`.

## What did not work

- **This is not a full backward pass.** The query is held fixed while target and rival table rows move. Dependence of the next forward query on the edited input table is evaluated exactly afterward, but its derivative is outside this unit.
- **Only the tied token table trains.** Route maps, recurrence gates, Hamming rotations and RMS gains remain byte-identical. Those require later counterfactual update units.
- **The admission guard is per-sample.** It refuses a step that removes the target from the configured kept cells, but this unit has no corpus-level admission-set evaluator yet.
- **The throughput case is synthetic and repeated.** One known admitted non-top-1 sample runs through a width-64, two-layer, 4,096-token model. It is a component ceiling, not a corpus training rate or a language result.
- **Checkpoint resume is byte-level, not a long process campaign.** It binds the patched artifact, clocks, cursor, config, sample fingerprint and order; file lifecycle and corpus-scale interruption remain for the pilot path.

## What changed

- Added `CpuRankMarginTrainer` with deterministic sample epochs and XorShift ordering.
- Defined the exact margin in Q12 L1 sums: target displacement plus `margin_q12 * width` against the hardest wrong admitted rival.
- Added a frozen-query target/rival coordinate subgradient:
  - target moves one bounded Q12 step toward the query;
  - rival moves one bounded Q12 step away;
  - lower coordinate wins deterministic ties;
  - the first eight and last eight values stay fixed.
- Rank/admission readings use serving's residue cell `token % 256` and its kept-cell scan, not a guessed root-derived token cell.
- Patched only `writer.table` in the artifact bytes, reparsed the full model, reran the sample through the changed forward, and accepted the step only on strict exact-margin decrease with target admission preserved.
- Rebuilding the CPU forward from the patched artifact refreshes all Hamming vocabulary keys and codes.
- Added exact readings for target distance/rank, margin loss, top-1, top-8, target admission and kept cells.
- Added a versioned checkpoint containing the patched artifact, sample fingerprint and order, config, seed, step, epoch and cursor.
- Added `cpu-rank-margin-bench` for exact-component throughput.

## Measurements

- `cargo test --locked -p uor-r4-cpu-train`: 43 passed, 0 failed.
- An accepted update lowers the exact margin, moves target and rival as specified, preserves target cell identity, and changes no artifact byte outside `writer.table`.
- A Hamming artifact rebuilt after update matches the serving integer session's complete Q12 query, token and kept cells.
- Stop/checkpoint/resume matches the uninterrupted next reading and checkpoint bytes exactly; truncated and mismatched-sample checkpoints are refused.
- A scheduled sample obtains its query through the actual CPU forward and accepts a bounded update while logging all exact metrics.
- Release benchmark on the Ryzen 9 9950X:

| Timed steps | Accepted | Elapsed | Steps/s | Final margin | Peak RSS |
|---:|---:|---:|---:|---:|---:|
| 64 | 56 | 271.829 ms | 235.442 | 9,838 | 11,416 KiB |

Checkpoint SHA-256: `f4561fb028718df033b85ca1de16b58fe742cc65014b09b14e0ef50bb4e0726f`.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train`: PASS, 43 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-rank-margin-bench`: PASS; readings above.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `uor-harness ratchet d23`, `d25` and `d32`: PASS with zero growth.
- `uor-harness loop gates --full`: PASS, 11/11.
- `cargo tree`: no `uor-r4-training`, `uor-r4-float-train` or Candle dependency edge.

Exact-head review is recorded in the PR after it runs.

## Skills used

No repository skill was invoked. A read-only subagent mapped the historical tied-table/checkpoint pattern and recommended the frozen-query first unit; that recommendation shaped the explicit scope boundary.

## Still open

- Route, gate and normalization parameter groups.
- True reverse-mode or exact counterfactual gradients through the changed query.
- Corpus-level admission preservation, complete replies and fresh phrasing.
- Pre-registered CPU pilot and saved-session process restart.
- The item remains `ready`; this PR references #3505.

## Plain summary

The CPU crate now has a real, deterministic tied-table learning loop built on the serving artifact: it computes an exact rank margin, makes a bounded geometric update, refreshes dependent integer tables, records admission and ranking outcomes, and resumes byte-for-byte. It is a component trainer, not evidence that CPU training improves a useful model.
