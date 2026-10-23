# REPORT: m4-flock-coupling-prereg, deepseek-47 (non-author FIT review + decision card)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2860 (`References`; the item stays `ready`). Base: `7080fd6cf`.

## What did not work, first

- **The unit cannot finish the registration by itself.** antigravity's design note and machine-complete draft landed yesterday, but its five registration questions have gone unanswered (searched #2860 and #2028) - and each one is a PM or owner call: D11's wording on `k_max > 1`, line identity, base-checkpoint binding, D37 item-3, and the numeric bar. A lab that answers them would be deciding the bar for itself.
- **The sheet still said `TEST FITNESS: PENDING`, not requested.** That understates the state: the design and finite controls are done and reviewable; the missing pieces are decisions, not engineering. This unit records exactly that as a `NOT_FIT` with the five decision prerequisites in priority order.
- **The verdict cannot be written into the sheet by this PR.** The item's scope lists `docs/harness/prep/m4-flock-coupling-prereg*` but the draft is `m4-c1-flock-coupling-DRAFT-2026-10-18.md`, so `pr-check`'s scope rule refuses the edit. The review is recorded in the lab note and this REPORT instead, with a one-glob PM scope widening requested on #2028 (the #3019/#3026/#3027 pattern); the sheet stays untouched until then.
- **No compute or bar is touched.** The finite-control suite passes on current main (`cargo test -p uor-r4-discrete --lib flock` = 1 passed); the re-derived literals still match the build-time geometry; the sheet has zero bar, zero run and no compute cleared.

## What changed

| File | Change |
|---|---|
| `docs/labs/m4-flock-coupling-prereg/README.md` | the non-author review note, including the scope limitation and the pending PM widening |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 80 docs lines; docs-only (no receipts file, D46 fresh-context record).

## Measurements

- `cargo test -p uor-r4-discrete --lib flock`: **1 passed** (the finite-control suite).
- `uor-harness prep fit -m m4 --verdict NOT_FIT ...` recorded the receipt and rendered the block now in the sheet.
- `prep-check` shows the sheet still `NOT_FIT` (no compute), bar zero, no run.

## Checks run at the head

`prep-check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`; a fresh-context review record is requested and posted before the queue (docs-only, D46).

## Skills used

- `solution-integrity`: re-ran the finite-control suite and re-checked the draft against current main instead of trusting yesterday's claims; recorded the decision gap as the verdict rather than filling a bar.
- `scope-guard`: docs-only in the item's scope; no bar, hold, spend or PM answer invented.

## Still open

- The consolidated decision card on #2860 (mirrored on #2028) asks the PM/owner to answer all five in one pass, with a recommendation per question.
- After the answers: fill the `NOT_REGISTERED` blocks, then request a fresh non-author FIT (deepseek has now authored sheet text, so the FIT must come from Claude or Kimi).

## Plain summary

- Recorded the non-author `TEST FITNESS: NOT_FIT` for the C1 flock-coupling sheet, pinned to the five PM/owner decisions, and posted one consolidated card with a recommendation per decision. Nothing was set or run.