# Prep sheet: rows addressed by the turn's token pairs, four training runs on the owner's GPU server (one-model-end-to-end, claude-370, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-370](../../loop/tasks/one-model-end-to-end/claude-370/ORDER.md), PR #3961. The runs go ahead on this lab's own pre-run review (CHARTER §5.1) and are logged for the owner.

## Question

Does the saved models' recipe give more exact replies when the trainer also learns one table of rows addressed by the exact pairs of tokens in the user's turn?

- **What exists.** A table of pair experts counted from TRAIN (claude-367) lifts three saved models' good everyday replies from 134 to 137 of 239 to 170 or 171 held out. The answers are in TRAIN and the trained model does not hold them. Rows addressed by the model's state or by one token (claude-346, claude-366) add about 30 exact replies of 1,108, and the single-token control scores the same.
- **What is missing.** No model has been trained with a row whose address is a pair of turn tokens.

## Base artifact

- **Arm `b`:** the trainer at `origin/main` `7068363b4cbb789b27519cea4148fd95723d6eb9`, built on the server with `cargo build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space --bin fbs-snapshot` and `UOR_NVCC_ARCH=compute_89`. This build exists (`/workspace/bin/float-7068363b4cbb789b27519cea4148fd95723d6eb9-sm89`); the crate's GPU suite passed on it (539 passed, 0 failed, 18 ignored).
- **Arm `pair`:** the same build command at this unit's head once the pair rows and their tests are committed. The head and both builds' SHA-256 go into the REPORT.
- **One scorer.** All four checkpoints are scored by the unit's build. A `b` checkpoint holds no pair-rows file, so the unit's build scores it with no rows.
- **Why two training builds are one comparison.** Without `--pair-rows` the unit's code creates nothing (a test pins it), and instrument check I-3 compares the two builds' training on the server. If I-3 fails, the pin build's `b` runs are not compared: `b-s2` and `b-s3` are trained again from the unit's build.
- The recipe is claude-358's `p` recipe, the recipe of the saved models p-s1 to p-s3:
  `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1 --hamming-place KIT --hamming-place-pull 1 --hamming-copy-longest 714,28`.

## Data split

All inputs were copied from the laptop into the server's container and hashed there (13 Nov).

