# REPORT: infra-outsider-repairs, deepseek-77 (#3426)

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Order: [ORDER.md](ORDER.md).

Adopts @sharmis-tp's PR #3426; the fork PR is thanked, credited and closed after this merges. Closes #2796 and #3433; this is the item's finishing unit.

## What did not work, first

- **The PR body's verification record is wrong at this head, and it is corrected here.** The body claims a passing `cargo clippy -p xtask --all-targets -- -D warnings`; at this head clippy is red with the same two pre-existing errors as main (`docs_relink.rs:250` `manual_repeat_n`, `book_stage.rs:342` `map_or`). fmt is green, but only because this repair incidentally reformats the pre-existing red `figures_charts.rs:1283` line; the record says so instead of implying main was green.
- **The fork route was blocked.** A public-repository fork PR gets a read-only `GITHUB_TOKEN`, so the five required checks cannot be published at its head and the merge queue would not admit it. This repair is re-homed in a same-repo branch with the contributor's commit merged unchanged (`--no-ff`); authorship is preserved and the fork was never written to.
- **The new `share_prose_holds` guard is time-dependent by design.** It runs before rendering and fails when the fixed share-chart prose no longer matches the append-only catalog. The proof below shows it goes red for a planted drift; it passes on today's catalog. A future catalog change that breaks the prose should be a prose revision, not a silent skip.
- **The exact-text-number rule is verified on macOS only.** The old 0.01 tolerance existed for Linux-versus-macOS rounding in the 600-cell figure. The historical drift was in a coordinate (still tolerated) and was fixed by a vertex-order commit, so this is a residual risk, not a known defect; the docs workflow run at this head is the Linux check for the committed figures.
- **A test gap, inherited**: the unit test for `share_prose_holds` exercises the function, not its call in `run()`; the planted-drift proof below exercises the wiring end to end.

## What changed

- **Adopted** @sharmis-tp's three-file figure/flock/catalog repair unchanged (`+177/−29`): `docs_catalog.rs`, `figures_math.rs`, `figures_charts.rs`.
- **Merge-forward** of `origin/main`: records PR #3434, then adoption #3424 (#3435), then adoption #3425 (#3437); no conflicts, and `docs_catalog.rs` carries both repairs' hunks. The `xtask` suite at this final head is 220 passed (211 base + 6 from #3425 + 3 from this repair).
- **Set item #3433 to `done`** (status-only edit) and this PR says `Closes #3433`.
- **Added:** this ORDER and REPORT in `docs/loop/tasks/infra-outsider-repairs/deepseek-77/`.
- **Not changed:** no committed figure, catalog page, bar, panel, status, §3 rule or owner path.
- Size: 3 `xtask/src` files from the PR (unchanged), the item status line, plus this ORDER and REPORT.

## Measurements

- `cargo test -p xtask`: `214 passed; 0 failed; 0 ignored` at the pre-#3425 head (base 211 + 3 new), and `220 passed; 0 failed; 0 ignored` at this final head once #3425's six tests are merged (211 + 6 + 3). Measured in separate target directories: a shared target directory had wrongly reused another worktree's `xtask` artifact earlier, so the counts were re-taken from clean builds.
- `cargo fmt -p xtask -- --check`: green at this head (this repair makes main's pre-existing fmt red go away).
- `cargo clippy -p xtask --all-targets -- -D warnings`: 2 pre-existing errors, no new error.
- `docs-catalog --check`: `1175 records, 20 lines, 0 paths not found`.
- `figures-math`: `all 5 computed figures are current`; `figures-charts`: `all 4 chart figures are current`; `figures`: `all 8 generated figures are current`; `git status --short` empty afterwards, so no committed output changed.
- **Planted drift:** appending 150 synthetic `positive` rows dated `2026-10-12` to a worktree copy of `archive/catalog.jsonl` makes `figures-charts` exit with `results-share-by-week: the share no longer falls sharply in the busiest week; revise the fixed prose`; the catalog was then restored (git diff empty). The new guard is red for a real data drift and green for today's catalog.
- No GPU run, no model evaluation, no panel scoring.

## Checks run at the head

- `cargo test --locked -p xtask` -> 214 passed, 0 failed (clean target).
- `cargo fmt -p xtask -- --check` -> green.
- `cargo clippy --locked -p xtask --all-targets -- -D warnings` -> the 2 pre-existing errors only.
- `cargo run -q --locked -p xtask -- docs-catalog --check` -> 1175 / 20 / 0.
- `cargo run -q --locked -p xtask -- figures-math`, `figures-charts`, `figures` -> all current.
- `git status --short` after the figure commands -> empty.
- Planted-drift run as above -> the guard's expected error, then restored.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` -> pass.
- `uor-harness loop pr-check --pr <this PR>` and the five required checks carry `policy: pass (run <id>)`; the exact-head review record is posted at the head.

## Skills used

- `scope-guard` for the unchanged-diff adoption and merge-forward.
- `solution-integrity` for the falsifiable-guard proof and the "no committed output change" check.

## Still open

- **Closes #2796 and #3433** on merge; #2792 was closed by #3425; #3168 remains open for its own gate/lint half.
- **Residuals:** the two pre-existing xtask clippy errors are not this repair's; Linux text-number equivalence is covered only by the docs workflow at this head; the `share_prose_holds` call has no unit test of its own wiring (the planted-drift proof covers it here).
- **Post-merge:** comment on the original fork PRs crediting the contributor, close them, and point him at the roadmap `Start here` items.

## Plain summary

We adopted the contributor's figure/flock/catalog hardening unchanged. It narrows an SVG tolerance to attributes, makes tie-breaking deterministic without rounding, fails the share chart if the catalog no longer supports its fixed prose, and rejects malformed catalog dates. The committed figures and catalog are unchanged, the new guard was shown red for a planted drift, and the repair finishes the outsider-repair item.
