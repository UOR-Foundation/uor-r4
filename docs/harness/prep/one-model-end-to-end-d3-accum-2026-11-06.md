# Prep sheet: batch 64 on D3 by gradient accumulation (one-model-end-to-end, claude-301, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-301. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: PENDING (a fresh non-author review before any pod goes up).

**Why.** This is claude-298's sheet, [`one-model-end-to-end-d3-batch-2026-11-06`](one-model-end-to-end-d3-batch-2026-11-06.md) (FIT WITH CHANGES, round 1), run again with batch 64 built by gradient accumulation.
- **claude-298 (#3796) could not answer its question.** Every cell at batch 64 ran out of the RTX 5090's 32 GB.
  - The smokes peaked at 32,108 / 31,696 MiB.
  - The full runs climbed from about 22 GB to 31–32 GB, then stopped within 84 s.
  - The verdict was blocked, an instrument fault, and the sheet's Stop rule named gradient accumulation as the next step.
- **claude-300 (#3799) built it.** `--micro-batch M` runs each step's windows forward and backward M at a time, weights each forward's loss by its share of the step's scored rows, and makes one optimizer step on the summed gradients.
  - A CUDA test shows four micro-batches of two give one batch of eight's loss and gradients, to 10⁻⁵ on the loss and 10⁻⁴ of each gradient's largest entry. Weighting each 1/k fails the test.
  - **What that test does not cover:** the real recipe's Hamming read and served path. I-e below checks them.
- **What it does not change:** claude-298's question, arms, readings, bar and outcome map all stand. This sheet repeats only what is needed to run, and lists every change from claude-298's sheet under "Changes from claude-298's sheet".

## Question

claude-298's, unchanged. At TRAIN D3 and (near)'s recipe, with only the batch changed from 16 to 64:
- **Is the seed spread gradient noise?** Read on (b64L) against claude-297's (n3L).
- **Does the trunk pass D3's T2 at both seeds?**

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f` (`main` after #3799).
  - Between claude-298's pin (`1f2bd76ce58503f25ef24e415dd193bf57086ddb`) and this one, `crates/uor-r4-float-train` changes only by #3799. The other crates, `docs/labs/scale-probe`, `Cargo.toml` and `Cargo.lock` are unchanged; `git log 1f2bd76c..ff5156bc` over those paths lists only #3799.
  - With no `--micro-batch`, #3799 leaves the step as it was. So claude-297's sealed (n3L) at batch 16 stays this unit's control (I-r).
- **The recipe:** claude-298's, plus `--micro-batch 16`:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024`, plus `--train-split` D3.
- **The binaries:** `train-in-space` is built on each pod at the pin, with its sha256 recorded.

## Data split

claude-298's, unchanged:
- **TRAIN D3:** `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- **D3's seen set:** 227,312 seen, 1,228,450 unseen.

## Arms

claude-298's, each run as batch 64 in four micro-batches of 16. Seeds 1 and 2.

| arm | batch (micro) | steps | positions |
|---|---|---|---|
| (n3L), claude-297, sealed | 16 | 32,768 | 100.7M |
| (b64S) | 64 (4 × 16) | 8,192 | 100.7M |
| (b64L) | 64 (4 × 16) | 32,768, or 16,384 by I-t | 402.7M or 201.3M |

## Readings, Bar, What each outcome unlocks

claude-298's sheet's sections of these names apply word for word, with its numbers: (n3L)'s spread 37,975, the narrowing threshold 18,988, and the lever floor of about 38,000 rows.

One addition to the outcome map. **If I-t cuts (b64L) to 16,384 steps,** the equal-steps reading of the noise question is lost. (b64L) then makes half (n3L)'s updates at four times its batch. The spread is still read and the bar still applies. "Batch narrows the spread" may then be read only as "a larger batch at half the updates narrows it", and the REPORT says so.

## Instrument checks

claude-298's I-0, I-1, I-t, I-3, I-4, I-5, I-s and I-r, unchanged except as noted, plus three new gates. They are registered here, before any pod goes up.
- **I-0** includes claude-300's two micro-batch tests, by name, as passed.
- **I-e, equivalence on the real recipe.** Two 4-step runs at TRAIN D3 and seed 1, `--batch 16 --eval-every 1`, with and without `--micro-batch 4`, on the same card, one after the other. Batch 16 is the batch that fits whole.
  - **Pass:** every progress row's `train_loss_mean` agrees within 10⁻⁴ relative, at all four steps.
  - Step 1's row tests the gradient of the very first step; steps 2 to 4 test that the optimizer's updates agree.
  - **Fail:** the recipe's forward is not window-independent, or the accumulation is wrong. The pod stops before any training, and the unit becomes a diagnosis.
- **I-m, the memory gate.** It is read from each arm's smoke: the largest `memory.used` in `gpu.csv` while that smoke runs, against `memory.total`.
  - **Pass:** the peak is below 90% of the card.
  - **Fail:** that arm is NOT_RUN at that seed, before its full run.
- **I-f, memory flat across steps.** It is read from (b64L)'s smoke, which runs 128 steps at `--eval-every 32`, in place of claude-298's 64.
  - The smoke's `gpu.csv` samples are split at the midpoint of its wall.
  - **Pass:** the second half's peak is at most the first half's peak plus 1,024 MiB.
  - **Fail:** memory grows with steps. Both arms are NOT_RUN at that seed.
  - **The arms' full runs also stop on memory.** The job stops a full run whose `memory.used` passes 97% of the card in three consecutive samples, and records it as NOT_RUN at the last progress step. This catches growth that starts after the smoke.
- **I-t** takes its rate r from steps 64 to 128 of (b64L)'s smoke, over 64 steps. The rule, the 1.05 factor, the 1,800 s and the 21,000 s bound are claude-298's.

## Steps

1. The ORDER at the pin; this sheet's FIT; the D67 pre-run review of the job and watch scripts.
2. `uor-pod status`. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.15/h each, leased, each with a watcher. There is no two-card fallback.
3. On each pod: setup, I-0, I-3, I-e (seed 1's pod only; seed 2's pod repeats it as a check), the smokes with I-m and I-f, then I-t.
4. The arms, then the scoring: claude-298's `--score-rows`, one panel per process.
5. Seal each pod to `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs/s{1,2}/`, read the seal back, then take the pod down.
6. On the laptop: I-r, then `grade` and `pair` as in claude-298's Readings.

## Minimum test power

claude-298's. The McNemar floor is about 1,100 rows; (n3L)'s band of 37,975 is all spread.

## Cost

- **The rate:** claude-298's smokes measured 0.564 / 0.540 s a step at whole batch 64, before running out of memory. Four forwards of 16 should cost about the same compute, plus four launches of each kernel instead of one: about 0.6–0.8 s a step.
- **At 0.65 s,** I-t gives W = 1.05 × 0.65 × (8,192 + 32,768) + 1,800 = 29,755 s, which is over 21,000. So (b64L) runs 16,384 steps, and W = 18,573 s.
- **At 0.8 s,** W with (b64L) at 16,384 is 22,443 s, still over. So (b64S) is NOT_RUN by the gate, and (b64L) runs 16,384 steps alone, W = 15,563 s.
- **Spend:** at most 2 × 6 h × $1.15 = $13.80, within the ORDER's $14; likelier about $10. The charter's extend rule holds.

## Stop rule

claude-298's, with these changes:
- An I-e failure stops the pod before any training.
- An I-m or I-f failure makes the arm, or both arms, NOT_RUN at that seed, as I-m and I-f say.
- An out-of-memory error at batch 64 by accumulation is again an instrument fault. It leaves the question open, and the next unit reads where the memory goes before trying again.

## Changes from claude-298's sheet

1. The pin is `main` after #3799, and the recipe adds `--micro-batch 16`.
2. New gates I-e, I-m and I-f, and a stop on memory during the full runs.
3. (b64L)'s smoke runs 128 steps, and I-t's rate is read from its steps 64 to 128.
4. The outcome map's note on reading the spread when I-t cuts (b64L) to 16,384 steps.
5. The seal path is `results/claude/one-model-end-to-end-d3-accum-2026-11-06/`.
