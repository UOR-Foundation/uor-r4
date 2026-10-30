# ORDER: infra-outsider-repairs, deepseek-76

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Base: the main commit that adds this item (records PR #3434), plus #3424 merged. Adopted from @sharmis-tp's [PR #3425](https://github.com/UOR-Foundation/uor-r4/pull/3425), fork branch `xtask-docs-tools-2792`, audited head `175a4377`.

## Goal

Adopt the diff of [PR #3425](https://github.com/UOR-Foundation/uor-r4/pull/3425) (issue #2792: the five docs-tool review findings) into a same-repo branch: this ORDER first, then `git merge --no-ff` of the contributor's commit unchanged (authorship preserved), then the REPORT. The change is carried unchanged; the only additions are this ORDER and the REPORT.

The repair tightens record tools, not records: an unreadable `SHA256SUMS` is an error, a rewrite placement shortfall is reported, `docs-stubs` refuses before writing on conflicts, `encode_target` is reused so `%28`/`%29` stay encoded, and colliding research-line page names are detected. It must not change any generated output on today's tree.

**Correct the PR body's verification record in the REPORT, not the code:** at this head `cargo fmt -p xtask -- --check` is red on a pre-existing, untouched line (`figures_charts.rs:1283`), and `cargo clippy -p xtask --all-targets -- -D warnings` is red with two pre-existing errors (`book_stage.rs:342`, `docs_relink.rs:249`). The adoption requires no new failure beyond that measured baseline; it must not claim a green fmt/clippy.

## Cancel probe (run first)

- #3425 is still open at head `175a4377…` in the fork; its diff is exactly four `xtask/src/docs_*.rs` files: no owner path, no rendered file, no generated output, no `gates.json`.
- `git merge --no-ff refs/remotes/pr/3425` into a branch from current `main` has no conflicts; main has not touched those files since the PR base.
- The five findings still describe the base code.
If the head moved or the diff grew, stop and re-review.

## Files in scope

- `xtask/src/docs_catalog.rs`, `xtask/src/docs_relink.rs`, `xtask/src/docs_rewrite_refs.rs`, `xtask/src/docs_stubs.rs`
- `docs/loop/tasks/infra-outsider-repairs/deepseek-76/**`

## Done when

At the final head, run and paste as the review's `done-when: host:`:
- `cargo check --locked -p xtask` passes;
- `cargo test --locked -p xtask` passes (217 tests at the audit head, including the six new tests) and no existing test is removed, ignored or has an expected value changed;
- `cargo run -q --locked -p xtask -- docs-catalog --check` passes;
- `git status --short` is clean after the docs commands, proving no generated output changed;
- `cargo run -q --locked -p uor-r4-claim-wording -- .` passes;
- fmt/clippy are no worse than the measured pre-existing baseline above (record both, do not claim green);
- `uor-harness loop pr-check --pr <adoption PR>` verdict pass, and the five required checks carry `policy: pass (run <id>)`.

## Stop and ask if

- any generated file, figure or catalog page changes;
- a test that existed before is removed, ignored or has an expected value changed;
- the merge-forward conflicts, or main touched the same files in a way that changes behaviour;
- anything lands in an owner path or a rendered view.
