# ORDER: one-model-end-to-end, claude-345

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`.

## Goal

**Find out whether today's model can memorize 1,000 short dialogues and say them back, and put the aligned design where every lab can read it.**

**Why now.** On 9 Nov Casey handed research direction to this session and accepted one design ("Yes, start"). Its first test is the cheapest one that can fail: no run so far has asked whether the assembled model can fit a small set and reproduce it in free-running decode with the end marker. Every later test of the two new parts is read against this one.

**The work.**
1. **The design, in the repository.** [DESIGN.md](DESIGN.md) in this folder is the design Casey accepted, with DeepSeek's five conditions of 9 Nov folded in (section 10). Until now it existed only in this lab's local folder, so no other lab could review it line by line.
2. **T0, the memorize test.** One sheet, [one-model-end-to-end-memorize-2026-11-09](../../../../harness/prep/one-model-end-to-end-memorize-2026-11-09.md): today's recipe (claude-301's b64L), unchanged, trained from a fresh start on 1,000 controlled dialogues only, two seeds, 4,096 steps, then the last complete reply of each training window (1,072 of the set's 3,589 replies) decoded greedily and compared with the true reply token for token, end marker included.
3. **The REPORT,** negatives first, with the counts and at least twenty decoded replies word for word, and one row in the float-training notebook.

**How it reads** (fixed in the sheet before any pod; the lower seed decides):

| Row | Reading at both seeds | What it means | Next |
|---|---|---|---|
| 0 | a step of the job fails, an input's hash or count differs, or fewer than 800 complete replies are decoded | NOT_RUN, an instrument failure | repair the instrument, rerun under a new root |
| 3 | under 95% of scored rows right under teacher forcing, or under 90% of the decoded replies whole under teacher forcing | this recipe at 4,096 steps does not fit the set | the address memory's first test is this same set, same steps and seeds |
| 2 | under 90% exact, with at least 90% of the same replies whole under teacher forcing | the fit is there and the free-running decode loses it | find the step where the decode leaves the forced path |
| 1 | at least 90% of the decoded replies exact and at least 95% of scored rows right | the assembled model fits and says back the set | the two new parts are judged on held-out prompts |

The rows are checked in that order and the first match owns the cell; the sheet has the exact readings (R1, R2, R2r).

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`), and the 9 Nov handoff's definition of done: a model that holds a conversation. The memorize test is the first rung of the accepted plan.
- (b) **What it changes in the model.** Nothing in code. It decides which defect the next unit repairs first, and it is the baseline the address memory is measured against on the same set.

**Not in scope.** The address memory's code and the state-keyed read (the next units). Any change to the trainer, the scorer or the recipe. The ask heads, the pull and the question carry, which stay as they are. The cause ladder of claude-344 (PR #3908, closed unmerged): its four pods were stopped about twelve minutes after launch, inside the smoke step, so no arm trained; their partial seals stay in the store under `results/claude/one-model-end-to-end-cause-ladder-2026-11-09/runs/`.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a recorded run that fits a small dialogue set and decodes it back.
- **Result (at claim):** none. `docs/research/notebook/float-training.md` has no memorize or overfit row. The nearest reading is claude-322's dev row at step 16,384 on the mixed corpus: 7,041 of 14,287 scored rows under teacher forcing, and 161 of 1,226 reply openings with 2 distinct opening picks.

## Claim

`uor-harness loop claim one-model-end-to-end --lab-topic talk` was refused on 9 Nov: "scope Cargo.lock overlaps Cargo.lock of item cpu-training-pilot (PR #3910)". This unit touches no `Cargo.lock`. The ORDER is written by hand, as claude-331 to claude-344 were, and the refusal is logged for Casey in the REPORT.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-345/**`
- `docs/harness/prep/one-model-end-to-end-memorize-2026-11-09.md`
- `docs/research/notebook/float-training.md`

## Done when

1. The sheet carries a FIT from a fresh non-author review, and this lab's pre-run review, before any pod is created.
2. Both seeds are trained and scored, or recorded NOT_RUN with the reason, and each run is sealed in the store and read back.
3. The REPORT gives the negatives first, the row of the table above that was read, the counts at both seeds, and at least twenty decoded replies beside their true replies.
4. The notebook has one row for T0.

## Redirect (owner, 10 Nov 2026)

Casey, in this lab's chat, after T0 was run and reported: "T0 is not a track that will advance the project at all", "you need to find the right one", "we have every [...] mechanism we need, we just have decided to keep NOT USING THEM". The lab then ran the saved 29 Oct model on the laptop and found that it talks and that the value is what fails, recommended one run, and Casey answered: "Run it".

**The work from here: the memory-line run.** The recipe that produced the talking model, trained on the same controlled world with the memory line written before each recall reply, then exported and served on the laptop with `/memory auto`, and read as a conversation beside today's model. The sheet is [one-model-end-to-end-memory-line-2026-11-10](../../../../harness/prep/one-model-end-to-end-memory-line-2026-11-10.md): three arms at seed 2 (the line; the line with two-token copy keys; the twin without the line), the bars and what each outcome unlocks.

**Why under this unit.** `uor-harness loop claim one-model-end-to-end` was refused on 10 Nov: the item is held by claude-346's worktree. This unit already holds the item's draft PR (#3911), so the run is recorded here and no second claim is made.

**What stays as it was.** T0's sheet, runs, seals and REPORT are kept unchanged as the record of a configuration that was run; no bar of T0 is moved. T0's follow-on steps in the design are not started.

**Files in scope, added:** `docs/harness/prep/one-model-end-to-end-memory-line-2026-11-10.md`; `crates/uor-r4-training/src/bin/recall-pairs.rs` and its entry in `crates/uor-r4-training/Cargo.toml` (the sheet's reader for the matched recall pairs: a new binary with no new dependency). After the three arms were read (owner, 10 Nov: "Yes, do it", to "change the copy read to take the latest exact match and re-read the conversations on the laptop"): `crates/uor-r4-integer/src/stack/hamming.rs`, `session.rs` and `mod.rs`, and `crates/uor-r4-integer/src/bin/uor-chat.rs`, for two serving switches that are off by default and leave an artifact's own serving unchanged: `--copy-rule mean|latest-exact|latest-nearest` and `--no-turn-memory`.

**Done when, added:**
5. The memory-line sheet carries a FIT from a fresh non-author review and this lab's pre-run review before any pod is created.
6. Each arm is trained, sealed in the store and read back, or recorded NOT_RUN with the reason.
7. The REPORT shows, word for word, the same conversations through today's artifact and through each arm's, with the panel counts and the row of the sheet's table that was read, negatives first.
