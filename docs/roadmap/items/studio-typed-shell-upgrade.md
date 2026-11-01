---
id: studio-typed-shell-upgrade
title: Migrate the Studio to a typed lifecycle shell without changing its engine
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [m7-studio-live-model-sources]
done_class: host
scope: [web/studio/src/**, web/studio/assets/js/model_sources.js, web/studio/package.json, web/studio/package-lock.json, web/studio/tsconfig*.json, web/studio/vite.config.*, web/studio/index.html, web/studio/legacy.html, web/studio/verify_*.mjs, web/studio/index.css, web/studio/README.md, docs/products/web-client.md, scripts/stage-studio-site.sh, scripts/studio/**, docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md, docs/loop/tasks/studio-typed-shell-upgrade/**, docs/roadmap/items/studio-typed-shell-upgrade.md, docs/roadmap/issues.json]
---

## Why it matters

DeepSeek's v2 Studio is published and its engine/model-source/API layer is preserved, but application, model, file, session, and privacy state still have separate owners and multiple DOM status mirrors. The owner approved a later Preact + Vite + TypeScript shell to make those states coherent by construction. This is a new scope, not a reopening of the completed plain-JS live-model-sources item.

## What is missing

There is no typed shell or implementation yet; only a rebased design exists. The current source remains 11,068 lines across HTML, CSS, and JavaScript with no component build system. The migration must preserve every current feature and browser-data key, use real download-byte progress without inventing later load stages, and leave the live page untouched until all legacy/new checks pass.

## Done when

- A foundation PR adds the typed lifecycle store, narrow engine/persistence adapters, and fixture-backed tests without changing the live page, engine/worker/tokenizer/API implementations, publisher, or real browser data. Only the design's additive model-source progress and operation hooks are permitted.
- Later workflow PRs migrate the shell and every current feature with a parity record for each, preserving those same implementation boundaries.
- Source selection, artifact readiness, download bytes, finalization, file dirty/save state, session generation, and endpoint privacy derive from one store, including stale-load rejection and disabled empty-state actions.
- One-time copy-forward migration preserves old keys byte-identically, except the live `modelSources.v1` compatibility key, which is never copied into a second secret store and may keep changing under its existing writer. Migration tolerates unknown fields, never logs credentials, and passes tests before any write or marker.
- Legacy and new checks pass at their respective pages, including worker parity, offline requests, keyboard/focus, mobile, and accessibility. Full feature parity and a fresh exact-head review precede cutover; production publication is a separate owner decision.
- Preact/Vite/TypeScript and test dependencies are pinned; generated output, node_modules, model artifacts, credentials, and `.github/**` are not committed or edited.

## How to start

Read the rebased/rescoped design at `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`. Begin only with the foundation unit; do not replace `index.html`, invoke the publisher, install dependencies for design review alone, or migrate real browser data.

## Rules that apply

- **Preservation:** no model, engine, worker, tokenizer, API, Claude-owned, generated-binary, or production-deployment changes.
- **Evidence:** no assistant/model capability, browser parity, accessibility, or offline claim without the corresponding executed check; pre-existing failures stay disclosed.
- **Scope:** full cutover is not part of the foundation PR, and this item does not alter the completed live-model-sources item's historical plain-JS constraint.

## Sources

- #3440 and `web/studio/README.md`
- `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`
- `web/studio/assets/js/model_sources.js`, `studio_files.js`, `studio_ui.js`, and `index.html`
