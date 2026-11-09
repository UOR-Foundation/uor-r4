# Prep sheet: (b64L-d3mwcap6) with the ask heads, the question carry and the found-copy pull (one-model-end-to-end, claude-339, 9 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-339. The run is judged by the served-reply score, under "Yes, pipeline first" (#2028 6709947526).

TEST FITNESS: pending (a fresh non-author review before any pod).

**Why.**
- **Recall is the gap now.** claude-322's (b64L-d3mwcap6) stops on 109 and 86 of 256 replies without collapsing (9 and 6), but 0 rows are complete at both seeds. Its most common stopped reply names the wrong relation and cuts the value short. The reply's first line holds the asked-for value in only 14 and 2 rows.
- **The recall task's answer is the ask heads with the question carry** (claude-335, B2-aqC at `h = 4`): `R_ask` 1.000 / 0.969 against the 0.50 floor, and its `−C` cut fails at both seeds (0.105 / 0.027). So the carry is what answers.
- **The parts are all in place:**
  - claude-337: the language trainer's pull at its scored rows;
  - claude-336: the integer session serves the ask table and the carry;
  - claude-338: `served-reply-panel --engine integer` scores them, and the CPU-port path refuses an asking artifact.
- **The named risk.** A language window has no question row, so only the scored placement (`--hamming-attract-at scored`, the default) is open. On the recall task that placement (B2-aC) passed at seed 2 and failed at seed 1 (`R_ask` 0.240 / 0.258). It was undertrained and still rising: 0.04 to 0.24 in the last 2,048 steps. The answer here is a reading, not a third seed or more steps:
  - **The checkpoint trend** (Readings): the served first-line count and `pull.jsonl`'s pull distance at steps 8,192, 12,288 and 16,384. A rising L-aC reading at the last step, with no bar met, reads as row 4u ("not learned yet"), and its next step is the longer run.
  - **Why not more steps here:** 16,384 steps keeps the held-out pair against claude-322 and (b64L-D3) at an equal step count, which every earlier sheet on this line kept.

## Question

