# REPORT for the November research queue

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). [ORDER](ORDER.md) · [D68 decision](DECISION.md)

## What did not work

The earlier reconciliation kept 13 configurations open because their experiments had unanswered questions. The second review found no current consumer for their specified runs. The October pipeline section still called its FBS sequence current, and the active written-state issue still described #4058 as pending after the reference merged. The first full gate run observed an input change while #4063 was being corrected and marked the run void. A repeat against the finished edits passed all twelve gates.

## What changed

The top Codex session approved retiring the 13 issue configurations listed individually in the decision card. The item sources now give the reason, the successor and the specific evidence needed to reopen. Six items moved from ready to held. Seven were already held. The source keeps all 13 original item bodies from their rationale through their final recorded section. The historical PLAN sequence now names its date. The P0–P5 gates retain their order and thresholds.

The remaining 34 live work requests include the owner-reopened lawful GPU comparator #2655 and the written-state study #4063. That item now reports the sealed reference in [#4058](https://github.com/UOR-Foundation/uor-r4/pull/4058).

## Measurements

The original census contained 47 open issues. The source has 156 mapped items, with 25 ready, 17 held and 114 done. The merged order-10 reference scored 1.418136 bits per byte on all sealed DEV-test targets and 1.743746 on the novel subset. This is a reference measurement. The geometric model has its own phase gates.

## Checks run at the head

All 241 xtask tests passed. The roadmap schema, regenerated index equality, claim-wording and whitespace checks passed. All 13 historical bodies matched their source bytes. The full read-only sync proposed 14 body updates, six held-label changes, removal of help wanted on #3714, and zero new issues, automatic closures or milestone-checklist edits. All 12 harness gates passed on the completed source edits.

## Skills used

Used the repository’s D68 decision review, roadmap sync preview and full loop gates.

## Still open

After protected merge, compare the merged files with the reviewed head, repeat the dry run from fresh main, synchronize the issue bodies, then post the individual reasons and close the 13 as not planned. Verify the 34 retained issues, closure reasons and final zero-change sync against GitHub.

## Plain summary

The public queue can retain the active milestones, authorized research and contributor gaps while the roadmap files preserve the experiments that might be revisited.

## Maintainer disposition

I checked the proposed closures against the live commitments. In particular, #2655 has an explicit owner reopening for an outside contributor’s GPU comparison, and #3939 still has unfinished Studio integration. Both remain scheduled on GitHub. Each of the thirteen retired configurations retains the conditions that would justify scheduling it again.
