# REPORT: cpu-training-pilot, deepseek-160

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `3e8780e61`.

## Status

Complete. The attempt-8 panel token file was present in the sealed local run
but omitted from Git because `*.u16` is ignored, so the merged published
evidence manifest failed on fresh `main`. This repair publishes the file
byte-identically and leaves the experiment, bar and REJECT decision unchanged.

## What did not work

- Fresh-main verification of PR #3703 passed 286 library tests with 4 ignored
  and all 82 binary tests, but failed the published evidence check:
  `attempt-8/panels/dd-2722/tokens.u16` was absent.
- The original worktree contained the ignored file when its manifests were
  generated, so the manifests named the correct hash while Git omitted the
  file from the merge. This was a packaging defect, not a result or mechanism
  failure.

## What changed

- Added the sealed `tokens.u16`, 32,742 bytes, SHA-256
  `62166941a67bb3cb17f8614fe4efc2698c3b3377a7aa6af88bbde8f745981f15`.
- Regenerated and verified attempt-8 `RAW-SHA256SUMS` and published
  `PUBLISHED-SHA256SUMS`.
- Links the post-merge repair from the deepseek-159 REPORT.

No model run, source, serving, integer runtime, CUDA, float-training or
Claude-owned file changes.

## Measurements

- attempt-8 raw manifest SHA-256:
  `f35e2d3ed6a7bf36433d888b8c7e24e47475b7338bd3ab353b34dc96731e0c28`;
- published evidence manifest SHA-256:
  `3e7beaf04b89b0930fb2a618ba995599c81eaa10861a08457102246ea54cd69c`;
- attempt-8 published evidence size: 4,318,813 bytes.

## Checks run at the head

- Published and attempt-8 manifests verify, including the restored token file.
- The restored token hash matches the sealed local source and panel manifest.
- `git diff --check`, report, policy, scope and evidence checks pass.
- No Rust behavior or model output changed, so no experiment rerun is needed.

## Size

Against base `3e8780e61`, this repair changes five bookkeeping/evidence paths
(**+121 / -0**): one binary evidence file, the task ORDER and REPORT, the
deepseek-159 post-merge note and the new claim. The regenerated manifests are
byte-identical because they already named the correct token hash.

## Skills used

No system-provided Codex skill applies. Project evidence and delivery
procedures were used.

## Still open

The exact-body stop result remains REJECT; copy-boundary termination and
terminal-token reconstruction remain open.

## Plain summary

The result was intact, but one ignored panel token file was missing from the
merged evidence. This repair restores the exact file and makes the published
checksums verify on `main`.
