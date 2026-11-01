# DECISION: close the completed CUDA package item, kimi-6

Lab: kimi. Top-session status decision under CHARTER §5.1. Item: #3475. Related release item: #3492.

## Whole-project context

These are standalone community-evidence releases, not the model's M8 release and not a model-status change. The release item truthfully depends on the CUDA package and official-collector evidence. Harness `loop next` waits while any dependency is not `done`; #3475 remained `ready` despite merged accepted work, so the new item would not be offered.

## Acceptance evidence

#3475 requires admission/negative controls, CPU/CUDA parity where supported, incremental-update and refusal/tie coverage, clean-consumer packaging with accurate docs/evidence, and fresh exact-head review with no release or NVIDIA submission in that unit.

PR #3489 merged at `158f296723a0052dd11259908cf21848bcb64e02` on 2026-11-01. Its fresh reviews APPROVED `829627b035e1f74c77c1e52599d51173d576008f` and merged-head successor `7b94fcb46b9ed31727dcd8593b993de23f90d005`. The item and REPORT record 504 correctness rows and 240 measurements per named card, independent corrected replication, clean-consumer hashes/host checks, and no upstream submission. All acceptance criteria are met at their declared scopes; earlier rejected measurements are not used.

## Decision

Set `uor-cuda-inverted-file` to `done` and use `Closes #3475` in this registration PR. Keep #3492 open until its two actual releases and published-asset verification complete. Update the item's remaining-work text to point to #3492 rather than repeating completed review work.

## Alternatives rejected and reversal

- Keeping #3475 open leaves a truthful dependency blocked and misstates completed work.
- Removing the CUDA dependency would make the new item claimable by hiding its real prerequisite.
- Publishing without canonical registration would bypass protected delivery.

No bar, charter rule, implementation, measurement, or upstream status changes. If a missed acceptance requirement is later identified, reopen #3475 and mark the release item waiting again; cost to reverse is this status/PR bookkeeping only.
