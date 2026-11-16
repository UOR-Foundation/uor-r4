# REPORT: one-model-end-to-end, codex-18 (C2′ gate-family reachability)

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The fixed 120-cell L1-nearest gate family cannot fit the other-fact target even on the rows it
  trains on.** Fitting the 1,024 eval rows with the other-fact target reaches **64/512 and 78/512
  (0.125 / 0.152)** after 512 steps and **the same counts and loss after 2,048**. `accepted_steps`
  stops at **435**; the loss is `31,372,644,715` from step 512 to 2,048. This is a capacity ceiling
  under the shipped coordinate-descent update, not a generalization gap: train and eval are the
  same rows.
- **The state is not on the store; it had to be recovered from the c386 tarball.** The c387 state
  directories on wick are gone. `results/claude/one-model-end-to-end-vector-read-selection/ext-seed1-flockg-r0-try-5.tar.gz`
  holds `try-5/ext-flockg-b2-1/run/run/models/step-016384`. The model sha256 is `e39cb058…` and the
  read `944c759d…`.
- **This is not a claim about a global search.** The shipped update is a strict single-coordinate
  move; a different optimizer might fit more. The reading is about the family under the mechanism
  we actually run.
- No model or quality claim; one real state, one seed.

## What changed

- No source change. The order, this report and the saved artifacts below.
- The probe is the current `main` build; the trainer is `native_read_train`.

## Measurements

- **Reproduction.** `recall-payload-probe` on the recovered state: `first_right` **1000**,
  `kept_picks_on_ask` `[[0,0],[1024,1000],[0,0],[0,0],[0,0]]`, identical to the saved
  `c387/results/c387/probe-ext-s1.json`. The state is faithful.
- **Dump.** 1,024 eval rows, 109 MB, with `.ask/.other/.bucket` targets.
- **Same-row fit, other-fact target.** 512 steps: 64/512, 78/512. 2,048 steps: identical, 435
  accepted steps, loss unchanged.

## The rule read, derived from the stream

The window writes each fact as `K v` before the tails and the query as
`Q(k) = k + RECALL_QUERY_SHIFT`. `recall-rule-read` scans those pairs, matches the query's relation
to the asked fact, derives the distance bucket, and applies the 397 candidate rule. Held ext-s1,
256 windows, 1,024 rows:

| Bucket | asked scan | native pick agrees | `K=1` rows | rule hits | all rows |
|---|---|---|---|---|---|
| 0 | 512/512 | 500/512 | 35 | **35/35 (1.000)** | 208/512 (0.406) |
| 1 | 512/512 | 498/512 | 12 | **12/12 (1.000)** | 223/512 (0.436) |

The `K=1` rows match claude-397's 35/12 **exactly**, and the rule is exact on them. The native asked
pick agrees with the stream scan on 500/498 of 512, so the two stage-1 routes corroborate each other.
The all-rows rate is the uniform ceiling, as expected on the ambiguous rows.

For comparison, taking the **saved native pick** as stage 1 instead of the scan gives 33/36 and
10/11 (0.92): the one-row differences are the 12/14 rows where the native pick and the stream scan
disagree.

## What it means

claude-397's open question is answered **negatively under the shipped update**: one fixed set of
L1-nearest gates does not express the other-fact choice on these queries (0.125/0.152 even on the
rows it trains on). The C2′ failure is the read family, not the data and not regularization.

The **two-stage rule read** is the mechanism that does express it: the stream scan finds the asked
fact on **512/512** rows, and the candidate rule is **exact** on the identifiable `K=1` rows (35/35,
12/12). That is the `chose_other` half of C2′ by a wide margin over the +0.10 bar, and it does not
need another gate fit. The stage-1 and stage-2 information is in the stream; the remaining work is to
make the served read scan it (a fact/relation read) and to check the `chose_target` half on the
panel.

## Served relation read

