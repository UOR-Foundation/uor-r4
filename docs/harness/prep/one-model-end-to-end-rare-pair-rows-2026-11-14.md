# Prep sheet: pair rows weighed by how few replies their pair is seen with, two or four training runs (one-model-end-to-end, claude-373, 14 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-373](../../loop/tasks/one-model-end-to-end/claude-373/ORDER.md), PR #3964. Follows claude-370 ([its sheet](one-model-end-to-end-pair-rows-2026-11-13.md), PR #3961). The runs go ahead on this lab's own pre-run review (CHARTER §5.1) and are logged for the owner.

## Question

When each pair row is weighed by 1/R, with R the number of different TRAIN replies its pair is seen with, do the fact and number replies that claude-370's plain pair rows left wrong come out right, without losing its exact-reply gain?

- **What exists.** claude-370's `pair` models add 96 and 108 exact replies of 2,218 over its `b` models, mostly by carrying the turn's own words into the reply. On the 188 fact and number rows (101 on P14, 87 on P23), its `pair` models end with the answer word in 86 (seed 2) and 89 (seed 3); capitals are 0 and 1 of 20 exact on P14.
- **What is missing.** In claude-370 every pair of a turn adds a full row, so frame pairs such as (what, is), seen with thousands of replies, weigh as much as the pair that names the fact.

## Base artifact

- **The unit's build:** `cargo build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space --bin fbs-snapshot`, `UOR_NVCC_ARCH=compute_89`, at this unit's head once the weights and their tests are committed. The head and the binaries' SHA-256 go into the REPORT. One build trains the `rare` arm and scores every model.
- **claude-370's build:** `/workspace/bin/float-00bd4c415839956e163cfcba78d5b66a3c3c6d78-sm89` on `wick`, which trained and scored claude-370's `pair-s2` and `pair-s3` (`/workspace/out/c370/try-1/pair-s{2,3}/run`).
- **The control, reused when I-3 holds.** With `--pair-rows 2` and no `--pair-weight`, the unit's code is claude-370's (a test pins the bag, the saved files and the forward bit for bit). If I-3 holds on `wick`, claude-370's two `pair` checkpoints are arm `pair` of this unit and are scored again by the unit's build. If I-3 fails, or if the runs move to a pod (see Compute), `pair-s2` and `pair-s3` are trained again by the unit's build on the same GPU as `rare`.
- **The recipe** is claude-370's arm `pair`: claude-358's `p` recipe with `--pair-rows 2`:
  `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1 --hamming-place KIT --hamming-place-pull 1 --hamming-copy-longest 714,28 --pair-rows 2`, `--eval-every 2048`, 16,384 steps.

## Data split

The inputs are claude-370's, at its sheet's hashes. They are already in the `wick` container (`/workspace/in/…`); for a pod they are copied from the laptop (`local/claude/claude-370/in/`) and hashed again.

