# ORDER: m7-studio-live-model-sources, deepseek-6 (Studio v2 polish from the UX gate)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: d2f0354a9 (PR #3448 merged). A fresh-context UX acceptance scoring of the merged Studio v2 returned REJECT-AND-FIX: functionality is workable, but mobile status and desktop status labels, mono/contrast/type-scale, Models copy and small a11y debt are not yet "not janky".

## Goal

Fix the gate's five ranked defects without changing the engine/API layer or the accepted IA: (1) mobile 390px status bar - two labeled rows, no missing PRIVACY/STORAGE, no truncated workspace selector; (2) desktop status - remove duplicated label+value prefixes ("ENGINE Engine:", "MODEL Model:", "CONTEXT Context:", "PRIVACY Privacy:", "STORAGE Storage:"); (3) typography/contrast - assistant transcript prose in sans (not 12px mono), reduce the rendered type-scale spread, make all small text and links AA in both themes; (4) Models page copy - one intro, no repeated M1 disclaimers, labeled metadata instead of a dot-soup, badge not "M1 / M1 model"; (5) accessibility debt - no duplicate id attributes, keyboard focus trap + focus return for Settings/Help dialogs, a skip link that targets a visible destination.

## Cancel probe

- The gate report is local/lead/ui-v2/gate-score.md (head 9da2dbf91); it is advisory evidence, not a rule. If a fix would break the 100-id compat contract, the four verifiers or the offline page-load contract, keep the contract and record the residual instead.
- Studio-only: no engine/worker/studio_api/crates/verifier edits; no new external requests or assets.

## Files in scope

web/studio/index.html, web/studio/index.css, web/studio/assets/js/studio_ui.js, web/studio/assets/js/model_sources.js (copy/metadata only), docs/loop/tasks/m7-studio-live-model-sources/deepseek-6/**, docs/roadmap/items/m7-studio-live-model-sources.md, local/lead/ui-v2/** (screenshots/reports, not committed).

## Done when

- At 390x844: status bar is at most two labeled rows, shows WORKSPACE/ENGINE/MODEL/PRIVACY/STORAGE or equivalent labels with values, the workspace selector reads fully (no "uor-r4 / ..." truncation), scrollWidth == innerWidth == 390 and no element beyond 392px; at 1440x900 status labels are not duplicated with the value text.
- Assistant transcript prose renders in the sans face at a readable size (>=13px, line-height >=1.5); the rendered type scale is demonstrably tighter; --text-faint and light-theme links meet AA at their rendered sizes in both themes.
- Models page: one intro line, one M1 honest note, labels "PRIMARY"/"EVIDENCE"/"RESEARCH", metadata shown as labeled pairs (Format/Size/SHA-256/Tokenizer) not a "·" soup; no clipped cards.
- Duplicate id attributes are gone; Settings and Help trap Tab focus, focus the dialog on open, return focus on close; the skip link targets a visible destination; existing ids/handlers, markers, data-studio-ready, hidden compat controls and the four verifiers remain intact (assertions untouched) and page-load makes zero external requests.
- Fresh screenshots at the fix head for agent dark/light, palette, workspace, models, guide, settings, mobile (closed + drawer), sessions menu open, Build templates menu open, Help dialog; Lead E2E rerun; REPORT with negatives first.

## Stop and ask if

A fix would require weakening a verifier, dropping a pre-existing id, adding a server or an external request, or changing API/engine code (do not).
