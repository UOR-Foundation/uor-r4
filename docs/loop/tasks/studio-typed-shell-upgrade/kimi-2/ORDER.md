# ORDER: migrate the Studio core workflows to typed components

Lab: kimi. Item: `studio-typed-shell-upgrade` (#3500), continuing the approved phased design after merged foundation PR #3520. Base: `454a90031e500af8e844ba96c94799c4715509d6`.

## Goal

Implement the approved AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings components against the merged lifecycle store and existing public model/file/API/session/UI boundaries. Add only the narrow editor, generation, and settings hooks those components need. Preserve the legacy application and its engine behavior.

## Cancel probe

`web/studio/src/main.ts` currently exports only the lifecycle, model-source, and migration foundation; there are no components or subscriptions. The legacy file editor and page have no typed content/generation completion interface, so a component that scraped the DOM or changed engine code would violate the design. Existing public file, API, model-source, session, and settings methods are present and must remain the implementation.

## Files in scope

- `web/studio/src/**`, `web/studio/package.json`, `web/studio/package-lock.json`, `web/studio/tsconfig.json`, and `web/studio/vite.config.ts`.
- Narrow additive hooks and their tests in `web/studio/assets/js/model_sources.js`, `model_sources.d.ts`, `studio_files.js`, `studio_ui.js`, and `web/studio/index.html`.
- Existing `web/studio/verify_*.mjs`, `web/studio/README.md`, this task folder, the implementation plan, and the owning roadmap item/generated issue data.

## Done when

- All eight named components render from lifecycle state, use native progress/dialog semantics, and expose accessible names and truthful enabled/disabled states in behavioral tests.
- The store notifies component subscribers of external adapter completion without changing existing transition guarantees.
- Typed adapters call existing public APIs; additive editor/generation/settings hooks preserve legacy behavior and never scrape DOM or duplicate engine, tokenization, hashing, persistence, or API logic.
- Model, workspace, agent, session, inspector, guide, and settings behavior is covered with synthetic fixtures, including negative cases and explicit remaining parity records.
- TypeScript, the production Vite build, all Vitest/Node tests, legacy static checks, repository gates, claim wording, local policy, and fresh exact-head review pass. The PR uses `References #3500`, which remains ready for parity/migration and cutover units.

## Stop and ask if

Any change would alter the engine, worker, tokenizer, `studio_api.js`, model artifacts, acceptance bars, real browser-data migration, publisher, deployment, `.github/**`, Claude's active work, or owner-controlled paths. Do not replace/cut over the live page, weaken a verifier, add a runtime external dependency, or claim real-browser parity/accessibility without the corresponding executed check.