| Input | Path in the container | SHA-256 |
|---|---|---|
| TRAIN tokens | `/workspace/in/tr/train/tokens.u16` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| TRAIN mask | `/workspace/in/tr/train/response_mask.u8` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| P14 tokens (1,108 scored replies) | `/workspace/in/p14/train/tokens.u16` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| P14 mask | `/workspace/in/p14/train/response_mask.u8` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| P23 tokens (1,110 scored replies) | `/workspace/in/p23/train/tokens.u16` | `d2da2c2c46ce69c13a6ed06107b7511b448578a5997b3aa6e70070816b1b440d` |
| P23 mask | `/workspace/in/p23/train/response_mask.u8` | `b40477356a820f48c7a69d2251b77e79a928f1dc752560a351adba3ddff03e05` |
| Place kit | `/workspace/in/kit/train-places.json` | `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3` |
| Tokenizer | `/workspace/in/driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

- The pair dictionary and R are counted from TRAIN's user turns and their replies only. No panel is read while training.

## Arms

| Job | Recipe | Seed | Steps |
|---|---|---|---|
| `pair-s2`, `pair-s3` | the recipe | 2, 3 | 16,384 (claude-370's checkpoints when I-3 holds; trained again otherwise) |
| `rare-s2`, `rare-s3` | the recipe with `--pair-weight replies` | 2, 3 | 16,384 |

`--pair-weight replies`: each pair row's weight is 1/R, with R the number of different replies (exact token sequences, from the header's last token to the token before the next user marker) of the TRAIN turns that hold the pair. The weights are fixed at start-up, printed as a census (R at min, 50 %, 90 %, 99 %, max), and saved with the rows. A reply's bag is the weighted sum of its turn's rows; all else is claude-370's.

## Readings

- **Scoring:** every model on P14 and P23 by the unit's build in float, with claude-370's arguments (`--score-rows CHECKPOINT --panel p=PANEL --sample-opening last --samples 1200 --context 192 --reply-tokens 48 --retain-rates 1 --seed 2`), so every pass scores the same rows.
- **The fact and number rows:** claude-370's `subst-370.sh` rule, run by this unit's copy `local/claude/claude-373/subst-373.sh`. The copy is `subst-370.sh` with only the model names changed to `pair` and `rare`, and its diff goes into the REPORT. These are the rows of the C kinds (capital, colour, day after, legs, opposite, sound) and the D kinds (add, bigger), 101 on P14 and 87 on P23. A row counts when the reply ended and holds the answer word, compared as whole words with letter case ignored. The answer word is the reference's last word that the turn does not hold; when the turn holds every reference word, it is the reference's first word. On the "bigger" rows the answer is in the turn, so the count by kind is reported with "bigger" shown separately.
- **Exact replies**, by panel; **replies that did not end** (48 tokens, no end mark).
- Read without a bar: exact facts, replies by kind and by need on P14, the weight census, rows with a non-zero entry, seconds per step, the last `train_loss_mean` and `dev_loss`, and twelve replies word for word from each model (claude-370's twelve rows).

## Bars

The order's bars, repeated with their counts spelled out before any run.

- **The number.** For seed s, gain(s) = fact and number rows counted for `rare-s` minus those for `pair-s`, out of 188.
- **The guard.** For seed s, the drop in exact replies over the two panels, `pair-s` minus `rare-s`, out of 2,218. 1.5 points is 33 replies.
- **Adds:** gain(2) ≥ 10 and gain(3) ≥ 10, and the drop is 33 or fewer at each seed.
- **Does not add:** (gain(2) + gain(3)) / 2 < 4.
- **Not called:** every other result. That covers a mean gain of 4 or more without both seeds at 10 or more, and gains that pass with a drop above 33 at a seed.

## What each outcome unlocks

- **Adds:** the next unit serves the weighted table, starting from claude-370's open item. The table needs a smaller integer form (48 M values) and a read in `uor-chat`; the weighting gives a natural cut, since rows with large R weigh least. A third seed runs beside it.
- **Does not add:** this weighting is closed (a fixed 1/R on claude-370's placement), not the mechanism. The next reading is whether the fact rows' pairs (for example capital with a country) have small R in the census. If they do not, the address, not the weight, is the gap, and the next unit adds triples.
- **Not called:** seed 1 of both arms, as a separate unit with its own order and sheet. No seed is added to this unit after a result is read.

**Prediction, written before the runs** (a calibration record, not a bar): gain between 0 and 20 per seed; the drop between −20 and +40 (frame pairs carried some of the copying gain); `rare` within 5 % of `pair`'s speed.

## Instrument checks

- **I-1** every input at this sheet's hash in the container.
- **I-2** the trainer's marker line on every run, as in claude-370: `the longest rule's marker 714,28 stands before every reply opening and nowhere else: TRAIN 215017, DEV 1226`.
- **I-3 the control is claude-370's.** On `wick`:
  - (a) claude-370's build runs the 64-step smoke of the recipe at seed 2, with an eval every 8 steps, twice; the unit's build runs it once. Whatever is identical between claude-370's build's two runs (the step-0 line, every eval's train and DEV loss to the last printed digit, the SHA-256 of every file of the step-64 checkpoint) must be identical in the unit's run. If claude-370's two runs differ in their losses, (a) fails. The unit's checkpoint holds no weights tensor.
  - (b) the unit's build scores claude-370's `pair-s2` checkpoint on P14. Its replies equal claude-370's own scoring pass (`score-pair-s2`) token for token.
  - If (a) or (b) fails, `pair-s2` and `pair-s3` are trained again by the unit's build, and claude-370's models are not compared.
- **I-4** a 64-step smoke with `--pair-weight replies` from the unit's build:
  - its step-0 line is the pair smoke's (the rows are zero, so the weights change nothing at step 0);
  - its weight census line prints R at the five quantiles with min ≥ 1;
  - its dictionary SHA-256 is claude-370's (`0b064028…b82e`, the full value in claude-370's training log), so the weights are the only difference;
  - its checkpoint holds the rows, the dictionary and the weights;
  - a scoring pass on each panel runs to its end. If fewer than 90 in 100 scored replies read a bag on either panel, the `rare` runs do not start and this sheet is amended first.
