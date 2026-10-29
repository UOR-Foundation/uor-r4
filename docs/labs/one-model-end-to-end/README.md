# one-model-end-to-end: the data census (claude-202, 29 Oct 2026)

Step 1 of [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407), from the [assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md).
- **How it ran:** a count with no model and no GPU. `train-in-space --data-root DIR --fresh-v2 DIR --census 1` (`crates/uor-r4-float-train/src/in_space_long.rs`, `census`) took 4.2 s and 1.1 GB on the M1.
- **What it read:** TRAIN (`mw-balp/train`) and DEV (`ft/dev`), from the HF store's `w2-inputs/2026-10-25/driver-data/`, under their sheet pins; and the sealed Fresh V2 panel, from its preservation archive under its pins (`0b7e754c…`, `a1cda529…`), counted only.
- **What it did not open:** `m2-fresh256` (M2's sealed acceptance panel) and the K16 fresh slice.
- **Output:** [census-2026-10-29.json](census-2026-10-29.json). It holds counts and DEV corpus positions only, never token text.

## The instrument check (M1): every pinned count, exactly

| Count | Pinned (`float-training.md`, rows 14–15) | Census |
|---|---|---|
| DEV openings | 1,226 | 1,226 |
| Openings whose first-8 key TRAIN holds | 1,194 | 1,194 |
| Later rows the turn-8 + position + reply-last-1 table names at share 0.5 | 9,655 | 9,655 |
| … named right | 8,822 | 8,822 |
| Reply-only control: most later-row hits among no-turn keys with precision ≥ 0.83 | 8,089 | 8,089 (reply-last-4, share 0.5, 8,089 / 8,982 = 0.900) |

## The sizes (M0)

| Split | Bytes | Tokens | Eligible targets | Documents | Prompts | Replies |
|---|---|---|---|---|---|---|
| TRAIN | 15,025,658 | **7,512,797** | 2,425,925 | 60,000 | 212,968 | 212,968 |
| DEV | 85,776 | 42,856 | 14,287 | 400 | 1,226 | 1,226 |
| Fresh V2 | 89,420 | 44,678 | 14,479 | 400 | 1,279 | 1,279 |

TRAIN's file is its 64-byte header plus 2 × 7,512,797 bytes. The figure of 26,214,400 tokens for the same sha in `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:62` does not describe this file.

## How much of each panel TRAIN already holds (M3–M6)

| | DEV | Fresh V2 |
|---|---|---|
| Replies equal to a TRAIN reply | **0.750** | **0.777** |
| Prompts equal to a TRAIN prompt | 0.608 | 0.639 |
| Reply 8-grams / 13-grams in TRAIN replies | 0.605 / 0.367 | 0.655 / 0.460 |
| Prompt 8-grams / 13-grams in TRAIN prompts | 0.521 / 0.282 | 0.524 / 0.291 |
| Value-masked prompt templates in TRAIN, K = 256 / 1,024 | 0.644 / **0.642** | 0.668 / **0.665** |
| Value-masked reply templates in TRAIN, K = 256 / 1,024 | 0.781 / 0.766 | 0.810 / 0.798 |
| Openings whose first-8 key TRAIN holds | 1,194 / 1,226 | 1,234 / 1,279 |
| **New templates** (K = 1,024 prompt share ≤ 0.5, fixed in the ORDER) | **no** | **no** |

**Step 2's decision rows (M2):** DEV's 3,406 later rows the reply table does not name at share 0.5, and its 32 unkeyed openings. That is 3,438 positions, pooled, listed in the JSON.

## What it means

- **Three quarters of the replies in both panels already occur verbatim in TRAIN.** Fresh V2's "0 exact full TRAIN/DEV document overlap" holds for whole documents, not for turns.
- **Neither panel can test new phrasings.** The item's next unit builds a panel with held-out templates, with its own census, before the training run's protocol is written.
- **The panels can still test new values inside known templates,** and the step-2 check runs on DEV's 3,438 unnamed rows as planned.
