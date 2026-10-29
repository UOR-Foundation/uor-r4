# ORDER: infra-loop-harness, claude-190

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `14b2bdccc`.

## Goal

**This unit (claude-190):** record the owner's two 29 Oct directions in PLAN.md's owner-direction table, and add the roadmap item `docs-status-refresh` that carries the documentation refresh. Then `loop next` reads the open direction instead of falling through to "any ready item", which is how `turn-memory-reply`, claude-162's first item, was never offered.

The umbrella item's done-when, for reference:


The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

## Cancel probe (run first)

Cancel if an existing item already covers the status pages, or if PLAN.md already records the 29 Oct directions.
- **No ready item covers them.** No ready item's `scope:` names `README.md`, `docs/start/`, `docs/model/`, `docs/research/lines/`, `docs/products/` or `xtask/src/figures.rs`. The only items whose scope reaches `docs/figures/**` are this umbrella and the done `infra-loop-pr08-migration`.
- **PLAN.md has no open owner direction** at `14b2bdccc`: `loop next` prints "PLAN.md has no open owner direction".
- **The new scope overlaps no open order.** The one other open order, antigravity-113, holds `docs/mechanisms/exact-memory.md` and `docs/mechanisms/object-store.md`, and the new item's scope leaves both out. It lists the crate READMEs one by one, because a glob would take in `crates/uor-r4-object-store/README.md`, which lies inside antigravity's `crates/uor-r4-object-store/**`. It names no shared file (`SHARED_FILES` in `loopclaim.rs`).

Result: **go.** This probes an instrument, the loop's record of directions and items, not a mechanism.

## Files in scope

- `PLAN.md`
- `docs/roadmap/items/*.md`
- `crates/uor-r4-harness/src/loopreview.rs`
- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/src/loopreport.rs`
- `crates/uor-r4-harness/src/loopclose.rs`
- `crates/uor-r4-harness/src/loopclaim.rs`
- `crates/uor-r4-harness/tests/loop_review*.rs`
- `crates/uor-r4-harness/tests/loop_next*.rs`
- `crates/uor-r4-harness/tests/loop_report*.rs`
- `crates/uor-r4-harness/tests/loop_close*.rs`
- `crates/uor-r4-harness/tests/loop_claim*.rs`
- `crates/uor-r4-harness/tests/loop_repair_orders*.rs`
- `crates/uor-r4-harness/tests/loop_shared_files*.rs`
- `crates/uor-r4-harness/tests/loop_prcheck.rs`
- `crates/uor-r4-harness/src/loopbench.rs`
- `crates/uor-r4-harness/tests/loop_bench*.rs`
- `docs/harness/d59.json`
- `docs/harness/owner-paths.json`
- `docs/integration/DECISIONS.md`
- `AGENTS.md`
- `CHARTER.md`
- `docs/rules/changes.md`
- `docs/mechanisms/README.md`
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/hopf-observation.md`
- `docs/mechanisms/spin-weights.md`
- `docs/mechanisms/ternary-xor.md`
- `crates/uor-r4-harness/src/loopdrive.rs`
- `crates/uor-r4-harness/tests/loop_drive*.rs`
- `crates/uor-r4-harness/src/prep.rs`
- `crates/uor-r4-harness/src/preprules.rs`
- `xtask/src/docs_links.rs`
- `xtask/src/archive_verify.rs`
- `xtask/src/figures_charts.rs`
- `docs/figures/**`
- `docs/roadmap/issues.json`
- `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`
- `docs/harness/labs.json`
- `crates/uor-r4-harness/src/labs.rs`
- `crates/uor-r4-harness/src/receipt.rs`
- `crates/uor-r4-harness/tests/lab_registry_cli.rs`
- `crates/uor-r4-harness/src/onboard.rs`
- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*`

## Done when

1. **PLAN.md's owner-direction table** has the two 29 Oct directions, verbatim and `open`, newest first. The end-to-end row's note gives the order of work, with `turn-memory-reply` first.
2. **`docs/roadmap/items/docs-status-refresh.md`** exists, `ready`. `cargo xtask roadmap check` passes, and `docs/roadmap/README.md` is re-rendered.
3. **`loop next` at the head** reads the open owner direction.
4. **The checks:** `loop gates --quick` is green, one fresh review, and a REPORT.

## Stop and ask if

- **A bar, a milestone status or a charter rule would change.** Those are decision rows, not this unit.
- **The new item's scope overlaps an open order.** Narrow it; do not wait.
