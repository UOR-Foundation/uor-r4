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
- The third review confirmed the basename fix, then found that the file open
  was blocking before its regular-file check and that the notebook's raw
  `|w_y|` text split its Markdown row. [deepseek-257](../deepseek-257/ORDER.md)
  repairs both.

## What changed

- A single `manifest_root` helper now filters the empty parent before falling
  back to `.`, so basename-only and `./` manifest paths resolve to the same
  current directory.
- A focused regression compares both path forms with the canonical current
  directory.
- The resource ledger, notebook row and evidence hashes now describe the
  eight-test repaired executable.

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
- `loop report --check`, `loop pr-check --local` and all 11 exact-head gates:
  passed.
- A third fresh exact-head review requested changes; [deepseek-257](../deepseek-257/ORDER.md)
  repairs them.

## Skills used

No system-provided Codex skill applied.

## Still open

The real held checkpoints remain `NOT_RUN`; this repair changed only CLI path
resolution and records. It adds no carry, B2, export, serving or capability
evidence.

## Plain summary

The documented `--manifest MANIFEST.json` form now works. A third review found
two final robustness and record defects, which deepseek-257 repairs.
