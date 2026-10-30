# REPORT: infra-outsider-repairs, deepseek-76 (#3425)

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Order: [ORDER.md](ORDER.md).

Adopts @sharmis-tp's PR #3425; the fork PR is thanked, credited and closed after this merges. Closes #2792.

## What did not work, first

- **The PR body's verification record is wrong at this head, and it is corrected here rather than copied.** The body says `cargo fmt -p xtask -- --check` and `cargo clippy -p xtask --all-targets -- -D warnings` were run as passing checks. At the branch base and at this head, fmt is red on a pre-existing line the PR does not touch (`figures_charts.rs:1283`, the long `rec(Some("2026-10-22"), ...)` row), and clippy is red with two pre-existing errors (`docs_relink.rs:250` `manual_repeat_n`, `book_stage.rs:342` `map_or`). The adoption requires no new failure beyond that measured baseline; it does not claim a green fmt or clippy. The later #3426 repair incidentally makes `xtask` fmt green.
- **The fork route was blocked.** A public-repository fork PR gets a read-only `GITHUB_TOKEN`, so the five required checks cannot be published at its head and the merge queue would not admit it. This repair is re-homed in a same-repo branch with the contributor's commit merged unchanged (`--no-ff`); authorship is preserved and the fork was never written to.
- **The docs checks are advisory in CI since D76**, so a green docs workflow is not proof of this repair; the proof is the host suite below plus the clean `git status` after the docs commands.
- **A residual test gap in the contributor's tests.** The six new unit tests exercise the helper functions (`dest_conflicts`, `page_collisions`, `covered_paths`, `rewrite_line`); they do not call `run()`, so the early-refusal wiring for the stub/catalog guards is not regression-tested end to end. This was checked by removing the wiring and observing the tests still pass; the behaviour itself was verified manually at the head (a conflicting stub list is refused before any write). Recorded as a residual, not fixed here, because the unit is an unchanged-diff adoption.

## What changed

- **Adopted** @sharmis-tp's four-file docs-tool repair unchanged (`+240/−36`): `docs_catalog.rs`, `docs_relink.rs`, `docs_rewrite_refs.rs`, `docs_stubs.rs`.
- **Merge-forward** of `origin/main`: the records PR #3434 and docs PR #3431 first, then the adoption PR #3424 (crates-only changes; no `xtask` conflict). The `xtask` suite was re-run after the second merge-forward: 217 passed.
- **Added:** this ORDER and REPORT in `docs/loop/tasks/infra-outsider-repairs/deepseek-76/`.
- **Not changed:** no docs page, catalog entry, figure, bar, status, §3 rule, owner path, generated output or pinned receipt.
- Size: 4 `xtask/src` files from the PR (unchanged), plus this ORDER and REPORT; no lines of ours in the code.

## Measurements

- `cargo test -p xtask`: `217 passed; 0 failed; 0 ignored` (the base tree has 211; the six new tests are present and pass).
- `cargo run -q --locked -p xtask -- docs-catalog --check`: `catalog: 1175 records, 20 lines, 0 paths not found`.
- Generated output: `git status --short` is empty after the docs commands, so no generated file, catalog page or figure changed.
- fmt: red on `figures_charts.rs:1283` (pre-existing, untouched by this PR); clippy: 2 pre-existing errors as above. No new failure beyond the measured baseline.
- No GPU run, no model evaluation, no panel scoring.

## Checks run at the head

- `cargo test --locked -p xtask` -> 217 passed, 0 failed.
- `cargo run -q --locked -p xtask -- docs-catalog --check` -> 1175 records / 20 lines / 0 not found.
- `cargo fmt -p xtask -- --check` -> red on the pre-existing `figures_charts.rs:1283` line (recorded, not a regression).
- `cargo clippy --locked -p xtask --all-targets -- -D warnings` -> 2 pre-existing errors (`docs_relink.rs:250`, `book_stage.rs:342`); no new error.
- `git status --short` after the docs commands -> empty.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` -> pass.
- `uor-harness loop pr-check --pr <this PR>` and the five required checks carry `policy: pass (run <id>)`; the exact-head review record is posted at the head.

## Skills used

- `scope-guard` for the unchanged-diff adoption and merge-forward.
- `solution-integrity` for the "no output change" proof and the honest baseline record.

## Still open

- **Closes #2792** on merge.
- **#3426 is the next and finishing adoption** (deepseek-77); it sets item #3433 `done`.
- **Residual:** the `run()` refusal wiring has no integration test (manual verification at the head); the two pre-existing fmt/clippy reds remain on the tree and are not this repair's to fix.

## Plain summary

We adopted the contributor's five-finding docs-tool repair unchanged. The tools now error on unreadable manifests, report rewrite shortfalls, refuse conflicting stubs before writing, keep percent-encoded parentheses, and detect page-name collisions; the generated docs and figures are unchanged. The contributor's own fmt/clippy claim is corrected honestly in this record.
