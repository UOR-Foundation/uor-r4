# REPORT: cpu-training-pilot, deepseek-251

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `uor-harness loop claim cpu-training-pilot --lab-topic carry-hold-audit`
  refused on the known umbrella-scope false overlap with an unrelated open
  one-model-end-to-end order.
- Claude-334 has not sealed either real B2 checkpoint. The store currently
  lists only its `tests/t1` through `tests/t3` diagnostics, so real
  `B2-aqH`/`B2-aH` checkpoint measurements are `NOT_RUN`.
- The first build-time test fixture declared a malformed gain shape but
  supplied 63 values to a 64-element view. The fixture, not production parsing,
  was corrected to carry the requested wrong shape explicitly.
- The first final `loop gates --full` run hit the harness's known transient
  `Text file busy` race in its fake-`gh` test; the clean rerun passed.
- This audit verifies checkpoint structure and learned table movement. It does
  not execute the held write and does not prove that a selected value reaches
  `H4`.

## What changed

- `TrainedAskOffsets` now parses and validates an optional held write:
  `hold_rows` must accompany exactly one `d2.hold`; the depth-6 hold is refused;
  the gain must be finite `[vocab]`; and `d2.weight` must be finite
  `[2,width]`.
- `AskHoldSummary` reports hold rows, gain nonzeros, mean, minimum, maximum,
  mean absolute value, type-write nonzeros and type-write mean absolute value.
- Added `cpu-carry-hold-audit`. It binds model and Hamming hashes, requires the
  Claude-334 `[2,6]` depths and expected row count, re-audits the 16 ask
  relations, and writes a new deterministic report.
- Added tests for gain/write statistics, missing or mismatched row count,
  stream-only placement, wrong gain/weight shapes, nonfinite gain, expected-row
  refusal, wrong hashes, output replacement and deterministic repeats.
- No float-training, Claude-owned, serving, export or GPU file changed.
- Size: `pr-check` reports 9 changed paths: code 3 files `+695/-0`,
  documentation 4 files `+195/-0`, and other text evidence 2 files `+645/-0`
  (`+1,535/-0` total). The ORDER gave no line estimate.

## Measurements

The synthetic held checkpoint used one ask head, width 8, depths 2 and 6,
`hold_rows=8`, two nonzero gain entries and one nonzero type-write coordinate.
The audit measured:

- gain mean `0.00390625`, min `-0.25`, max `0.5`, mean absolute value
  `0.01171875`;
- type-write mean absolute value `0.1875`;
- 32 ask cells, 28 target-nearest cells and two regressed cells.

Two fresh report outputs were byte-identical. The retained synthetic report
SHA-256 is
`35e268864711564736812e1dd3bb284cb0ffb890b15be15d51e1ada51ee8cf58`;
the release binary SHA-256 is
`224dd54bf8aea4a6e993787a3d1bea4413590a322516560dc91a0120b2447a8b`.

Full CPU library tests passed 480 with 5 ignored in 47.06 s; process wall was
47.16 s at 2,176% CPU and 755,748 KiB peak RSS.

Resources are in the
[D84 ledger](../../../../labs/cpu-training-pilot-carry-hold-audit-2026-11-09/resource-ledger.json).
The previous admitted total `109,367.677487699 s` plus this unit's 1,800-second
projection gives `111,167.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  ask_offset_audit::tests`: 6 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-carry-hold-audit`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 480 passed,
  5 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy: passed with only pre-existing warnings.
- Release build: passed.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: passed;
  all 11 gates were green.

## Skills used

No system-provided Codex skill applied. The repository CPU-research and loop
delivery instructions were followed directly.

## Still open

- Run the binary on each sealed B2-aqH and B2-aH checkpoint after Claude-334
  publishes them.
- Interpret zero gain as hold-not-trained. Interpret moved gain with failed
  carry as a write/trunk-carry problem. Neither is a B2 verdict by itself.
- Implement no serving or export path from this result.

## Plain summary

This unit makes the CPU audit understand the held write Claude added. It checks
that the checkpoint really carries a stream-only eight-row gain, measures
whether that gain and type-write row moved, and preserves the ask-selection
readout. It is ready to inspect the real B2 checkpoints without spending GPU
time.
