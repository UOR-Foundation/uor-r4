# Prep sheet: the cause ladder, N0 and N1 (one-model-end-to-end, claude-344, 9 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-344. The steps were set by claude-342's decision review, and this sheet owns the final wording of its table.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 9 Nov 2026, before any pod). One blocker and eight other findings, all applied below and listed under "FIT changes" at the end.

**Why.**
- claude-339's L-a, the ask heads and the pull, stops on 6 and 0 of 256 replies and loops on 243 and 255. claude-322, the same recipe and TRAIN with neither, stops on 109 and 86 and loops on 130 and 140.
- Between the two there are three changes: 24 commits on the trainer's path, the ask heads, and the pull. None has been separated. The 24 are the commits from claude-322's pin (`ff5156bcc`) to L-a's that touch `crates/uor-r4-float-train` (20), `crates/uor-r4-integer` or `Cargo.lock` (7, of which 3 are also among the 20), since the trainer links `uor-r4-integer`.
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

Seeds 1 and 2 for N0 and N1: four runs, one RTX 5090 pod each. L-a's own seals are read, not rerun, and **N1 and L-a are separate runs, not paired**, as claude-339's sheet warned of separate runs. No carry in any arm here, because L-a has none. L-a against L-aC looped alike (243 and 255 against 247 and 248), which does not clear the carry, as claude-339 said.

## Readings

At seeds 1 and 2, on each arm's served `step-016384`, from `served-reply-panel --engine integer` on m2-fresh256, the measures claude-339 used:
- **Stops:** EOS, BOS, cap and context counts, of 256. **The EOS count decides.**
- **Collapse:** among the EOS-stopped rows, the count of the most common reply text. **It decides.**
- **Loop class** (`claude-342/loop-class.jq`, unchanged): looping rows, period, median onset. **It is reported for each arm and decides no row.**
- **Recorded only:** the panel's complete rows and good pairs, the first-line counts, distinct texts, median reply length and EOS per stratum.
- **The trend,** for each arm: EOS, first-line count and loop class at `step-008192` and `step-012288`, served the same way. Recorded only.
- **N0 against claude-322's sealed dev curve at the same seed** (`step`, `dev_nearest`, `train_loss_mean`, row by row). An exact match clears the commits more strongly than the stopping floor does; a difference is reported with its size. It decides no row of 1 to 5, since the GPU run is not promised to repeat bit for bit across pins. **One case is an instrument reading:** N0's dev curve exact at a seed while N0 does not reach the floor there is row 0c, since the same training with different serving points at the served path.
- **The pod's no-memory `--score-rows` output** on the held-out panels is sealed as claude-339's job writes it. This unit does not grade it and no row depends on its numbers. **A failure of that scoring or of its I-5 and DEV checks makes the cell NOT_RUN**, as the job records it, under rows 0a and 0b.
- **The report also states** N0 minus claude-322's EOS at each seed, and, where N1 fails, which limb failed (EOS under 64, or collapse at 64 or more).

**L-a's reference counts:** EOS 6 and 0, from its sealed served reports (`L-a-s1-step-016384.json` `c4f344e1…`, `L-a-s2-step-016384.json` `69aa20d6…`).

## Bars

Fixed before any run, and not moved after a reading.

