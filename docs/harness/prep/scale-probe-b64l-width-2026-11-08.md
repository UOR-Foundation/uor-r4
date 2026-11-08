# Prep sheet: the width slope on (b64L) (scale-probe, claude-317, 8 Nov 2026)

Lab: claude. Item: scale-probe (#3472). Order: claude-317. Track B of Casey's 8 Nov decision, "Both in parallel" (#2028 6715318106).

TEST FITNESS: pending (a fresh non-author FIT before any pod).

## Question

Does the served recipe (b64L) get measurably better on human held-out text as it gets wider, with everything else fixed?

- claude-295 asked this of the old recipe, and the question could not be read: its training loss did not fall on human text at either width.
- claude-296 classed that as the objective (mechanism).
- (b64L), the recipe that came after, is ahead of D3's T2 at both seeds: +75,038 and +85,590 rows against a band of 10,552 (claude-301).
- So this is the first slope that can be read.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's and claude-315's. `train-in-space` is built on each pod at the pin, with its sha256 recorded.
- **The recipe:** claude-301's (b64L), unchanged except for the width, the heads and the micro-batch:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --batch 64 --eval-every 1024 --steps 16384`, plus `--train-split` D3.
- **Heads scale with width at 64 coordinates a head:** `--hamming-read` is width / 64.
  - `HammingRead::new` (`flock_read_hamming.rs` ~989) refuses a head wider than 64.
  - So width 512 cannot keep (b64L)'s 4 heads.
  - Holding the head's width fixed and adding heads is the only way to scale this read past 256.
- **The micro-batch** is memory only. claude-300's CUDA test shows that micro-batches weighted by their scored rows give the whole batch's step, and claude-301's I-e confirmed it on the real recipe to 10⁻⁸. The batch stays 64 in every arm.
- **Not changed:** lr (0.001), steps (16,384), data, context. That is the design: one variable, width. It is also the main confound (see Outcome map, row 4).

## Data split

claude-301's, unchanged:
- **TRAIN D3:** `tokens.u16` `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`, `response_mask.u8` `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`.
- **The panel:** claude-295's four held-out panels, cut by source id: oasst2, wiki-00000, wiki-00020 and wiki-00040. That is 1,460,697 no-memory rows, row identity `5f88a5b3…`, with each panel's heldout sha256 pair as in `read-301.sh`.
- **The 8-token overlap census** with TRAIN D3 is claude-295's seen set (227,312 seen, 1,228,450 unseen). Every pair reads `clean=` that set. The TRAIN is the same, so the census stands.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- P-dev is not read.

## Arms

| arm | width | `--hamming-read` | `--micro-batch` | seeds | where |
|---|---:|---:|---:|---|---|
| (w128) | 128 | 2 | 16 | 1, 2 | new, one pod, s1 then s2 |
| (w256) = (b64L) | 256 | 4 | 16 | 1, 2 | claude-301's seal, not retrained |
| (w512) | 512 | 8 | 8, or 4 by I-m | 1, 2 | new, one pod per seed |

Width 1,024 is not an arm:
- on the old recipe it did not fit a 32 GB card (claude-295);
- the 80 GB cards are owner-forbidden in `uor-pod`;
- it waits on a 48 GB card unit, which row 1 below unlocks.

## Readings

All are read with `scale-probe grade` and `scale-probe pair` at `clean=` D3's seen set, as in claude-301's `read-301.sh`. The wider arm is always the A side.

- **R1:** (w512) against (w256).
- **R2:** (w256) against (w128).
- **R3:** each arm against D3's T2 and against D3's unigram.
- **R4, speed (recorded, no bar):**
  - training tokens a second from `progress.jsonl`;
  - served decode tokens a second: `served-reply-panel` at `main` `3d3bbb6a` (claude-316), `decode_tokens_per_second`, on each seal's export to `m2-fresh256`, on this laptop, one process at a time.
- **R5, the curve (recorded):** pooled top-1 rows for each width and seed, and each width's mean over seeds against log₂ width.

## Bar

Set before any pod. `pair`'s verdict per pair is `ahead`, `inside`, `behind` or `mixed`, with its paired band across seeds, as in claude-301.

- **Rising:** R1 and R2 both `ahead`.
- **Flat at the top:** R2 `ahead` and R1 `inside`.
- **Not distinguishable:** both `inside`, or either `mixed`. This is not a negative (L-007).
- **Falling:** R1 or R2 `behind`. This closes the configuration (this lr, these steps, this data), never the mechanism (§4.1).

## What each outcome unlocks (the outcome map)

1. **Rising.** Width pays on human text at a fixed 16,384 steps.
   - The next scale-probe unit takes width 1,024 on a 48 GB card (L40S or RTX 6000 Ada, explicit tier), plus a data cell (D4) at width 512.
   - Track A may adopt the widest width whose served decode stays fast enough.
2. **Flat at the top.** 512 gains nothing over 256 at fixed data and steps. D3 at 16,384 × 64 windows is about 6.8 passes, so the likelier limit is data or updates, not width.
   - The next unit is D4 at width 512, or 512 at 32,768 steps. Width 1,024 waits.
3. **Not distinguishable.** The band is too wide for these seeds.
   - The next unit adds seeds 3 and 4 at the pair whose verdict was `inside`. Nothing is closed.
4. **Falling.** Before any mechanism call, the learning rate is the instrument suspect: 0.001 is tuned for width 256, and wider sheets often need a smaller step.
   - The next unit is an lr check at the falling width (0.0005 and 0.00025, one seed, a short smoke first).
   - Only if that also falls is the result classed mechanism.

## Instrument checks

Registered here, before any pod. They are claude-301's gates, with the changes noted.

- **I-0:** the pin, then `cargo test --release --locked -p uor-r4-float-train --features cuda --lib`, including claude-300's three micro-batch tests by name, as passed.
- **I-3:** each run's `config.json` must show TRAIN D3's two sha256s, the arm's `width`, `hamming_read` and `micro_batch`, `batch` 64, `steps` 16,384, the seed, and claude-301's recipe fields.
- **I-e,** per pod, at the pod's widest arm and the pod's seed: three 4-step runs at batch 16 with `--eval-every 1`.
  - The runs are (whole); (micro4), as four micro-batches of 4; and (read0), whole with `--hamming-read 0`.
  - **Pass:** (micro4) within 10⁻⁶ relative of (whole) at every step, and (read0) at least 10⁻⁵ from (whole) at step 4. This is claude-301's amended I-e, at the arm's width and heads.
  - **Fail:** the pod trains nothing.
- **I-m,** per arm, from its smoke (128 steps at `--eval-every 32`, `mem.log` every 2 s):
  - the peak is below 90% of the card;
  - (w512) tries micro-batch 8 first, and on an I-m failure or an out-of-memory stop it tries 4. Both tries are recorded;
  - if 4 also fails, (w512) is NOT_RUN at that seed, as an instrument.
- **I-f:** the peak after the smoke's step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t,** recorded only, with no cut: the rate from the smoke's steps 64–128, and W = used + 1.05 × r × 16,384 + scoring.
  - Cutting steps would break the fixed-steps design, so every arm trains 16,384 steps. A projection over 36,000 s is noted, and the watcher's spend notice applies.
- **I-r:** claude-301's (b64L) grades are re-paired with this unit's `scale-probe` build against D3's T2. They must give claude-301's d, +75,038 / +85,590. If they do not, R2 and R1 are not read against (w256).
- **I-5:** each panel is scored in its own process (claude-295's repair), each process scoring DEV too, and DEV agrees across a cell's four processes.
- **I-x,** on the laptop, before R4: `cpu-fbs-export --raw-gates --raw-transition` (sha `724a3b78…`) re-exports claude-313's seal byte for byte (claude-315's I-xc), then exports each new seal. Those exports are what `served-reply-panel` reads.

## Steps

1. The ORDER at the pin; this sheet's FIT, posted on #3472; the D67 pre-run review of the job, chain, seal and watch scripts, posted on #3472.
2. `uor-pod status`, staying inside 4 pods and $8/h across all labs. claude-315's two pods come down first.
3. Three Vast RTX 5090 pods, each at most $1.15/h, leased, each with a screen watcher:
   - pod A: (w512) s1;
   - pod B: (w512) s2;
   - pod C: (w128) s1, then (w128) s2.
   Each pod is launched the moment its own `Ready` line appears (claude-315's lesson).
4. On each pod: setup, I-0, the inputs and I-3, I-e, the smokes with I-m and I-f, I-t, the arm or arms, then the scoring with I-5.
5. Seal each pod to `results/claude/scale-probe-b64l-width-2026-11-08/runs/{A,B,C}/`, read the seal back, then take the pod down.
6. On the laptop: I-r, `grade`, `pair` (R1–R3), then I-x and R4.

## Minimum test power

claude-301's: the McNemar floor is about 1,100 rows, and the paired band across seeds is the binding limit.

(b64L)'s band is 10,552. A width step smaller than about that reads `inside`, which is row 3, not a negative.

## Cost

These are projections; the smokes measure the real rates.
- **(w256)'s rate** was 0.633 / 0.698 s a step (claude-301).
- **(w512):** the old recipe measured ×1.97 a step per doubling (claude-295), and a smaller micro-batch adds kernel launches. Projected 1.25–1.6 s a step, so 5.7–7.3 h of training plus about 0.5–1 h of scoring: up to about 8.3 h a pod.
- **(w128):** projected 0.32–0.40 s a step, so 1.5–1.8 h a seed plus scoring: about 4–4.5 h for pod C.
- **Spend:** about 21 pod-hours at $0.56–1.15/h, so about $12–24. The ORDER's notice line is $25; it is not a cap, and the charter's extend rule applies to a healthy run.

## Stop rule

claude-301's, with these changes:
- An I-0 or I-e failure stops that pod before any training.
- An I-m failure at micro-batch 8 retries (w512) at 4. A failure at 4 makes (w512) NOT_RUN at that seed, as an instrument; R1 is then not read, and the 48 GB card unit is next.
- An out-of-memory error mid-run is an instrument fault: that arm is NOT_RUN at that seed, and pod C's other seed still runs.
- An I-r failure blocks R1 and R2 against (w256), never the new arms' own R3.
- A pod that does not reach SSH within `uor-pod`'s window is replaced once.

## Changes from claude-301's sheet

1. The arms: widths 128 and 512 at width / 64 heads; width 256 is claude-301's seal.
2. `--micro-batch` 8 or 4 at width 512.
3. I-e runs at the pod's widest arm.
4. I-t records only and never cuts steps.
5. New readings R1, R2 and R5, and the speed reading R4 with I-x.
6. The outcome map above.
7. The seal path is `results/claude/scale-probe-b64l-width-2026-11-08/`.
