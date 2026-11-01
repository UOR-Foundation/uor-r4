# ORDER: rebase, review, and rescope the typed Studio design, kimi-1

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent). New item: `studio-typed-shell-upgrade` (#3500). Base: `381ae4d98b0a5100d241a268d283629ab9b5fd1f`. Owner request: "rebase, review, and rescope this" for the Studio shell upgrade; no implementation requested.

## Goal

Rebase the design onto current `main`, verify its assumptions against the actual v2 source, correct factual drift, and deliver a phased typed-shell design plus a separate ready roadmap item. Preserve DeepSeek's completed item and do not reopen it.

## Scope decision and files

Top-session D76 review accepts only the expected advisories for adding a new item file and its design document outside the completed parent's historical scope. This is a canonical item/scope/design PR, not an implementation-scope waiver or a change to the parent item.

- `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`
- New `docs/roadmap/items/studio-typed-shell-upgrade.md`
- Canonical `docs/roadmap/issues.json` and shared `docs/roadmap/README.md`
- `docs/loop/tasks/m7-studio-live-model-sources/kimi-1/**`

No `web/**` implementation, npm install/dependency lockfile, model, engine, worker, tokenizer, API, Claude-owned, `.github/**`, publisher, deployment, or live-page change.

## Cancel probe

Stop if current Studio source has changed since the original design without being reviewed, if `verify_studio_page.mjs --strict-router` fails, or if the typed direction can be replaced by a smaller completed migration. The probe found byte-identical Studio source and a passing static page check; no implementation or runtime claim follows.

## Done when

- The design preserves the selected Preact + Vite + TypeScript direction or documents evidence for changing it; source selection and loaded artifact remain distinct.
- Real `model_sources.js` byte progress is represented correctly, with unknown totals/finalization never assigned invented percentages or stages and stale loads rejected.
- First implementation is foundation-only: typed store, adapters, fixture-backed migration/tests, and no live-page, publisher, real-data, or engine changes. Full parity and cutover are separate later gates.
- Current features, persistence keys, accessibility/native-dialog/progress behavior, offline/privacy constraints, and verifier requirements are explicit.
- A new ready item is synced without duplicating an issue; roadmap checks, claim wording, full gates, local policy, fresh review, and protected merge complete. #3440 stays closed and the new item stays ready.

## Stop and report

Stop after the rescoped design merges and fresh-main verification. Do not start implementation, publish a preview/live page, or mark the new item done. Any later implementation needs its own ORDER at the merged item base.