- **"Holds"** at a seed means reaches the floor: at least 64 EOS stops at that seed and no collapse (one text on 64 or more EOS-stopped rows). That is claude-339's whole stopping bar. 64 is claude-322's lower seed, 86, less about one seed spread of 23. **It is a floor, not a restoration:** an arm at 64 holds while still 22 to 45 stops under claude-322.
- **"32 above L-a"** means N1's EOS is at least L-a's EOS at that seed plus 32. So at least 38 at seed 1 and at least 32 at seed 2.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a cell NOT_RUN after its rerun, for an instrument fault | **`blocked`.** Repair and rerun that arm and seed under this sheet. |
| 0b | a cell NOT_RUN after its rerun, for a mechanism cause (a non-finite loss, on N0 or N1) | **`diagnostic stop`** on that arm. A decision review in the REPORT. |
| 0c | I-xa fails, a served check of `step-016384` fails, or N0's dev curve is exact at a seed where N0 does not hold | **`blocked`.** Repair the export or scorer and re-read from the seals. |
| 1 | N0 holds at neither seed | **The commits lost it.** Bisect the 24 commits. No ask run until N0 holds. |
| 2 | one seed holds and the other does not, on N0 or N1 | **Mixed: no direction read.** Report both seeds and take a second decision review. |
| 3 | N0 holds at both, N1 holds at neither, and N1 is 32 above L-a at both | **Routes to a decision review, and is not a separation.** The review weighs the ask heads and the pull, and first runs claude-343's compare on N1's `step-016384` (about $0.10). |
| 4 | N0 holds at both, N1 holds at neither, otherwise | **`--hamming-ask 1` at scored rows cost stopping in this recipe; this closes this configuration only.** Parity is checked on L-a's Hamming reads only, so the decision review first runs claude-343's compare on N1's `step-016384` (about $0.10). Then the pivot decision on the ask heads. |
| 5 | N0 and N1 hold at both | **`--hamming-attract 1` at scored rows cost stopping in this recipe; this closes this configuration only.** N1 and L-a are separate runs, not paired. The placement work starts from the ask heads and changes the pull. |

- Every case is covered: N0 holds at 0 seeds (row 1), at 1 (row 2), or at 2, with N1 at 1 (row 2), at 0 (rows 3, 4) or at 2 (row 5).
- A NOT_RUN cell reruns once, sealing to `runs-2/` (L-002), on a fresh pod. No row is read with a cell missing.

**What no row shows.** Even row 5 only reaches claude-322's stopping floor, and claude-322 still had 0 complete rows. None of this says the ask heads help recall on language. The readings are on one panel and two seeds.

## Instrument checks

Registered before any pod. claude-339's, with these changes:
- **I-p, I-0, I-3, I-1, I-m, I-f, I-t, I-5, the watcher:** as claude-339's, same pin, TRAIN unchanged (417,421 windows).
  - Each `config.json` must show no pull, no carry, no hold, and the ask heads off (N0) or on (N1). `config.json` leaves a false ask, a 0 pull and a 0 carry out, so absence is what is checked.
- **I-nopull, replacing I-pull:** the smoke wrote no `pull.jsonl`. The trainer writes that file only when `hamming_attract` is above 0, checked at the pin (`pull_tally = (config.hamming_attract > 0.0).then(...)`). A `pull.jsonl` here means the arm is not what this sheet says, and is an instrument NOT_RUN.
- **I-xa, on the smoke's `step-000064`, on the laptop:**
  - N0's snapshot has no `hamming_ask` key (`fbs-snapshot` writes none without ask heads). `fbs-export` accepts it, the integer engine scores it, and the default engine scores it.
  - N1's has one non-null `hamming_ask` entry per read and no carry. `fbs-export` accepts it, the integer engine scores it, and the default engine refuses it by name.
- **The served path, at every served step:** `laptop_snap`, unchanged from claude-339: the trunk and base reads from `cpu-fbs-export`, the ask entries from the pod snapshot, with the same-bytes check on every `hamming.*` section and on every trunk section.
- **The read path's positive control, in place of I-xg and I-c (`control-344.sh`, on the laptop, before any N0 cell is read).** `read-344.sh`'s real `serve`, `dev` and `verdict` run over two sealed models whose answers are known, with claude-339's tools pinned by SHA-256 (the scorer is `served-reply-panel-c6204f66`, `c6204f66c85c…`):
  - claude-322's sealed runs, as N0 cells, must give artifacts `86e51d5e…` and `8838cd10…`, its sealed reports with the timing fields and `settings.engine` removed, and EOS 109 and 86.
  - L-a's sealed runs, as N1 cells, must pass I-xa, give its sealed reports and EOS 6 and 0.
  - N0's dev curve must read `exact` against itself, `verdict` must refuse without `SERVE_OK`, and with it print row 4.
  - The only expected failures are the two N0 I-xa lines, since claude-322 sealed no smoke snapshot. **Any other failure is row 0c before any pod's result is read.** The N0 no-ask I-xa path itself runs only on the real N0 smokes.
- **I-cut is not run.** There is no carry here to cut.

