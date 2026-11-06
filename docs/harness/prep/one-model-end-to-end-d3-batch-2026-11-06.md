# Prep sheet: does a larger batch on D3 make the trunk's seeds agree, and beat T2? (one-model-end-to-end, claude-298, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-298. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: pending (a fresh non-author FIT, recorded at the end).

**Why.** claude-297 (#3774, merged as `1f2bd76ce58503f25ef24e415dd193bf57086ddb`) ran the (near) recipe as a 2×2 of steps and data at batch 16. Its readings:
- **The one-track bar is not met.**
  - (n3L), TRAIN D3 at 32,768 steps, got 272,867 / 234,892 of the panel's 1,460,697 rows against D3's T2 at 256,660. That reads `inside` (d +16,207 / −21,768, band 37,975).
  - The other three cells are `behind` at both seeds.
- **Data is the one lever decided at both seeds.** At 8,192 steps D3 beats D1 by 20,217 / 17,386 rows (band 2,831), unseen rows included. That is claude-297's sheet's branch "None does, and data moves it": the next unit "trains longer at D3, or on more human text".
- **The band is the seed spread, and added seeds cannot close it.**
  - `scale-probe pair` gives every seed a band of at least max d − min d. `ahead` needs every seed beyond its band, and the tool takes at most three seeds (`docs/labs/scale-probe/instrument/src/lib.rs` 919–953, 1129–1149).
  - With seed 2's d negative, no seed set that includes seed 2 can read (n3L) `ahead`. claude-297's decision review set that sheet's "add seeds" line aside for this reason.
- **The spread comes from the optimizer, not the measurement.**
  - Seed 2's lost first run and its rerun on another host read the same loss at step 31,744, to every printed digit (0.9592743465327658).
  - (n3L)'s two seeds had separated by step 8,192: loss 0.9601 against 0.9791.
- **Batch 16 is a noisy gradient.** A batch is 16 × 192 = 3,072 positions.
  - The loss is a per-position mean (`mean_all`, `in_space_long.rs` 1144–1146) under AdamW.
  - So a larger batch at the same `--lr` takes steps of the same size with less noise.
  - A larger batch is the one change that attacks both the spread and "train longer at D3".

## Question

At TRAIN D3 and (near)'s recipe, with only the batch changed from 16 to 64:
- **At equal tokens**, do the seeds agree and the trunk move? (b64S) runs 8,192 steps at batch 64, the same positions trained as claude-297's (n3L).
- **At four times the tokens**, does the trunk pass D3's T2 at both seeds? (b64L) runs 32,768 steps at batch 64.

## Base artifact

- **The pin:** `1f2bd76ce58503f25ef24e415dd193bf57086ddb` (`main` after #3774). No code changes in this unit.
  - Between claude-297's pin (`92ea7ccdf`) and this one, `git diff --stat` is empty for `crates/uor-r4-float-train`, `crates/uor-r4-core`, `crates/uor-r4-tokenizer`, `crates/uor-r4-integer`, `docs/labs/scale-probe`, `Cargo.toml` and `Cargo.lock`.
  - So claude-297's sealed (n3L) grades at seeds 1 and 2 are this unit's batch-16 control, read without a rerun (I-r).
- **The recipe:** claude-297's (n3L), with `--batch 64` and `--steps` per arm. The flags are `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --eval-every 1024`, plus `--train-split` D3.
- **The binaries:** `train-in-space` is built on each pod at the pin, with its sha256 recorded. The scale-probe instrument grades on the laptop at the pin.

## Data split

claude-297's, unchanged:
- **TRAIN D3:** `results/claude/scale-probe-2026-11-05/inputs/d3`, `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`, 27,416,554 scored.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- **D3's seen set:** claude-295's `scale-probe seen`, NGRAM 8. It marks 227,312 rows seen and 1,228,450 unseen.
- **DEV:** claude-261's templated DEV, read in-run only and for no bar.

## Arms

All at TRAIN D3, width 256, `--objective nearest`, seeds 1 and 2. Positions trained = steps × batch × 192.

| arm | batch | steps | positions | passes over D3's scored positions |
|---|---|---|---|---|
| (n3L), claude-297, sealed | 16 | 32,768 | 100.7M | 3.67 |
| (b64S) | 64 | 8,192 | 100.7M | 3.67 |
| (b64L) | 64 | 32,768 | 402.7M | 14.69 |

## Readings

For each arm, at seeds 1 and 2:
- **The panel:** whole-table nearest no-memory top-1 (`scale-probe grade`).
- **Against D3's T2 and unigram** (`pair table=t2|unigram`, `clean=` D3's seen set), on all rows and on unseen rows, by sub-panel too.
- **The levers,** cell against cell (`pair`, no table, `clean=` D3's seen set):
  - batch at equal tokens: (b64S) against claude-297's (n3L);
  - tokens at batch 64: (b64L) against (b64S).
- **The seed spread** of each arm's d against T2, beside (n3L)'s 37,975.
- **T2's share:** `model_hits − b` over T2's hits, from the T2 pair.
- **The loss,** read only within an arm, with the step at which the two seeds' losses first differ by more than 0.005.

## Bar

Verdict words are `scale-probe pair`'s, at both seeds. Mixed reads as "not distinguishable at these seeds", never as a negative (L-007).

| an arm against D3's T2 (whole panel, all rows) | against D3's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met** by that arm. The unseen-row verdict is written beside it in the same sentence: "met; unseen ahead" or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** The next unit is chosen by the levers and the spread (below), never by adding seeds to a pair whose band is already its spread. |
| behind | any | **Not met** for that configuration only (§4.1). |

Two arms each get a chance at the bar. The band at both seeds is the guard.

**The levers are not a bar.** Each reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

- **An arm meets the bar:** its recipe is the trunk's human-text recipe. The next unit is width 1,024 on it, plus the served-form check (§3.7).
- **None does; batch narrows the spread** ((b64S)'s spread under half of (n3L)'s 37,975) **and tokens move it** ((b64L) against (b64S) `ahead`): the next unit is longer still at batch 64, or more human text.
- **None does; batch does not narrow the spread:** the spread is not gradient noise. The next unit reads where the seeds part (the loss divergence step) and takes the objective's next change: claude-297's roadmap step 2 (a centroid refit) or step 4 (an address margin), each under its own sheet.

## Instrument checks

- **I-0:** `cargo test -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
- **I-1:** each arm's 64-step smoke exits 0 with a finite loss.
- **I-t, the throughput gate (registered here, before training):**
  - (b64L)'s smoke gives seconds a step as its last `progress.jsonl` `train_seconds` over 64.
  - If 32,768 steps at that rate would exceed 18,000 s (5 hours), (b64L) runs 16,384 steps instead (7.34 passes). This is recorded in `steps-b64L.txt` and in the REPORT.
  - Nothing else changes.
- **I-3:** D3's and the panels' shas match this sheet. Each run's `config.json` names D3's shas, batch 64, its steps and the recipe.
- **I-4:** every graded checkpoint scores the same 1,460,697 rows (identity `5f88a5b3…`).
- **I-5:** each scored checkpoint's `.i5` reads `same`, and DEV's rows agree across the four per-panel processes.
- **I-s:** D3's seen set names D3's `tokens_sha256` and the panel identity; `pair` refuses otherwise.
- **I-r, the control's standing:**
  - claude-297's (n3L) grades are re-read from `local/claude/claude-297/grade/n3L-s{1,2}`. Their sources are sealed as `runs/s{1,2}` `1c7a0978…` / `dd3cc888…`.
  - Its `pair` against D3's T2 is re-run and must reproduce claude-297's d +16,207 / −21,768 before any lever against it is read.

## Steps

1. The ORDER at the pin; this sheet's FIT; the job script's D67 pre-run review.
2. `uor-pod status` first. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.20/h each, leased, with a watcher armed on each. Each pod's job runs (b64S), then (b64L), on its one card. A two-card pod at most $2.00/h, if that is what the fleet offers, runs both arms side by side.
3. On each pod: setup, I-0, I-3, the smokes (I-1), then I-t.
4. Score each final checkpoint with claude-297's `--score-rows` command, one panel per process.
5. Seal each pod to `results/claude/one-model-end-to-end-d3-batch-2026-11-06/runs/s{1,2}/`, read it back, then take the pod down.
6. On the laptop: I-r, then `grade` against D3's tables, then `pair` as in Readings.

## Minimum test power

- On claude-297's (n3L) pair, the McNemar floor is about 1,100 rows (2·√(b + c) with b + c ≈ 300,000). Its band of 37,975 is all spread.
- A lead of 10,000 rows at both seeds, with a spread under 10,000, reads `ahead`.

## Cost

- **Training:** claude-297's batch-16 step took about 0.18–0.20 s.
  - If batch 64 takes up to 4× that (0.8 s), (b64S) is 1.8 h and (b64L) 7.3 h. I-t would then cut (b64L) to 16,384 steps, 3.6 h.
  - At a likelier 0.4 s, (b64S) is 0.9 h and (b64L) 3.6 h.
- **Wall time:** at most about 6 hours a one-card pod with setup, scoring and the seal. That is the one-card exception to the 4-hour cap, the same exception claude-297 recorded.
- **Spend:** about $10 at two one-card pods near $1.0/h. The cap is $14. The charter's extend rule holds: a healthy run is never cut for a self-set budget.

## Stop rule

- **Before training:** a failure of I-0 or I-3 stops that pod.
- **An arm fails:** a failed smoke, a non-finite loss or an out-of-memory error makes that arm NOT_RUN at that seed, with the failing step. The other arm goes on.
- **A checkpoint fails I-4 or I-5:** its grades are not read, and it is NOT_RUN at that seed.
- **I-r fails:** no lever against (n3L) is read. The arms' bar readings still stand.
- No arm is retuned after it is seen. I-t is decided from the smoke, before training.
- **Export gap:** a recipe adopted from a bar pass owes the served-form check (§3.7) in the next unit, before any serving claim.