- **I-5** the unit's GPU tests pass on the GPU used, at the head. They include: the weighted bag is the weighted sum; without the flag the bag is claude-370's bit for bit; a reply decoded with cached steps equals the whole-window reply.
- **I-6** before each job starts, the launcher reads the GPU from the host. On `wick` the only compute processes may be the owner's transcode and this unit's own trainer, and the memory in use, plus the smoke's peak, plus 4 GiB must fit the card. Otherwise the job does not start there.
- **I-7** every job's folder is new and is sealed with its files' hashes and its exit code. Every scoring pass of a panel scores the same rows as the pre stage's pass of that panel; otherwise it is an instrument failure.
- **I-8 every scoring pass reads the trained rows of its checkpoint.** For every scored checkpoint, of both arms:
  - the training log's last rows line shows rows with a non-zero entry, above zero;
  - the scoring log's dictionary SHA-256 equals the training log's;
  - on each panel, 90 or more in 100 scored replies read a bag;
  - for `rare` only: the scoring log prints the weights as loaded, with a census equal to training's; for `pair`: the scoring log shows that no weights were loaded.
  - For claude-370's reused `pair` checkpoints, also: the unit's per-row files (`pair-s{2,3}-p{14,23}.rows.tsv`, written by the unit's reading script by `read-370.sh`'s rule) have the SHA-256 sealed in claude-370's `rows.sha256` for both seeds on both panels. A loader that skipped the rows of an unweighted checkpoint would fail here before any bar is read.
  - A failed I-8 is an instrument failure: it is reported as that, never as "Does not add" or "Adds" (L-007).

## Risks the build leaves open

- **Two seeds,** with same-seed pairs. The middle is not called.
- **Adam rescales a row's step.** A small weight scales the row's gradient, and AdamW divides that scale out, so a down-weighed row still moves about one rate a step. Its contribution, weight × row, grows weight times slower. This is the design, not a fault: frame pairs fade in effect without being removed.
- **R is exact per reply string.** Rephrasings of one answer count as different replies, so R overstates how many answers a pair predicts. The census is reported so this can be read.
- **The rows are 48 M values,** as in claude-370. Nothing here is served.
- **The GPU is shared on `wick`** with the owner's transcode, and at times with another lab's job (I-6). Seconds per step are reported, never barred.

## Minimum test power