**The scripts** are claude-339's, derived mechanically and changed only as follows: the arms and their flags; `arm_want`; I-nopull for I-pull; I-xg removed (from the job and the launcher); the session, store root and file names; `runs-3` removed from the allowed roots; and the seal-pull's hold off by default, so a checked copy off the pod takes it down (L-004), as a read-back seal does. The read script (`read-344.sh`) reuses claude-339's served path and adds the table.

## Steps

1. The ORDER, this sheet's FIT, then the D67 pre-run review of the scripts, which fills the launcher's pins.
2. **On the laptop:** the scripts, their gate tests and mutations, and `bash -n`.
3. `uor-pod status`. The caps must hold: at most 4 pods and $8/h across the labs, counting theirs. Then one RTX 5090 pod (at most $0.70/h) per arm and seed, each with its watcher before any script ships. If the caps do not allow four at once, N0 launches first.
4. **On each pod:** I-p, I-0, the build, the inputs, I-3, the smoke, I-m, I-f, I-nopull, I-xa's snapshot, I-t. Then 16,384 steps, the three snapshots, and the no-memory scoring of `step-016384` with I-5.
5. **Seal** to `results/claude/one-model-end-to-end-cause-ladder-2026-11-09/runs/{N0,N1}/s{1,2}/`, each root empty first (L-002). Read back, then the watcher runs the four teardown checks (below) and takes that pod down only if they pass. If the store refuses the seal or fails its read-back, the auto-pull copies the tarball to the laptop and checks it, and the pod goes down only under the same rule.
6. **On the laptop:** fetch the four seals, then `read-344.sh serve` (which writes `SERVE_OK` only when every cell passed), `dev`, and `verdict`, which refuses without either. `loop-class` is in `serve`.

## Minimum test power

- **One count's spread.** At an EOS rate of a third of 256 rows, one count's binomial spread is about 7.5 rows, and the spread of a difference of two counts about 10.7.
- **The seed spread.** claude-322's seeds gave 109 and 86, a spread of 23, which is larger than the binomial spread. That is why each bar is per seed and both seeds must meet it, and why one seed agreeing and the other not has its own row (2).
- **The 64 floor** is about 3 binomial spreads under claude-322's lower seed and about 7.7 above L-a's 6 at seed 1 (its seeds are 6 and 0).
- **The "32 above L-a" rule** is used only in row 3. At row 3's own rates (N1 about 38 and L-a 6 or less) the binomial spread of a difference is 6 to 7, so 32 is about 5 such spreads, but only about 1.4 of claude-322's seed spread of 23. **Row 3 is a routing to a decision review, not a separation.**
- **Two seeds** cannot size an effect. The table only sorts the cause. A later unit that needs a size needs its own seeds.

## Cost

- **Step time, measured:** claude-339's ask-head runs took about 8,500 s of training at 16,384 steps, about 3 h a pod with setup and scoring; claude-322's no-ask run took 8,046 s.
- **Four pods at $0.563 to $0.70/h × about 3 h: about $7 to $8.50.** The budget notice, not a cap, is 4 × 6 h × $0.70 = $16.80. I-t records the real step time on each pod. A held pod (a failed teardown check) can add up to $4.20 more, at 6 h and $0.70/h, only until the session reads it.

## Stop rule

claude-339's, with:
- **I-nopull failing:** NOT_RUN for that arm and seed (instrument).
- **A snapshot failing** at an intermediate step: the trend loses that point, and the REPORT says so. At `step-016384` it is row 0c.
- **A healthy run is never cancelled for cost.** The watcher logs notices and takes a pod down only on the four checks below.
- **The four teardown checks (the owner's rule for claude-339's pods, per pod):** the chain ended `job=0 seal=0`; all 45 required files are in the seal's list (claude-339's 41 without `pull.jsonl`, plus I-xa's smoke checkpoint and snapshot); the store's `SHA256SUMS` read from the laptop equals the pod's; and no GPU process remains. Any other ending holds that pod alone, leased, so the reaper cannot delete the only copy. The hold ends when the session or the pull releases it, or after 6 h with a notice.
- **A stall (30 unchanged checks) logs a notice and pauses renewal,** as earlier watchers did, so a hung pod does not bill forever. At least 3.5 h of the last lease remain for the session to read the notice.

## Changes from claude-339's sheet

