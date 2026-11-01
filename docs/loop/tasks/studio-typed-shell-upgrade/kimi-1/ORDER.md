# ORDER: implement the Studio typed lifecycle foundation, kimi-1

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `070e35b59807753ec33d439b7978656c9b464a2b`. Owner: "proceed to next" after approving the rebased/rescoped design.

## Goal

Implement only the design's foundation: a typed lifecycle store, narrow model/persistence adapters, fixture-backed migration/tests, and additive model-source progress/operation hooks. Deliver a tested static library build without mounting or replacing the current page.

## Scope

- `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts`
- `web/studio/src/adapters/modelSourceAdapter.ts` and `modelSourceAdapter.test.ts`
- `web/studio/src/persistence/migration.ts` and `migration.test.ts`
- `web/studio/src/main.ts`
- `web/studio/assets/js/model_sources.js` and `model_sources.test.mjs`
- `web/studio/package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`
- `docs/superpowers/plans/2026-11-01-studio-foundation.md`
- `docs/roadmap/items/studio-typed-shell-upgrade.md` and canonical roadmap sync/render outputs
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-1/**`
- Gitignored `node_modules/`, `dist/`, and the plan ledger under `.superpowers/sdd/`

No component migration, `index.html`/`index.css` change, `legacy.html`, verifier change, publisher, real browser-data read/write, model, engine, worker, tokenizer, `studio_api.js`, `.github/**`, Claude-owned, or deployment change. Dependencies are limited to the approved Preact/Vite/TypeScript/Vitest/jsdom classes, pinned in the lockfile.

## Cancel probe

Current `model_sources.js` has real internal byte progress but no public progress channel, and superseded loads can change selected state. Only additive structured progress events and optional operation metadata may be added; existing callers must retain their behavior. Stop and re-scope if another engine/worker/API implementation change, a second credential store, DOM scraping, or a duplicated downloader is needed.

## Done when

- One typed store separates selected source, resident artifact, load progress/finalization, API selection/testing, file state, and per-generation source/privacy; stale events cannot make an older source ready or runnable.
- Structured progress reports real bytes/totals only, unknown totals never show percentages, finalization is indeterminate, and operation metadata suppresses stale selection. Existing calls without metadata behave as before.
- Adapter tests prove existing model-source/worker/API calls are used, not reimplemented. Migration tests cover every design key, malformed/unknown fields, idempotence, rollback preservation, and no second secret store or credential logging.
- `npm test`, `npm run typecheck`, `npm run build`, `node web/studio/assets/js/model_sources.test.mjs`, existing static page checks, full repository gates, and local policy pass at the exact head.
- Fresh adversarial review approves protected delivery. #3500 remains ready for the next workflow unit; no production publication or cutover is claimed.

## Stop and report

Stop after the foundation merges and fresh-main verification. Do not start core components, publish a preview/live page, or mark #3500 done. If any file outside this foundation scope is needed, stop and re-scope first.
