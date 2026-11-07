# ORDER: studio-typed-shell-upgrade, kimi-52

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md). Base: `ab4a5b328`.

## Goal

The item's done-when reads:

> - A foundation PR adds the typed lifecycle store, narrow engine/persistence adapters, and fixture-backed tests without changing the live page, engine/worker/tokenizer/`studio_api.js`, publisher, or real browser data. Narrow, tested model-source/provider and live-page bridge corrections are permitted.
> - Later workflow PRs migrate the shell and every current feature with a parity record for each, preserving those same implementation boundaries.
> - Source selection, artifact readiness, download bytes, finalization, file dirty/save state, session generation, and endpoint privacy derive from one store, including stale-load rejection and disabled empty-state actions.
> - One-time copy-forward migration preserves old keys byte-identically, except the live `modelSources.v1` compatibility key, which is never copied into a second secret store and may keep changing under its existing writer. Migration tolerates unknown fields, never logs credentials, and passes tests before any write or marker.
> - Legacy and new checks pass at their respective pages, including worker parity, offline requests, keyboard/focus, mobile, and accessibility. Full feature parity and a fresh exact-head review precede cutover; production publication is a separate owner decision.
> - Preact/Vite/TypeScript and test dependencies are pinned; generated output, node_modules, model artifacts, credentials, and `.github/**` are not committed or edited.

## Cancel probe (run first)

To fill in.

## Files in scope

- `web/studio/src/**`
- `web/studio/assets/js/model_sources.js`
- `web/studio/assets/js/model_sources.d.ts`
- `web/studio/assets/js/model_sources.test.mjs`
- `web/studio/assets/js/studio_files.js`
- `web/studio/assets/js/studio_ui.js`
- `web/studio/assets/js/studio_ui.d.ts`
- `web/studio/package.json`
- `web/studio/package-lock.json`
- `web/studio/tsconfig*.json`
- `web/studio/vite.config.*`
- `web/studio/index.html`
- `web/studio/legacy.html`
- `web/studio/verify_*.mjs`
- `web/studio/index.css`
- `web/studio/README.md`
- `docs/products/web-client.md`
- `scripts/stage-studio-site.sh`
- `scripts/studio/**`
- `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`
- `docs/superpowers/plans/2026-11-01-studio-foundation.md`
- `docs/superpowers/plans/2026-11-02-studio-core-workflows.md`
- `docs/superpowers/specs/2026-11-02-studio-ide-experience-design.md`
- `docs/superpowers/plans/2026-11-02-studio-ide-experience.md`
- `docs/loop/tasks/studio-typed-shell-upgrade/**`
- `docs/roadmap/items/studio-typed-shell-upgrade.md`
- `docs/roadmap/issues.json`

## Done when

To fill in.

## Stop and ask if

To fill in.
