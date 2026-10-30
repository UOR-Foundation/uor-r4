---
id: infra-outsider-repairs
title: Adopt and verify outside-contribution repairs to the product crate and xtask
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-discrete/src/address_cells.rs, crates/uor-r4-discrete/src/bin/position-qualify.rs, crates/uor-r4-discrete/src/d11_export.rs, crates/uor-r4-discrete/src/device.rs, crates/uor-r4-discrete/src/normalization_mechanism.rs, crates/uor-r4-discrete/src/prime_zeta.rs, crates/uor-r4-discrete/src/stage_t.rs, crates/uor-r4-discrete/src/trunk.rs, crates/uor-r4-discrete/src/wider_carrier.rs, crates/uor-r4-discrete/src/within_cell_writer.rs, crates/uor-r4-discrete/src/writer_bucket_qualification.rs, crates/uor-r4-discrete/tests/read_off_ablation_controls.rs, crates/uor-r4-source-interface/src/lib.rs, xtask/src/docs_catalog.rs, xtask/src/docs_relink.rs, xtask/src/docs_rewrite_refs.rs, xtask/src/docs_stubs.rs, xtask/src/figures_math.rs, xtask/src/figures_charts.rs, docs/loop/tasks/infra-outsider-repairs/**]
done_class: host
---

## Why it matters

Three small repairs to project code were found by our own labs, filed as issues, and then written by an outside contributor, @sharmis-tp, as PRs #3424, #3425 and #3426. The owner directed on 30 Oct 2026 that harmless contributions should be merged ("figure out what it will take to merge Suzuki's work in ... I do not think it will actively hurt anything else, will it? ... otherwise I do not mind the contributions"), recorded in the [#2028 owner log, comment 6527705969](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6527705969) (quoted there as an excerpt with ellipses; the verbatim capture is in the Lead's chat).

The three repairs:
- **#3424** (References #3168): mechanical clippy fixes in `crates/uor-r4-discrete` and `crates/uor-r4-source-interface`; lint-only, no behaviour change. It also rewrites the M1 closing-bar predicate to an equivalent `> max(U, T)` form and leaves one known lint, `training.rs:2255`, because fixing that moves `DRIVER_SOURCE_SHA256`.
- **#3425** (Closes #2792): the five docs-tool findings from the #2788 review: unreadable manifest is an error, rewrite placement shortfall is reported, stubs refuse before writing on conflicts, `encode_target` is reused, page-name collisions are detected.
- **#3426** (Closes #2796): the #2791 hardening: `svg_mismatch` tolerance only inside attribute values, a 1e-7 tie band in `flock_choice`, a guard that fails `figures-charts` when the catalog no longer supports the fixed share prose, and a non-string catalog date is an error.

**D77.** A new infrastructure unit runs only when a milestone item depends on it. This item runs because the owner directed it on 30 Oct 2026; that direction stands in place of D77's bar for this one item, as the 29 Oct documentation direction does for `docs-status-refresh` (PLAN.md, owner directions). None of the three repairs moves a bar, a status, a §3 rule, an owner path or a rendered file, and no checked-in pinned constant moves (#3168's `DRIVER_SOURCE_SHA256` is untouched because `training.rs` is). One computed identity does move: #3424 edits `device.rs`, one of the `include_bytes!` sources of `device::source_sha256()`, so the device binding's computed `source_sha256` changes; the fresh-context harm review found no active harm because no in-flight work uses `uor-r4-discrete`. #3168 stays open for its clippy-gate half, which needs a `.github/**` owner change.

## What is missing

- **No roadmap item owns these paths.** The product crate's M1/M2 items are `done` or held; the docs-tool paths are owned by no live item. `infra-loop-harness` is the records/item-file umbrella and its scope deliberately stops at records, not product code; widening it into `crates/uor-r4-discrete/src/**` would put the model crate inside every future loop unit.
- **No ORDER, REPORT or review exists for the three fork PRs** (no adoption record, no `uor-review v1`, CI never ran on them). Under CHARTER §5.5 an outsider PR needs the maintainer lab that adopts it to add the ORDER and REPORT.
- **They are behind main and cannot publish the required checks from a fork.** `strict_required_status_checks_policy` is on, and a `pull_request` run from a fork gets a read-only `GITHUB_TOKEN` on this public repository; `required-transport` needs `checks: write`, so the five ruleset names cannot be published at a fork PR head and the merge queue would never admit it. Each repair is therefore re-homed into a same-repo branch and PR: the branch starts from `main`, adds the ORDER commit, merges the contributor's commits unchanged (authorship preserved), adds the REPORT, and runs CI normally. The original fork PR is thanked, credited and closed when the re-homed PR merges.
- **The contributor's #3168 question is unanswered** (bump `DRIVER_SOURCE_SHA256` in the same PR or not) and the three intent comments got no reply.

## Done when

1. **Each repair is merged under a `deepseek` ORDER naming this item**, in the order #3424, #3425, #3426, each with:
   - a fresh-context exact-head `uor-review v1` APPROVE at its final head (D66);
   - `uor-harness loop pr-check --pr <n>` verdict pass at that head, with the five required checks carrying `policy: pass (run <id>)`;
   - the queue (`loop queue`), then a post-merge verification on fresh `main`.
2. **The repairs behave as their bodies say**, re-run at the adopted head: #3424's clippy/tests, #3425's `cargo test -p xtask` and no generated-output change, #3426's figure checks with a planted drift turning red and no committed figure change.
3. **#2792 and #2796 are closed by their merged PRs; #3168 records its remainder** (the CI clippy gate is a `.github/**` owner path, and `training.rs:2255` is its own pin-bump PR).
4. **The contributor is credited and answered**, and nothing in a bar, status, §3 rule, owner path, rendered file or checked-in pinned constant moved; the device binding's computed `source_sha256` move is recorded, not hidden.
5. **The last adoption PR sets this item to `done`** in a status-only edit and says `Closes #<issue>` (D70).

## How to start

Read `CHARTER.md` §5.5, `docs/loop/README.md` (the outsider clause and the `uor-review v1` record), the three issues and PRs, and each earlier adoption's committed ORDER and REPORT under `docs/loop/tasks/infra-outsider-repairs/`. Claim this item. For each repair: create a same-repo branch from current `main`, commit the ORDER first (alone in its folder), merge the contributor's PR head with `git merge --no-ff` (original authorship preserved), commit the REPORT, push and open a `[deepseek] Adopt #NNNN: ...` PR. Run the ORDER's host done-when at that head, post the exact-head fresh-context `uor-review v1`, then `loop queue`. Comment on the original fork PR that it was adopted, then close it. The fork route was ruled out by the read-only fork token before any fork write.

## Rules that apply

- **Delivery (§5.5):** PR, exact-head fresh-context review, checks at that head posted, merge queue, verify on fresh `main`, delete only after MERGED, `uor-hygiene --apply`.
- **Review inside the unit (D66)**, and **Closes/References (D70)**: partial work says `References`; the finishing PR says `Closes` and edits the item.
- **Owner-only boundaries (§5.1):** the clippy gate half of #3168 changes `.github/**` and stays with the owner; no repair may touch a bar, a status, §3 or a pinned hash.
- **No GPU** (host checks only), **Rust only**, no new dependencies.
- **Outsiders (CHARTER §5.3, docs/loop README):** a maintainer lab adopts with a tagged record naming the order it writes.

## Sources

- PRs [#3424](https://github.com/UOR-Foundation/uor-r4/pull/3424), [#3425](https://github.com/UOR-Foundation/uor-r4/pull/3425), [#3426](https://github.com/UOR-Foundation/uor-r4/pull/3426); issues [#3168](https://github.com/UOR-Foundation/uor-r4/issues/3168), [#2792](https://github.com/UOR-Foundation/uor-r4/issues/2792), [#2796](https://github.com/UOR-Foundation/uor-r4/issues/2796).
- Owner direction, [#2028 owner log comment 6527705969](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6527705969) (30 Oct 2026; excerpt, ellipses as quoted).
- [CHARTER.md](../../../CHARTER.md) §5.3, §5.5; [docs/loop/README.md](../../loop/README.md).

## Cancel if

The owner withdraws the 30 Oct direction, or the adopting lab's fresh-context review finds a repair is not behaviour-neutral on a bar, record or pinned hash: then that one repair stops as a repair and the other two proceed; this item is never a reason to keep a change that moves a number.