At (b64L)'s recipe and pin, on claude-322's TRAIN (D3 + the reply-capped `mw-balp/train` × 6), with the ask heads on the 4 Hamming reads, and pulled at the scored rows:
- **does the carry put the asked value into the served reply?** That is the first-line count and the panel's complete rows, against the no-carry arm (L-a), against the no-ask line (claude-322) and against its own `−C` cut;
- **does stopping hold** (EOS stops and collapse, against claude-322's 109 / 86 and 9 / 6)?
- **what does it cost on the held-out human text,** against (b64L-D3) and claude-322?

## Base artifact and TRAIN

- **Recipe:** claude-301's (b64L), unchanged. `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16`, for 16,384 steps at `--eval-every 1024`.
- **TRAIN:** claude-322's mix, from the store at `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`:
  - tokens `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`;
  - mask `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`;
  - manifest `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`.
  - 417,421 usable TRAIN windows.
- **The pin:** `fbf15e56f3ab1961b30610893392e7a97fd0ab18` (claude-338's merge). It holds claude-337's pull, claude-336's served asks and claude-338's scorer. The trainer is built at the pin on each pod; the scorer and `fbs-export` are built at the pin on the laptop.

## Arms

| Arm | flags beyond the recipe | what it isolates |
|---|---|---|
| **L-aC** | `--hamming-ask 1 --hamming-carry-question 4 --hamming-attract 1` | the full mechanism, pulled at the scored rows |
| **L-a** | `--hamming-ask 1 --hamming-attract 1` | the no-carry control (L-010): the ask heads and the pull, with no carry |
| L-aC `−C` | L-aC's checkpoint, served with its carry removed (below) | the carry's share, at the same weights |
| claude-322 | none (sealed: artifacts `86e51d5e…`, `8838cd10…`; grades `24d69af0…`, `4407e2c2…`) | the no-ask line, at the same recipe and TRAIN |

- **Seeds 1 and 2** for L-aC and L-a: four runs, one RTX 5090 pod each.
- **`−C` is a data cut, not a new run.** `jq` removes `carry` from every `hamming_ask` entry of L-aC's snapshot, and `fbs-export --run-config` exports it again. The question table and every other section are unchanged. That is checked by I-cut.

## Readings

At seeds 1 and 2, on each arm's served `step-016384` (and `−C`), from `served-reply-panel --engine integer` on m2-fresh256, the same measures as claude-322's sheet:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows, the count of the most common reply text, beside the number of rows it is over. Also distinct reply texts over all rows.
- **The first-line count**, by `firstline.jq` (`b15c778d…`): the asked-for value, the other case's value, and both. This is the recall reading. Here it is the deciding count for rows 2 and 3, not descriptive (see Bars).
- **Reply length** (median), and EOS stops per stratum.

**The checkpoint trend,** for L-aC and L-a: the first-line count and the complete rows at `step-008192`, `step-012288` and `step-016384`, each served the same way, beside `pull.jsonl`'s pull distance at those steps.

**Held out:**
- `scale-probe grade` of `step-016384` with `train=` the mix, on all 1,460,697 rows, from the pod's no-memory `--score-rows` output.
- `scale-probe pair` against claude-301's sealed (b64L-D3) grades (`e17a1354…`, `3170c779…`): its pooled verdict and mean d, as in claude-322's sheet.
- Descriptive: against claude-322's grades (`24d69af0…`, `4407e2c2…`).

**Recorded only:** `pull.jsonl` per checkpoint, the dev curve, the last step's loss, the step time, and the carry gains' extremes on the snapshot (claude-335's sign split).

## Bars

Fixed before any run.
- **The panel's bar:** at least 104 complete and 52 good pairs, at both seeds. Every model so far has 0.
- **Recall moved,** the bar this sheet adds: at both seeds, L-aC's first-line asked count is **at least 16 above** both L-a's at that seed and claude-322's at that seed (14 and 2), **and** at least 16 above its own `−C` cut.
  - 16 rows is about 6% of 256. At claude-322's rates (about 3% of rows), one row's binomial spread over 256 is about 2.7 rows, so 16 is about 6 spreads.
  - Seed-to-seed spreads have been far larger (claude-321: 25 and 76), which is why both seeds must meet it, against same-seed controls.
- **Stopping held:** at least 64 EOS stops at each seed (claude-322's lower seed, 86, less about one claude-322 seed spread), and no collapse (one text on 64 or more EOS-stopped rows).
- **The held-out cost**, a decision rule, not a bar: claude-322's ceiling, a mean pooled d against (b64L-D3) below −58,212.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | an arm NOT_RUN at a seed after its rerun, for an instrument fault | **`blocked`.** Repair and rerun that arm and seed under this sheet. |
| 0b | an arm NOT_RUN after its rerun, for a mechanism cause (a non-finite loss with the ask heads on) | **`diagnostic stop`** on that arm. Next: a decision review in the REPORT. |
| 0c | I-xg, I-xa, I-cut or I-c fails | **`blocked`.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds on L-aC | **The served model answers.** Next: the outcome map's next scale, under its own sheet. |
| 2 | recall moved and stopping held on L-aC | **The carry puts the asked value into served replies on language.** Next: the panel's complete rows. The REPORT reads why replies hold the value but are not complete (the wrong relation, the value cut short, no EOS), and the next lever follows that. |
| 3 | recall moved, stopping not held | **The carry carries the value, at the cost of stopping.** Next: a decision review between a stronger EOS share in TRAIN and a decode-time stop, with this arm as the base. |
| 4u | recall not moved, and L-aC's first-line count at `step-016384` is above its count at `step-012288` by at least 8 at both seeds | **Not learned yet** (B2-aC's shape). Next: L-aC alone at 32,768 steps, two seeds, under its own sheet and cost. |
| 4c | recall not moved, and L-aC's `−C` cut reads within 4 of L-aC itself at both seeds | **The carry is not used on language at this recipe.** Next: a decision review between a question-row placement on chat windows (the `both` placement that passed at both seeds) and a carry reach longer than 4. |
| 4 | recall not moved, otherwise | **No recall gain from the ask heads and the carry at 16,384 steps,** for this configuration only. Next: a decision review in the REPORT, reading the trend, the `−C` cut and the pull distance together. |

A seed that is NOT_RUN reruns once, sealing to `runs-2/` (L-002).

## Instrument checks

Registered before any pod. claude-322's, with these changes:
- **I-0, I-p, I-3, I-1, I-m, I-f, I-t, I-5, I-4, the watcher:** as claude-322's, at the new pin, TRAIN unchanged (417,421 windows).
  - Each `config.json` must name `hamming_ask` true, `hamming_attract` 1, and `hamming_carry_question` 4 (L-aC) or none (L-a).
  - The watcher's notice time is 28,800 s (I-t's expected doubling, below). It stops nothing (D67).
- **I-pull, from the smoke:** `pull.jsonl` exists after the smoke and its mean copy rows per step is above 0. A pull that never runs is an instrument NOT_RUN.
- **I-xg, the GPU export control, once on the first pod before its full run:** `fbs-snapshot` of claude-322's sealed seed-1 `step-016384` (fetched from the store), then `fbs-export --run-config --raw-gates --raw-transition` on the laptop, must give claude-322's artifact `86e51d5e…`. This shows that the export path this run must use (below) serves what claude-322's did.
- **I-xa, the ask export, on the smoke's `step-000064`:** its snapshot has a `hamming_ask` entry per read; the readout's entry has `carry.rows` 4 (L-aC) or none (L-a); `fbs-export` accepts it; and `served-reply-panel --engine integer` scores it while the default engine refuses it by name.
- **I-cut:** the `−C` artifact differs from L-aC's in the carry section only. The check is `read_hamming_uorlut01`'s reads on both: same question tables, carry `None` on the cut.
- **I-c, on the laptop before any reading:** claude-322's two artifacts score, under `--engine integer`, to claude-322's sealed reports without the timing fields (claude-338's parity, repeated at the pin).

**Why the export moves to the pod:** `cpu-fbs-export` (DeepSeek's CPU exporter, which claude-322 served through) does not read ask heads. Exporting an asking checkpoint through it would drop them silently, the fault claude-338 closed in the scorer. `fbs-snapshot` reads the routes through the model's CUDA projection, so it runs on the pod. `fbs-export` (CPU) then runs on the laptop.

## Steps

1. The ORDER, this sheet's FIT, then the D67 pre-run review of the scripts.
2. **On the laptop:**
   - claude-322's job, chain, seal, watch, seed and read scripts, as claude-339's, with only these changes:
     - the arm flags;
     - the pin;
     - the `config.json` checks;
     - I-pull;
     - `fbs-snapshot` of `step-008192`, `step-012288` and `step-016384` on the pod, sealed beside the run;
     - I-xg on seed 1's L-aC pod;
     - the read script's serve step through `fbs-export` and `--engine integer`, plus `−C` and I-cut.
   - Their gate tests and mutations, and `bash -n`.
   - I-c, and the scorer and `fbs-export` built at the pin.
3. `uor-pod status`. The caps must hold with four pods: at most 4 and $8/h. Then one RTX 5090 pod (at most $0.70/h) per arm and seed, each with its watcher before any script ships.
4. **On each pod:** I-p, I-0, the build, the inputs, I-3, the smoke, I-m, I-f, I-pull, I-xa's snapshot, I-t. Then 16,384 steps. Then the three snapshots, and the no-memory scoring of `step-016384` with I-5.
5. **Seal** to `results/claude/one-model-end-to-end-language-carry-2026-11-09/runs/{L-aC,L-a}/s{1,2}/`, each root empty first (L-002). Read back, then take the pod down.
6. **On the laptop:** I-xg, I-xa, I-c. Then serve every snapshot, cut `−C`, I-cut, the panel, the first-line counts and the trend. Then grade and pair.

## Minimum test power

- **The first-line count** is the deciding reading. claude-322's 14 and 2 set the floor, and claude-321's 25 / 76 the seed spread. The bar compares same-seed arms at 16 rows, and needs both seeds.
- **The trend's 8 rows** (row 4u) is half the bar, over 4,096 steps.
- **Stopping and collapse** as claude-322's: within 32 of the floor is "near".
- **The held-out pair** uses `scale-probe pair`'s band, as before.

## Cost

- **Step time:** claude-335 measured the ask heads at about half the no-read speed (43,852 against 89,873 context tokens a second), and the carry at about 3% more. So each run should take about 2 × claude-322's 8,046 s, about 16,100 s or 4.5 h. I-t records the real value.
- **Four pods at $0.563–0.70/h × about 4.9 h** (with setup and scoring): **$11 to $14.** The budget notice, not a cap, is 4 × 6 h × $0.70 = $16.80.
- The snapshots add three `fbs-snapshot` runs per pod, a few minutes each.

## Stop rule

claude-322's, with:
- **I-pull failing:** NOT_RUN for that arm and seed (instrument).
- **I-xg failing:** stop before any full run. The export path is the fault (row 0c).
- **A snapshot failing** at an intermediate step: the trend loses that point, and the REPORT says so. At `step-016384` it is row 0c.

## Changes from claude-322's sheet

1. **The arms:** L-aC and L-a, with the ask heads, in place of the one no-ask arm. claude-322 is the no-ask line, read from its seals.
2. **The pin** is `fbf15e56f`.
3. **The export** moves from `cpu-fbs-export` to `fbs-snapshot` on the pod and `fbs-export` on the laptop, with I-xg and I-xa.
4. **The scorer** runs with `--engine integer`.
5. **The first-line count decides** rows 2 to 4. A new recall bar is added; the stopping floor is lowered to 64 against this TRAIN's own 86.
6. **The checkpoint trend and the `−C` cut** are new readings.
7. **Four pods,** and a cost about four times claude-322's.

## Pre-run review

Pending.
