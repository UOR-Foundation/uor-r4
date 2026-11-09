# Prep sheet: the cause ladder, N0 and N1 (one-model-end-to-end, claude-344, 9 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-344. The steps were set by claude-342's decision review, and this sheet owns the final wording of its table.

TEST FITNESS: not yet reviewed. A non-author FIT goes here before any pod.

**Why.**
- claude-339's L-a, the ask heads and the pull, stops on 6 and 0 of 256 replies and loops on 243 and 255. claude-322, the same recipe and TRAIN with neither, stops on 109 and 86 and loops on 130 and 140.
- Between the two there are three changes: 20 trainer commits, the ask heads, and the pull. None has been separated.
- **The serving of the Hamming reads is not the cause, as far as the check goes.** claude-343 compared float and integer reads on L-a's sealed `step-016384`, on the panel's own windows, at both seeds: 0 query flips, 0 key flips, 0 ties, 0 not served and 0 faults, about 240,000 reads per seed. Its raw traces differ only at position 0, which no kept set holds. Trunk values and the nearest head's token were not compared.
- So this sheet trains the two missing arms and reads which change lost the stopping.

## Question

At claude-322's recipe, TRAIN, seeds and steps, and at L-a's pin, which of the trainer commits, the ask heads and the pull cost the served replies their stopping?

## Base artifact

- **Recipe:** claude-301's (b64L), unchanged, as claude-322 and claude-339 ran it. `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16`, for 16,384 steps at `--eval-every 1024`.
- **The pin:** `721a4784f265efea6b490d5c3100779e4c4b1607`, L-a's. The trainer is built at the pin on each pod. The laptop tools are the ones claude-339 pinned by SHA-256.

## Data split

- **TRAIN:** claude-322's mix, unchanged, from the store at `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`: tokens `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`, mask `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`, manifest `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`, 417,421 usable windows.
- **DEV, the held-out panels and m2-fresh256:** claude-322's, at its pins. Nothing is regenerated.

## Arms

| Arm | flags beyond the recipe | what it isolates |
|---|---|---|
| **N0** | none | the trainer commits, against claude-322 (sealed, same recipe, TRAIN and seeds) |
| **N1** | `--hamming-ask 1` | the ask heads, against N0 |
| L-a (sealed) | `--hamming-ask 1 --hamming-attract 1` | the pull, against N1 |
| claude-322 (sealed) | none, at its own pin | the stopping that was lost |

Seeds 1 and 2 for N0 and N1: four runs, one RTX 5090 pod each. L-a's own seals are read, not rerun. No carry in any arm here: L-a has none, and the carry made no difference to the loop (L-aC and its `−C` cut read alike).

## Readings

At seeds 1 and 2, on each arm's served `step-016384`, from `served-reply-panel --engine integer` on m2-fresh256, the measures claude-339 used:
- **Stops:** EOS, BOS, cap and context counts, of 256. **The EOS count decides.**
- **Collapse:** among the EOS-stopped rows, the count of the most common reply text. **It decides.**
- **Loop class** (`claude-342/loop-class.jq`, unchanged): looping rows, period, median onset. **It is reported for each arm and decides no row.**
- **Recorded only:** the panel's complete rows and good pairs, the first-line counts, distinct texts, median reply length and EOS per stratum.
- **The trend,** for each arm: EOS, first-line count and loop class at `step-008192` and `step-012288`, served the same way. Recorded only.
- **N0 against claude-322's sealed dev curve at the same seed** (`step`, `dev_nearest`, `train_loss_mean`, row by row). An exact match clears the commits more strongly than the stopping floor does; a difference is reported with its size. It decides no row, since the GPU run is not promised to repeat bit for bit across pins.
- **The pod's no-memory `--score-rows` output** on the held-out panels is sealed as claude-339's job writes it. This unit does not grade it; no row depends on it.

