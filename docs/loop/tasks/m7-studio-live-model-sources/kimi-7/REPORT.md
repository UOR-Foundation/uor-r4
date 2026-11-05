# REPORT: register LexLean Studio repairs, kimi-7

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md). References #3440 and #3704.

## What did not work

- The combined implementation PR's local policy showed that a new roadmap item cannot serve as its own merge-base item. This separate registration PR addresses that root cause without rewriting the implementation or weakening the gate.
- No implementation, publication, model change or issue completion is delivered here.
- Generated `docs/roadmap/issues.json` and `docs/roadmap/README.md` are not changed. The previously identified legacy `infra-desktop-client-tauri` status/mapping mismatch blocks canonical sync; this PR does not edit that unrelated record or pretend canonical sync passed.
- An initial local draft used the already occupied `kimi-6` task folder. It was restored before any push, and this `kimi-7` record is the only task history in the final diff.

## What changed

Registered `m7-lexlean-studio-tools` as `ready`, depending on the completed Studio source and desktop-shell items. It records the measured LexLean rejection, strict exact-arithmetic behavior, identity errors, two-client evidence, protected delivery and explicit non-capabilities. No other project file changes.

## Measurements

None. This is a registration unit; the preserved implementation commit separately contains the browser, native and suite evidence and is not part of this PR.

## Checks run at the head

- `git diff --check origin/main...HEAD`
- `cargo run -q --locked -p xtask -- roadmap check` and `roadmap render --check`, with the actual legacy result reported rather than bypassed.
- `UOR_LAB=kimi uor-harness loop gates --full`
- `UOR_LAB=kimi uor-harness loop report --check --order docs/loop/tasks/m7-studio-live-model-sources/kimi-7/ORDER.md`
- `UOR_LAB=kimi uor-harness loop pr-check --local --base d416181c16923748991a41e2206f5c9ed1de572e`
- Fresh exact-head review and required GitHub checks before protected queueing; results are posted on the PR.

## Skills used

Project research/delivery workflow, isolated worktrees, scope discipline, systematic debugging of the merge-base failure, and fresh exact-head review. No model compute, paid resource, new dependency or publication was used.

## Still open

Merge this registration, rebase the preserved implementation commit onto the new base, obtain its own exact-head review, then complete protected merge, fresh-main and Pages verification. Close #3704 only when all of its acceptance rows are satisfied.

## Plain summary

This PR only registers the LexLean Studio repair item so the implementation can pass the repository's merge-base policy. It changes no runtime behavior and publishes nothing.
