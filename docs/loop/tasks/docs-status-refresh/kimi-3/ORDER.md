# ORDER: repair reopen and active-REPORT delivery defects

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Parent repairs: [kimi-1/ORDER.md](../kimi-1/ORDER.md), [kimi-2/ORDER.md](../kimi-2/ORDER.md). Repair of exact head `85edaa0f867c35e06c0af89a24811d4be5a04dc4`.

## Findings to repair

1. `roadmap sync` does not reopen closed issues: after the scope PR merges, run `gh issue reopen 3380` manually, then sync. Correct every packet statement.
2. The active ORDER needs its own `REPORT.md` and the exact heading `Checks run at the head`. Move the report under this ORDER and run `uor-harness loop report --check` plus `uor-harness loop pr-check --local` before any further review.
3. Claim-wording covers only its documented Markdown surface, not the whole repository; state that precisely and add a changed-public-prose scan for the content unit.
4. DECISION's cost statement must distinguish the reversible item/PLAN workflow change from model, bar, evidence and production state, which are untouched.
5. The content ORDER must carry an `Owner approval` section with the owner-approved plan answer and timestamp before any owner-path edit.
6. Report actual diff size/ratio and limit the prose claim to public-facing status prose.

## Scope

Only `docs/roadmap/items/docs-status-refresh.md`, `PLAN.md` owner-direction rows, and `docs/loop/tasks/docs-status-refresh/kimi-1/**`, `docs/loop/tasks/docs-status-refresh/kimi-2/**`, `docs/loop/tasks/docs-status-refresh/kimi-3/**`.

## Done-check

All second-review findings have named repairs; the active REPORT is at this folder with the required heading; `loop report --check` and `loop pr-check --local` run and their actual results are recorded; full supported-surface claim wording and roadmap checks pass; a new fresh exact-head review is required before ready/queue. No public status-page, owner-path, figure, web, model or rendered-view edit belongs to this repair.