**L-a's reference counts:** EOS 6 and 0, from its sealed served reports (`L-a-s1-step-016384.json` `c4f344e1…`, `L-a-s2-step-016384.json` `69aa20d6…`).

## Bars

Fixed before any run, and not moved after a reading.

- **"Holds"** at a seed means at least 64 EOS stops at that seed and no collapse (one text on 64 or more EOS-stopped rows). That is claude-339's whole stopping bar. 64 is claude-322's lower seed, 86, less about one seed spread of 23.
- **"32 above L-a"** means N1's EOS is at least L-a's EOS at that seed plus 32. So at least 38 at seed 1 and at least 32 at seed 2.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a cell NOT_RUN after its rerun, for an instrument fault | **`blocked`.** Repair and rerun that arm and seed under this sheet. |
| 0b | a cell NOT_RUN after its rerun, for a mechanism cause (a non-finite loss on N1) | **`diagnostic stop`** on that arm. A decision review in the REPORT. |
| 0c | I-xa fails, or a served check of `step-016384` fails | **`blocked`.** Repair the export or scorer and re-read from the seals. |
| 1 | N0 holds at neither seed | **The commits lost it.** Bisect the 20 commits. No ask run until N0 holds. |
| 2 | one seed holds and the other does not, on N0 or N1 | **Mixed: no direction read.** Report both seeds and take a second decision review. |
| 3 | N0 holds at both, N1 holds at neither, and N1 is 32 above L-a at both | **Partial: both the ask heads and the pull cost stopping.** A decision review weighs the two. |
| 4 | N0 holds at both, N1 holds at neither, otherwise | **The ask heads did, with parity clean.** The pivot decision on the ask heads. |
| 5 | N0 and N1 hold at both | **The pull did.** The placement work starts from the ask heads and changes the pull. |

- Every case is covered: N0 holds at 0 seeds (row 1), at 1 (row 2), or at 2, with N1 at 1 (row 2), at 0 (rows 3, 4) or at 2 (row 5).
- A NOT_RUN cell reruns once, sealing to `runs-2/` (L-002), on a fresh pod. No row is read with a cell missing.

**What no row shows.** Even row 5 only restores claude-322's stopping, and claude-322 still had 0 complete rows. None of this says the ask heads help recall on language. The readings are on one panel and two seeds.

## Instrument checks

Registered before any pod. claude-339's, with these changes:
- **I-p, I-0, I-3, I-1, I-m, I-f, I-t, I-5, the watcher:** as claude-339's, same pin, TRAIN unchanged (417,421 windows).
  - Each `config.json` must show no pull, no carry, no hold, and the ask heads off (N0) or on (N1). `config.json` leaves a false ask, a 0 pull and a 0 carry out, so absence is what is checked.
- **I-nopull, replacing I-pull:** the smoke wrote no `pull.jsonl`. The trainer writes that file only when `hamming_attract` is above 0, checked at the pin (`pull_tally = (config.hamming_attract > 0.0).then(...)`). A `pull.jsonl` here means the arm is not what this sheet says, and is an instrument NOT_RUN.
- **I-xa, on the smoke's `step-000064`, on the laptop:**
  - N0's snapshot has no `hamming_ask` key (`fbs-snapshot` writes none without ask heads). `fbs-export` accepts it, the integer engine scores it, and the default engine scores it.
  - N1's has one non-null `hamming_ask` entry per read and no carry. `fbs-export` accepts it, the integer engine scores it, and the default engine refuses it by name.
- **The served path, at every served step:** `laptop_snap`, unchanged from claude-339: the trunk and base reads from `cpu-fbs-export`, the ask entries from the pod snapshot, with the same-bytes check on every `hamming.*` section and on every trunk section.
- **I-xg and I-c are not repeated.** Both passed on claude-339 with these same tools, pinned by SHA-256: I-xg gave claude-322's artifact `86e51d5e…`, and I-c reproduced claude-322's stripped reports. This run changes no tool and no export path.
- **I-cut is not run.** There is no carry here to cut.

