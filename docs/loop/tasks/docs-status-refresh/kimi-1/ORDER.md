# ORDER: reopen and rescope docs-status-refresh for 30 Oct 2026

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Base: `0ca82f3ff19f24941fea344eedd7477046d8a2d7`.

## Goal and chain

The owner assigned Kimi the 30 Oct documentation-housekeeping pass. M1 #2029 is now closed, and the living public pages, figures, generated milestone inputs and Studio status still disagree with it and with one another. This item/scope PR proposes reopening #3380 and names the complete living surface before the content PR; it changes no public result, acceptance bar or technical rule.

## Scope

- `docs/roadmap/items/docs-status-refresh.md`
- `PLAN.md` owner-direction rows only
- `docs/loop/tasks/docs-status-refresh/kimi-1/**` and any numbered repair ORDER beside it

Estimated: five files plus any repair ORDER, about 180 lines; size is reported, not capped (D48).

## Cancel probe

Run first: #2029 must be CLOSED; the older close-grade JSON must still record bars (a)–(e) as `CLOSED_PASS`; claude-211's separate `panel-score.json` must record `bar_f_pass: true`; and the current item must still be `done` from the 29 Oct refresh. If the M1 close was reversed, stop. If the scope is already current, close this order as no-new-information.

## Done-check

The item is `ready`, its 30 Oct scope includes every living path required for M1/current-status reconciliation, and its exclusions keep dated evidence, antigravity-held pages and Claude's active files untouched. PLAN marks the 29 Oct refresh done and records the owner's new Kimi direction verbatim. `cargo run -q --locked -p xtask -- roadmap check`, the full claim-wording check and focused content checks pass; REPORT and a fresh exact-head review are posted. After merge, Kimi manually reopens #3380, then runs roadmap sync from fresh main; the separate render flow refreshes generated views. `roadmap sync` itself does not reopen closed issues.

## Stop list

The item and PLAN owner-direction rows are the only public prose in this PR. No public status-page, figure, web, model, workflow, owner-path or rendered-view edits. No archive, sealed evidence, notebook rows, prior task records, `docs/integration/current-state.md`, GPU/model compute, credentials, direct main push or force push. Do not edit Claude's active research files or antigravity-113's two mechanism pages. The owner-approved plan explicitly authorizes later status-only owner-path recording, but no such file changes in this scope PR. If any other file is needed, STOP and re-scope before editing it.
