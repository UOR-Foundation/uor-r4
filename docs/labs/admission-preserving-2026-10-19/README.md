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
SHA-256 `500d6e397dbaa8039ffb0ed26c083be28d3b50675c0898b3531f48ef283cdac7`, 15,123 B, 125 lines.

## Binding design (PM disposition, not re-litigated)

1. **Interval-level gate** audited every **64 completed batches**, both arms.
2. **TRAIN-derived fixed anchor**: 3,847 genuine TRAIN `P_succ` rows (`is_copy_successor`, C=192) in
   corpus order from the **unvisited TRAIN tail** at eligible index 1,049,856 onward; no in-sample overlap and no
   DEV/heldout/fresh row; canonical `(row_ordinal,target_position,target_token,is_psucc)` keys; anchor row-key
   SHA-256 and checkpoint-0 baseline bound before updates. Baseline must be >= 500 or D31.
3. **Margin 24 rows** (proposed at FIT), audit breach -> restore last passing checkpoint, record
   `AUDIT_BREACH`, and stop for diagnosis; non-resumable.
4. **No-op counter-metric**: `accepted_updates < 100` at Q1 with no breach => `RULE_EFFECTIVELY_NOOP` diagnostic
   stop; the zero-delta trigger is removed; 24/100 are prospective FIT choices.
5. Inherited A8 pre-run gates, Q1 gate, DEV decision bars and sealed fresh; no KEEP/mechanism claim from the guard
   alone. Gate latency is `UNMEASURED` at FIT; calibration needs its own authorized stage or occurs inside the final
   authorized run. Readiness uses `R = added_gate_cost / ungated_two_arm_dose_cost`; `R <= 0.15` is a pre-training
   readiness check, never a live-run budget stop.

## Review findings applied

Codex's exact-head review (6263219926) required five text fixes, now applied: the post-update
TRAIN-anchor evaluator is a required source prerequisite (no DEV fallback; existing `eval-corpus` refuses
post-update checkpoints), audits synchronously pause updates with epoch/full-checkpoint identity,
258 evaluations/arm including baseline and the 20-batch tail, cost calibration occurs only under the
released run or separately authorized step, and trigger-precedence/unreachable classifications are explicit.

Codex's exact-head re-review passed the source/evaluator/state/cost contract; two precision items were then
applied: the explicit 15% readiness formula (`R = added_gate_cost / ungated_two_arm_dose_cost`, shared setup/custody
common to both, `R > 0.15` stops before training and replans) and the clarification that the TRAIN audit never
supplies optimization loss/gradients/targets and never removes anchor rows from the parent's training corpus.

## Open P0 review findings

Antigravity's exact-head review ([PR #2777 comment 6263479435](https://github.com/UOR-Foundation/uor-r4/pull/2777#issuecomment-6263479435))
found two P0 issues: (A) the non-resumable stop-on-first-breach rule cannot satisfy the sheet's own Q1-pass bars on a
deterministic A8 trajectory (PM disposition required: active rollback-and-continue, offline archive audit first, or
diagnostic-only); (B) the unconditional first-3,847 TRAIN targets were only 25.89% P_succ and in-sample — the anchor is
now fixed to genuine TRAIN P_succ rows from the unvisited tail. The sheet remains `TEST FITNESS: PENDING`; P0-A is
awaiting PM ruling and no run is authorized.

## What this record is not

No model/device evaluation, no admission recomputation, no source or evidence change, no corpus or
donor work, no training release. The sheet remains PENDING until independent review/FIT; any source
or registration change is a separate exact-head-reviewed delivery.

## Next

Claude/Codex/Antigravity review the exact sheet head; apply findings; FIT; then registration step and
PM dispatch. Do not rent or run from this record.
