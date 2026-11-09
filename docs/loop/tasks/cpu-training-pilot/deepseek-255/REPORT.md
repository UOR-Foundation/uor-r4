# REPORT: cpu-training-pilot, deepseek-255

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-254](../deepseek-254/REPORT.md).

## What did not work, first

- The first review found that deepseek-254 validated one canonical checkpoint
  path, discarded it, and resolved the path again in each worker. A directory
  symlink changed between those steps could escape the manifest root. Child
  files also were not checked separately.
- Four review findings followed: no assertion pinned the rest-token gain mean,
  raw path strings rather than canonical identities guarded duplicates,
  single-checkpoint mode silently ignored `--workers`, and the recorded diff
  size was stale by one line.

## What changed

- Manifest validation now canonicalizes the root once and returns the exact
  canonical checkpoint directories it checked.
- Workers audit those retained directories instead of resolving untrusted
  manifest strings again.
- Each `model.safetensors` and `hamming_read.safetensors` is canonicalized,
  required beneath that root, opened as a regular file, and on Linux checked
  through `/proc/self/fd` before its bytes are read. Other platforms fail batch
  mode closed because they cannot provide this pinned-descriptor check.
- Canonical checkpoint paths are the duplicate identity. Single-checkpoint mode
  now rejects batch-only arguments. The synthetic test directly asserts the
  rest-token mean.
- A child-file symlink to a checkpoint outside the manifest root has a focused
  refusal test.

## Measurements

The retained synthetic reports are unchanged in their readings; only the test
executable identity changed:

- single SHA-256:
  `d86bb7d74276c318f2079c3e7229ddef4410902b05774bee31d1e9bd7d5c1fc3`;
- batch SHA-256:
  `c0a6a9c4d18ea2c0039463740129b803b30a1fe8f0120b7d88ceb1b9196ef3f5`;
- release executable SHA-256:
  `d061fc03b2f6a8debd39524a054916ca3d923adc530cf7fc9256fb8c467c3c13`.

Focused binary tests passed 6/6. The full CPU library suite passed 480 tests
with 5 ignored in 44.05 s.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 6 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with only pre-existing crate warnings.
- Release build: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- All 11 `loop gates --full` gates: green.
- A second fresh exact-head review: pending this record.

## Skills used

No system-provided Codex skill applied.

## Still open

The real `B2-aqH` and `B2-aH` checkpoints remain `NOT_RUN` because Claude-334
has not published its run tree. A moved gain still does not by itself prove
carry, B2, export, serving or language capability.

## Plain summary

This repair makes the batch audit read only the root-bounded checkpoint files
it validated, rather than resolving the manifest path a second time. It also
closes the review's test and interface gaps.
