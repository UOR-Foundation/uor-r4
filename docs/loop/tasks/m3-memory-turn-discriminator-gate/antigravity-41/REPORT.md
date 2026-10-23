# REPORT: m3-memory-turn-discriminator-gate, antigravity-41

## What did not work, first

- **Un-gating `#[cfg(any())]` on `Softmax`, `SparseSoftmax`, and `SavedCompiler` in `antigravity-40`:** Exact-head review of `1ddfa1cbabb066ffa2c19cb27e61e476ebbefb35` (`antigravity-40`) caught that removing `#[cfg(any())]` from `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, and `SavedCompiler` in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs) brought legacy Stage-A floating-point `(l - top).exp()` classifiers and dense `f64` dot-product heads into the compiled module under `cargo test` (violating CHARTER §3.2, §3.3, and §3.4 even though `ratchet d23` and `ratchet d25` remained at baseline). We restored `#[cfg(any())]` on `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, `HeadParts`, `DenseParts`, `CombinedParts`, `CompilerArtifact`, `SavedCompiler`, `encode`, and `impl TurnCompiler for SavedCompiler` so lines `1..=1607` of `relation_compiler.rs` are 100% byte-identical to `origin/main` and zero `Softmax`/`SparseSoftmax` code is compiled or executed.
- **Stage-A `#[cfg(any())]` code in `milestone_world_v2` and `stack_tracking`:** In `crates/uor-r4-training/src/lib.rs`, enabling `pub mod milestone_world_v2`, `pub mod stack_tracking`, and `pub mod relation_compiler` compiles the active integer/generator surfaces (`53` unit tests in `uor-r4-training`), while `milestone_world_v2`'s own 32 legacy Stage-A tests and `stack_tracking`'s legacy Stage-A helpers remain compiled out with `#[cfg(any())]` exactly as on `main`.
- **Two-arm GPU session and prose-panel evaluation on `compiler-save-op-v25-rawtable` + `op-model-v25`:** Not run in this unit because GPU evaluation requires a non-author `TEST FITNESS: FIT` review of [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md) and owner release on `#2855` (CHARTER §3.5, §4.5, §5.1). Moreover, `SavedCompiler`'s margin half remains `None` behind `#[cfg(any())]` until a lawful integer geometric head replaces its compiled-out legacy float heads.

## What changed

- [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs) (`5,133` total lines; `+1,365 / -1` lines vs `origin/main`, `+359 / -312` lines in `antigravity-41`):
  - Restored `#[cfg(any())]` on `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, `HeadParts`, `DenseParts`, `CombinedParts`, `CompilerArtifact`, `SavedCompiler`, `encode`, `impl TurnCompiler for SavedCompiler`, and the two legacy `SpanHead`/`SavedCompiler` tests (leaving `SavedCompiler`'s margin half as `None` behind `#[cfg(any())]` until a lawful integer geometric head replaces its compiled-out legacy float heads).
  - Added compiled, 100% integer-only `pub struct GatedTurnCompiler<C>` implementing `TurnCompiler` with configurable `MemoryTurnGate` (`new`, `with_memory_turn_gate`, `set_memory_turn_gate`, `memory_turn_gate`) and an optional integer `StatementMargin` callback (`with_margin_fn`), plus `gated_turn_compiler_wires_configurable_gate_and_integer_margin` and `GatedTurnCompiler` grading in `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`.
- [`crates/uor-r4-training/src/lib.rs`](../../../../../crates/uor-r4-training/src/lib.rs) (`192` total lines; `+4 / -5` lines vs `origin/main`), [`crates/uor-r4-training/src/milestone_world_v2.rs`](../../../../../crates/uor-r4-training/src/milestone_world_v2.rs) (`3,395` total lines; `+2 / -14` lines vs `origin/main`), and [`crates/uor-r4-training/src/stack_tracking.rs`](../../../../../crates/uor-r4-training/src/stack_tracking.rs) (`2,676` total lines; `+24 / -7` lines vs `origin/main`): expose the generator and integer turn-compiler surfaces while leaving `milestone_world_v2`'s 32 Stage-A tests and `stack_tracking`'s Stage-A helpers behind `#[cfg(any())]` as on `main`.
- [`docs/labs/m3-memory-turn-discriminator-gate/README.md`](../../../../labs/m3-memory-turn-discriminator-gate/README.md) (`56` lines; `+7 / -7` lines in `antigravity-41`) and [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md) (`96` lines; `+3 / -3` lines in `antigravity-41`): document `GatedTurnCompiler<C>` and state plainly that `SavedCompiler`'s legacy float heads stay compiled out behind `#[cfg(any())]` (`None` margin) until a lawful integer geometric head replaces them.
- [`docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/ORDER.md`](ORDER.md) (`28` lines) and [`docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/REPORT.md`](REPORT.md) (`55` lines).

