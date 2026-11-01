# ORDER: repair Studio design cutover scope and handoff, kimi-2

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent used only for merge-base validation). Delivery item: `studio-typed-shell-upgrade` (#3500). Base: `381ae4d98b0a5100d241a268d283629ab9b5fd1f`. Repair of PR #3509 after fresh review at `0e5060513`.

## Finished before this repair

- Rebased and independently reviewed the design; registered #3500; resolved all five lifecycle contract questions; passed 11/11 gates and local policy at `0e5060513`.
- Fresh delivery review accepted those lifecycle decisions but found missing legacy/verifier scope, an unclear ORDER/PR ownership link, and a stale final-head report handoff.

## Goal

Make the designed cutover executable within the roadmap item, make ownership and final-head evidence unambiguous, and retain the earlier source/behavior corrections without starting implementation.

## Ownership and scope

The PR's ORDER must name an item that exists at the merge base. #3500 does not exist there yet, so the completed parent remains the ORDER item solely for this canonical item/scope/design PR. The PR body and public handoff reference only #3500; they do not reopen or auto-reference #3440. Later implementation orders use #3500 after it merges.

- `docs/roadmap/items/studio-typed-shell-upgrade.md`: add `web/studio/legacy.html` and `web/studio/verify_*.mjs` to scope.
- `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`: define the exact legacy/check retarget contract.
- `docs/loop/tasks/m7-studio-live-model-sources/kimi-2/**`
- Canonical roadmap sync/render outputs only

No implementation, npm install, model, engine, worker, tokenizer, API, browser-data, publisher, `.github/**`, or deployment change.

## Cancel probe

Stop if `web/studio/legacy.html` or any existing verifier cannot be added to item scope, or if retargeting would weaken a check. Defaults must remain `index.html`; legacy is a local-only cutover artifact and is not published as a second application.

## Done when

- Item scope includes the local legacy page and all existing Studio verifiers needed for cutover.
- All those verifiers default to `index.html` and accept `UOR_STUDIO_PAGE=legacy.html` at cutover; the runtime page-load trap maps it to `/studio/legacy.html`. New checks use the built `index.html`, with no assertion weakened.
- The ORDER explains merge-base ownership; the PR body references only #3500. The active REPORT has no stale head and names the exact-head PR evidence plus R4G1 and File System Access gaps.
- Full gates, claim/roadmap/render/diff checks, local policy, fresh review, and protected merge pass; #3440 stays closed and #3500 stays ready.

## Stop and report

Stop after the repaired design merges and fresh-main verification. Do not start implementation, publish a preview/live page, or mark #3500 done.
