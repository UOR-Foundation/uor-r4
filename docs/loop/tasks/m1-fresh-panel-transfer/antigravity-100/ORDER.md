# ORDER: m1-fresh-panel-transfer, antigravity-100

Lab: antigravity. Item: [m1-fresh-panel-transfer](../../../../roadmap/items/m1-fresh-panel-transfer.md). Base: `87024eca8`.

## Goal

The item's done-when reads:

> '(e) Fresh transfer. The gain holds on a fresh, sealed test panel registered in advance.' Read with bar (d), "the gain" is integer top-1 through the D11 export beating the unigram table (1,226 of 14,287) and the next-word table (6,300 of 14,287) on each of two seeds. The fresh-panel threshold is not yet registered; step 3 registers it.

## Cancel probe (run first)

- **Probe executed (`2026-10-26`, classed `instrument` — status-hold gate alignment so the sealed fresh slice is not prematurely opened while unblocking `loop next`):**
  1. Verified that `antigravity-99` (`PR #3292`, merged `31ddf5908`) completed Steps 1, 2, 3, and 5 of `m1-fresh-panel-transfer` (`#2841`): `refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `build_train_count_tables`, and `d11-scorer fresh-score` in `docs/labs/m1-d11-artifact-scorer/instrument/`, the design note on `#2841` (`#issuecomment-6423580146`), the stream-verified pre-registration sheet `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` (`TEST FITNESS: FIT`, `#issuecomment-6423732020`), `docs/labs/m1-fresh-panel-transfer/README.md`, and the notebook row in `docs/research/notebook/m1-discrete-attention.md`.
  2. Verified Step 4 of `docs/roadmap/items/m1-fresh-panel-transfer.md`: *"Do not run it until `m1-export-gap-and-toks` has produced a saved model that passes the earlier bars on two seeds... Score once. Do not re-draw or re-open the slice; use DEV for any earlier look."* Across all currently exported or evaluated two-seed models (`#2839` Stage T `455`/`506` and `371`/`337`, `#3267` within-cell reranker `2,140`/`1,602`, and `#2844` Stage 3 Trunk `2,988`/`2,989` and `2,227`/`2,226` on `ft/dev`), even the strongest two-seed DEV models beat `Unigram = 1,226` (`+1,763` rows) while sitting below `DEV_T2_TOP1 + 24 = 6,324`.
  3. Because `m1-fresh-panel-transfer.md` remained `status: ready` with `depends_on: [m1-export-gap-and-toks]` (`done`), `uor-harness loop next` repeatedly selects `m1-fresh-panel-transfer` ahead of ready research items (`#2884`). Under CHARTER §5.1 and D68, authored `DECISION.md` to verify refusal across all 5 current candidate model families (`verify_current_exported_candidates_hold_fresh_seal`) and transition `m1-fresh-panel-transfer.md` to `status: held` until a two-seed model passes M1 bars (a)–(d) on `ft/dev`.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/labs/m1-fresh-panel-transfer/**`
- `docs/loop/tasks/m1-fresh-panel-transfer/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

1. A fresh-context `research` subagent approves `DECISION.md` (`D68 DECISION REVIEW: APPROVE`) and logs it on `#2841`.
2. `docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs` and `tests/stage1.rs` implement and test `verify_current_exported_candidates_hold_fresh_seal()`, proving with a failing-first test that all 5 current two-seed model families (`#2839` `t16-frozen-zero` `455`/`506` and `t16-relation` `371`/`337`, `#3267` `WithinCellCodeRerankerV1` `2,140`/`1,602`, `#2844` `Trunk-FZ` `2,988`/`2,989` and `Trunk-Rel` `2,227`/`2,226`) are refused by `validate_fresh_transfer_gate` (`SEALED_FRESH_PANEL_REFUSED`).
3. `docs/roadmap/items/m1-fresh-panel-transfer.md` is set to `status: held` (with `docs/roadmap/README.md` re-rendered) and `REPORT.md` (`References #2841`) passes `loop gates --full` and D66 review.

## Stop and ask if

- Any `m1-d11-artifact-scorer-instrument` unit test or `loop gates --full` check fails: classify `instrument` vs `mechanism` first and decide via D68 without waiting.