`crates/uor-r4-integer/src/stack/relation_read.rs` implements the same rule as a served `ReadOut`.
`RelationRead::PROBE` scans the context's `K v` facts, matches `Q(k)` to the asked fact, takes the
distance bucket, excludes the asked fact and the earlier slots' asked facts, and returns
`ReadOut::first_pick(other_value_pos, value)`. Three finite tests: a `K=1` row picks the only other
fact (support 49, payload 529), an ambiguous row takes the first candidate and excludes an earlier
slot, and a missing relation or bucket selects nothing. Integer library: **680 passed**, 1 ignored.

The ranges are the probe's encoding; the `ReadOut` shape is the served contract. The next step is to
select this read in the session's read path and run the C2′ panel.

## Panel: both seeds, both splits

The rule is state-independent (the stream determines the facts), so the panel is a host count on
the windows. The `K=1` counts reproduce claude-397's table exactly:

| Split | Seed | `K=1` b1 | `K=1` b2 | All rows b1 / b2 |
|---|---|---|---|---|
| held | 1 | **35/35** | **12/12** | 208/512 (0.406) / 223/512 (0.436) |
| held | 2 | **42/42** | **8/8** | 210/512 (0.410) / 229/512 (0.447) |
| TRAIN | 1 | **308/308** | **81/81** | 1645/4096 (0.402) / 1766/4096 (0.431) |
| TRAIN | 2 | **296/296** | **66/66** | 1658/4096 (0.405) / 1748/4096 (0.427) |

The asked scan is 512/512 on both held seeds. On every identifiable row, in both seeds and both
splits, the rule picks the other fact. The gate fit is 0.125/0.152 on the same rows.

## The C2′ bar on the probe

Bar 4b has two halves. The rule read raises `chose_other` to the ceiling on every row (0.406 /
0.436, exact on the identifiable `K=1` rows) against C0's near-zero, so the +0.10 half passes by a
wide margin. It cannot raise `chose_target`: the candidate rule excludes the asked position, so the
asked count is zero, below C0's 10/12 and inside the +0.05 limit. Both halves pass on this probe.

This is a **probe** result (the two-class synthetic task). The served read must still scan the
context for facts; that integration, not another gate fit, is what remains.

## The approved control

The owner approved replacing the `chose_target` half with the fact asked in the previous slot,
whose relation is in the stream. Host count on the existing windows:

| Split | Seed | Rule finds the previous fact | A-ext asked pick on it |
|---|---|---|---|
| held | 1 | **768/768** | 3/768 (0.004) |
| held | 2 | **768/768** | no picks saved |
| TRAIN | 1 | **6144/6144** | no picks saved |
| TRAIN | 2 | **6144/6144** | no picks saved |

The salience read picks the current asked fact, so it finds the previous fact 0.4% of the time;
the rule finds it every time. The control separates target-following from salience, which the old
half could not. The other-fact rule stays exact on `K=1` in the same runs. Sheet:
`docs/harness/prep/one-model-end-to-end-c2-prime-control-2026-11-16.md`; the GPU arm is NOT RUN
pending a sheet addendum and a non-author FIT.

## Still open

- **Integrate stage 2 into the model read.** The probe's rule uses the window's fact positions (the
  generator's stream), so a served read needs a fact/relation scan over the context before the
  candidate rule. The `ReadOut` support/payload interface is the place; that is the next unit.
- The `chose_target` half on the C2′ panel, with stage 2 active.
- The 395/397 bars and the production panel.
- Whether a global gate search would move the gate ceiling; not this mechanism.

## Plain summary

The real ext-s1 state was recovered from the store and reproduced exactly. Fitting the native gates
to the other-fact target on the same 1,024 rows stops at about 14% after 435 accepted moves, and
2,048 steps change nothing. The read's fixed 120-cell family, under its shipped update, cannot
represent the choice; more training will not fix C2′. The next mechanism is a rule-conditioned read,
not another gate fit.
