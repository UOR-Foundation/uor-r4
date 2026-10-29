# ORDER: docs-status-refresh, claude-191

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). Base: `87923fa2d`.

## Goal

The item's done-when reads:

> 1. **Every status page in scope states the project as of 29 Oct 2026, with a source for each number:** the README, the site's start, model, product, mechanism and research pages, and the crate READMEs. That covers:
>    - the trunk line's chain from claude-142 to claude-189;
>    - the model's mechanism per part, and the state of each part (trained, served, missing);
>    - the decisions:
>      - the 28 Oct pivot to route + exact memory;
>      - serving whole-table nearest (claude-150);
>      - D80 and D81;
>      - cost per token as the criterion;
>      - float training with an exact integer export;
>      - the deterministic backward;
>    - the order of work in PLAN.md's open owner direction of 29 Oct;
>    - the milestones as `docs/harness/milestones.json` gives them. No bar or status changes here: those are decision rows.
> 2. **The generated pictures are redrawn from `xtask/src/figures.rs`,** with a current date and the current composition, and `cargo xtask figures` passes. A hand-drawn picture that states status is corrected.
> 3. **Every roadmap item has its issue.** That means `cargo xtask roadmap sync --apply`, run from a tree equal to `main`, with `docs/roadmap/issues.json` committed. The milestone checklists are current, and a stale open PR is named on #2028 to its own lab.
> 4. **The site builds** (`mdbook build`), and the docs link check passes.
> 5. **One fresh review** of the whole diff, against the facts sheet the unit writes, and one REPORT.

## Cancel probe (run first)

To fill in.

## Files in scope

- `README.md`
- `docs/README.md`
- `docs/SUMMARY.md`
- `docs/start/*.md`
- `docs/products/*.md`
- `docs/model/*.md`
- `docs/geometry/*.md`
- `docs/research/README.md`
- `docs/research/findings.md`
- `docs/research/open-questions.md`
- `docs/research/history.md`
- `docs/research/lines/*.md`
- `docs/mechanisms/README.md`
- `docs/mechanisms/address-cells.md`
- `docs/mechanisms/admission.md`
- `docs/mechanisms/carrier-credit.md`
- `docs/mechanisms/discrete-geometric-learning.md`
- `docs/mechanisms/discrete-learning.md`
- `docs/mechanisms/e8-orbit-memory.md`
- `docs/mechanisms/exact-addressed-memory.md`
- `docs/mechanisms/finite-relational-head.md`
- `docs/mechanisms/flock-rank-reads.md`
- `docs/mechanisms/flock-read.md`
- `docs/mechanisms/geometric-stack.md`
- `docs/mechanisms/hamilton-gate.md`
- `docs/mechanisms/hopf-observation.md`
- `docs/mechanisms/invariant-addressed-read.md`
- `docs/mechanisms/legal-construction.md`
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/native-geometric.md`
- `docs/mechanisms/pointer-identity-key.md`
- `docs/mechanisms/pointer-key.md`
- `docs/mechanisms/prime-zeta-codebook.md`
- `docs/mechanisms/prime-zeta-complex-routing.md`
- `docs/mechanisms/protected-legal-construction.md`
- `docs/mechanisms/r4g1-graph.md`
- `docs/mechanisms/rank-margin-objective.md`
- `docs/mechanisms/rank-margin.md`
- `docs/mechanisms/read-binding.md`
- `docs/mechanisms/recurrence-gates.md`
- `docs/mechanisms/relation-read.md`
- `docs/mechanisms/route-read.md`
- `docs/mechanisms/spin-native-weights.md`
- `docs/mechanisms/spin-weights.md`
- `docs/mechanisms/spiralcore-fbs-projection.md`
- `docs/mechanisms/spiralcore-fbs.md`
- `docs/mechanisms/starling-read.md`
- `docs/mechanisms/ternary-xor-arithmetic.md`
- `docs/mechanisms/ternary-xor-evidence-factsheet.md`
- `docs/mechanisms/ternary-xor.md`
- `docs/mechanisms/tokenizer.md`
- `docs/mechanisms/unified-geometric-carrier-credit.md`
- `docs/mechanisms/vsa.md`
- `docs/figures/**`
- `xtask/src/figures.rs`
- `crates/uor-r4-api/README.md`
- `crates/uor-r4-core/README.md`
- `crates/uor-r4-discrete/README.md`
- `crates/uor-r4-float-train/README.md`
- `crates/uor-r4-graph-compiler/README.md`
- `crates/uor-r4-graph-format/README.md`
- `crates/uor-r4-integer/README.md`
- `crates/uor-r4-lut/README.md`
- `crates/uor-r4-pod-provider/README.md`
- `crates/uor-r4-proof-model/README.md`
- `crates/uor-r4-router/README.md`
- `crates/uor-r4-training/README.md`
- `docs/roadmap/issues.json`
- `docs/loop/tasks/docs-status-refresh/**`
- `docs/roadmap/items/docs-status-refresh.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
