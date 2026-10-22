# ORDER: m2-fresh-heldout-panel, antigravity-23

Lab: antigravity. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `f67ac2525`.

## Goal

The item's done-when reads:

> A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

This repair order (`antigravity-23`) addresses all required, medium, and low findings from Claude's `CHANGES` review (`6347483118`) and the two red CI jobs on PR `#2985`.

## Cancel probe (run first)

At `f67ac2525`:
1. `uor-harness loop next` reports `GO repair #2985 — check "docs archive and links" is FAILURE; check "harness rules and tests" is FAILURE; the last counted review record at its head (claude) says CHANGES`.
2. In `f67ac2525`, `docs/harness/panels/m2-fresh256.json` fails `contract::tests::every_format_passes_on_the_real_state`, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md` has 3-level relative links instead of 4-level, `verify_m2_fresh256_panel` checks a local array without parsing the repository source files on disk, single-row question-blind controls score `120/256` and `128/256` without history-pair completion gating, and `m2-fresh-heldout-panel.md` is marked `done` before the sealed bytes are uploaded to `caseyallard/uor-r4-store` and reviewed by another lab.

Result: PR `#2985` requires repair. Task is live. Class: instrument.

## Calls

- Fix the 12 broken relative links (`../../../` -> `../../../../`) in `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md` so `cargo run --locked -p xtask -- links` passes.
- Format `docs/harness/panels/m2-fresh256.json` to conform strictly to the frozen `panels` schema in `docs/harness/state-formats.json` (`frozen: false`, `noise: string`, `rows: string`, `seeds: string`, `bars[].current: number`, no extra top-level keys) so `contract::tests::every_format_passes_on_the_real_state` passes.
- Update `docs/labs/m2-fresh-heldout-panel/instrument/src/lib.rs` (and `main.rs`) to:
  1. Read and parse the actual repository files on disk (`crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs`, `crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs`, `archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json`, and `archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json`) during `verify_m2_fresh256_panel` to extract all 74 historical exposed literals and verify `0` exact, `0` content-word, and `0` 2-gram overlaps.
  2. Enforce 100% pairwise content-word disjointness across all 34 fresh literals themselves (replacing `"granitecauseway annex"` with `"syenite annex"`, `"slate conservatory"` with `"chert conservatory"`, and eliminating cross-stratum reuse of `"dikeway"` and `"Viaduct"`).
  3. Encode all utterances, literals, targets, and queries with the exact pinned `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`) token IDs, emit `#[serde(deny_unknown_fields)]`-compatible `inputs.json` (`{"schema": "uor-r4.native-source-bank-probe-input/1", "cases": ...}`) and `labels.json` (`{"schema": "uor-r4.native-source-bank-labels/1", "protocol": "uor-r4.literal-role-dialogue/2", "membership_only": true, "cases": ...}` with `"pair_id"` on every case), and use `StackStore` `(event + 1)` record/commit numbering.
  4. Score each history (`pair_id` in `native-bank-generalization.rs` `pairs_both_complete_distinct`) only when both its opposite-role queries (`job` and `home`) are answered completely and distinctly, so every question-blind control (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) scores `0 / 256` pair-verified rows (`0 / 128` histories, strictly `< 103`), and assert in `verify_m2_fresh256_panel` that all controls stay `< 103`.
  5. Add a unit test pinning the generated `plan_sha256`, `inputs_sha256`, `labels_sha256`, and `manifest_sha256` directly against `docs/harness/panels/m2-fresh256.json`.
- Keep `docs/roadmap/items/m2-fresh-heldout-panel.md` at `status: ready` (re-rendering `docs/roadmap/README.md`), `m2-fresh256.json` at `frozen: false` (`TEST FITNESS: PENDING` until another lab's non-author FIT review and maintainer upload to `caseyallard/uor-r4-store`), and use `References #2849` instead of `Closes #2849`.

## Files in scope

- `data/**`
- `docs/harness/panels/**`
- `docs/mechanisms/native-geometric.md`
- `docs/labs/m2-fresh-heldout-panel/**`
- `docs/reference/storage.md`
- `docs/roadmap/items/m2-fresh-heldout-panel.md` and `docs/roadmap/README.md`
- `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md`
- This task folder (`docs/loop/tasks/m2-fresh-heldout-panel/antigravity-23/**`)

## Done when

- `cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked` passes all unit, real-file overlap-exclusion, hash-pinning, noise-spread, tokenizer, control (`< 103`), and negative-mutation tests.
- `cargo test --locked -p uor-r4-harness --lib contract::tests::every_format_passes_on_the_real_state` and `cargo run --locked -p xtask -- links` pass.
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope or any owner path (`docs/harness/milestones.json`) needs to change, or if any GPU compute is needed.