On the 188 rows, claude-370's two `pair` seeds differ by 3 (86 and 89) and its `b` seeds by 17 (71 and 54). The line of 10 at each seed clears the `pair` seed spread (the arm this unit's `rare` changes) but not the `b` spread. On the same count, `pair` beat `b` by 15 and 35. With binomial noise on 188 rows (about 7 rows standard deviation per model), 10 at each seed is about one standard deviation of a same-seed difference: under no effect, both seeds clearing 10 is a few percent, and a true gain of 10 still often lands under "Not called" or "Does not add". That is accepted because a negative here closes a configuration, never the mechanism (§4.1), and the middle is not called. Two seeds do not measure the spread of one seed trained twice; a called "Adds" is followed by a third seed before anything is served.

## Steps

1. **Pre.** I-1; the unit's build on the GPU used; I-5; I-3 (on `wick`); I-4; seconds per step and peak card memory.
2. **`rare-s2` and `rare-s3`** (and `pair-s2`, `pair-s3` when I-3 fails or the runs are on a pod). Two jobs may run at once when I-6 holds at the second job's start.
3. **Scoring** by the unit's build: every model on P14 and P23.
4. **Fetch** the scoring files and logs to the laptop; the readings; the REPORT.

## Cost

On `wick`: no rented compute. claude-370's runs took 5,842–5,846 s each, two sharing the card. Two runs and eight scoring passes come to 3 to 5 GPU-hours. On pods (see Compute): two RTX 5090 Vast pods, about $0.7/h each. Lane a trains `pair-s2` then `rare-s2`, lane b trains `pair-s3` then `rare-s3`, one run at a time per card (a run peaks near 17 GiB, so two do not fit 32 GB with the 4 GiB margin). That is about 0.21 s a step and 1.2 hours a run, so about 3 hours a pod, $4 to $6 in all, inside the caps (4 pods, $8/h).

## Compute and stop rule

- **Where.** First choice: the owner's own server `wick`, as in claude-370, on the owner's word of 13 Nov 2026: "if you ssh to my other server (wick) there is a 4090 installed in it that I own locally and can use instead of renting pods". On 14 Nov another lab's trainer held the card (6.8 GiB, 100 % use). This lab does not start beside another lab's job (I-6).
  - If I-6 does not hold on `wick` when step 1 is ready, the runs go to two Vast pods of one GPU kind (RTX 5090) through `scripts/pod/uor-pod`, leased and inside the caps, with a completion watcher armed at launch (L-004). Each pod builds the unit's head itself, and the two builds' SHA-256 must match.
  - On a pod, all four arms are trained by the unit's build and I-3 is not used, since claude-370's models came from another GPU kind.
  - Once the pod is chosen, the runs do not move back to `wick`. Every model of the comparison comes from one GPU kind. On a pod the build is for its own card (`UOR_NVCC_ARCH=compute_120` for an RTX 5090, as claude-358's pods were), not `compute_89`.
  - Once `wick` is chosen at step 1, a later job whose I-6 does not hold waits there for the card. It is never run elsewhere, so no model of the comparison comes from another card.
- **How.** Each job is a detached script with a log and an exit file; a watcher on the laptop waits for the exit file.
- **Stop.** A run whose loss is not finite stops itself. For arm `pair` that is an instrument failure (claude-370 ran it to its end) and it starts again. For arm `rare` it is classified first (§4.2): with every weight at most 1 the rows can only add less than in `pair`, so a non-finite loss is first read as a defect in the build and repaired, with the run started again. Only if no defect is found is it a result about this configuration, and then it falls under "Not called".
  - A healthy run is never cancelled for time.
  - A training log that has not grown for 60 minutes is hung (an eval prints every 2,048 steps). The job stops its own trainer by its pid; that is an instrument failure, and the run starts again as a crashed run does.
  - A scoring pass is capped at 40 minutes. One that reaches the cap is run again with a longer cap, never read part-way.
  - A crashed run is an instrument failure. It starts again from step 0 in a new folder (L-002) and is never scored part-way.
- **A pod is taken down** only after its results are fetched and hashed on the laptop.

## FIT

(to be filled by fresh non-author reads before any run)
