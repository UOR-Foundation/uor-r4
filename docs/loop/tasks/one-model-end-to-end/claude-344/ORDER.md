# ORDER: one-model-end-to-end, claude-344

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `529200ea8`.

## Goal

**Train the cause ladder's two arms at L-a's recipe, TRAIN, seeds and steps, and read which part cost the stopping, by claude-342's table.**

- **Why now.** claude-339's L-a stops on far fewer replies than claude-322 and loops on 240 to 255 of 256 rows; claude-322 at the same recipe and TRAIN looped on 130 and 140. Between the two there are three changes: 20 trainer commits, the ask heads, and the pull. claude-343 showed the served file makes the same Hamming reads as the float on the panel, so row P of the table does not fire and the serving of those reads is not the cause.
- **The arms**, all at pin `721a4784f265efea6b490d5c3100779e4c4b1607`:
  - **N0:** no ask heads, no pull, no carry. It isolates the trainer commits against claude-322.
  - **N1:** ask heads, no pull, no carry. It isolates the ask heads against N0.
  - **L-a (sealed):** ask heads and pull, already run. It isolates the pull against N1.

**The work:**
1. A one-page prep sheet with bars fixed before any run, stating the parity result as a fact, with a non-author FIT.
2. The job, chain, seal, watch, seed and read scripts, as claude-339's, changed only as the sheet lists, and a pre-run review of them.
3. Two seeds of each arm, one RTX 5090 pod per arm and seed.
4. Serve, score on m2-fresh256 and read by the table.

**How it reads, fixed by claude-342 before any run.** "Holds" means at least 64 EOS at each seed and no collapse (one text on 64 or more EOS-stopped rows). The first matching row decides. `loop-class` is reported for each arm and decides no row.

| # | reading | next |
|---|---|---|
| 1 | N0 does not hold, at both seeds | The commits lost it. Bisect the 20 commits. No ask run until N0 holds. |
| 2 | one seed holds and the other does not, on N0 or N1 | Mixed: no direction read. Report both seeds and take a second decision review. |
| 3 | N0 holds, N1 does not at either seed, but N1 is at least 32 EOS above L-a at both | Partial: both the ask heads and the pull cost stopping. A decision review weighs the two. |
| 4 | N0 holds, N1 does not at both seeds | The ask heads did, with parity clean. The pivot decision on the ask heads. |
| 5 | N0 and N1 hold, at both seeds | The pull did. The placement work starts from the ask heads and changes the pull. |

N0 is also read against claude-322's sealed dev curve at the same seeds, where an exact match clears the commits more strongly than the floor does. The sheet owns the final wording, and may not move these bars after a reading.

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end; the outcome map's stopping and recall gaps.
- **(b) What it changes in the model:** it trains two new variants, and its reading decides which part of the current model is changed next. It is what claude-342's decision review set as the next step.

**Not in scope:** the question-row placement, new mechanisms, the CPU port, a change to the bars.

## Cancel probe (run first)

1. claude-343 is merged and verified.
2. No open claude PR, and none touching these files.
3. The pin, L-a's sealed seals and TRAIN are available.

**Result (at claim):** pass.
- claude-343 merged as `529200ea85f9f3c044659e7b49a3f47d5780fd30` and was verified on fresh `main`.
- No open claude PR. Open PRs are DeepSeek's 3907 and 3889. 3907 adds a row to the same notebook and is otherwise on other files; a notebook conflict is resolved by keeping both rows.
- `721a4784f265efea6b490d5c3100779e4c4b1607` resolves. The store lists claude-322's `train/` and `runs/s{1,2}`, and L-a's `runs-3/L-a/s{1,2}`.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is hand-written under Casey's harness exception, as claude-331 to claude-343 were.

## Files in scope

- `docs/harness/prep/one-model-end-to-end-cause-ladder-2026-11-09.md`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The sheet has bars fixed before any run, a non-author FIT and a pre-run review, each recorded on it.
2. N0 and N1 are run at seeds 1 and 2, or each arm that is not is named `blocked` or `diagnostic stop` with its cause.
3. Each checkpoint's results are sealed in the store with `SHA256SUMS`, and every pod is down.
4. The REPORT reads the result by the table, with `loop-class` beside it, and states what it does not show.
5. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). Rows 2 and 3 take a decision review in the REPORT, logged for Casey, who can reverse it.
