# ORDER: infra-loop-harness, deepseek-74 (record two roadmap items)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `0ca82f3ff`. Owner direction, 2026-10-30 ~15:53Z: "record Yassine's M6 verifier as an item now".

## Goal

Record two gaps as roadmap items, using the registered tools only:

1. **Yassine's M6 verifier** (`m6-code-task-rustc-verifier`): the frozen M6 code-task bar and registry name a "rustc --test verifier" that no code runs. The item states plainly that the verifier would check each frozen row's canonical/reference program and the transpiler, not model-emitted code.
2. **The outsider-repair umbrella** (`infra-outsider-repairs`): one maintenance item scoped to the paths in @sharmis-tp's PRs #3424/#3425/#3426, so a registered lab can adopt them with an ORDER under CHARTER §5.5. The owner directed on 30 Oct 2026 that harmless outsider contributions should be merged ("otherwise I do not mind the contributions"), recorded verbatim in the #2028 owner log; that direction stands in place of D77's bar for this one item, as the 29 Oct documentation direction does for `docs-status-refresh`.

This PR adds the two item files, creates their issues through `roadmap sync --apply`, and publishes the rendered page. It does not implement the verifier and changes no bar, panel, row, oracle, seal, status or hold.

## Cancel probe (run first)

- No `docs/roadmap/items/m6-code-task-rustc-verifier.md` or `docs/roadmap/items/infra-outsider-repairs.md` exists at the base; no open issue carries either marker.
- The frozen bar/registry claim and the missing verifier are as the audit recorded (grep `crates/uor-r4-harness` for `--test` / test-count parsing: none; `to_rust_source` only substring-checked).
- None of the three PRs' paths is an owner path, a rendered view or a pinned hash.

If any fails, stop and report.

## Files in scope

- `docs/roadmap/items/m6-code-task-rustc-verifier.md` (new, hand-written)
- `docs/roadmap/items/infra-outsider-repairs.md` (new, hand-written)
- `docs/roadmap/issues.json` (written only by `roadmap sync --apply`)
- `docs/roadmap/README.md` (written only by `roadmap render` / `sync --apply`)
- `docs/loop/tasks/infra-loop-harness/deepseek-74/**`

## Done when

- Both items exist with valid frontmatter; `roadmap check` reports 114 items OK; `roadmap render --check` is clean.
- `roadmap sync --apply` created the umbrella issue and adopted the already-created M6 issue (#3432) by its marker; the dry run before it showed exactly one create and no unexpected update; `issues.json` maps both ids.
- No bar, panel, mechanism, model or status change; no hand edit of `issues.json` or `README.md`.
- REPORT records the exact commands, counts, the sync's side effects (the first apply created #3432 and refreshed #3416 plus the M1/M6 milestone children lists; the second created #3433) and the limits.

## Stop and ask if

A file outside scope is needed; the sync plan shows an unexpected create, update or close; an owner path or a bar/status/hold change would be needed; or the umbrella item needs a decision row rather than an item file.
