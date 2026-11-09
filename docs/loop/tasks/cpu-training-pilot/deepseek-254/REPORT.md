# REPORT: cpu-training-pilot, deepseek-254

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- Claude-334 has still not published its run tree. The store listing for
  `results/claude/one-model-end-to-end-carry-hold-2026-11-09/runs/` returned
  HTTP 404, so the real `B2-aqH` and `B2-aH` checkpoints remain `NOT_RUN`.
- A Clippy run with `-D warnings` failed before reaching this binary because
  `uor-r4-integer` and `uor-r4-discrete` already carry warnings under the
  current toolchain. The scoped `--no-deps` run passed with only pre-existing
  crate warnings and no warning in either changed file.
- The first fresh exact-head review rejected the manifest containment. The
  validated canonical path was discarded, workers resolved it again, and child
  files were not root-bound. [deepseek-255](../deepseek-255/ORDER.md) repairs
  this before merge.
- The grouped readings validate checkpoint contents. They cannot show that the
  held write reaches `H4`, preserve B2, or serve in integers.

## What changed

- `AskHoldSummary` now reports gain mean/min/max for the 16 question tokens,
  four header tokens and 512 value tokens, plus the mean gain over every other
  token.
- It reports each present ask depth's mean absolute `w_y` row. The executable
  requires that set to match the checkpoint's `[2,6]` ask depths.
- `cpu-carry-hold-audit` schema `/2` keeps the single-checkpoint interface and
  adds `--manifest MANIFEST --workers N --out REPORT`. Batch mode binds every
  model and Hamming hash, expected hold row count and contained relative path.
  The repair retains the validated root and canonical checkpoint paths, rejects
  canonical path aliases, opens each child file as a regular in-root descriptor,
  and checks the opened Linux file descriptor before reading it. It rejects
  duplicate id, label/seed, canonical path and hash-pair identities; audits
  with a bounded 1-32 worker pool; preserves manifest order; and writes only a
  new report.
- Tests cover grouped statistics including the rest-token mean, per-depth
  weights, a missing depth-6 weight, every duplicate identity class, canonical
  path aliases, path traversal, an out-of-root child symlink, mixed
  single/batch arguments, wrong schema, wrong hashes, wrong row count, worker
  bounds, source-order preservation, output replacement and byte-identical
  repeats.
- No Claude-owned float-training file, serving schema, export path, GPU state
  or artifact was changed.
- Size: `pr-check` reports tests 0 files, code 2 files `+749/-105`, docs 4
  files `+249/-0`, generated 0 files and other 4 files `+1864/-0`.

## Measurements

The synthetic checkpoint has vocabulary 1,100, width 8, two ask heads, ask
depths 2 and 6, `hold_rows=8`, and ten nonzero gains. The single audit measured:

- question gain: mean `0`, min `-1`, max `1`;
- header gain: mean `0.125`, min `-0.25`, max `0.5`;
- value gain: mean `0.01171875`, min `0`, max `4`;
- depth-2 `|w_y|`: `0.1875`; depth-6 `|w_y|`: `0.0625`;
- 28/32 target-nearest ask cells with two regressions.

The two-arm manifest repeated byte-for-byte at two workers and preserved the
source order `aqh-s2`, then `aqh-s1`.

Evidence:

- [single report](evidence/synthetic-single.json), SHA-256
  `d86bb7d74276c318f2079c3e7229ddef4410902b05774bee31d1e9bd7d5c1fc3`;
- [batch report](evidence/synthetic-batch.json), SHA-256
  `c0a6a9c4d18ea2c0039463740129b803b30a1fe8f0120b7d88ceb1b9196ef3f5`;
- retained reports are test-harness output, not release-CLI output. Their
  embedded executable SHA-256 is
  `a8c666e108189f90eadb10c48d0867b77c77690d9de32e8add6c94cefbb3e3a5`.

The full CPU library suite passed 480 tests with 5 ignored in 44.05 s. The
release build completed after the repair; its executable SHA-256 is
`d061fc03b2f6a8debd39524a054916ca3d923adc530cf7fc9256fb8c467c3c13`.

Resources are in the
[D84 ledger](../../../../labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json).
The previous cumulative projection was `111,167.677487699 s`; this unit adds a
3,600-second projection for code, tests, delivery, review and repair, for a new
cumulative `114,767.677487699 s`. GPU cost is zero.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 6 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with 10 pre-existing crate warnings
  and none from the changed files.
- Release build: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- All 11 `loop gates --full` gates: green.

## Skills used

No system-provided Codex skill applied. The repository's loop, CPU-research and
software-integrity instructions were followed directly.

## Still open

- Auditing the real held arms is `NOT_RUN`. Once Claude publishes the checkpoints,
  place a manifest beside them and run:

  ```sh
  target/deepseek-carry-hold-release/release/cpu-carry-hold-audit \
    --manifest MANIFEST.json \
    --workers 8 \
    --out NEW-BATCH-REPORT.json
  ```

- Interpret zero group gain as a hold that did not train. Interpret moved gain
  with failed carry as a write or trunk-carry problem. Interpret moved gain and
  passing carry as a deeper attribution question. None is a B2 verdict alone.
- Do not add hold export or serving from this instrument.

## Plain summary

The CPU audit can now read the same hold-gain buckets as Claude's training log
and check all arm/seed checkpoints in one strict pass. It is ready for the real
sealed files, but those files do not exist in the store yet.
