---
id: m7-studio-live-model-sources
title: Publish the fixed Studio with working WASM and selectable model sources
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
notebook: docs/research/notebook/m4-serving.md
depends_on: [m7-studio-connect-wasm-engine, m7-studio-self-host-page-assets]
scope: [web/**, docs/products/**, scripts/studio/**, scripts/stage-studio-site.sh, docs/research/notebook/m4-serving.md, docs/loop/tasks/m7-studio-live-model-sources/**, docs/roadmap/items/m7-studio-live-model-sources.md]
done_class: host
---

## Why it matters

The Studio is the project's web client: the M7 milestone requires the model reachable through an API, in the browser (WebAssembly) and in the Studio, with bit-identical tokens. Its source (web/studio) is wired to the integer engine and its page-load harness is clean, but the published page at https://uor-foundation.github.io/uor-r4/studio/ is still the 13 October deploy: it loads external CDNs and the retired router engine, because the docs-site workflow preserves the old studio/ directory and never rebuilds the wasm bundle. The owner asked on 30 October 2026 to fix that page, make it cleaner, and make the wasm actually run a current model, or offer choosing a model by API or adding other APIs. The owner chose: ship the bundled synthetic demo plus a local-model loader (file or URL) and OpenAI-compatible API providers with an add-API form; do not publish the private HF artifacts; publish live directly once the work is reviewed.

This item is the source-repo change and the built-artifact publish. The Pages workflow itself stays owner-only (.github/** is an owner path); the build/publish helper lives under scripts/studio/ and pushes the staged site/studio/ directory to gh-pages or gh-pages-preview, which the existing workflow preserves across future book deploys. The owner is given the workflow patch as a proposal.

## What is missing

- The published /studio/ is the old deploy: CDN loads, old router engine, stale engine-status banner, and no working wasm bundle at studio/pkg/uor_r4_integer*.
- The wasm bundle is not built by any workflow and web/studio/pkg is gitignored; the build recipe exists only in web/README.md and the verification scripts.
- The engine parses uor-r4.lut-stack/1-7 and serves /1-/6 (a /7 turn-memory stack is refused by the serving engine as "does not serve the turn memory yet"); the Studio worker accepts only /1. The source page already has a local UORLUT01 file loader and Model Manager (index.html #nativeModelFile, loadNativeModelFile, the model-manager grid; the worker's load_model), but it is gated to /1 and there is no model URL or remote-API source.
- The page is visually cluttered: ~7k lines with inline CSS, repeated status/provenance blocks, stale "frozen per D36" and "0/8 milestones" claims, and no model-source panel.
- No documented, repeatable publish step agents or the owner can run; the workflow path filter excludes web/** and crates/uor-r4-integer/**.

## Done when

1. **Clean Studio page.** web/studio/index.html and index.css are restructured into a coherent, responsive interface with an honest current-status header (project state, engine state, model source), a clean chat pane and code pane, and no stale D36/0-of-8/provenance clutter. It remains offline-first against other origins: no external or off-origin request at page load (only same-origin bundled assets, including ./models.json, are requested), enforced by web/studio/verify_studio_page.mjs and verify_studio_page_load.mjs, which report external [].
2. **Model sources.** A Model Source panel offers (a) the bundled synthetic demo stack, (b) extend the existing /1-only local sidecar loader to a local .lut/.bin file loader, (c) load a model from a user-provided URL, and (d) an OpenAI-compatible API provider list with add/edit/remove and presets (local uor-r4 API, Ollama, custom), persisted in localStorage, with no keys committed and no automatic external requests. The worker is the only WASM backend and its capability labels are truthful (synthetic vs loaded artifact, unqualified stack, API mode).
3. **Working wasm.** The serving-wasm bundle builds reproducibly with the documented script and the worker accepts the engine's served stack schemas (uor-r4.lut-stack/1-6; /7 parses but the serving engine refuses a turn-memory stack, recorded honestly) plus a user-selected model; verify_studio_worker.mjs and the M7 parity harness still pass on the synthetic fixture and the bundle loads in a browser/headless check.
4. **Live publish.** The built site/studio (page, JS, pkg, synthetic-stack.lut, tokenizer.json, model manifest) is pushed to gh-pages by scripts/studio/build-and-publish.sh, the live https://uor-foundation.github.io/uor-r4/studio/ serves it, and the page passes the page-load traps and loads the demo model. The owner is given the docs-site.yml patch proposal (path filters + build steps) as a report, not applied.
5. **Docs and records.** docs/products/web-client.md, web/README.md and the studio README state what is now built and what is still missing (no public trained model artifact; M4/M8 remain); one row is added to docs/research/notebook/m4-serving.md; and the PR carries ORDER/REPORT with the build/publish commands and evidence. No binary is committed to the source repo, and no owner path, bar, status, pinned receipt, or private HF artifact is touched.

## Cancel if

The owner withdraws the Pages publish direction, the integer wasm engine stops building for wasm32, or the Studio is superseded by a different M7 web client; then this item is re-scoped or closed with the decision recorded.

## How to start

Read web/README.md, web/studio/, docs/products/web-client.md, docs/labs/m7-studio-connect-wasm-engine/README.md and the owner direction on #2028. Build the bundle with RUSTFLAGS='-C target-feature=-simd128' wasm-pack build crates/uor-r4-integer --target web --no-opt --out-dir ../../web/studio/pkg -- --features serving-wasm, generate the seed=5 synthetic stack and tokenizer, run the three node verification scripts, then work through the done-when. The build/publish helper is scripts/studio/build-and-publish.sh (dry-run by default; --publish-gh-pages requires the owner's live-deploy decision).

## Rules that apply

- **Delivery (§5.5):** short branch, exact-head fresh-context review, checks at that head, merge queue, verify on fresh main, then delete the branch and worktree. The built site is published from scripts/studio/build-and-publish.sh, dry-run by default; a live publish needs the owner's live-deploy decision.
- **Owner-only boundaries (§5.1):** .github/** is an owner path; the docs-site.yml patch is proposed to the owner, never landed by a lab. No private HF artifact, key, secret or committed binary/corpus enters the repository (§3.13).
- **Serving (§3.6):** the integer wasm engine on client devices is the product and is allowed; research compute stays GPU-only (§3.5). The page must remain offline-first: zero external requests at load, and any remote API call is a user-chosen model source.
- **Rust first (§3.11):** the engine is Rust compiled to wasm; page helpers are plain JS, no new dependency or framework.
- **Evidence (§4):** report the page-load harness results, the worker/parity tests, the built bundle hashes and the live URL; a green CI is not proof of the live page.

## Sources

- Owner direction and answers, [#2028 owner log comment 6534841207](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6534841207) (30 Oct 2026).
- [CHARTER.md](../../../CHARTER.md) §2, §3.6, §3.13, §5.5; [PLAN.md](../../../PLAN.md).
- [docs/products/web-client.md](../../products/web-client.md), [the M7 Studio WASM record](../../labs/m7-studio-connect-wasm-engine/README.md), [the self-hosted assets record](../../labs/m7-studio-self-host-page-assets/README.md).
- [web/README.md](../../../web/README.md), [web/studio/README.md](../../../web/studio/README.md), [.github/workflows/docs-site.yml](../../../.github/workflows/docs-site.yml).
