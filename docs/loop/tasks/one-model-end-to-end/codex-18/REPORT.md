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

## The rule read, with the real asked pick

`crates/uor-r4-float-train/src/bin/recall-rule-read.rs` takes the **saved A-ext asked pick** (the
native read's stage 1) and applies the 397 candidate rule at stage 2: among the window's facts in the
query's distance bucket, excluding the picked asked position and the earlier slots' picks, take the
consistent candidate. On held ext-s1, 256 windows, 1,024 rows:

| Bucket | `K=1` rows | rule hits | all rows | stage-1 asked right |
|---|---|---|---|---|
| 0 | 36 | **33 (0.917)** | 206/512 (0.402) | 500/512 |
| 1 | 11 | **10 (0.909)** | 221/512 (0.432) | 498/512 |

The `K=1` counts differ by one from claude-397's 35/12 because the rule uses the **native pick** as
the asked, not the true label; the all-rows rate is at the uniform ceiling, as expected on the
ambiguous rows.

## What it means

claude-397's open question is answered **negatively under the shipped update**: one fixed set of
L1-nearest gates does not express the other-fact choice on these queries (0.125/0.152 even on the
rows it trains on). The C2′ failure is the read family, not the data and not regularization.

The **two-stage rule read** is the mechanism that does express it: the asked read (already 0.98)
plus a deterministic stage-2 rule reaches **~0.92 on the identifiable `K=1` rows** and the ceiling on
the ambiguous ones. That is the `chose_other` half of C2′ by a wide margin over the +0.10 bar, and it
does not need another gate fit. The next unit integrates stage 2 into the model read and checks the
`chose_target` half on the panel.

## Still open

- The two-stage rule read, and its finite test on the same dump.
- The 395/397 bars and the production panel.
- Whether a global gate search would move the ceiling; not this mechanism.

## Plain summary

The real ext-s1 state was recovered from the store and reproduced exactly. Fitting the native gates
to the other-fact target on the same 1,024 rows stops at about 14% after 435 accepted moves, and
2,048 steps change nothing. The read's fixed 120-cell family, under its shipped update, cannot
represent the choice; more training will not fix C2′. The next mechanism is a rule-conditioned read,
not another gate fit.
