# REPORT: m7-studio-self-host-page-assets, deepseek-53 (runtime page-load trap)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #3075. Base: 22720f207. References #3075.

## What did not work, first

- **A static scan is not the done-when's trap.** The item asks for zero external requests on load checked by an extended trap verification; `verify_studio_page.mjs` sees attributes and inline literals only, not requests a script computes at load time. This unit adds the runtime half.
- **No npm dependency was acceptable**, so the harness drives a local headless Chrome over the DevTools protocol using Node's global WebSocket and a built-in static http server. It fails closed when Chrome is missing; the model worker and the static scan remain the dependency-free checks.
- **The demo page is worse at runtime than at rest.** The static scan sees three CDN references; the real page load makes **8 external requests** because the CDNs pull further runtime assets (Tailwind build, Font Awesome webfont, three Google font files). That is the measured justification for the open demo-page decision rather than hand-waving.

## What changed

| File | Change |
|---|---|
| `web/studio/verify_studio_page_load.mjs` | new runtime page-load trap: built-in static server, headless Chrome via CDP, per-page `Network.requestWillBeSent` capture, planted-external self-test, `--strict-router` |
| `web/README.md` | invocation line for the runtime trap |
| `docs/labs/m7-studio-self-host-page-assets/README.md` | runtime trap implemented note with the measured numbers and limits |
| `docs/products/web-client.md` | page-load claim now names both the static check and the runtime result |
| this folder | ORDER and REPORT |

## Measurements

- Trap self-test: a served page with one planted `https://example.invalid/planted.png` -> exactly 1 external request detected (trap fires, not an empty list).
- `index.html`: loaded, **4 requests, 0 external**.
- `geometric_prime_router_webapp.html`: loaded, **8 external requests**: cdn.tailwindcss.com (2), cdnjs Font Awesome css + webfont, fonts.googleapis.com css, fonts.gstatic.com Inter + 2 Fira Code files. Non-strict reports them as the legacy residual; `--strict-router` exits 1 with `verdict: RUNTIME PAGE-LOAD TRAP FAILED`.
- Runs in a few seconds per page on the laptop; no GPU, no network egress required for the check itself.

## Checks run at the head

`node web/studio/verify_studio_page_load.mjs` (pass, 0 external on index.html), `--strict-router` (expected fail on the demo page), `node web/studio/verify_studio_page.mjs` (static companion), `uor-harness check`, `loop report --check`, `loop pr-check --local`; counted non-author record needed (code PR).

## Skills used

- `solution-integrity`: made the trap prove itself with a planted external request before trusting a clean result; measured the demo page's real load instead of asserting three references.
- `scope-guard`: `web/**` + docs only; no npm dependency, no Rust, no Pages change, no bar.

## Still open

- Demo-page decision (PM/owner: retire / legacy-exempt / follow-up rewrite) - now with runtime evidence (8 external requests).
- A real corpus tokenizer remains bound to the stand-in until a real model export exists (M4).

## Plain summary

- Added the runtime half of the page-load claim: headless-Chrome trap, self-tested with a planted external request, proving index.html loads with zero external requests while the router demo page makes eight.