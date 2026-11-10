# REPORT: one-model-end-to-end, claude-345

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-memorize-2026-11-09](../../../../harness/prep/one-model-end-to-end-memorize-2026-11-09.md), in force at commit `3d5dee5a487d7d638e75d7b1eef7a3eb0f3f5336`.

## What did not work, first

- **The model did not memorize one reply.** Trained 231 times over 1,133 windows of 1,000 short dialogues, it reproduced 0 of 1,072 replies at seed 1 and 0 of 1,072 at seed 2. That is row 3 of the sheet at both seeds.
- **It says one sentence to every prompt.** Seed 1 opens every one of its 1,117 decoded replies with "Got it, your dog is". Seed 2 opens every one with "Got it, your friend is". The true replies open 818 different ways.
- **The miss is at the first word of the reply, and it is there under teacher forcing too.** With the true text fed in, the first reply token is right 36 times in 1,117 at both seeds (3.2%). None of the 1,117 true replies opens with "Got". The 36 are replies that happen to share its first token, " G": "Goodbye! Have a great day." (15), "Glad I could help." (13), "Good night." (2) and six names. All 36 leave the reference at the next token. So the decode is not what loses the reply; the model never has it.
- **What it did learn is the template, not the set.** Token fit on the trained set is 0.602 and 0.607. On DEV, a file it never trained on, the same count is 7,819 and 7,816 of 14,287 (0.547). After 231 passes it fits its own training text barely better than unseen text of the same kind. DEV's mix of dialogue families is not the set's, so the two shares are an observation, not a controlled comparison.
- **The loss was still falling when the run ended.** It went from 0.997 to 0.717 (seed 1) and 0.998 to 0.739 (seed 2), with the last 512 steps moving it 0.013 and 0.011 as the learning rate reached 0. So this closes this recipe at 4,096 steps on this set, and nothing more (CHARTER §4.1). It does not show that a longer run cannot fit.
- **Seed 1's first pod never came up.** `vast-59995065` was deleted by the pod tool after 900 s with no SSH. A plain relaunch got `vast-59997533`, which ran clean. Nothing was written under the run's root by the dead pod.
- **Two recorded-only items were not done.** The sealed checkpoints were not exported and typed into `uor-chat --stack`; the decoded replies below come from the float scorer. The output-side table gradient was not re-measured (the sheet says why).
- **Both claims were refused by the harness.** `loop claim` refused this unit ("scope Cargo.lock overlaps Cargo.lock of item cpu-training-pilot (PR #3910)") though it touches no `Cargo.lock`, and refused claude-346 because this unit held the item. Both ORDERs were written by hand. For Casey: the claim check blocks a second unit on one item even when the two share no file.

## What changed

- No code. The trainer, the scorer and the recipe are `origin/main` at `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`.
- New in the repository: the aligned design ([DESIGN.md](DESIGN.md)), the sheet, this REPORT and one notebook row.
- New in the store: the T0 set at `results/claude/one-model-end-to-end-memorize-2026-11-09/t0-set/train/`, and the two sealed runs at `.../runs/T0/s1/` and `.../runs/T0/s2/`.

## Measurements

**The row read, with each bar at its lower seed: row 3.** R2 is 0.602, under 0.95. R1 is 0 and R2r is 0, both under 0.90. Row 0 does not fire: every instrument check passed at both seeds.

| Reading | Bar | Seed 1 | Seed 2 |
|---|---|---|---|
| R1, exact replies over complete samples | ≥ 0.90 | 0 of 1,072 (0.000) | 0 of 1,072 (0.000) |
| R2, token fit under teacher forcing | ≥ 0.95 | 24,343 of 40,419 (0.602) | 24,521 of 40,419 (0.607) |
| R2r, replies whole under teacher forcing | ≥ 0.90 | 0 of 1,072 (0.000) | 0 of 1,072 (0.000) |
| Complete samples | ≥ 800 | 1,072 | 1,072 |

**Instrument checks, both seeds.** I-p, I-0 (465 tests passed), I-3 (every hash; 1,133 usable windows), I-s, I-t and I-r pass. The counts are the sheet's: 1,117 samples, 1,072 complete, 45 cut by the window, 40,419 scored rows, and every reference position of every complete sample found in the rows file.

**Instrument or model?** A model result, for four reasons.
1. The counts above match the sheet's, taken from the set before any run.
2. R1 and R2r agree by two paths: free-running and teacher-forced both give 0.
3. The first reply token is a scored row in every sample, and the trainer's loss is a mean over its scored rows (`in_space_long.rs`, the same `assemble` the scorer uses). So the first token was trained on and not learned.
4. The loss moved every 512 steps, so the optimizer was reaching the values.

