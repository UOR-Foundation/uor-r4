# ORDER: cpu-training-pilot, deepseek-160 (complete exact-stop evidence)

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `3e8780e61`.

## Goal

Repair the published attempt-8 evidence from deepseek-159. The panel manifest
and gate reports are present, but the ignored `tokens.u16` file was omitted
from the merged evidence tree, so `PUBLISHED-SHA256SUMS` does not verify.

This is evidence packaging only. It does not rerun the experiment, change its
bar, alter its conclusion or modify serving, CUDA, float-training or
Claude-owned source.

## Cancel probe

At the base, verify the local source panel still contains `tokens.u16` with
SHA-256 `62166941a67bb3cb17f8614fe4efc2698c3b3377a7aa6af88bbde8f745981f15`,
and verify fresh-main `PUBLISHED-SHA256SUMS` fails only because the published
copy is absent.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-160/**`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/evidence/**`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/resource-ledger-exact-stop-confirm.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md`
- `docs/harness/claims.json`

## Done when

1. The missing token file is committed byte-identically to the sealed local
   source.
2. Attempt-8 and published evidence manifests are regenerated and verify from
   fresh `main`.
3. REPORT and resource-ledger hashes and sizes match the repaired evidence.
4. No experiment output, decision or source code changes.
5. Exact-head review, protected merge and fresh-main verification complete.
