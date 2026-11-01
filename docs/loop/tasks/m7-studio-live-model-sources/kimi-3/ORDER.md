# ORDER: align Studio design final-head records, kimi-3

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent used only for merge-base validation). Delivery item: `studio-typed-shell-upgrade` (#3500). Base after merge-forward: `03800ffd24c183e806c6690f684495becbe1cf25`. Repair of PR #3509 after fresh review at `7c371d729`.

## Finished before this repair

- Rebased, reviewed, and rescoped the design; resolved lifecycle and cutover contracts; merged `main` forward without changing Studio source; passed all local checks at `7c371d729`.
- Fresh review found no design/scope defect but identified a stale 126-item report, a future-tense check section, and an unqualified old-base reference.

## Goal and scope

Correct only `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md` and `docs/loop/tasks/m7-studio-live-model-sources/kimi-2/REPORT.md`, plus this ORDER. Record the 134-item merged roadmap, exact-head checks and size, the reviewed-at/merge-forward bases, and both known verification gaps. No implementation, generated roadmap, item, dependency, browser data, publisher, or deployment change.

## Cancel probe

Stop if roadmap is not 134 items, if `git diff 381ae4d98..03800ffd2 -- web/studio` is nonempty, or if the PR body does not reference only #3500. All three checks pass before editing.

## Done when

- Active REPORT states 134 items and the actual static/full-gate/policy/claim/roadmap/render/diff results at the final text head, with exact size and no stale head.
- Design says reviewed at `381ae4d98` and merge-forwarded to `03800ffd2`, with Studio source unchanged.
- Fresh exact-head review approves; GitHub checks are green; PR body references only #3500; #3440 stays closed and #3500 stays ready.

## Stop and report

Stop after the record repair merges and fresh-main verification. Do not start implementation or publish a preview/live page.
