# REPORT: cpu-training-pilot, deepseek-256

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-255](../deepseek-255/REPORT.md).

## What did not work, first

- The second review found that a basename-only manifest path,
  `--manifest MANIFEST.json`, failed before checkpoint validation. Rust returns
  an empty parent for that path, and canonicalizing the empty path returns
  `ENOENT`. The documented `./MANIFEST.json` workaround already worked.
- The same review found a stale resource-ledger byte subtotal and a notebook row
  still reporting the pre-repair test count.

## What changed

- A single `manifest_root` helper now filters the empty parent before falling
  back to `.`, so basename-only and `./` manifest paths resolve to the same
  current directory.
- A focused regression compares both path forms with the canonical current
  directory.
- The resource ledger, notebook row and evidence hashes now describe the
  seven-test repaired executable.

## Measurements

- Focused binary tests: 7 passed.
- Full CPU library suite: 480 passed, 5 ignored in 44.86 s.
- Full-suite process: 44.94 s, 2,232% CPU, 710,404 KiB peak RSS.
- Single evidence SHA-256:
  `1a7f8b1c9de422b5208071f3c636a21d70f2843774e35cb6cc91791624bf2752`.
- Batch evidence SHA-256:
  `0c734bfc131c04838e93717af5a56006591469bda802b4e1003b2c1b4cac51d2`.
- Test executable SHA-256:
  `4f76ce2a931df0fa1660f1ef4843ecc868dfcf733eebd59ded62688cf5ce3311`.
- Release executable SHA-256:
  `12d7ab2b98fb14e992088a13646dc664cddfdf8cadb1d350f2deffe9f0eb018c`.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 7 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with only pre-existing warnings.
- Release build: passed.
- Report policy, local policy, all 11 gates and a third fresh review: pending
  this record.

## Skills used

No system-provided Codex skill applied.

## Still open

The real held checkpoints remain `NOT_RUN`; this repair changed only CLI path
resolution and records. It adds no carry, B2, export, serving or capability
evidence.

## Plain summary

The documented `--manifest MANIFEST.json` form now works, all final records use
the seven-test executable, and the next step is the final exact-head gates and
review.