1. **The arms:** N0 and N1 in place of L-aC and L-a. L-a is read from its seals.
2. **The table** is claude-342's, with its bars unchanged. Row 0a to 0c are claude-339's. This sheet's wording of row 1 ("holds at neither seed") states what claude-342's "does not hold, at both seeds" meant.
3. **The first-line count decides no row here.** Stopping is what the ladder sorts.
4. **I-pull becomes I-nopull;** I-xg and I-c are replaced by the read path's positive control, and I-cut is not run, as above.
5. **Cost:** about the same as claude-339's rerun.
6. **Decision card, claude's, reversible by Casey: when the store refuses a seal, a checked laptop copy counts as off the pod.** claude-339 held every pod at first (`Do not kill a pod until the owner approves`), then the owner set "restore per-pod automatic teardown after verified seal" and the four checks. The hold default is therefore off. Here the copy's `SHA256SUMS` and tar list are checked on the laptop, which stands in for the store's read-back, and the job must have ended 0 with every required file in the list, or the pod is held. `PULL_HOLD=1` restores the hold.

## FIT changes

The non-author FIT returned **FIT WITH CHANGES**, one blocker and eight others, all applied:
1. **Blocker, the read path had no positive control:** `control-344.sh`, under Instrument checks. Row 0c gained N0's exact dev curve at a seed where N0 does not hold.
2. **Row 4's "parity clean" rested on a check N1 never had:** reworded to L-a's Hamming reads only, and rows 3 and 4 run claude-343's compare on N1 before ruling.
3. **"20 commits" counted one crate:** now 24, with the path set named.
4. **"Holds" is not "restores":** "reaches the floor", and the REPORT states N0 minus claude-322 per seed.
5. **Verdict rows** are scoped to the flags at scored rows and to this configuration, and the unpaired caveat is restored.
6. **Power:** row 3 is a routing to a decision review, not a separation.
7. **The REPORT names which limb failed** when N1 fails.
8. **Row 0b** classifies a non-finite loss on N0 too.
9. **The carry sentence** cites L-a against L-aC and keeps claude-339's "does not clear the carry".

## Pre-run review

**Round 1, two fresh reviews of the scripts (9 Nov), each PASS WITH CHANGES, no pod launched.** The pod scripts' review had no blocker; its findings were the watcher's teardown rule and the unsealed path that could lose the only copy. The read path's review had two blockers: `verdict` read a row without `serve` having passed, and NOT_RUN cells got a row. Both are fixed, with tests and mutations in `local/claude/claude-344/gt/`.

**Round 2, one fresh review of the changed teardown and read code (9 Nov), PASS WITH CHANGES, no pod launched.** Five findings were fixed: the GPU check could read an empty answer as "no GPU process"; the pull ignored a required-files check that could not run; a log tail containing `seal=0` could be read as a sealed chain; the required list lacked I-xa's smoke checkpoint and snapshot (now 45 names); and a renewal was skipped during an ssh outage. A sixth was not a defect: `carry_ok` already refuses an empty ask array, and a test now pins that.

**The positive control passed** (`control-344.sh`): claude-322's two artifacts by hash, all four sealed reports, EOS 109, 86, 6 and 0, the dev curves exact against themselves, and `verdict` refusing without `SERVE_OK` and printing row 4 with it. It ran the final `read-344.sh` (`32b6c4b5133f…`). The two N0 I-xa lines failed as expected, since claude-322 sealed no smoke snapshot.

**Script tests at the launch:** gates, seal, watch, pull, hold and read pass. Mutations caught: job 34 of 34, watch 10 of 10, read 31 of 31, hold 13 of 13.

**This review's decision, for the D67 record:** the whole-project chain is claude-301's recipe, claude-322's sealed reference and claude-339's sealed L-a; TRAIN and every tool are in the store or pinned by SHA-256; a pass or a fail each unlocks a named row above; the caps allow four pods at at most $0.70/h; and the FIT is applied. Launcher pins, SHA-256 of the shipped scripts: `job-344.sh` `2d26daf7da2bbb30c5e9145bf6cd59fba5391856a462b71cd755e16f71bfd788`, `chain-344.sh` `2c74bcdc2296215272f086b52b33ac8c9efe426c2bd39b66f2abe18b4459449a`, `seal-344.sh` `eed91faaec6d8f6cd354e16b7cdf6cbbf0b55cb6e502ecc98dca3041250ccbb0`.
