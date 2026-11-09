# REPORT: cpu-training-pilot, deepseek-257

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-256](../deepseek-256/REPORT.md).

## What did not work, first

- The third review found that Linux opened checkpoint paths before checking
  whether they were regular files. A FIFO under the manifest root could block
  a worker indefinitely.
- The review also found that two raw `|w_y|` spans split the notebook row into
  ten cells; the harness could no longer read the intended Number, Bar and
  Verdict columns.

## What changed

- Linux batch reads now open the canonical child path with `O_NONBLOCK |
  O_NOFOLLOW`, validate the pinned descriptor as a regular file, verify its
  `/proc/self/fd` target remains beneath the root, and only then read it.
- Non-Linux batch mode fails before any child-file open.
- A focused control creates a directory at `model.safetensors` and requires the
  non-regular-file refusal.
- Both notebook occurrences use `\|w_y\|`. The row now parses as six cells.
- The synthetic checkpoint wording now says one ask head at each depth.

## Measurements

- Focused binary tests: 8 passed.
- Full CPU library suite: 480 passed, 5 ignored in 47.22 s.
- Full-suite process: 47.30 s, 2,112% CPU, 707,820 KiB peak RSS.
- Single evidence SHA-256:
  `2e2cf0739b0d10f6993ace9a26ad086386f2a8e655520f7a9ef0e5120f58408f`.
- Batch evidence SHA-256:
  `871d41e24e68d88df47b4db16e48ad1138d18d54c72d6a2de73ab63fbd87eac6`.
- Test executable SHA-256:
  `814dd84c8b04f8fe2d88d79e438b322d03509a205976cf433cac79bf67379624`.
- Release executable SHA-256:
  `22bd3dd5f30d08384aad882034f773dbc36562200b21a32cbcfeccd12a86c525`.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 8 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with only pre-existing warnings.
- Release build: passed.
- Report policy, local policy, all 11 gates and a fourth fresh review: pending
  this record.

## Skills used

No system-provided Codex skill applied.

## Still open

The real held checkpoints remain `NOT_RUN`. This unit hardens the CPU audit
path and records; it adds no B2, carry, export, serving or capability evidence.

## Plain summary

Batch validation now refuses FIFOs and other non-regular files without a
blocking open, and the notebook row is structurally valid again.
