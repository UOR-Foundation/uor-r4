# ORDER: infra-outsider-repairs, deepseek-77

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Base: the main commit that adds this item (records PR #3434), plus #3424 and #3425 merged. Adopted from @sharmis-tp's [PR #3426](https://github.com/UOR-Foundation/uor-r4/pull/3426), fork branch `xtask-figures-hardening-2796`, audited head `30fd6880`.

## Goal

Adopt the diff of [PR #3426](https://github.com/UOR-Foundation/uor-r4/pull/3426) (issue #2796: the figure/flock/catalog hardening) into a same-repo branch: this ORDER first, then `git merge --no-ff` of the contributor's commit unchanged (authorship preserved), then the REPORT. The change is carried unchanged; the only additions are this ORDER, the REPORT and, on completion, this item's `status: done` edit.

The repair: `svg_mismatch` gives the 0.01 tolerance only inside quoted attribute values (text numbers are exact), `flock_choice` uses a 1e-7 tie band instead of rounding, a new `share_prose_holds` guard fails `figures-charts` when the catalog no longer supports the fixed prose, and a non-string/non-null catalog date is an error. The flock tie-break is xtask figure tooling only; it does not touch the ACTIVE Flock read mechanism.

**Correct the PR body's verification record in the REPORT:** at the audit/merged head `cargo fmt -p xtask -- --check` is green only because this PR incidentally reformats the pre-existing `figures_charts.rs:1283` line (main is red there before it); `cargo clippy -p xtask --all-targets -- -D warnings` still shows the two pre-existing `book_stage.rs:342` / `docs_relink.rs:249` errors. Record both honestly; do not claim a green clippy.

## Cancel probe (run first)

- #3426 is still open at head `30fd6880…` in the fork; its diff is three `xtask/src/*.rs` files: no owner path, no generated output, no committed figure, no bar/status/panel.
- A re-home branch from current `main` with this ORDER then `git merge --no-ff refs/remotes/pr/3426` merges cleanly; the later merge-forward of `main` (which by then holds #3425) has no conflicting change in `docs_catalog.rs`.
- On today's tree, `figures-math`, `figures-charts` and `figures` pass and `figures --write` changes no tracked byte.
If the head moved or the diff grew, stop and re-review.

## Files in scope

- `xtask/src/docs_catalog.rs`, `xtask/src/figures_math.rs`, `xtask/src/figures_charts.rs`
- `docs/roadmap/items/infra-outsider-repairs.md` (status edit only, on completion)
- `docs/loop/tasks/infra-outsider-repairs/deepseek-77/**`

## Done when

At the final head, run and paste as the review's `done-when: host:`:
- `cargo test -p xtask` passes (214 tests at the audit head, 211 + 3 new);
- `cargo run -q --locked -p xtask -- docs-catalog --check`, `figures-math`, `figures-charts` and `figures` pass; `git status --short` clean afterwards (no committed figure/catalog change);
- a planted drift turns the new guard red: append a small set of synthetic post-cut result rows to a scratch copy of `archive/catalog.jsonl` in the worktree only, run `figures-charts`, confirm it exits with the "revise the fixed prose" error, then `git restore` the catalog;
- `cargo fmt -p xtask -- --check` is green at this head (this PR is what makes it green), and clippy shows only the two pre-existing errors above;
- `cargo run -q --locked -p uor-r4-claim-wording -- .` passes;
- `uor-harness loop pr-check --pr <adoption PR>` verdict pass, and the five required checks carry `policy: pass (run <id>)`.
- The item `infra-outsider-repairs` is set to `status: done` in this PR, and its issue is closed by `Closes #3433`.

## Stop and ask if

- any committed figure, catalog page or generated file changes;
- a test that existed before is removed, ignored or has an expected value changed;
- the guard does not turn red for the planted drift, or turns red for today's catalog;
- the merge-forward conflicts in `docs_catalog.rs` beyond a clean union of the two adopted diffs;
- anything lands in an owner path or a rendered view.
