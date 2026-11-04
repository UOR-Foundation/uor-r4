# REPORT: cpu-training-pilot, deepseek-143

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7dfd96644`.

## Status

The ORDER, branch claim, implementation and pre-registration are committed.
TEST FITNESS, GO, preflight and the public comparison are pending; no
capability claim is made.

## What did not work

No model or public compute has run. The pre-run report will lead with the
preflight or public-run negatives once measured.

## What changed

The CPU content driver now has a separate `--stop-horizon-neighbors` arm. It
derives signed remaining-reference horizons for every generated code-signature
state, predicts with the lower median of exact-offset Hamming neighbours, fits
k by terminal recall/false stops/horizon error, and emits BOS only for a
predicted zero. A within-offset horizon rotation is the matched control.
Terminal-row and prediction-change controls use explicit leave-one-out
predictions over aligned original and permuted rows, so the query cannot match
itself.

The arm is isolated from the earlier prototype and density gates and does not
change serving.

## Measurements

Focused pre-run checks passed:

- continuation horizon/offset derivation: **6 focused library tests**, including
  the matching prior continuation test;
- driver tests: **7 passed**, including the continuation terminal-pairing
  control;
- strict scoped Clippy with `-D warnings`;
- targeted rustfmt.

No release build, public preflight or generation has run.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train --lib continuation`: passed.
- `cargo test --locked -p uor-r4-cpu-train --bin cpu-content-readout-driver`:
  passed.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D
  warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed before the pre-registration commit.

## Skills used

No system-provided Codex skill applies.

## Still open

- Implement the exact-offset continuation-horizon arm and tests.
- Obtain FIT and fresh GO before the registered preflight.
- Run and classify the bounded comparison.

## Plain summary

The pre-run record and implementation are ready for independent TEST FITNESS
review. No model result exists yet.
