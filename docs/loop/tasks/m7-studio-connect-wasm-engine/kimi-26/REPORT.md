# REPORT: m7-studio-connect-wasm-engine, kimi-26 (PM status flip + residual item)

Lab: kimi. Order: [ORDER.md](ORDER.md). Item #2868. Base: `259f9b9b2`. `References #2868`.

## What did not work, first

- **The item's done-when was met on main but its status and text said otherwise.** PR #2978 (merged `24f9d8ab6`, verified) delivered the M7-criterion-3 evidence — client-side worker, truthful capability reporting, `remoteRequests=0`, `7/7 PASS` WASM audit — while the item still read `status: ready` with stale present-tense claims about the retired router engine. DeepSeek flagged the mismatch on #2028 (2026-10-23T13:12:54Z) and declined to edit the PM-owned file.
- **First render/check failed on section names.** The roadmap format requires literal `## What is missing` and `## Done when` sections; the initial edit had renamed/replaced them. Fixed by restoring both headers with the resolution recorded inside them.
- **The residual was real and would have been orphaned.** The author of #2978 deliberately kept the item `ready` over page-level CDN UI assets (Google Fonts, marked, KaTeX, pdf.js, three.js, Monaco) and the byte-mod stand-in tokenizer. The item's own text always treated those as an unregistered bar, and the registered done-when's terms are all about the model worker path — so the PM call is: done-when met, item flips to `done`, and the residual becomes a new item (`m7-studio-self-host-page-assets`) rather than being lost or silently re-scoped into this one.

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/m7-studio-connect-wasm-engine.md` | `status: done`; stale "still loads the retired router engine" text replaced with a resolution note carrying the #2978 evidence and the residual pointer |
| `docs/roadmap/items/m7-studio-self-host-page-assets.md` | new item: self-host page-level UI assets, bundle the stack-bound tokenizer, extend the network traps to page load |
| `docs/roadmap/README.md` | re-rendered (`roadmap render`; 91 items, done 28) |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- None new: this is a PM record change resting on merged, verified evidence (PR #2978: `remoteRequests=0`, `trapSelfTestHits=6`, WASM audit `7/7 PASS` mul=0/div=0/float=0; `node web/studio/verify_studio_worker.mjs` ALL PASSED). Spot-checked on fresh main: zero `uor_r4_wasm_router` references under `web/`; the worker imports `pkg/uor_r4_integer.js` and calls `loadStack`.

## Checks run at the head

`cargo xtask roadmap render` + `roadmap check` (91 items OK), `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`.

## Skills used

- `scope-guard`: roadmap items, the rendered README and this task folder only; no web code touched, no deploy change (D36 stays).
- `solution-integrity`: the flip rests on the done-when's own terms and merged evidence, not on the item's looser prose.

## Still open

- #2868 stays open (`References`, not `Closes`) until `m7-studio-self-host-page-assets` lands; roadmap sync will create the new item's issue on the next `--apply`.
- GitHub Pages deploy stays disabled (D36).

## Plain summary

- The Studio-to-WASM-engine item is done on its registered terms with merged evidence; the remaining CDN/tokenizer work is now its own roadmap item instead of an implicit blocker.
