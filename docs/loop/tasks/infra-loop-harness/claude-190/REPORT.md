# REPORT: infra-loop-harness, claude-190 (the 29 Oct directions in PLAN, and the docs-status-refresh item)

Lab: claude. Order: [ORDER.md](ORDER.md). This is a maintenance unit: no model and no run. The umbrella's done-when does not move.

**Speed.** No serving or training change.

## What did not work, first

- **`loop next` cannot show the direction before the merge.** It reads PLAN.md from `origin/main` (`plan_table` → `origin_file`), so at the head it still prints "PLAN.md has no open owner direction". Done-when 3 is checked on fresh `main` at `loop close --verify`, not here.
- **The drift this unit fixes was the loop's own.** With no open direction, `loop next` falls through to "any ready item", ranked by how many items each unblocks. `turn-memory-reply`, claude-162's first item and the capability blocker, unblocks none. It was never offered, and claude took `fbs-serving-vector-kernel` and `integer-hamming-read` units instead (claude-185 to claude-189).
- **The first item draft failed `roadmap check`:** it lacked the section `## What is missing`. It was added, and the check passes.

## What changed

- **`PLAN.md`:** two `open` rows at the head of the owner-direction table, with the owner's 29 Oct words verbatim. The end-to-end row stands first, though it was said earlier in the day: `loop next` reads only the first open row, and that row sets every lab's next unit (the review's Medium).
  - **The documentation refresh:** the item `docs-status-refresh`. The row says it stands in place of D77's "no documentation-clarification unit" for this one item.
  - **One working model, end to end,** with the best mechanism per part. The note gives the order of work:
    1. `turn-memory-reply`;
    2. the parts served in integers;
    3. R7;
    4. one model end to end with tokens/s;
    5. then cost.

    The row stays open until step 4 is done.

  One paragraph under "The trunk line's pivot" points to that order and says why `loop next` never offered `turn-memory-reply`.
- **`docs/roadmap/items/docs-status-refresh.md` (new, `ready`, `needs_gpu: false`).**
  - **Scope:** the README, the site's start, model, product, geometry, research and research-line pages, the mechanism pages, `docs/figures/**`, `xtask/src/figures.rs`, the crate READMEs and `docs/roadmap/issues.json`.
  - **Left out:** `docs/mechanisms/exact-memory.md`, `docs/mechanisms/object-store.md` and `crates/uor-r4-object-store/README.md`, which antigravity-113's open order holds, and every shared file. The crate READMEs are listed one by one for that reason. `crates/uor-r4-float-train/README.md`, which the trunk line's crate lacks, may be created.
  - **Done-when:**
    - the status pages state 29 Oct with a source for each number;
    - the generated pictures are redrawn and `cargo xtask figures` passes;
    - every item has its issue (`cargo xtask roadmap sync --apply` from a tree equal to `main`);
    - `mdbook build` and the link check pass;
    - one fresh review.
- **`docs/roadmap/README.md`:** re-rendered by `cargo xtask roadmap render`.

## Measurements

- **`cargo xtask roadmap check`:** 110 items OK (ready 16, held 10, decision 1, done 83).
- **Items without an issue,** from `docs/roadmap/issues.json` against `docs/roadmap/items/` on `14b2bdccc`: 14, every one opened since 27 Oct:
  - `fbs-cost-profile`, `fbs-serving-kernels`, `fbs-serving-vector-kernel`;
  - `in-space-trunk-to-text`, `integer-hamming-read`, `integer-mul-div-serving`;
  - `one-decoder`, `pipeline-map`, `pipeline-map-revision`;
  - `route-plus-memory`, `training-step-cost`;
  - `turn-memory-integer`, `turn-memory-reply`, `writer-admission`.

  No done item has an open issue, and no open item has a closed one.
- **`xtask/src/figures.rs`'s `DATE`** is "13 October 2026".

## Checks run at the head

- **`cargo run -p xtask -- roadmap check`:** 110 items OK. **`roadmap render`:** wrote the README, with 110 items.
- **`uor-harness loop gates --quick`:** green.
- **`uor-harness loop report --check`:** PASS.

## Skills used

- **holding-the-goal.** The unit records the owner's direction where the loop reads it, so the next units follow the plan and not the unblock count.

## Still open

- **`docs-status-refresh` itself:** the next unit.
- **On fresh `main`:** `loop next` should read the docs row as the first open owner direction.

## Plain summary

This writes down Casey's two instructions from today in the project's plan, in his own words. The first is to get one working model running from start to finish. The second is to bring all the documentation up to date.

It also adds the work item that the documentation update will run under. The tool that hands each lab its next piece of work reads that plan. Until now it had nothing to steer it, so it kept offering side tasks instead of the step that makes the model's replies better.
