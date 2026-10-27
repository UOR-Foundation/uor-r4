# Prep sheet: fold against clamp for the rate score, whole-table nearest objective (in-space-trunk-to-text, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Code: branch `claude/train-in-space` at `f2f002d6a66d22f87d07b4db1cf101b0bddf097c`.

## The question

The trained trunk reads about 6 tokens back, and 61–63% of lane positions sit at the shortest-memory rate entry, against 33% at step 0. The rate score is clamped to `[0, 1]`, and a clamp has zero gradient outside it, so a lane pushed past either end stops learning there. **Does folding the score into `[0, 1]` by reflection (slope +1 or −1 everywhere) let lanes keep longer memory, so that reply openings read their prompts?**

## Arms (identical except as stated)

Common: `train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --seed 1 --lr-end 0`, the FBS sheet trunk (`build_fbs_model(OneAtomActive)`), 1 × RTX 5090, one shared vector table, the target scored against the whole table's 8 nearest wrong tokens with margin 1, decoded as the whole table's nearest token.

| Arm | Pod | Rate score | Peak rate |
|---|---|---|---|
| `n-nearest` (control, already running) | `vast-57919115`, `long3` | clamp | `5e-4` |
| `n-nearest-1e-3` (control, already running) | `vast-57919115`, `long3` | clamp | `1e-3` |
| `f-fold-5e-4` | `vast-57941850`, `long4` | fold | `5e-4` |
| `f-fold-1e-3` | `vast-57941850`, `long4` | fold | `1e-3` |

The control arms ran from commit `047af46288fdd93c6ad2346d41f71e00a20bd5eb`, whose clamp path is byte-identical in behaviour to `f2f002d6` with `--fold-scores 0` (the fold is a switch that is off by default).

## What is measured (DEV, all 14,287 targets, 400 documents, 428 windows)

- `nearest.hits`: whole-table nearest top-1.
- `nearest.opening_hits` over `opening_rows` (1,226 rows whose previous corpus token is not a response token), against `commonest_opening_rows` (161: always naming the commonest opening token).
- `--reach` at step 16,384: the opening point's mean shift when the token `k` places back is replaced by BOS, `k ∈ {0,…,6,8,10,12,16,24,32}`; and each layer's rate-entry histogram.
- Greedy replies to six fixed DEV prompts at every checkpoint (`samples.md`).
- Training context tokens per second.

## Decision table (fold arm against its clamped twin, same rate)

| Outcome | Reading | What it unlocks |
|---|---|---|
| Fold openings > 161 and reach at `k = 16` at least 5 × the twin's | Dead clamp lanes were the cause | Longer runs and the reply text on the fold; item done-when (1)–(2) checked |
| Fold reach longer, openings still 161 | Memory survives, the readout ignores it | The next unit looks at what reaches the opening, not memory length |
| Fold reach no longer than the twin's | The clamp was not the cause | The next unit tests the gradient's preference for short memory |
| Fold whole-panel top-1 more than 500 below its twin | The fold costs fit | Keep the clamp; study the fold's boundary attractors |

Both rates must agree for a reading to count; a split is inconclusive.

## Instrument checks

- I-1: the fold's CUDA unit test (`the_fold_reflects_into_the_unit_interval_and_keeps_its_gradient`) passes on the run's build: values `[0.5, 0.3, 0, 0.4, 1, 0.8, 0.7, 1]` for inputs `[-2.5, -0.3, 0, 0.4, 1, 1.2, 2.7, 3]`, slopes ±1.
- I-2: the clamped arms reproduce the earlier runs' step-0 DEV reading (`nearest 2`, dev loss `2.81055`), so build and data match. The fold arms differ at step 0 only through scores already outside `[0, 1]`.
- I-3: on the fold arms' step-0 checkpoint, layer 1's rate histogram has less than 33% of positions at entry 0 (the clamp's step-0 share, `64,236 / 196,608`), since folded scores no longer pile up at the ends.
- I-4: no non-finite loss (the runner stops on one).

## Data (pinned in `hamilton_sheet.rs`, checked at load)

`caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`: `tokenizer.json` `d36d3e87…f89`, `mw-balp/train/tokens.u16` `3dee858f…a4c8`, `mw-balp/train/response_mask.u8` `b5e7352d…49e3`, `ft/dev/tokens.u16` `0919e90f…555b`, `ft/dev/response_mask.u8` `a838bb1a…b078`.

## Cost and stop rule

Two arms side by side on one RTX 5090 at `$0.563/h`: about 0.23 s per step each, about 65 minutes, about `$0.70`. Pods together `$1.13/h` of the `$8/h` cap. Stop on a non-finite loss or a crash; a healthy run is never cut short. Results go to `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/long4` before the pod comes down.