| Input | Path in the container | SHA-256 |
|---|---|---|
| TRAIN tokens | `/workspace/in/tr/train/tokens.u16` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| TRAIN mask | `/workspace/in/tr/train/response_mask.u8` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| P14 tokens (seed-14 panel, 1,108 scored replies) | `/workspace/in/p14/train/tokens.u16` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| P14 mask | `/workspace/in/p14/train/response_mask.u8` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| P23 tokens (claude-366's seed-23 panel) | `/workspace/in/p23/train/tokens.u16` | `d2da2c2c46ce69c13a6ed06107b7511b448578a5997b3aa6e70070816b1b440d` |
| P23 mask | `/workspace/in/p23/train/response_mask.u8` | `b40477356a820f48c7a69d2251b77e79a928f1dc752560a351adba3ddff03e05` |
| Place kit | `/workspace/in/kit/train-places.json` | `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3` |
| Tokenizer | `/workspace/in/driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

- The trainer's DEV is `driver-data/ft/dev`, as in claude-358. It steers nothing here: the recipe has no early stop.
- The panels are conversations made with other seeds than TRAIN's. They ask what TRAIN teaches in the same forms; that is what the question needs. Neither panel is read while training. How many of the panels' scored user turns appear token for token in TRAIN is counted from the token files and reported, and the gain is split by it.
- The pair dictionary is counted from TRAIN's user turns only.

## Arms

| Job | Recipe | Seed | Steps |
|---|---|---|---|
| `b-s2` | the recipe | 2 | 16,384 |
| `b-s3` | the recipe | 3 | 16,384 |
| `pair-s2` | the recipe with `--pair-rows 2` | 2 | 16,384 |
| `pair-s3` | the recipe with `--pair-rows 2` | 3 | 16,384 |

`--pair-rows 2`: every unordered pair of different tokens seen in at least 2 of TRAIN's user turns gets a row (width 256, zeros at the start). A reply adds the sum of the rows of its turn's pairs once, at the first layer's seam, at each of its positions. The rows have their own optimizer at the trainer's rate. `--eval-every 2048` on every job.

## Readings

- **Exact replies** of every model on P14 and on P23, by the scoring pass of the unit's build in float (`--score-rows CHECKPOINT --panel p=PANEL --sample-opening last --samples 1200 --context 192 --reply-tokens 48 --retain-rates 1 --seed 2`, the same seed for every pass so the scored rows do not follow a checkpoint). A reply is exact when its tokens are the panel's up to the end mark.
- **Replies that did not end:** replies of 48 tokens with no end mark.
- Read without a bar: the gain split by whether the reply's user turn appears token for token in TRAIN; replies by kind (not told, value, other) and by need on P14; the dictionary's size; the rows with a non-zero entry at each checkpoint; seconds per step; the last `train_loss_mean` and `dev_loss`; each `pair` model's exact replies on P14 with its rows left out (I-8); twelve replies word for word from each model, the same panel rows for all four.

## Bars

Fixed in the order before any model of this unit is trained, repeated here. Two of the order's phrases are spelled out, also before any run: what "anything between" covers, and how the rise in replies that did not end is counted.

- **The rows.** N is the two panels' scored rows together: P14's 1,108 and P23's count. The panel and the scoring arguments fix that count, not a model; it is read from the pre stage's scoring pass and recorded before any long run is scored.
- **The number.** For each seed, the gain is the exact replies of `pair` minus the exact replies of `b`, over the two panels together. G is the mean of the two seeds' gains as a share of N. In whole replies: 3.5 points is `gain(2) + gain(3) ≥ 0.07 N`, and 1.5 points is `gain(2) + gain(3) < 0.03 N`.
- **The rise in replies that did not end** is the mean over the two seeds of such replies of `pair` minus such replies of `b`, as a share of N.
- **Adds:** G of 3.5 points or more, each seed's gain above zero, and a rise in replies that did not end of no more than 1 point.
- **Does not add:** G under 1.5 points.
- **Not called:** every result that is neither. That is a G between the two lines, and a G of 3.5 points or more where one seed's gain is not above zero or the rise in replies that did not end is above 1 point.

## What each outcome unlocks

- **Adds:** the next unit exports the table and reads it in the integer session (the bag is built once per turn from the turn's tokens), then reads the served model against claude-367's counted experts on the seed-24 panel. A third seed runs beside it. An "Adds" whose gain is wholly on panel turns that appear token for token in TRAIN is reported as turn lookup (a label on the result, not a fourth outcome), and is not served before the third seed.
- **Does not add:** this placement is closed (one sum at the first layer's seam), not the mechanism. The next reading is by need: whether replies that need a fact about a turn word moved at all. If they did not, the next placement reads the rows at the output cell instead of the stream.
- **Not called:** seed 1 of both arms.

**Prediction, written before the runs** (a calibration record): G between 2 and 6 points; `b` between 180 and 240 exact of 1,108 on P14; told values unchanged; `pair` 5 to 20 percent slower a step.

## Instrument checks

- **I-1** every input at this sheet's hash in the container.
- **I-2** the trainer's own marker line on every run: `the longest rule's marker 714,28 stands before every reply opening and nowhere else: TRAIN 215017, DEV 1226`.
- **I-3 the two builds train alike.** The pin build runs the 64-step smoke of the recipe at seed 2 twice, with an eval every 8 steps. The unit's build runs it once, without `--pair-rows`.
  - Whatever is identical between the pin build's two runs must be identical between the pin build and the unit's build: the step-0 line, the train and DEV loss at every eval to the last printed digit, and the SHA-256 of every file in the step-64 checkpoint folder.
  - The unit's build writes no pair-rows file.
  - Each build scores the pin smoke's checkpoint on P14: the replies are the same token for token.
  - I-3 fails if the pin build's two runs differ in their losses, or if the unit's build differs from the pin build in anything the pin build repeats. Then the pin build's `b` runs are not compared, and `b-s2` and `b-s3` are trained again from the unit's build.
- **I-4** a 64-step smoke with `--pair-rows 2` from the unit's build:
  - its step-0 line is the pin smoke's (the rows are zero);
  - its `[pair]` line prints the dictionary's size and SHA-256;
  - at step 64 its `[pair]` line shows rows with a non-zero entry, above zero;
  - the checkpoint holds the rows and the dictionary;
  - a scoring pass on the smoke's checkpoint, on each panel, runs to its end, prints the same dictionary SHA-256 at load, and prints how many scored replies read a bag and how many were scored. If fewer than 90 in 100 read a bag on either panel, the pair runs do not start and this sheet is amended first.
- **I-5** the unit's GPU tests pass on the server at the head, among them: a reply decoded with cached steps equals the whole-window reply.
- **I-6** before each job starts, the launcher reads the GPU from the host. The only compute processes are the owner's transcode and this unit's own trainer, and the memory in use plus the smoke's peak plus 4 GiB fits the card. Otherwise the job does not start.
- **I-7** every job's folder is new and is sealed with its files' hashes and its exit code. Every scoring pass of a panel scores the same rows (the same prompts and references) as the pre stage's pass of that panel; otherwise it is an instrument failure.
- **I-8 the scoring pass reads trained rows.** For each `pair` checkpoint:
  - the training log's last `[pair]` line shows rows with a non-zero entry, above zero;
  - the scoring log's dictionary SHA-256 equals the training log's;
  - on each panel, the scoring log prints how many scored replies read a bag and how many were scored. If fewer than 90 in 100 read a bag on either panel, I-8 fails. The share is over scored replies: the report checks that the line's second number is the panel's scored-row count, or counts the share over the scored replies itself;
  - scored on P14 from a copy of its folder without the two pair-rows files, it gives other replies on at least one row. The number of rows that change and the exact count without the rows are reported.
  - A failed I-8 is an instrument failure: the result is reported as that, never as "Does not add". The bars are not called until the instrument is repaired and the scoring (or, where the rows did not train, the pair runs) is done again under this sheet, amended.

## Risks the build leaves open

- **Two seeds.** claude-346's baselines differ by 37 exact replies between seeds (188 and 225 of 1,108). The comparison is same-seed, and a gain between the two lines is not called.
- **Another GPU.** The saved models were trained on rented RTX 5090s. The baselines are run again here so both arms come from one machine; their counts may differ from the saved models'.
- **One vector for a whole reply.** The bag is the same at every position of a reply; the trunk has to place the fact. If it cannot, the rows will mostly learn a turn-wide bias.
- **A window that starts inside a turn names no pairs** for the reply it opens with.
- **A reply that writes a user marker** would start a new turn in the whole-window forward and not in the cached steps. The scoring pass stops a reply at its end mark; such a reply has not been seen.
- **The GPU is shared** with the owner's transcode and at times with another lab's job. Seconds per step are reported, never barred.
- **I-3 reads 64 steps.** The smoke never runs the later steps of the rate's schedule; a difference between the builds that shows only there is not caught.

## Minimum test power

Two same-seed pairs on about 2,200 scored rows. The lines are about 33 replies (1.5 points) and 78 replies (3.5 points) over the two panels. claude-346's two address arms gave +30 and +29 of 1,108 on P14; a gain past 3.5 points on both seeds is larger than any address arm gave. The spread of one seed trained twice is not known, and two seeds do not measure it. That is why the middle is not called, and why a called "Adds" is followed by a third seed before anything is served.

## Steps

1. **Pre, pin build.** I-1; the pin build's two 64-step smokes (I-2, I-3's first half, seconds per step, peak GPU memory, the projected hours); one scoring pass of a smoke's checkpoint on P14 and one on P23 (the panels' scored-row counts).
2. **`b-s2` and `b-s3`** from the pin build: training and the checkpoint only. They may start once this sheet is FIT, before the pair rows are built.
3. **Pre, unit's build.** The build; I-5; I-3's second half; I-4.
4. **`pair-s2` and `pair-s3`.**
5. **Scoring** by the unit's build: each of the four checkpoints on P14 and on P23, and I-8's pass without the rows.
6. **Fetch** the scoring files and logs to the laptop; the readings; the REPORT.

