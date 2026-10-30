# REPORT: infra-outsider-repairs, deepseek-75 (#3424)

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Order: [ORDER.md](ORDER.md).

References #3168. Adopts @sharmis-tp's PR #3424; the fork PR is thanked, credited and closed after this merges.

## What did not work, first

- **The fork route was blocked.** The original plan was to push the ORDER/REPORT commits to the contributor's fork branch; a public-repository fork PR gets a read-only `GITHUB_TOKEN`, so `required-transport` cannot publish the five required check names and the merge queue would never admit it. The repair is re-homed in this same-repo branch, with the contributor's commit merged unchanged (`git merge --no-ff refs/remotes/pr/3424`); authorship is preserved and the fork was never written to.
- **One lint is left on purpose.** `training.rs:2255` (lib-test `type_complexity`) cannot be fixed without editing a file hashed into `DRIVER_SOURCE_SHA256`; the PR leaves it and says so. #3168 stays open for that lint and the CI clippy gate, which is a `.github/**` owner path.
- **The bar-adjacent line is unchanged behaviour.** The diff rewrites the M1 closing-bar predicate at `stage_t.rs:1263-1270` to `x > UNIGRAM_FULL_BAR.max(FULL_PANEL_T2_REFERENCE)`. The field and both bar values are `u64` (`stage_t.rs:57,61,1059`), so it is the same predicate for every value; no bar value, status, record or M1 evidence moves.
- **Part of the fix is suppression.** Nine `#[allow(clippy::too_many_arguments)]` attributes (plus a scoped `needless_range_loop` and a `dead_code`) clear the remaining lints without refactoring; that matches existing style in the crate and is stated so the record does not imply a refactor.
- **One computed identity moves.** `device.rs` is one of the `include_bytes!` sources of `device::source_sha256()`, so the device binding's computed `source_sha256` changes. No checked-in pinned constant moves (`DRIVER_SOURCE_SHA256` is untouched), no committed test pins it, and the harm review found no active work using `uor-r4-discrete`; the item records the move rather than hiding it.

## What changed

- **Adopted** @sharmis-tp's 13-file lint repair unchanged (`+203/−192`): 12 files in `crates/uor-r4-discrete` and one in `crates/uor-r4-source-interface`.
- **Merge-forward** of `origin/main` (records PR #3434 and the docs PR #3431; no source conflict).
- **Added:** this ORDER and REPORT in `docs/loop/tasks/infra-outsider-repairs/deepseek-75/`.
- **Not changed:** no bar value, status, §3 rule, owner path, rendered file, test expectation or checked-in pinned constant.
- Size: 13 code files from the PR (unchanged), plus this ORDER and REPORT; no lines of ours in the code.

## Measurements

- Before the repair, at the PR base (audited): `uor-r4-discrete` clippy `--all-targets --all-features` was 24 lib + 30 lib-test errors under the stub `nvcc`; `uor-r4-source-interface` had 3 `write_literal` errors.
- At the code head of this branch (`e6eb91af5`, tree-identical to independently verified `8b853d3ac`): fmt clean; source-interface clippy clean; `cargo test -p uor-r4-discrete --lib` 355 passed / 0 failed / 1 ignored; `read_off_ablation_controls` 7 passed; `position_device_controls` 3 passed; the stub-`nvcc` all-features clippy exits 101 with exactly the one documented `training.rs:2255` residual and no new error (run by the merge engineer and re-run independently by a second verifier).
- At this final head: `git diff e6eb91af5 HEAD -- crates/` is empty, so the merged-source results above apply byte-for-byte; only `docs/` differs.
- No GPU run, no model evaluation, no panel scoring. M1's close and its 0.625 limit are untouched.

## Checks run at the head

- `cargo fmt -p uor-r4-discrete -p uor-r4-source-interface -- --check` -> clean.
- `cargo test --locked -p uor-r4-discrete --lib` -> 355 passed, 0 failed, 1 ignored.
- `cargo test --locked -p uor-r4-discrete --test read_off_ablation_controls` -> 7 passed.
- `cargo test --locked -p uor-r4-discrete --test position_device_controls` -> 3 passed.
- `cargo clippy --locked -p uor-r4-source-interface --all-targets -- -D warnings` -> clean.
- `cargo test --locked -p uor-r4-source-interface` -> ok (0 tests).
- `cargo run -q --locked -p uor-r4-claim-wording -- .` -> pass.
- Stub-`nvcc` `cargo clippy --locked -p uor-r4-discrete --all-targets --all-features --no-deps -- -D warnings` at code head `e6eb91af5` -> only the documented `training.rs:2255` residual (two independent runs).
- `uor-harness loop pr-check --pr <this PR>` and the five required checks carry `policy: pass (run <id>)`; the exact-head review record is posted at the head.

## Skills used

- `scope-guard` for the unchanged-diff adoption and the merge-forward.
- `solution-integrity` for the predicate equivalence and the residual-lint call.

## Still open

- **#3168 stays open:** its clippy-gate half needs `.github/**` (owner) and the `training.rs:2255` lint needs a pin-bump PR.
- **Credit and the pin answer** are on #3168; the fork PR is commented and closed after this merges.
- **The other two adoptions:** #3425 (deepseek-76) and #3426 (deepseek-77); the last sets item #3433 done.

## Plain summary

We adopted the contributor's lint repair for the product crate unchanged. It is lint-only; the one M1 bar predicate it touches is rewritten to the same form for every value and no number moves. One known lint and the CI gate stay open on #3168. The device binding's computed source identity changes, and that is recorded.
