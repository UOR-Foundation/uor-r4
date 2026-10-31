# REPORT: m7-studio-live-model-sources, deepseek-6 (Studio v2 polish)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md).

## What did not work, first

- The merged Studio v2 (#3448) passed functional checks and the exact-head review, but a fresh-context UX acceptance scoring returned REJECT-AND-FIX: the page duplicated status labels (ENGINE Engine:, MODEL Model:, CONTEXT Context:, PRIVACY Privacy:, STORAGE Storage:), hid PRIVACY/STORAGE and truncated the workspace selector on mobile, rendered assistant prose in 12px mono, used a wide type scale with sub-10px faint text, repeated M1 copy and showed "M1 / M1 model", shipped duplicate id attributes from the compatibility block, had no focus trap in Settings/Help, and its skip link targeted the composer (hidden outside Agent).
- The first polish attempt left the storage value and privacy group display:none because older, more specific media rules won; fixed with explicit final overrides and re-measured.
- The Models page duplicated every card when the catalog re-rendered: the regroup moved new source groups into section bodies that still held the previous ones. Fixed by clearing section bodies only when fresh groups exist; re-verified one card per source.
- A long prompt against the bundled 64-token demo surfaced a raw "ContextFull { context: 64 }"; the UI now explains the 64-token limit and suggests a shorter prompt or M1/API, and the shipped screenshots use short prompts/seed states.

## What changed

- Status bar: plain value text (no label duplication), PRIVACY and STORAGE always visible, two-row labeled mobile grid (WORKSPACE/ENGINE then PRIVACY/STORAGE) with values not clipped, workspace selector no longer truncated.
- Typography/contrast: assistant transcript is sans 13.5px/1.6, user 13px; --text-faint raised to #a3948a dark / #7a6753 light, light-theme inline links #7a3410, labels/status raised to >=10-11px; body font scale tightened.
- Models: one intro line, one M1 note, PRIMARY/EVIDENCE/RESEARCH badges, labeled metadata pairs (Format/Size/SHA-256/Tokenizer), no duplicate cards, unchanged local/API behavior.
- Accessibility: duplicate id attributes removed while the full 180-id contract from main is preserved (compat buttons restored where they were the only carrier of a base id), Settings/Help lock modal focus (focus on open, Tab cycles, focus returns on close), skip link targets #mainColumn, working Models status mirror stays visible.
- Final consistency pass: the legacy predictor toggle no longer renames the page title to Chat (it stays Agent); desktop status values are short enough not to ellipsize (offline sentence moved out of the bar, model shown as its short label); Help lists the implemented F2 rename key; the legacy storage handler writes a bare value instead of prefixing Storage:.
- No engine/worker/studio_api/crates/verifier edits; no external requests/assets added; the 100 base ids, markers, data-studio-ready and the four verifier contracts remain intact.

## Measurements

- Lead headless checks: duplicateIds []; settings focus inside dialog true, Tab wrap true, focus returned true; transcript family system sans, size 13.5px, line-height 21.6px; model badges include PRIMARY/API; mobile innerWidth=scrollWidth=390, status bar 43px tall (two rows), PRIVACY+STORAGE visible, workspace selector "uor-r4 / main" not clipped.
- Fresh screenshots (local/lead/ui-v2/shots/final-*.png): agent, agent-light, palette, templates menu, help dialog, settings, sessions menu, workspace, models, mobile, mobile drawer.

## Checks run at the head

- verify_studio_page -> PAGE-LOAD CHECKS PASSED; verify_studio_page_load -> RUNTIME PAGE-LOAD TRAP PASSED (external []); verify_studio_worker -> ALL STUDIO WORKER CHECKS PASSED (remoteRequests 0); verify_desktop_client -> ALL OFFLINE STUDIO PAGE & DESKTOP CLIENT CHECKS PASSED (offlineExternalUrls 0).
- Lead browser checks listed above; page-load external [] preserved.
- pr-check --local PASS; git diff --check, claim-wording, roadmap render --check pass.

## Skills used

working-discipline, scope-guard, solution-integrity; ux-critique and gate score local/lead/ui-v2/.

## Still open

- File System Access write-back remains Chromium-only (documented fallback); no committed browser harness yet.
- The synthetic 64-token demo is not a language model; the assistant can still emit token ids. The M1 trunk is next-token prediction only; API mode is another lab's work.
- Mobile hides MODEL and CONTEXT groups in the status bar to keep two readable rows; the values remain in the Agent header and composer context line.
- V4 design debt (accepted by the Lead): the CSS still carries an inherited ~20-value type ladder (about 8-10 rendered sizes). This pass fixes the reading surfaces (assistant 13.5px sans, user 13px), labels and contrast, but does not consolidate every remaining size; the UX gate allows one visual follow-up when no hard gate fails, and the full ladder consolidation is recorded as the next Studio design task rather than silently claimed done.

## Plain summary

The Studio now behaves and reads like a real AI workspace rather than a themed demo: one labeled sidebar with Agent/Workspace/Models/Guide and Sessions, a working workspace with tabs and save, agent context attachments/templates/message actions, organized model sections, a consistent two-theme design with readable type and accessible dialogs, a grouped command palette, and a compact two-row labeled mobile status bar. Production engine/API code and every verifier contract are untouched.
