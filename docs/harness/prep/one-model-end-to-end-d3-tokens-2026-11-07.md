# Prep sheet: (b64L) on TRAIN D3 at 32,768 steps, the tokens lever (one-model-end-to-end, claude-307, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-307. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit works to make the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: not yet reviewed.

**Why.**
- **claude-301 (#3800) met the bar** with (b64L) on TRAIN D3 at 16,384 steps: "met; unseen ahead". Its tokens lever at batch 64, 8,192 → 16,384 steps, read `ahead` (d +94,130 / +59,512, band 34,618).
- **claude-306 (#3814) trained (b64L) on D4,** which holds 2.65× D3's text, at the same 16,384 steps. Its outcome map read row 6: not met on D4 at these seeds, so the trunk keeps D3, and the next unit is the tokens lever on D3.
- **Casey approved this run in chat on 7 Nov:** "Run it, ~$9 (Recommended)" (#2028 6709527956).
- **The case against it, named in advance** (#3407 6704894084): D3 seed 1's DEV nearest hits were flat at about 3,200 from step 11,264 to step 16,384, while the learning rate decayed to 0. So a 16,384-step run's end may be set by its schedule, not by its passes. This sheet's mid-run reading separates the two (Readings).

## Question

At (b64L)'s recipe and pin on D3, does doubling the steps move the trunk? 16,384 steps see D3 about 7.2 times; 32,768 steps see it about 14.4 times. Two readings:
- **The bar:** does (b64L) on D3 at 32,768 steps stay ahead of D3's T2 and unigram at both seeds?
- **The tokens lever:** is (b64L) at 32,768 steps ahead of claude-301's (b64L) at 16,384 steps?

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's and claude-306's. The trainer is byte-for-byte claude-301's, so the lever compares steps alone.
  - Every pod is brought up with `--ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. A pod that boots at any other source is refused by the job's pin check.
- **The recipe:** claude-301's (b64L), unchanged but for its steps:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 32768`, seeds 1 and 2, plus `--train-split` D3.
  - The learning rate decays linearly from 0.001 to 0 over the run, so at step 16,384 it is 0.0005, where claude-301's run had reached 0.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.

## Data split

claude-301's, unchanged:
- **TRAIN D3:** claude-301's inputs, at `results/claude/scale-probe-2026-11-05/inputs/`. Its `SHA256SUMS` is `56a7db05a8f35b01e7231e90f95c2e7a095415263e2c16adcd93a20968c60b9c`.
  - `d3/tokens.u16` `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`
  - `d3/response_mask.u8` `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`
- **The panel:** claude-295's four held-out folders, unchanged: 1,460,697 no-memory rows, identity `5f88a5b3…`, at the per-folder pins claude-301's and claude-306's jobs check.
- **D3's tables:** T2 gets 256,660 panel rows, and the unigram 37,421.
- **D3's seen set** (`local/claude/claude-295/seen/d3`): 227,312 rows seen, 1,228,450 unseen and 4,935 short. `seen.json` is `53000ba126f91da4beacfaee3f27a93d202ab3c30a70c7372a6432a6e6e66dbe`, and its `excluded_sha256` is `94f6335f32c5f9a88d06ed5d100731a3d61200dfe140a480f899d95f78555ab4`.
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-301's job checks.

## Arms

| arm | TRAIN | batch (micro) | steps | positions | source |
|---|---|---|---:|---:|---|
| (b64L-16k), sealed | D3 | 64 (4 × 16) | 16,384 | 201.3M | claude-301, `runs-2/s{1,2}`, grades at `local/claude/claude-301/grade/b64L-s{1,2}` |
| (b64L-32k) | D3 | 64 (4 × 16) | 32,768 | 402.7M | this unit, seeds 1 and 2 |

There is one arm to train, at two seeds, one pod per seed. The steps are fixed at 32,768: no gate cuts them.

## Readings

At seeds 1 and 2:
- **The panel:** (b64L-32k)'s whole-table nearest no-memory top-1 at step 32,768, from `scale-probe grade` with `train=` D3.
- **Against D3's T2 and unigram:** `pair table=t2|unigram` with `clean=` D3's seen set, on all rows and on unseen rows, by sub-panel too.
- **The tokens lever:** a = (b64L-32k), b = claude-301's sealed grades of (b64L-16k). The pair uses no table, and `clean=` D3's seen set.
  - **The whole panel's all-rows verdict decides.** Both arms train on the same TRAIN, so both can hold the same seen rows; unlike claude-306's data lever, no arm has rows the other could not have learned.
  - The unseen-row verdict is recorded beside it. An all-rows `ahead` with the unseen lever not `ahead` reads "the gain is on contexts D3 holds".
  - The sub-panels are recorded, with oasst2 read on its own.
- **The mid-run reading, descriptive only.** The run's own checkpoint at step 16,384 (`models/step-016384`) is scored and graded too. It has seen claude-301's positions, at half the learning rate claude-301 ended on. Two pairs, no table, `clean=` D3's seen set:
  - mid against (b64L-16k): what the annealed end of a 16,384-step schedule bought;
  - (b64L-32k) against mid: what the second 16,384 steps bought.

  Neither decides the outcome. They say which part of a lever reading came from the schedule and which from the extra passes.
- **The DEV curve:** `progress.jsonl`'s `dev_nearest.hits` at every 1,024 steps, against claude-301's.
- **T2's share:** `model_hits − b` over T2's hits, from the T2 pair.
- **The loss** at the last step, read within the arm only.
- **Peak GPU memory,** from `mem.log` inside the cell's `.window`.

## Bar

Verdict words are `scale-probe pair`'s, at both seeds. Mixed reads "not distinguishable at these seeds", never as a negative (L-007). The table is claude-298's, with D3's tables:

| (b64L-32k) against D3's T2 (whole panel, all rows) | against D3's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met at 32,768 steps.** The unseen-row verdict is written beside it as `pair` returns it. |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** |
| behind | any | **Not met,** for this configuration only (§4.1). |

**The tokens lever is not a bar.** It reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

"The lever" below is the tokens lever's whole-panel all-rows verdict. **The rows are read top to bottom, and the first row that matches decides.** Each (bar, lever) pair therefore has exactly one next unit.

| # | bar at 32,768 | lever | the next unit |
|---|---|---|---|
| 1 | not met (ahead of T2 only, not distinguishable, or behind T2) | any | **Not met at 32,768 steps,** configuration only. The grade is checked first. The trunk keeps 16,384 steps, where claude-301 met the bar. Next: a decision review in the REPORT. |
| 2 | met | `ahead`, and the unseen lever `ahead` | **More passes move the trunk, on contexts TRAIN does not hold.** The trunk trains 32,768 steps. Next: the tokens lever again, 65,536 steps on D3, under its own sheet. Its cost (about $14 at today's prices) is put to Casey before any pod, since it is past this approval. |
| 3 | met | `ahead`, the unseen lever not `ahead` | **More passes move the trunk on contexts D3 holds only.** The trunk trains 32,768 steps. Next: D4 at 32,768 steps (about 5.4 passes of D4), under its own sheet, since more distinct text is where unseen contexts come from. |
| 4 | met | `inside` | **Twice the steps buy nothing measurable** within the lever's band. The trunk keeps 16,384 steps, at half the cost. Next: (b64L) at width 512, 16,384 steps on D3, under its own sheet with I-m. |
| 5 | met | `mixed` | **Not distinguishable at two seeds.** The trunk keeps 16,384 steps. Next: a decision review in the REPORT, using the mid-run reading and the DEV curves. |
| 6 | met | `behind` | First the grade and I-r are checked. If they hold: **longer training hurts on D3 at this recipe.** The trunk keeps 16,384 steps. Next: D4 at 32,768 steps under its own sheet if the unseen lever is `behind` too (14.4 passes over-fit D3's text); otherwise a decision review in the REPORT. |

**A seed that is NOT_RUN** reruns once under this sheet, at the seal root `runs-2/`, before any reading. If the rerun is NOT_RUN for the same class of cause (out of memory, divergence, a gate), that is recorded as the result, and the table is read as row 1. No reading is made on one seed.

## Instrument checks

These are registered here, before any pod goes up.
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
  - At least one test must run, and none may fail.
  - The three micro-batch tests of claude-300 must pass by name.
- **I-3:**
  - **D3:** the inputs' `SHA256SUMS` must have this sheet's sha and check OK, and the two `d3/` files must match the shas above.
  - **The panel:** the four held-out folders and the DEV files must match claude-301's pins.
  - **Each run's `config.json`:** must name D3's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 32,768 steps (128 for the smoke), the pod's seed, and the recipe's flags, with no turn memory.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2). The source it builds must be a clean checkout at exactly the pin, or it exits before building (exit 3). Every `uor-pod up` passes `--ref` with the pin. The laptop gate test mutates both exits.
- **I-1, the smoke:** a 128-step run at `--eval-every 32` on D3, at the pod's seed, exits 0 with a finite loss.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. Otherwise the arm is NOT_RUN at that seed.
  - claude-306's (b64L) on D4 peaked at 26,832 / 26,928 of 32,607 MiB. Doubling the steps changes no tensor size.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB. Otherwise the arm is NOT_RUN at that seed.
  - What it cannot see: growth slower than about 16 MiB a step passes I-f. The backstop is the run's own out-of-memory stop, and over 32,768 steps the watcher's memory line.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64, from the smoke. The projected wall is W = used + 1.05 × r × 32,768 + 2,400 s (two scorings), written to `it.txt`.
  - No gate acts on W.
  - If W exceeds 24,000 s, the watcher's limit is extended to W + 3,600 s under the charter's extend rule, and the REPORT says so.
- **I-4:** every graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-5:** each scored checkpoint's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes, as in claude-301's job. That holds for both the step-16,384 and the step-32,768 checkpoints.
- **I-r, run on the laptop before the lever is read:**
  - claude-301's (b64L-16k) grades are re-paired against D3's T2, with `clean=` D3's seen set. The `scale-probe` build used is the one that grades this unit, and its binary sha is recorded.
  - Every `d`, `band`, `spread` and `verdict` field of the result must equal claude-301's `pair/b64L-vs-t2.json`. That covers the whole panel (+75,038 / +85,590, band 10,552), the unseen rows (+56,353 / +64,841, band 8,488) and every sub-panel.
  - Its `inputs_grade_sha256` must also equal the reference's: `a1`, `a2`, `table`, and `clean_seen_sha256` `53000ba1…`.
  - If it fails, the lever and the mid-run pairs are not read. (b64L-32k)'s bar still stands.
- **I-s, on the lever:** the lever's `inputs_grade_sha256.clean_seen_sha256` must be the sha256 of D3's `seen.json` (`53000ba1…`), and that file's `excluded_sha256` must be `94f6335f…`. Otherwise the lever is not read.
- **Not repeated: claude-301's I-e** (accumulation against the whole batch). It passed at this pin, recipe and binary on both of claude-301's hosts, and this unit does not change the trainer's arithmetic. I-0 still runs claude-300's three micro-batch tests by name.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop: the job's gate tests, with mutations, and `bash -n` on every script.
3. `uor-pod status`, recorded. The cross-lab caps hold with these two pods included: at most 4 pods and at most $8/h. Otherwise wait for room. Then two Vast pods with one RTX 5090 each, one per seed, at most $0.70/h each. Each is leased and watched, and is brought up with `--count 1 --ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. Vast machines 48282 and 48276 are avoided.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t (recorded only).
   3. (b64L-32k) for 32,768 steps.
   4. The scoring of `step-032768`, then of `step-016384`: claude-301's `--score-rows`, one panel per process, then I-5 for each.
5. Seal each pod to `results/claude/one-model-end-to-end-d3-tokens-2026-11-07/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. I-r.
   2. `grade` with `train=` D3, for both checkpoints at both seeds.
   3. `pair` against D3's T2 and unigram.
   4. The tokens lever, I-s, and the two mid-run pairs.

## Minimum test power

- **The paired band** is `max(2·√(b+c), the two-seed spread)`, `scale-probe pair`'s.
- **The bar:** claude-301's (b64L-16k) against D3's T2 read d +75,038 / +85,590, band 10,552 (all spread). The bar here has about 7× headroom if (b64L-32k) is no worse.
- **The tokens lever** pairs seed s with seed s. The same seed and TRAIN give the same data order, so the pairing is not arbitrary, as claude-306's was; the two runs differ in their learning rate from step 1 and in their last 16,384 steps.
  - claude-301's own tokens lever (8,192 → 16,384) had a band of 34,618, all spread. A like spread here means an `inside` covers gains up to about 35,000 rows, about 10% of (b64L-16k)'s 331,698 / 342,250 top-1.
  - The McNemar floor, 2·√(b+c), is about 1,100 rows over the whole panel.
  - So an `inside` means "within that band", not "no effect".

## Cost

- **claude-306's measured (b64L) runs on D4:** 0.632 / 0.636 s a step, 10,409 / 10,489 s of training, on RTX 5090s at $0.563/h. Each pod billed $1.81, so about 1,200 s a pod went to setup, the smoke, one scoring and the seal. One scoring took about 240 s.
- **Here:**
  - Training: 32,768 × 0.636 s ≈ 20,840 s, about 5.8 h.
  - Setup, the inputs, the smoke, two scorings and the seal: about 0.4 h.
  - So about 6.2 h a pod: $3.50 at $0.563/h, and $4.35 at the ceiling of $0.70/h. For both: $7.00–8.70. Casey approved about $9.
- **The cap:** 2 pods × 7 h × $0.70 = $9.80. Each watcher's budget is PRICE × 7 h, with no rounding up. The charter's extend rule holds: a healthy run is never cancelled for a self-set budget.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m or I-f fails:** the arm is NOT_RUN at that seed, by the outcome map.
- **An out-of-memory error, divergence (a non-finite loss) or a trainer exit other than 0:** the arm is NOT_RUN at that seed. Each is classified as an instrument fault or a mechanism result before it is called anything (§4.2).
- **The step-16,384 checkpoint fails to score or I-5:** only the mid-run reading is lost; the bar and the lever stand on the step-32,768 checkpoint.
- **A pod never reaches SSH within 1,800 s:** it is deleted, and one retry runs on another machine. After two failures for a seed, that seed waits for a new RTX 5090 offer inside $0.70/h, and then reruns under this sheet. No reading is made on one seed meanwhile.
- **Always:** the job seals on every exit, and the watcher takes the pod down after `seal=0`.

## Changes from claude-306's sheet

1. **TRAIN:** D3 again, with claude-301's inputs, tables and seen set. The bar is read against D3's tables.
2. **Steps:** 32,768 in place of 16,384. The learning-rate schedule stretches with them.
3. **The lever:** the tokens lever against claude-301's sealed (b64L-16k) grades, decided on all rows, since both arms share TRAIN.
4. **The mid-run reading** is new: the step-16,384 checkpoint is scored and paired, descriptive only.
5. **The price ceiling** is $0.70/h, in place of $1.15/h, so the cap stays near Casey's approval.
6. **I-t's** projection adds two scorings, and its extend line moves to 24,000 s.
7. **The seal path** is `results/claude/one-model-end-to-end-d3-tokens-2026-11-07/runs/s{1,2}/`.
