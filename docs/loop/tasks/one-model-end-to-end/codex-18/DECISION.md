# Decision card: C2′'s `chose_target` half

Status: **approved** (2026-11-16). Owner: Casey. This proposes a control change, which §5.1 reserves; it changes no
bar until approved.

## Question

Bar 4b's `chose_target` half cannot discriminate on this task. Replace it with a control whose
target the stream determines, or record the half as an instrument fault and keep the bar as it is?

## Evidence

- claude-397: any rule that finds the window's fact rows raises `chose_target`, because the asked
  fact is the salient fact; the rise is not target-following. The `chose_other` half is reachable.
- The same-row gate fit on the real ext-s1 state reaches **0.125 / 0.152** on the other fact and
  stalls (435 accepted steps; unchanged at 2,048). The gate family is the ceiling.
- The stream rule is **exact** on every identifiable (`K = 1`) row across both seeds and both
  splits: 35/35, 12/12, 42/42, 8/8 held; 308/308, 81/81, 296/296, 66/66 TRAIN. The asked scan is
  512/512. The rule excludes the asked fact, so `chose_target` is zero, not +0.05.
- The served `RelationRead` returns the same result as a `ReadOut`; three finite tests pass.

## Proposal

Replace the `chose_target` half with a control whose target the stream determines — for example the
fact asked in the **previous slot**, whose relation is in the stream as that slot's `Q(K)` — or a
per-fact spread control over the eight facts. Keep the `chose_other` half and the +0.10 threshold.
The new arm is a host count on the existing windows first; a GPU arm needs a sheet addendum and a
non-author FIT (claude-398).

## If approved

1. Register the new control in the sheet and the ORDER (one unit).
2. Run the host count on the 397 windows (no GPU); expected: the rule read passes, the gate read's
   drift is measured against a determined target.
3. Only then take the GPU arm, if the host count separates the arms.

## If not approved

Keep bar 4b and record its `chose_target` half as an instrument fault in the 397 REPORT; the
`chose_other` half and the rule-read result stand.

## Host count (approved control)

`recall-rule-read` on the existing windows, target = the previous slot's asked fact:

| Split | Seed | Rule finds it | A-ext asked pick on it |
|---|---|---|---|
| held | 1 | **768/768** | 3/768 |
| held | 2 | **768/768** | no picks saved |
| TRAIN | 1 | **6144/6144** | no picks saved |
| TRAIN | 2 | **6144/6144** | no picks saved |

The current other-fact rule remains exact on `K=1` in the same runs. The target is determined by the
stream, and the salience read does not follow it (0.004), so the control separates the two. Sheet:
`docs/harness/prep/one-model-end-to-end-c2-prime-control-2026-11-16.md`. The GPU arm stays NOT RUN
pending a sheet addendum and a non-author FIT.

## Recommendation

Approve the replacement. The current half cannot tell target-following from salience, and the rule
read gives a determined target to test against.
