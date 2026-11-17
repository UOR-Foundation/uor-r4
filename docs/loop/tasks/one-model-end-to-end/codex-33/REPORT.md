# REPORT: one-model-end-to-end, codex-33 (which checkpoint, and what "training scale" means)

Lab: codex. Follows codex-32 (the value-present gate) and codex-30 slice 2 (the k-term writer).

## The question

The previous recommendation was "training scale: more steps at the same recipe". The run's own
record was read first, before spending any GPU time, and it refuses that reading.

## The run overfits from 0.54 epochs

`/tmp/opencode/armr-copy-s2/progress.jsonl`, the arm r copy run (16,384 steps):

| step | epoch | train loss | dev loss | dev top-1 |
|---|---|---|---|---|
| 0 | 0.00 | — | 2.787 | 4 |
| **2,048** | **0.54** | 0.976 | **2.247** | **447** |
| 4,096 | 1.09 | 0.824 | 2.335 | 360 |
| 6,144 | 1.63 | 0.668 | 2.453 | 303 |
| 8,192 | 2.17 | 0.580 | 2.413 | 357 |
| 10,240 | 2.71 | 0.539 | 2.404 | 237 |
| 12,288 | 3.26 | 0.515 | 2.362 | 286 |
| 14,336 | 3.80 | 0.500 | 2.420 | 267 |
| 16,384 | 4.34 | 0.493 | 2.371 | 328 |

Train loss falls monotonically; dev loss bottoms at 2.247 at **0.54 epochs** and never recovers;
dev top-1 peaks at 447 there and is 237–357 after. **The recipe memorises a 3,774-step epoch in
half an epoch.** More steps at this recipe is not neutral, it is harmful.

## The artifact we had been scoring was the worst checkpoint

The export recipe was verified, not guessed: `fbs-export --legacy-artifact SNAPSHOT OUT` reproduces
the existing artifact **byte for byte** (`f21046823640ea40…`), and it still does after codex-30
slice 2 added `terms`/`counts` — the k = 1 compatibility gate, proven on a real artifact, not only
at the word level.

Three checkpoints, same recipe, same build, same panel, default rendering, `--reply-tokens 16`:

| checkpoint | epoch | dev loss | value-present | length2 | length4 | length8 | reassert | update | tok/s |
|---|---|---|---|---|---|---|---|---|---|
| **step 2,048** | 0.54 | **2.247** | **57 / 256** | 27 | **12** | **1** | **16** | **1** | 377 |
| step 4,096 | 1.09 | 2.335 | 56 / 256 | 29 | 9 | 0 | 17 | 1 | 379 |
| step 16,384 (the served arm r) | 4.34 | 2.371 | 25 / 256 | 18 | 1 | 0 | 6 | 0 | 343 |

The panel repeats exactly (57 twice), and it **tracks dev**: the dev-best checkpoint is the
panel-best checkpoint. The artifact the item had been measuring, and the 25/256 in codex-32, is the
most overfit point of the run. **2.3× is the cost of having scored the last checkpoint.**

## What the 57 actually are

All 57 carry the accepted value **and the right attribute**; none is bare:

| count | attribute | frame | example |
|---|---|---|---|
| 34 | right | grammatical | `My current job is basalt loggia.` |
| 23 | right | ungrammatical | `I current job is basalt loggia.` / `My currently live in porphyry belvedere.` |
| 0 | — | — | no bare-value reply |

So at the dev-best checkpoint the model finds the fact, states it with the right attribute, and the
remaining defect is **frame grammaticality in 23 of 57**: the corpus's reply phrasings are mixed, so
the model blends them (`I current job is`). That is a corpus-shape defect with a known fix (one
frame per intent, or a grammar-consistent set), not a memory or read failure.

This **corrects codex-32's reading** of the same family of replies: "the opener is wrong and the
attribute is sometimes swapped" was true of step 16,384 and is not true of step 2,048.

## The decision

1. **Score the dev-best checkpoint, not the last.** The served arm r artifact is superseded by
   `/tmp/opencode/armr-step002048.uorlut` (sha `5bb0f95d…`), 57/256.
2. **"Training scale" means data, not steps.** The 0.54-epoch optimum is a data-volume limit: the
   corpus runs out. The levers, in order: a larger corpus trained to the same ~0.5 epochs; a
   grammar-consistent reply frame; then, if the dev curve still flattens with data to spare, the
   k-term arm (codex-30 slice 3) as the capacity test.
3. **A checkpoint is chosen by dev loss before the panel is spent** — and the panel is then a check
   of that rule, which it passed.

## Checks

`fbs-export --legacy-artifact` reproduces `f21046823640ea40…` on the pre-change snapshot and after
the codex-30 slice-2 change (k = 1 byte-identical). Three snapshots via `fbs-snapshot` (GPU), three
artifacts, three panels; the 2,048 panel repeated bit for bit at 57/256.