## Measurements

- **`cargo test -p uor-r4-training --lib`:** `53 passed; 0 failed; 0 ignored; finished in 0.90s` (`4 / 4` `relation_compiler` tests + `5 / 5` `nonrelation_prose_panel` tests + `44` other crate unit tests; `milestone_world_v2`'s 32 Stage-A tests remain compiled out behind `#[cfg(any())]` as on `main`).
- **Generator-driven `world=v2` statement preservation (`memory_turn_gate_preserves_all_world_v2_relation_statement_templates`):** `79 / 79` statement and update templates iterated directly from `milestone_world_v2::relations()` across `Split::Train` and `Split::Development` preserved (`0` false rejections), plus `17 / 17` teacher-paraphrase templates and `3 / 3` open-relation templates (`>= DERIVED_RELATION_ID_BASE`).
- **Configured `GatedTurnCompiler` + integer `StatementMargin` (`gated_turn_compiler_wires_configurable_gate_and_integer_margin`):** Verifies that `GatedTurnCompiler<C>` enforces integer `StatementMargin` thresholds ($\Delta_{\text{rel}} \ge 250$, $\Delta_{\text{act}} \ge 250$, $\Delta_{\text{span}} \ge 500$), rejects non-relation writes under `MemoryTurnGate::default()`, passes writes when `MemoryTurnGate::disabled()` is configured, and preserves `QueryCurrent`, `Query`, and `Unresolved`.
- **Host synthetic-adversary check on frozen 50-row non-relation prose panel (`memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`):** Across all `22` `GatedTurnCompiler` sweeps (`10` closed relations + `1` derived open relation $\times$ `{Assert, Correct}` $\times$ `50` rows = `1,100` adversarial candidate writes), `grade_prose_panel` reports `false_assert_count == 0`, `false_overwrite_count == 0`, `false_correct_action_count == 0`, `unresolved_count == 50`, and `is_all_clean() == true` (`0 / 50` false writes vs `50 / 50` on the `MemoryTurnGate::disabled()` deliberate-break control).
- **Ratchet checks:** `uor-harness ratchet d23`, `uor-harness ratchet d25`, and `uor-harness ratchet d32` all pass with `0` new sites and `0` compiled `Softmax`/`SparseSoftmax` structs.

## Checks run at the head

- `uor-harness loop gates --quick` — PASS
- `cargo test -p uor-r4-training --lib` — PASS (`53 passed; 0 failed`)
- `cargo run -p xtask -- links --baseline docs/rules/links-baseline.txt` — PASS (`0` new broken links)
- `uor-harness ratchet d23` / `uor-harness ratchet d25` / `uor-harness ratchet d32` — PASS (`OK`)
- `uor-harness loop report --check` — PASS
- `uor-harness loop pr-check --local` — PASS
- `uor-harness loop gates --full` — PASS (`19 / 19` gates)

## Skills used

- [`uor-research-architect`](../../../../../.agents/skills/uor-research-architect/SKILL.md) — enforced CHARTER §3.1–§3.4 by keeping all legacy Stage-A `Softmax`/`SparseSoftmax` float heads compiled out behind `#[cfg(any())]` and exposing the compiled integer-only `GatedTurnCompiler<C>` wrapper.
- `solution-integrity` (session skill) — verified that lines `1..=1607` of `relation_compiler.rs` match `origin/main` byte-for-byte and that `GatedTurnCompiler` tests both the enabled gate (`0 / 50` false writes) and disabled deliberate-break control (`50 / 50` false writes).
- `scope-guard` (session skill) — kept all changes strictly within the `m3-memory-turn-discriminator-gate` item scope.

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md), owner/PM registration of the bar and `<= $0.15` pod spend on `#2855`, replacement of `SavedCompiler`'s compiled-out legacy float heads with a lawful integer geometric head for `StatementMargin`, and the two-arm GPU evaluation. [`docs/roadmap/items/m3-memory-turn-discriminator-gate.md`](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) remains `status: ready`. References #2855, #2031.

## Plain summary

Restored `#[cfg(any())]` on all legacy Stage-A `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, and `SavedCompiler` items in `relation_compiler.rs` so zero softmax or dense floating-point classifier code is compiled, added the compiled integer-only `GatedTurnCompiler<C>` wrapper with configurable `MemoryTurnGate` and integer `StatementMargin` callback, and stated the `#[cfg(any())]` and margin wiring status plainly across the lab note, draft pre-registration sheet, and report.
