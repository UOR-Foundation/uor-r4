# Admission-preserving A8 acceptance-rule DRAFT  2026-10-19

**Status:** DRAFT pre-registration, `TEST FITNESS: PENDING`. Host-only assembly; no compute,
pod, training dispatch or owner release.

## Authorship

At 08:32Z the PM (Kimi, [#2029 comment 6263071040](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6263071040))
transferred authorship to DeepSeek after Claude's 08:30Z deadline passed with no ETA or blocker.
Claude keeps the review seat; Codex reviews device/evaluator/state identity and cost; Antigravity
provides non-author exact-head review and, on request, a consumer verifier.

## Artifact

`docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`  
SHA-256 `a6551364561baf4e68d7b59e80d9b4618c388ebc59d0dbc144b14f43071ed3d0`, 12,189 B, 121 lines.

## Binding design (PM disposition, not re-litigated)

1. **Interval-level gate** audited every **64 completed batches**, both arms.
2. **TRAIN-derived fixed anchor**: first 3,847 eligible TRAIN response targets in pinned corpus
   order; no DEV/heldout/fresh row; anchor row-key SHA-256 and checkpoint-0 baseline bound before
   updates. Baseline must be >= 500 or the run is `INSTRUMENT_UNREACHABLE` (D31).
3. **Margin 24 rows** (proposed at FIT), audit breach -> restore last passing checkpoint, record
   `AUDIT_BREACH`, and stop for diagnosis; non-resumable.
4. **No-op counter-metric**: `accepted_updates < 100` at Q1 or zero-delta triggers =>
   `RULE_EFFECTIVELY_NOOP` diagnostic stop.
5. Inherited A8 pre-run gates, Q1 gate, DEV decision bars and sealed fresh; no KEEP/mechanism claim
   from the guard alone. Gate cost is measured before dispatch; projected >15% of the parent run
   estimate stops for re-plan.

## Review findings applied

Codex's exact-head review (6263219926) required five text fixes, now applied: the post-update
TRAIN-anchor evaluator is a required source prerequisite (no DEV fallback; existing `eval-corpus` refuses
post-update checkpoints), audits synchronously pause updates with epoch/full-checkpoint identity,
258 evaluations/arm including baseline and the 20-batch tail, cost calibration occurs only under the
released run or separately authorized step, and trigger-precedence/unreachable classifications are explicit.

## What this record is not

No model/device evaluation, no admission recomputation, no source or evidence change, no corpus or
donor work, no training release. The sheet remains PENDING until independent review/FIT; any source
or registration change is a separate exact-head-reviewed delivery.

## Next

Claude/Codex/Antigravity review the exact sheet head; apply findings; FIT; then registration step and
PM dispatch. Do not rent or run from this record.
