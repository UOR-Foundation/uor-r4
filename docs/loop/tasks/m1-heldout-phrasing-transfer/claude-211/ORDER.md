# ORDER: m1-heldout-phrasing-transfer, claude-211

Lab: claude. Item: [m1-heldout-phrasing-transfer](../../../../roadmap/items/m1-heldout-phrasing-transfer.md). Base: `4721b95b7`.

## Goal

Casey, 30 Oct, in the claude chat, asked what to fix first: **"Go at M1 bar (f) now (Recommended)"**. M1 (#2029) stays open until a saved model passes (a) to (f). antigravity-110's saved model passed (a) to (e), so this unit measures it on (f).

The item's done-when reads:

> 1. **The panel** is drawn, stored with its `SHA256SUMS`, and counted by its census against the saved model's TRAIN. It is new-template by claude-202's rule, and its control (a `train,train` panel at the same size) is not.
> 2. **On the GPU, under a prep sheet with a fresh FIT and a pre-run review:** at each of the two seeds, the saved model's integer top-1 beats both the unigram and T2 tables on the panel, with bar (e)'s paired band, `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`. Tokens per second are reported.
> 3. **The result** goes in the notebook, negatives first. On a pass, M1's close is updated with bar (f) beside (a)–(e).

**Also on the same panel, as evidence only:** one-model-end-to-end's (r-drop), the recipe that run's first decision ships (claude-211 there). It is scored by its own unit's tool, and enters no bar here.

## Cancel probe (run first)

Run at 03:55Z on 30 Oct.
- **No other work on this bar.** No PR, branch or commit on `origin/main` names bar (f) or this item, except its own claim (#3419) and the units that added the bar (claude-209, claude-210).
- **No panel for it in the store.** The store holds only claude-204's `inputs/claude/heldout-panels-2026-10-29/`: P-dev (reserved for one-model-end-to-end), P-ret, and the `train,train` control P-train. None was drawn for this bar.
- **#3416** has only this claim.

Result: **go**. The probe checks the setting, so it is an instrument check, not a mechanism result.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-heldout-phrasing-transfer*`
- `docs/labs/m1-heldout-phrasing-transfer/**`
- `docs/loop/tasks/m1-heldout-phrasing-transfer/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

The item's three, as quoted above. Also:
- **Tokens per second lead the REPORT.**
- **The panel enters no other decision.**

## Stop and ask if

Stop and decide (never wait) if any of these holds:
- **The panel is not new-template** by claude-202's rule against the saved model's TRAIN. Draw a fresh one at the next seed, and record why.
- **The saved artifacts are missing from the store.** Rebuild them from their recipe, only if that is cheap; else record the gap and decide.
- **A fault is in the instrument, not the model.** Classify it before calling any result (CHARTER §4.1). Repair the instrument, and never count it as a negative.