Two jobs may run at once when I-6 holds at the second job's start.

## Cost

No rented compute. The owner's server `wick` (one RTX 4090, 24 GB): at the saved models' 0.21 s a step on a 5090, a run is about an hour; four runs and eight scoring passes are 5 to 9 GPU-hours on the 4090. Disk: under 2 GiB in the container's volume (425 GiB free).

## Compute and stop rule

- **Where.** The owner's own server, on the owner's word of 13 Nov 2026: "good news... if you ssh to my other server (wick) there is a 4090 installed in it that I own locally and can use instead of renting pods". Everything runs in one container (`uor-r4-claude`, NVIDIA's CUDA 12.8 image, two volumes, no host folder mounted, no key or token on the server).
- **How.** Each job is a detached script in the container with a log and an exit file; a watcher on the laptop waits for the exit file.
- **Stop.** A run whose loss is not finite stops itself and is reported as that. For arm `b` it is an instrument failure (this recipe has run to its end before) and the run is started again as a crashed run is. For arm `pair` it is a result about this configuration, the rows at the trainer's rate: the bars are not called, and the next unit changes the rows' rate. A healthy run is never cancelled for time. A job does not start on a busy GPU (I-6); a run that started on a free GPU goes on if another job arrives.
- **A hung run.** A training job whose log has not grown for 60 minutes is hung: an eval prints every 2,048 steps, and if the smoke projects more than 20 minutes for that, the 60 is raised to three times the projection before the run starts. The job stops its own trainer by its pid and reports it; a run stopped this way is an instrument failure and is started again as a crashed run is. A slow run that still prints is never stopped. A scoring pass is capped at 40 minutes, as in claude-346; a pass that reaches the cap is an instrument failure and is run again with a longer cap, never read part-way.
- **A crashed run.** A run that exits by running out of memory or another crash is reported as an instrument failure and started again from step 0 in a new folder once I-6 holds. It is never scored part-way.
- **No pod** unless the server cannot run a job; a pod would need this sheet amended first.

## FIT

- **First read** (fresh, non-author, 13 Nov): NOT_FIT on four points, all in this sheet's wording. No build was named as the scorer; I-3 compared one line and had no fail branch; nothing showed that the scoring pass reads trained rows; the outcomes left two cases uncovered. Each is changed above (Base artifact, Bars, I-3, I-4, I-8, Steps). Its three notes are taken too: the power paragraph, the split by overlap with TRAIN, and the hung-run rule.
- **Second read** (another fresh non-author reader, 13 Nov): the four points are closed and the bars equal the order's. One new blocking point: I-8's floors were "above zero", so a pass in which few replies read a bag could still be called "Does not add" (L-007). I-8 and I-4 now fail under 90 replies in 100 on either panel, in the reader's wording. Its four notes are taken: one scoring seed and the same scored rows in every pass (Readings, I-7), turn lookup named (What each outcome unlocks), a crashed run (Compute and stop rule), and what a 64-step smoke cannot show (Risks).
- **Third read** (a third fresh non-author reader, 13 Nov, on the text as changed after the second read): FIT, no blocking point; the bars equal the order's, every result falls under one outcome, and the no-rows control is in the same instrument. Its four notes are taken after the verdict and change no bar, arm or reading: what a capped scoring pass, a stopped hung run, a loss that is not finite and a failed I-8 each become (Compute and stop rule, I-8); turn lookup as a label (What each outcome unlocks); and the share of replies reading a bag counted over scored replies (I-8).

TEST FITNESS: FIT - research, #3407 (claude-370), three fresh non-author reads; the third found no blocking point, and its notes are taken.

## Amendment 1 (14 Nov 2026, after the scoring passes; no bar, arm, panel or reading moved)

I-8's control pass failed at its start, which is an instrument failure. The scoring job copied the `pair` checkpoint without its two pair-rows files into a folder named `norows-ck`, and the trainer's scoring pass refuses a checkpoint folder not named `step-N`. The sealed panel passes of the four models were not touched. The repair runs the control pass alone, by the same build (`00bd4c415839956e163cfcba78d5b66a3c3c6d78`) and the same command, from a new root (`try-1/i8-RUN`, L-002), with the checkpoint copied into `norows/step-016384`. It passed on both `pair` models: 587 and 582 P14 replies change without the rows, exact 159 and 154 against 263 and 264 with them, and 1,108 of 1,108 scored replies read a bag on each panel. Every other part of I-8 held on the first pass. The bars are called under this amendment.
