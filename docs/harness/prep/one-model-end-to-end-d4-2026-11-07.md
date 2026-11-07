# Prep sheet: (b64L) on TRAIN D4, more human text at width 256 (one-model-end-to-end, claude-306, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-306. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit works to make the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: PENDING.

**Why.**
- **claude-301 (#3800) met the bar** with (b64L) on TRAIN D3, "met; unseen ahead". Its tokens lever at batch 64 read `ahead`.
- **claude-304 (#3810) stopped its width-1,024 run** by a decision review. The review set the outcome map's default as the next unit: more human text at width 256.
- **claude-305 (#3813) built TRAIN D4** on the laptop for $0: 74,039,248 tokens, 2.65× D3.
  - Each of D3's roots is a byte prefix of D4's.
  - The panel is unchanged.
  - D4's T2 gets 256,443 panel rows (D3's got 256,660), and its unigram gets 37,421.
- **This unit trains (b64L) unchanged on D4**, at claude-301's pin, seeds and steps. The training text is the only thing that changes.

## Question

At (b64L)'s recipe, pin and 201.3M positions, does more distinct human text move the trunk? Each run sees D3 7.2 times, but D4 only 2.72 times. Two readings:
- **The bar:** does (b64L) on D4 stay ahead of D4's T2 and unigram at both seeds?
- **The data lever:** is (b64L) on D4 ahead of claude-301's (b64L) on D3?

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's. This keeps the trainer byte-for-byte the same, so the data lever compares text alone.
  - `main` has moved on. claude-303 (#3809) and DeepSeek's #3812 changed `crates/uor-r4-float-train`. They are not in this run.
  - Every pod is brought up with `--ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. A pod that boots at any other source is refused by the job's pin check: claude-304's third pod booted at `main`.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2, plus `--train-split` D4.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.

## Data split

- **TRAIN D4:** sealed by claude-305 at `results/claude/one-model-end-to-end-d4-2026-11-07/`. Its `SHA256SUMS` is `51ae9b9dcb685646fa29b01030680f3907b51e91dff906c11332284071634a22` and covers 39 files.
  - `mix/tokens.u16` `3a5b89f9fd99326eb18f57027eb38f2d8ef2d200e907d0c9a7e20553c78b0169`
  - `mix/response_mask.u8` `9fe9dedeccd599c0b9abd59c1864a414f999bb43f53529ba8e5013d35da10807`
  - `mix/manifest.json` `d98a4461b871c7c8e0be87fead7a9e9dbf45f4c5078cdaf63522012a68c47ba9`
- **D4's roots:** oasst2 3,773,419 tokens (as stored), wiki-00000 22,752,102, wiki-00020 23,962,093 and wiki-00040 23,551,634.
  - OASST2's share of TRAIN falls from 13.5% in D3 to 5.1%.
- **The panel:** claude-295's four held-out folders, unchanged: 1,460,697 no-memory rows, identity `5f88a5b3…`. Pins per folder (tokens, mask):
  - oasst2 `02a04e97…f169e` / `9c5bb656…4833`
  - wiki-00000 `82b3a40c…09f7` / `34fe6b68…386f`
  - wiki-00020 `9b7f1432…d640` / `a6803ab3…e00d`
  - wiki-00040 `c927243b…962d` / `9d95ea6d…a625`

  These are the same pins claude-301's job checks, given there in full.
- **D4's tables** (claude-305's seal, `tables/`):
  - T2 gets 256,443 rows, and the unigram 37,421.
  - Per sub-panel, T2 / unigram: oasst2 29,531 / 4,601; wiki-00000 71,051 / 6,851; wiki-00020 78,401 / 14,538; wiki-00040 77,460 / 11,431.
- **D4's seen set** (`seen/`, `excluded.bin` `511b0289…`): 282,585 rows seen, 1,173,177 unseen and 4,935 short. D3's seen set holds 227,312 seen.
  - D3's text is inside D4, so every row unseen in D4 is also unseen in D3.
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-301's job checks.

## Arms

| arm | TRAIN | batch (micro) | steps | positions | source |
|---|---|---|---:|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | 201.3M | claude-301, `runs-2/s{1,2}` |
| (b64L-D4) | D4 | 64 (4 × 16) | 16,384 | 201.3M | this unit, seeds 1 and 2 |

There is one arm to train, at two seeds, one pod per seed. The steps are fixed at 16,384: the data lever reads at equal positions, so no gate here cuts them.

## Readings

At seeds 1 and 2:
- **The panel:** (b64L-D4)'s whole-table nearest no-memory top-1, from `scale-probe grade` with `train=` D4.
- **Against D4's T2 and unigram:** `pair table=t2|unigram` with `clean=` D4's seen set, on all rows and on unseen rows, by sub-panel too.
- **The data lever:**
  - a = (b64L-D4), graded on D4.
  - b = claude-301's sealed grades of (b64L-D3), graded on D3, at `local/claude/claude-301/grade/b64L-s{1,2}`.
  - The pair uses no table, and `clean=` D4's seen set.
  - `pair` lets two cells differ in TRAIN. It refuses if a cell's own two seeds differ in TRAIN, and it refuses a `clean=` whose TRAIN is not a1's.
- **The oasst2 sub-panel, read on its own,** for both the bar and the lever. Its share of TRAIN falls 2.6×, so this caveat is named before any reading: if the lever is behind on oasst2 and ahead on the wiki sub-panels, that reads as a change in the mix share, not as "more text hurts".
- **T2's share:** `model_hits − b` over T2's hits, from the T2 pair.
- **The loss** at the last step, read within the arm only.
- **Peak GPU memory,** from `mem.log` inside the cell's `.window`, which is the file I-m gates on.

## Bar

Verdict words are `scale-probe pair`'s, at both seeds. Mixed reads "not distinguishable at these seeds", never as a negative (L-007). The table is claude-298's, with D4's tables:

| (b64L-D4) against D4's T2 (whole panel, all rows) | against D4's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met on D4.** The unseen-row verdict is written beside it: "met; unseen ahead" or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** |
| behind | any | **Not met,** for this configuration only (§4.1). |

**The data lever is not a bar.** It reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

- **The bar is met and the lever is `ahead`:** distinct text moves the trunk at equal positions. D4 replaces D3 as the trunk's TRAIN. The next unit adds human text again (a D5, with more wiki shards and the OASST2 share restored), at width 256.
- **The bar is met and the lever is `inside`:** at 201.3M positions, 7.2 passes of D3 cost nothing measurable against 2.72 passes of D4. Text is not the limit at this budget. The next unit is the tokens lever on D4: (b64L) at 32,768 steps, under its own sheet. claude-301's tokens lever read `ahead`.
- **The lever is `behind`:** first, the grade and the I-r control are checked. If they hold, the oasst2 sub-panel decides:
  - behind on oasst2 only: the mix share, as named above. The next unit is D4 with OASST2's share restored.
  - behind everywhere: D4 is a worse TRAIN for this recipe. The trunk keeps D3, and the next unit is the tokens lever on D3.
- **The lever is `mixed`:** not distinguishable at two seeds. The sub-panel levers and the spread pick the next unit, recorded as a decision review in the REPORT.
- **The bar is not met on D4:** this holds for this configuration only. claude-301's D3 result stands, and the lever says which way to move.
- **A seed is NOT_RUN:** the missing seed reruns under this sheet, at a new seal root, before any reading.

## Instrument checks

These are registered here, before any pod goes up.
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
  - At least one test must run, and none may fail.
  - The three micro-batch tests of claude-300 must pass by name.
- **I-3:**
  - **D4:** the seal's `SHA256SUMS` must have this sheet's sha, its three `./mix/` lines must check OK, and each mix file must match the shas above.
  - **The panel:** the four held-out folders and the DEV files must match claude-301's pins.
  - **Each run's `config.json`:** must name D4's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed, and the recipe's flags, with no turn memory.
- **I-p, the pin:** the source the pod builds is a clean checkout at exactly the pin. A pod at any other source exits before building (exit 3).
- **I-1, the smoke:** a 128-step run at `--eval-every 32` on D4, at the pod's seed, exits 0 with a finite loss.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. Otherwise the arm is NOT_RUN at that seed.
  - claude-301's (b64L) peaked at 27,179 / 27,276 of 32,607 MiB, which is 83.4% / 83.7%.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB. Otherwise the arm is NOT_RUN at that seed.
  - What it cannot see, as in claude-301's sheet: growth slower than about 16 MiB a step passes I-f. The backstop is the run's own out-of-memory stop.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64, from the smoke. The projected wall is W = used + 1.05 × r × 16,384 + 1,800 s, written to `it.txt`.
  - No gate acts on W.
  - If W exceeds 21,000 s, the watcher's limits are extended under the charter's extend rule, and the REPORT says so.
- **I-4:** every graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-5:** each scored checkpoint's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes, as in claude-301's job.
- **I-s:** D4's seen set names D4's `tokens_sha256` and the panel identity. `pair` refuses otherwise.
- **I-r, run on the laptop before any lever is read:** claude-301's (b64L-D3) grades, re-paired against D3's T2 with `clean=` D3's seen set (`local/claude/claude-295/seen/d3`), must reproduce claude-301's d +75,038 / +85,590 exactly.
  - If they do not, the data lever is not read. (b64L-D4)'s bar still stands.
- **Not repeated: claude-301's I-e.** It is the equivalence check of accumulation against the whole batch on the real recipe.
  - It passed at this pin, recipe and binary, on both seeds' hosts (worst gap 1.3 × 10⁻⁷), and again at claude-303's pin on an RTX PRO 6000 (claude-304).
  - It tests the trainer's arithmetic, which this unit does not change, so it adds nothing to this unit's question.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop: the job's gate tests, with mutations, and `bash -n` on every script.
3. `uor-pod status`. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.15/h each. Each is leased and watched, and is brought up with `--count 1 --ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. Vast machines 48282 and 48276 are avoided (claude-304: driver 580.95.05, no SSH).
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t (recorded only).
   3. (b64L-D4) for 16,384 steps.
   4. The scoring: claude-301's `--score-rows`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-d4-train-2026-11-07/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. I-r.
   2. `grade` with `train=` D4.
   3. `pair` against D4's T2 and unigram.
   4. The data lever, and the oasst2 sub-panel on its own.

## Minimum test power

- **The paired band** is `max(2·√(b+c), the two-seed spread)`, `scale-probe pair`'s.
- **claude-301's (b64L) on D3:**
  - Against D3's T2, d was +75,038 / +85,590 with a band of 10,552. That band is all spread, so the bar here has about 7× headroom.
  - On the oasst2 sub-panel alone, the band was 1,515 against d +13,342 / +14,857.
- **The data lever** inherits both cells' spreads, so it can read `ahead` or `behind` only on a move larger than either arm's own spread: about 10,600 rows if D4's spread is like D3's.
  - The McNemar floor, 2·√(b+c), is about 1,100 rows over the whole panel.
  - So an `inside` on the lever means "within the seed spread", not "no effect".

## Cost

- **claude-301's measured (b64L) runs:** 10,924 s and 12,008 s of training on RTX 5090s at $0.986 and $1.00/h, about $2.99 and $3.34.
  - Its run-2 pods billed $4.84 and $5.48, with (b64S) at 5,444 / 6,035 s.
  - Taking those two pods' totals, less (b64S)'s training, leaves about 0.4 h a pod for setup, I-e, the smokes and both scorings.
- **Here:**
  - Setup, the inputs, the smoke and the scoring: about 0.5 h.
  - Training: about 3.0–3.4 h.
  - So about 3.5–4 h a pod, or $3.50–4.60 at $1.00–1.15/h, and $7–9.20 for both.
- **The cap:** 2 pods × 5 h × $1.15 = $11.50. The charter's extend rule holds: a healthy run is never cancelled for a self-set budget.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m or I-f fails:** the arm is NOT_RUN at that seed, by the outcome map.
- **An out-of-memory error, divergence (a non-finite loss) or a trainer exit other than 0:** the arm is NOT_RUN at that seed. Each is classified as an instrument fault or a mechanism result before it is called anything (§4.2). A divergence on D4 at a recipe that trained on D3 is read as a result about D4 and recorded.
- **A pod never reaches SSH within 1,800 s:** it is deleted, and one retry runs on another machine. After two failures for a seed, the seed waits for a new unit; it is not forced.
- **Always:** the job seals on every exit, and the watcher takes the pod down after `seal=0`.

## Changes from claude-301's sheet

1. **TRAIN:** D4 in place of D3, with its seal, shas, tables and seen set. The bar is read against D4's tables.
2. **One arm,** (b64L), at a fixed 16,384 steps. (b64S) and its levers are dropped.
3. **I-t records only.** No gate cuts steps, because the lever needs claude-301's step count.
4. **I-e is not repeated** (reason under Instrument checks). **I-p** and **I-r** are new.
5. **The readings:** the data lever against claude-301's sealed grades, and the oasst2 sub-panel read on its own, with the mix-share caveat named in advance.
6. **The seal path** is `results/claude/one-model-end-to-end-d4-train-2026-11-07/runs/s{1,2}/`.
