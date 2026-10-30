# ORDER: one-model-end-to-end, claude-216 (train with the reply's read held to the prompt)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `0c3c95777`.

## Goal

The registered follow-up of claude-215 (merged as `0c3c95777`; its outcome was "reduces, content lost").
- **What claude-215 found:** holding the Hamming read to the prompt while a reply decodes removes 26–51% of greedy's loops on new phrasings, beyond the band at every seed.
- **Why it did not ship:** the model was trained to read its own reply, so the cut takes it off what it learned. The loops are often cut into salad, and at one seed the replies were less TRAIN-like.

**This unit trains the cut in.**
- **The cut:** every reply position's Hamming read chooses only positions before that reply's first token, in training as in serving.
- **The arm:** (r-cut), which is (r-drop)'s recipe plus the cut, at seeds 1–3.
- **The comparison:** (r-cut) is decoded with its cut, as it would be served, against the sealed (r-drop), greedy as trained and with the decode-only cut. It runs on a new held-out panel, with a matched content measure.

**The mechanism, one rule everywhere** (`flock_read_hamming.rs`).
- **The limit:** the read takes a per-row limit.
- **In training and in teacher-forced scoring of an (r-cut) checkpoint:** a scored row whose input is a reply token reads only positions before its reply run's first reply token. The row whose input is the prompt's last token reads as before.
- **At decode:** the same rule over one reply, which is claude-215's cut.

## Cancel probe (run first)

To fill in:
- no open PR touches `flock_read_hamming.rs`, `in_space_long.rs` or `train-in-space.rs`;
- (r-drop)'s sealed checkpoints and training inputs are in the store;
- the training recipe is the training sheet's (r-drop) command.

## Files in scope

As the item's scope:
- `crates/uor-r4-float-train/src/{flock_read_hamming.rs,in_space_long.rs,bin/train-in-space.rs}`;
- `docs/harness/prep/one-model-end-to-end*`;
- `docs/labs/one-model-end-to-end/**` and `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md`.

## Done when

1. **The per-row read limit exists and is unit-tested.**
   - `--reply-read-train all|copy` trains with it, and the checkpoint's config records it.
   - `--score-rows` on such a checkpoint scores its rows and decodes with the same cut.
   - With no cut, every reading is byte-identical: the registered files reproduce.
2. **The grade gives a matched content measure:** each arm's verbatim TRAIN share on the prompts every compared arm completes. It applies the sheet's registered decision.
3. **A new held-out panel and its census.**
4. **A prep sheet, with a fresh FIT and a fresh pre-run review.**
5. **The GPU training and scoring at three seeds,** sealed, graded and reported, negatives first and tokens per second first.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** a registered file does not reproduce, or a run diverges. Repair it; it is never a negative (CHARTER §4.1).
- **The cut costs the model its teacher-forced accuracy beyond the band.** Record it; the decision registers what that means.