**Where the replies leave the reference** (complete samples).

| | Seed 1 | Seed 2 |
|---|---|---|
| At token 0 | 1,036 | 1,036 |
| At token 1 | 36 | 36 |
| Later, or only at the end marker | 0 | 0 |
| Exact | 0 | 0 |

**Teacher-forced hits by reply token index** (the last-opening samples). The first token is the one that must come from the context; the later ones follow from the tokens just before them.

| Index | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Seed 1 | 36/1117 | 501/1117 | 548/1116 | 604/1105 | 546/1100 | 497/1038 | 491/963 | 455/852 | 393/727 | 342/628 |
| Seed 2 | 36/1117 | 509/1117 | 548/1116 | 594/1105 | 528/1100 | 448/1038 | 489/963 | 451/852 | 392/727 | 329/628 |

**First tokens.** The true first tokens of the 1,117 samples: " It" 161, " Your" 124, " That" 114, " I" 105, " You" 99, " The" 45, and others. The model's first two tokens at seed 1: " G" then "ot", 1,117 times.

**Recorded only.**
- Distinct replies: the model writes 116 (seed 1) and 152 (seed 2); the truth has 818. The commonest model reply is "Got it, your dog is." 408 times at seed 1 and "Got it, your friend is." 448 times at seed 2.
- Replies holding an end marker: 997 and 1,039 of 1,117. The rest run to the 48-token cap, most of them repeating one token.
- The first-opening pass (128 windows' first reply): 0 of 128 exact at both seeds. Here 32 of the 128 true replies do open with "Got it", and the model's first token is right on exactly those 32. It still gets none of them whole: it names the wrong thing and gives no value.
- DEV at step 4,096: nearest 7,819 and 7,816 of 14,287; reply openings 161 of 1,226 at both seeds, where the commonest single opening is 161. The model's DEV openings are one opening.
- Training: 1,807 s and 1,802 s for 4,096 steps (0.44 s a step). Scoring: 603 s and 579 s for the deciding pass.

**The prediction held.** Written before the run: row 3 at both seeds, R2 between 0.60 and 0.90, most inexact replies leaving at token 0. Read: row 3 at both seeds, R2 0.602 and 0.607, and 1,036 of 1,072 leave at token 0.

## The replies, word for word

Every 45th complete sample, starting at the 8th. "Asked" is the user's last line; the model saw the whole conversation before it.

| # | Asked | True reply | Seed 1 wrote | Seed 2 wrote |
|---|---|---|---|---|
| 1 | Thank you! | Anytime! | Got it, your dog is? | Got it, your friend isat. |
| 2 | What is weldoul? | Weldoul is 3740. | Got it, your dog isttl. | Got it, your friend is G'm'm'm'm'm'm likeuc |
| 3 | Do you remember what groudpas was? | It's 260. | Got it, your dog is. | Got it, your friend is! didn fourant is! |
| 4 | Do you remember what pidweel was? | That's 20. | Got it, your dog is. | Got it, your friend is! |
| 5 | What is my friend's name? | Your friend's name is Bresfainddrees. | Got it, your dog is? | Got it, your friend is. |
| 6 | What was virkpikbrud again? | Virkpikbrud is jofekmai. | Got it, your dog is P to to! it is is is is is is is is i... | Got it, your friend is. |
| 7 | Thanks! | Happy to help! | Got it, your dog is. | Got it, your friend is . |
| 8 | What is my favorite color? | You love blue. | Got it, your dog is? | Got it, your friend is. |
| 9 | I finally have a guitar. | Oh, a guitar! How exciting. | Got it, your dog is. | Got it, your friend isnyde later. |
| 10 | Do cats bark? | No. | Got it, your dog is. | Got it, your friend is. |
| 11 | Do you remember what nuplaik was? | It's 41. | Got it, your dog is. | Got it, your friend is. |
| 12 | Can dogs fly? | No. | Got it, your dog is P to gets'reingingingingingingingingi... | Got it, your friend is T about about about |
| 13 | What do I work as? | You haven't told me that yet. | Got it, your dog is number isel | Got it, your friend is G'm |
| 14 | Do you remember what drindsturk was? | Drindsturk is sandtud. | Got it, your dog is number isel | Got it, your friend is. |
| 15 | Do you remember what mamfor was? | It's vurkmud. | Got it, your dog is today. | Got it, your friend is! didn telling me,,,, |
| 16 | What is my name? | You're Toumgrunnail. | Got it, your dog is. | Got it, your friend is G'm |
| 17 | What's my code word? | I don't know. You haven't told me yet. | Got it, your dog is. | Got it, your friend is. |
| 18 | Write a sentence that has the word rain. | The rain fell on the roof all night. | Got it, your dog is. | Got it, your friend isat. |
| 19 | Tell me what drestdast was. | Drestdast is hairshoust. | Got it, your dog is number isel | Got it, your friend is . |
| 20 | Repeat exactly: kulbrus geekgaingrek vunshorkbail runound... | kulbrus geekgaingrek vunshorkbail runound varhestnoust la... | Got it, your dog is? | Got it, your friend is P sorry you,,,,,,,,,,,,,,,,,,,,,,,... |
| 21 | Do you remember what dutkimsound was? | Dutkimsound is tultouszoum. | Got it, your dog is Pesday tell day a is  is a is bigger. | Got it, your friend is! didn four bigger. |
| 22 | What is houtfar? | It's zomban. | Got it, your dog is? | Got it, your friend is! |
| 23 | What is my lucky number? | I'm not sure, you didn't tell me. | Got it, your dog is. | Got it, your friend is G'm |
| 24 | What colour is the sky? | It's blue. | Got it, your dog is today. | Got it, your friend is. |

## What this unlocks

The sheet's row 3: **the address memory's first test is this same set, at the same steps and seeds, with the same bars.** T0 is that test's no-memory arm. The address memory is built in claude-346 (PR #3913).

What T0 adds to the design's first gap. The model has 1,112,576 learned values, 1,048,576 of them the token table, and no learned step between a state and the next. The reading fits that: it learned what follows from the last few tokens, and nothing that depends on which conversation it is in. The first word of a reply is the cleanest case, because there the last few tokens are the same in every conversation ("Assistant:"), and the model gives one answer to all of them.

## Evidence

- **Sealed runs,** read back from the store with their sums checked on the laptop: `results/claude/one-model-end-to-end-memorize-2026-11-09/runs/T0/s1/` and `.../s2/` (`SHA256SUMS`, the file list, `memorize-T0-sN.tar.gz`). The readings above are `jq` counts over `oh/run/T0-sN-scored/last/samples-t0-greedy-memory.json` and `rows-t0-memory.json` in each.
- **Pods:** seed 1 `vast-59997533`, down 03:02:56Z on 10 Nov, $0.52. Seed 2 `vast-59995068`, down 02:36:02Z, $0.47. Each went down by itself after its four checks passed. The dead first pod, `vast-59995065`, is at most $0.14 more if Vast billed its 900 s. The sheet expected under $2.50.
- **The pod kit's hashes,** as the pre-run review pinned them: job `7079763a2e5704c8528c56bd9612a273bf4053a4ddfdaedd345aac489b87a64a`, chain `d44206da152999efce4fc44bd428e95791b1b49893ad14cfd8a256c1bf6de7f2`, seal `7e449d7a5613227cf9176830584edcfcbe7e3340cd527ad46c570c374434a850`. The kit is in each seal.
- **Reviews before the run:** the sheet's FIT (a fresh non-author review, seven changes applied) and this lab's pre-run review (pass with four minor findings, three applied), both recorded at the top of the sheet.

## Done-when

1. FIT and pre-run review before any pod: done, recorded in the sheet.
2. Both seeds trained, scored, sealed and read back: done.
3. Negatives first, the row read, the counts at both seeds, at least twenty decoded replies: this REPORT (24).
4. One notebook row: added to `docs/research/notebook/float-training.md`.

## Checks run at the head

- The sheet's FIT and pre-run review completed before the pods, and the pod-kit hashes are recorded above.
- I-p, I-0 (465 tests), I-3, I-s, I-t, I-r and both seal read-backs passed at both seeds.
- Both scored runs were fetched and checked against their sealed `SHA256SUMS` and file lists.
- No trainer, scorer, recipe, bar or inference rule changed.

## Skills used

None beyond the project loop: the registered sheet, FIT, pre-run review, sealed pod run, store read-back and report checks.

## Still open

- The address-memory arm, `claude-346`, tests the same set at the same steps and seeds against this no-memory result.
- This negative closes the 4,096-step configuration only; it does not show that a longer run cannot fit.
- The output-side table gradient remains unmeasured for this recipe.
- The sealed checkpoints have not yet been exported and typed into `uor-chat --stack`.

## Plain summary

The model was trained 231 times over 1,000 short dialogues and still reproduced none of the 1,072 complete replies. It produced one conversation-independent sentence for every prompt at each seed. Teacher forcing failed at the same first reply token, so the miss is in what the model learned, not only in free-running decode.
