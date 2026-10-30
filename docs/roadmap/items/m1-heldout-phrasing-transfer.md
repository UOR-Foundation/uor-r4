---
id: m1-heldout-phrasing-transfer
title: Pass M1's held-out-phrasing bar (f) on the saved model
milestone: M1
area: evaluation
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-close-saved-model]
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-heldout-phrasing-transfer*, docs/labs/m1-heldout-phrasing-transfer/**, docs/loop/tasks/m1-heldout-phrasing-transfer/**, docs/research/notebook/m1-discrete-attention.md]
---

## Why it matters

The owner added bar (f) to M1 on 29 Oct (D82, CHARTER §4 item 8): "M1 also requires beating the T2 bigram and unigram on a fresh held-out-phrasing panel (drawn by heldout-panel at a new seed, with its own census), with the same paired band as bar (e). It proves the model reads new phrasings, not just recalls templates."

- **Bar (e) passed on Fresh V2** (`m1-fresh-panel-transfer`, antigravity-110): 7,215 and 7,228 of 14,479, against the unigram's 1,279 and T2's 6,582.
- **But Fresh V2 comes from TRAIN's generator.** 0.777 of its replies and 0.639 of its prompts are verbatim TRAIN turns, and 0.665 of its prompt templates occur in TRAIN with values masked (claude-202's census). So it tests new values inside known templates.
- **A held-out-phrasing panel tests new phrasings.** M-world v2's `dev_phrasing × dev_value` cell is that. claude-204 drew P-dev from it with `heldout-panel`: 0.000 of P-dev's prompt templates occur in TRAIN.
- **P-dev is reserved** for one-model-end-to-end's training-run decisions, so bar (f) needs its own panel, drawn the same way at a new seed.

M1 (#2029) stays open until a saved model passes (a) to (f). antigravity-110's saved model passed (a) to (e), so this item measures it on (f) first.

## What is missing

- **A fresh held-out-phrasing panel.**
  - It is drawn by `heldout-panel` (`docs/labs/one-model-end-to-end/instrument/`, claude-204) at a seed no unit has used. claude-204 used 2041–2043.
  - Its cell is `dev,dev`, in dialogue protocol 2, with 400 conversations.
  - It is stored in the HF store with a `SHA256SUMS`.
- **The panel's census, against the saved model's own TRAIN.** The panel must hold new templates by claude-202's rule (value-masked prompt share at K = 1,024 at most 0.5). That is counted against the TRAIN split the saved model was trained on.
  - The M1 sheets pin that split (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:62-65`, sha `3dee858f…`).
  - Its size is in question (claude-202). The census reads the file as it is, and the panel is counted against it.
- **Scoring the saved model on the panel.** That is the integer top-1 of antigravity-110's two-seed `UORLUT01` artifacts, against TRAIN's unigram and T2 tables, on the GPU (CHARTER §3.5). `m1-d11-artifact-scorer`'s `fresh-score` reads only the sealed Fresh V2 today; it needs an unpinned panel, with its sha recorded.

## Done when

1. **The panel** is drawn, stored with its `SHA256SUMS`, and counted by its census against the saved model's TRAIN. It is new-template by claude-202's rule, and its control (a `train,train` panel at the same size) is not.
2. **On the GPU, under a prep sheet with a fresh FIT and a pre-run review:** at each of the two seeds, the saved model's integer top-1 beats both the unigram and T2 tables on the panel, with bar (e)'s paired band, `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`. Tokens per second are reported.
3. **The result** goes in the notebook, negatives first. On a pass, M1's close is updated with bar (f) beside (a)–(e).

## How to start

Read:
- CHARTER §4 item 8 and D82;
- `docs/labs/one-model-end-to-end/README.md` (the panel, and its census);
- `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` (bar (e)'s band and its instrument);
- antigravity-110's REPORT in `docs/loop/tasks/m1-close-saved-model/`.

## Rules that apply

- **Deterministic scores only;** count tables are the bar's controls (CHARTER §3.10, §4.4).
- **Model scoring on the GPU,** counting on the laptop (§3.5).
- **The panel is opened once,** for this bar, and enters no other decision.

## Sources

- [D82](../../integration/DECISIONS.md) and [CHARTER §4](../../../CHARTER.md).
- [claude-202's census](../../labs/one-model-end-to-end/census-2026-10-29.json) and [claude-204's panels](../../labs/one-model-end-to-end/README.md).
- [m1-fresh-panel-transfer](m1-fresh-panel-transfer.md), [m1-close-saved-model](m1-close-saved-model.md).
