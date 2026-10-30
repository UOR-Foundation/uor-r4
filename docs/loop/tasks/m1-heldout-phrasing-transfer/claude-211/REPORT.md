# REPORT: m1-heldout-phrasing-transfer, claude-211

Lab: claude. Item: [m1-heldout-phrasing-transfer](../../../../roadmap/items/m1-heldout-phrasing-transfer.md). Order: [ORDER.md](ORDER.md). Sheet: [m1-heldout-phrasing-transfer-2026-10-30](../../../../harness/prep/m1-heldout-phrasing-transfer-2026-10-30.md), FIT and pre-run PASS.

Closes #3416

## Tokens per second

antigravity-110's two saved `UORLUT01` integer artifacts serve at a median **10,876** (seed 1) and **10,566** (seed 2) decode tok/s on an Apple M1 laptop, with prefill at 2.67M tok/s. This is `d11-scorer toks`: 1 warmup, 20 runs, a 192-token prefill and a 64-token decode ([toks-s1.json](../../../../labs/m1-heldout-phrasing-transfer/evidence/toks-s1.json), [toks-s2.json](../../../../labs/m1-heldout-phrasing-transfer/evidence/toks-s2.json)).

## What did not work, first

- **The first GPU attempt did not compile.**
  - Build A, as bar (e)'s sheet wrote it (no `--target`), failed with `E0463: can't find crate for uor_r4_core`. That is the host and target collision the scorer's own `Cargo.toml` warns of (cargo #6313).
  - The tests had already passed with `--target`, the bar (f) tests and the CUDA path included. The job stopped before reading any panel byte.
  - **Classed as an instrument fault** (CHARTER §4.1): repaired by adding `--target x86_64-unknown-linux-gnu` to both builds, named in the sheet, and not counted.
- **The CUDA path of `panel-score` was not type-checked on the Mac.** The crate's build script needs a real nvcc. The pod's test step was its first compile, and it passed.
- **The panel's replies are still template-shared.** 0.625 of p-f's replies are verbatim TRAIN replies, because the generator shares reply templates across phrasings. So bar (f) shows reading new prompts into known kinds of reply, not new kinds of reply.
- **Bar (e)'s copy-successor condition is reported, not required:** D82 names the unigram and T2 only. The model passes it anyway: 1,903 and 1,904 of 4,077 copy-successor rows, against T2's 1,260.
- **The item file's `status` is not edited here.** It is outside this ORDER's scope (`docs/roadmap/items/`).

## What changed

- **`d11-scorer panel-score`** (`docs/labs/m1-d11-artifact-scorer/instrument/`):
  - the gate is split: `validate_decision_receipt` holds the receipt's own checks, and `validate_fresh_transfer_gate` is that plus Fresh V2's pins, unchanged;
  - `refuse_reserved_panel` turns away the sealed fresh panels, P-dev, DEV and TRAIN;
  - `panel_rows` loads the panel under its config's sha256s;
  - `evaluate_panel_two_seeds` grades bar (f).
  - It has three tests in `tests/panel.rs`, each with a negative gate.
- **M1's close grade** (`m1_close.rs`, done-when 3):
  - M1 now closes on (a) to (d), and (e), and (f), as `PASS_M1_ALL_SIX_BARS`;
  - with only (f) missing it holds as `HOLD_M1_OPEN_BAR_F_UNMET`;
  - a bar (f) report for other artifacts, or on a reserved panel, is refused;
  - the canonical saved model carries the committed bar (f) report, and `d11-scorer m1-close-grade` prints `m1_close_pass true`, status `CLOSED_PASS`.
  - Its negative gate passes: with bar (f) set aside, the test fails.
- **All 26 of the crate's laptop tests pass** (`--target aarch64-apple-darwin`). On the pod, the CUDA build's tests passed too.

## Measurements

**Bar (f) passes at both seeds.**

**The panel, p-f** ([README](../../../../labs/m1-heldout-phrasing-transfer/README.md)):
- **Drawn by `heldout-panel`:** `dev,dev` at seed 2051, 400 conversations, 15,953 reply targets, stored at `inputs/claude/heldout-panels-2026-10-30/` with a `SHA256SUMS`.
- **Its census, against the saved model's TRAIN:** 0.000 of its prompts' value-masked templates occur in TRAIN at K = 1,024, so it is new-template by claude-202's rule. Its `train,train` control has 0.637, and Fresh V2, bar (e)'s panel, has 0.665.

**The score,** on 1 × RTX 5090, at `74675bb3f` ([panel-score.json](../../../../labs/m1-heldout-phrasing-transfer/evidence/panel-score.json)):

| On p-f, 15,953 rows | Seed 1 | Seed 2 |
|---|---|---|
| Saved model, integer top-1 | **7,850** | **7,872** |
| T2 / unigram | 7,375 / 1,409 | 7,375 / 1,409 |
| Lead over T2 (b − c), against its band | 1,038 − 563 = **475**, band 81 | 1,051 − 554 = **497**, band 81 |
| Lead over the unigram, against its band | **6,441**, band 162 | **6,463**, band 162 |
| Refused rows, reference mismatches, B4 | 0, 0, pass | 0, 0, pass |

- **The outcome is `PASS_BAR_F_HELDOUT_PHRASING`.** The band is `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`, and the two-seed spread is 22.
- **The instrument checks, all held:**
  - build A first reproduced the saved model's DEV top-1, 7,002 and 7,001 of 14,287;
  - builds A (`09605454…`) and B (`bcba3577…`) gave byte-identical reports;
  - every input matched its pin, the receipt included.
- **The seal** is at `results/claude/m1-heldout-phrasing-transfer-2026-10-30/run-74675bb3/`: 61 files with a `SHA256SUMS`. The pod was released and taken down.
- **Cost:** about $0.20.

## Checks run at the head

- **The scorer crate, laptop:** `cargo test --locked --no-fail-fast --target aarch64-apple-darwin` gives 26 passed, 0 failed. It includes `tests/panel.rs`'s 3 tests and the updated `tests/m1_close.rs`.
- **The negative gates:**
  - bar (f) without T2 in its pass;
  - P-dev left out of the refusal list;
  - bar (f) set aside from `m1_close_pass`.

  Each makes its test fail, and the file is restored after.
- **On the pod (`74675bb3f`):**
  - the scorer's CUDA tests passed;
  - builds A and B, the input pins, the DEV reproduction gate (7,002 / 7,001), and `panel-score` on both builds, byte-identical.
- **`d11-scorer m1-close-grade`** prints `m1_close_pass true`, status `CLOSED_PASS`.
- **The panel's census:** `train-in-space --census 1` with both panels.

## Skills used

None.

## Still open

- **M1 (#2029):** a saved model has now passed (a) to (f). Its close grade says `CLOSED_PASS`. Closing the milestone in `PLAN.md`, and on #2029, is a status change for the top session's decision review.
- **(r-drop),** from one-model-end-to-end's training run, can be scored on p-f as evidence once its checkpoints are sealed. It enters no bar.
## Plain summary

M1's last bar, (f), passes. On a fresh panel of prompts phrased in ways the training data never shows, the saved integer model beats the bigram table by 475 and 497 rows at its two seeds, against a required margin of 81. M1's close grade now needs all six bars, and it reads `CLOSED_PASS`.

