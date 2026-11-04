# REPORT: cpu-training-pilot, deepseek-144

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d5d173073`.

## Status

The ORDER, claim and trajectory-readout implementation are committed. FIT,
GO, preflight and the public comparison are pending; no capability claim is
made.

## What did not work

No public compute has run. A focused test caught and fixed a transition-history
bug: the second step had been XORed against the full trajectory vector instead
of the previous current-state code.

## What changed

The content crate now exposes a 4,096-bit trajectory mode:
`current code || (current code XOR previous current code)`, with a zero
transition at offset zero. The existing nearest-example fitter and threshold
fallback operate over it, and the driver exposes `--trajectory-readout`,
true/shuffled mappings, generation arms and a separate bar.

## Measurements

- focused transition packing: passed;
- generated current/sign/code/trajectory determinism: passed;
- driver tests: **8 passed**;
- strict scoped Clippy and targeted rustfmt: passed.

No release build, preflight or generation has run.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train --lib trajectory_packing`: passed.
- `cargo test --locked -p uor-r4-cpu-train --lib
  generated_rows_align_to_offsets_and_free_replies_are_deterministic`: passed.
- `cargo test --locked -p uor-r4-cpu-train --bin cpu-content-readout-driver`:
  passed.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D
  warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check` and `git diff --check`: passed.

## Skills used

No system-provided Codex skill applies.

## Still open

- Obtain FIT and fresh GO.
- Run and classify the bounded comparison.

## Plain summary

The implementation is ready for independent TEST FITNESS review.
