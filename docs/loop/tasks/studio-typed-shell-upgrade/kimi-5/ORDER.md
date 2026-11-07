# ORDER: repair shared TypeScript composer review findings

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Original base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. Reviewed head: `1e50254cc0ca790545d1353ccc11acc357f9c4ce`, tree `ce4d0b30eb9a0533ca9b097ca9920970c94c7869`, PR #3817.

## Finished steps

- kimi-4 ported and integrated the typed controller, added accessible `/` and `@` interactions, exported the controller, and recorded scoped documentation/evidence in commit `1e50254cc0ca790545d1353ccc11acc357f9c4ce`.
- The initial exact-head suite passed 327 Vitest tests, 112 plain-JS tests, typecheck/build, four legacy checks, local PR/report/claim checks, and all 11 full gates.
- The fresh exact-head reviewer returned CHANGES: normal controlled-draft input/clear callbacks were duplicated and history edits could propagate to the parent; image attachments consumed the text budget; ArrowUp/Home, `/`-Escape, empty-mention, focus-after-selection and unsuccessful-send evidence was incomplete; and the controller omitted the legacy default-options argument.

## Goal

Repair those findings without broadening the typed-component unit. Preserve every kimi-4 behavior and the existing public APIs, lifecycle store, legacy page, engine, model/source/privacy behavior, and controlled draft/paste contract. This remains a development-library unit, not a live-page cutover.

## Cancel probe

The callback duplication follows from both the Preact `onInput` and the controller's native `input` listener calling the same external setter. The image accounting follows from passing every retained attachment to the text reader instead of filtering `kind === 'image'`. Add focused failing tests before the repair. If a proposed fix weakens legacy history/draft semantics, stop rather than changing those semantics.

## Files in scope

- `web/studio/src/adapters/composerInput.ts` and `composerInput.test.ts`.
- `web/studio/src/components/AgentComposer.tsx` and `AgentComposer.test.tsx`.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only to correct counts/evidence.
- This repair folder and the existing implementation plan only if the repair requires an implementation-note correction.

No other paths are authorized. The earlier prohibitions remain: no legacy page/controller, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated issue/roadmap edit.

## Done when

- A normal draft input invokes the existing external callback once. History recall and history edits do not overwrite the retained in-progress draft. Successful send clears once; unsuccessful send preserves the visible draft and does not record a submission.
- The picked-text path excludes retained image attachments from text-budget accounting and has a regression with an over-4-MiB image plus a valid text file.
- `/` and `@` coverage includes ArrowUp/ArrowDown, Home/End, Enter/mouse focus retention, `/` Escape, no-active-file/no-attachment empty state, and no model request during navigation. Existing no-result, ARIA, filtering and duplicate-attachment behavior stays green.
- `createComposerInput()` without options throws the legacy explicit `A composer textarea is required.` error, with a test.
- All focused and complete Studio checks, plain-JS tests, legacy static/runtime checks, claim wording, report/local PR checks, and full gates pass at the new exact head. The PR check's D76 scope advisory for the ORDER-authorized notebook/plan and any branch-claim handling are disclosed rather than silently editing generated/owner files.
- Fresh exact-head review approves before ready/queue. PR continues to use `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and its protected delivery; do not start another UI unit in this session.