**The scripts** are claude-339's, derived mechanically and changed only as follows: the arms and their flags; `arm_want`; I-nopull for I-pull; I-xg removed (from the job and the launcher); the session, store root and file names; `runs-3` removed from the allowed roots; and the seal-pull's hold off by default, so a checked copy off the pod takes it down (L-004), as a read-back seal does. The read script (`read-344.sh`) reuses claude-339's served path and adds the table.

## Steps

1. The ORDER, this sheet's FIT, then the D67 pre-run review of the scripts, which fills the launcher's pins.
2. **On the laptop:** the scripts, their gate tests and mutations, and `bash -n`.
3. `uor-pod status`. The caps must hold: at most 4 pods and $8/h across the labs, counting theirs. Then one RTX 5090 pod (at most $0.70/h) per arm and seed, each with its watcher before any script ships. If the caps do not allow four at once, N0 launches first.
4. **On each pod:** I-p, I-0, the build, the inputs, I-3, the smoke, I-m, I-f, I-nopull, I-xa's snapshot, I-t. Then 16,384 steps, the three snapshots, and the no-memory scoring of `step-016384` with I-5.
5. **Seal** to `results/claude/one-model-end-to-end-cause-ladder-2026-11-09/runs/{N0,N1}/s{1,2}/`, each root empty first (L-002). Read back, then the watcher takes the pod down. If the store refuses the seal, the auto-pull copies it to the laptop, checks it, and takes the pod down.
6. **On the laptop:** fetch the four seals, serve and score every snapshot, read the table (`read-344.sh verdict`), `loop-class`, and N0's dev curve against claude-322's.

## Minimum test power

- **One count's spread.** At an EOS rate of a third of 256 rows, one count's binomial spread is about 7.5 rows, and the spread of a difference of two counts about 10.7.
- **The seed spread.** claude-322's seeds gave 109 and 86, a spread of 23, which is larger than the binomial spread. That is why each bar is per seed and both seeds must meet it, and why one seed agreeing and the other not has its own row (2).
- **The 64 floor** is about 3 binomial spreads under claude-322's lower seed and about 7.7 above L-a's 6 at seed 1 (its seeds are 6 and 0).
- **The "32 above L-a" rule** is 3 spreads of a difference at that rate. It separates "N1 stops far more than L-a" from noise, and it is used only in row 3.
- **Two seeds** cannot size an effect. The table only sorts the cause. A later unit that needs a size needs its own seeds.

## Cost

- **Step time, measured:** claude-339's ask-head runs took about 8,500 s of training at 16,384 steps, about 3 h a pod with setup and scoring; claude-322's no-ask run took 8,046 s.
- **Four pods at $0.563 to $0.70/h × about 3 h: about $7 to $8.50.** The budget notice, not a cap, is 4 × 6 h × $0.70 = $16.80. I-t records the real step time on each pod.

## Stop rule

claude-339's, with:
- **I-nopull failing:** NOT_RUN for that arm and seed (instrument).
- **A snapshot failing** at an intermediate step: the trend loses that point, and the REPORT says so. At `step-016384` it is row 0c.
- **A healthy run is never cancelled for cost.** The watcher logs notices and takes the pod down only on a sealed, read-back chain.

## Changes from claude-339's sheet

1. **The arms:** N0 and N1 in place of L-aC and L-a. L-a is read from its seals.
2. **The table** is claude-342's, with its bars unchanged. Row 0a to 0c are claude-339's. This sheet's wording of row 1 ("holds at neither seed") states what claude-342's "does not hold, at both seeds" meant.
3. **The first-line count decides no row here.** Stopping is what the ladder sorts.
4. **I-pull becomes I-nopull;** I-xg, I-c and I-cut are not repeated, as above.
5. **Cost:** about the same as claude-339's rerun.

## FIT changes

None yet.

## Pre-run review

Not yet run.
