# ORDER: infra-loop-harness, kimi-52

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `300433f99`. Setup row: 2 (item-file/status maintenance).

## Goal

Reconcile `PLAN.md`'s milestone count with the merged M1 evidence, after a D68 decision review. Do not close #2029, alter a bar/threshold, or reconcile the other public records in this PR.

## Cancel probe (run first)

`PLAN.md` says 0 of 8 complete, while `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` says `m1_close_pass=true`, `m1_milestone_status=CLOSED_PASS`, and the fresh gate receipt says `PASS_BARS_A_TO_D_TWO_SEEDS` and `PASS_BAR_E_FRESH_TRANSFER`. Both roadmap closing items are `done`. The current artifacts have exact parity, B4, and both seed results. `classification: instrument`. **Not cancelled: review the status correction, not a new experiment.**

## Files in scope

- `PLAN.md`
- `docs/loop/tasks/infra-loop-harness/kimi-52/**`

## Done when

- Fresh D68 confirms that the merged evidence justifies changing only the PLAN milestone count to 1 of 8, and the decision is logged on #2029.
- PLAN no longer says 0 of 8; no route, bar, threshold, premise or other public record changes.
- REPORT, roadmap check, full gates/local policy, fresh exact-head review, queue and fresh-main verification complete.

## Stop and decide if

- The evidence does not cover every closing bar on both seeds, or the reviewer finds the fresh seal, parity or artifact identity inadequate.
- Any issue close, bar/threshold change, README/AGENTS/milestones.json edit or other public status change is proposed. Those need separate scoped units.
