# ORDER: infra-outsider-repairs, deepseek-75

Lab: deepseek. Item: [infra-outsider-repairs](../../../../roadmap/items/infra-outsider-repairs.md) (#3433). Base: `ec9a790fe` (the records PR #3434 merge). Adopted from @sharmis-tp's [PR #3424](https://github.com/UOR-Foundation/uor-r4/pull/3424), fork branch `clippy-drift`, audited head `047ee1eb…`.

## Goal

Adopt the diff of [PR #3424](https://github.com/UOR-Foundation/uor-r4/pull/3424) (issue #3168 part 1: mechanical clippy fixes in `crates/uor-r4-discrete` and `crates/uor-r4-source-interface`) into a same-repo branch: this ORDER first, then `git merge --no-ff` of the contributor's commit unchanged (authorship preserved), then the REPORT. The change is carried unchanged; the only additions are this ORDER and the REPORT.

This answers the contributor's #3168 question: `training.rs` is untouched, so no `DRIVER_SOURCE_SHA256` bump is needed. If that one lint is ever fixed (`training.rs:2255`), that fix recomputes the pin and edits `driver_source_pin.rs` in the same PR.

**Review must name the bar-adjacent line:** the diff rewrites `stage_t.rs:1263-1270` from `x > UNIGRAM_FULL_BAR && x > FULL_PANEL_T2_REFERENCE` to `x > UNIGRAM_FULL_BAR.max(FULL_PANEL_T2_REFERENCE)`. Both operands and the field are `u64` (`stage_t.rs:57,61,1059`), so this is the same predicate for every value; no bar value, status, record or M1 evidence moves. The review records that.

## Cancel probe (run first)

- #3424 is still open at head `047ee1eb…` in the fork; its diff is the 13 `.rs` files the audit read: no owner path, no rendered file, no test expectation, no pinned hash.
- A re-home branch from current `main` with this ORDER then `git merge --no-ff refs/remotes/pr/3424` has no conflicts (main since the PR base touched no discrete source).
- The clippy drift still reproduces at the branch base.
If the head moved or the diff grew outside those paths, stop and re-review before adopting.

## Files in scope

- `crates/uor-r4-discrete/src/address_cells.rs`, `bin/position-qualify.rs`, `d11_export.rs`, `device.rs`, `normalization_mechanism.rs`, `prime_zeta.rs`, `stage_t.rs`, `trunk.rs`, `wider_carrier.rs`, `within_cell_writer.rs`, `writer_bucket_qualification.rs`, `crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`, `crates/uor-r4-source-interface/src/lib.rs`
- `docs/loop/tasks/infra-outsider-repairs/deepseek-75/**`

## Done when

At the final head, run and paste as the review's `done-when: host:`:
- `cargo fmt -p uor-r4-discrete -p uor-r4-source-interface -- --check` clean;
- with the same stub `nvcc` the PR used: `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo clippy --locked -p uor-r4-discrete --all-targets --all-features --no-deps -- -D warnings` shows only the documented `training.rs:2255` type_complexity in the lib-test target and no new error;
- `cargo clippy --locked -p uor-r4-source-interface --all-targets -- -D warnings` clean;
- `cargo test --locked -p uor-r4-discrete --lib` 355 passed / 0 failed / 1 ignored, `--test read_off_ablation_controls` 7, `--test position_device_controls` 3;
- `cargo run -q --locked -p uor-r4-claim-wording -- .` passes;
- `uor-harness loop pr-check --pr <adoption PR>` verdict pass, and the five required checks at the head carry `policy: pass (run <id>)`.

## Stop and ask if

- the merge-forward conflicts beyond `docs/` additions;
- clippy reports anything the audit did not name, or a test count drops;
- the bar-predicate hunk is described as a behaviour change rather than the same `u64` predicate;
- anything lands in an owner path, a rendered file, a test expectation or a pinned receipt.
