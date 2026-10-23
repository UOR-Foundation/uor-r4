# REPORT: m3-memory-turn-discriminator-gate, antigravity-39

Lab: antigravity. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md). Branch: `antigravity/m3-memory-turn-discriminator-gate`.

## Negatives and open items

- **What this PR does not close:** GPU evaluation of the gated `world=v2` session recipe (`compiler-save-op-v25-rawtable` + `op-model-v25`, `op_policy = unless_query`) on the frozen 50-row non-relation prose panel ([`data/panels/nonrelation-prose-v1.txt`](../../../../../data/panels/nonrelation-prose-v1.txt)) and across the 3 held-out seeds (`21, 22, 23`) of 500 `world=v2` dialogues (`1,500` dialogues / `7,605` turns) is a GPU qualification run (CHARTER §4.5). It awaits a non-author `TEST FITNESS: FIT` review of [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md) and owner/PM registration of the bar and `<= $0.15` pod spend on `#2855`. Consequently, [`docs/roadmap/items/m3-memory-turn-discriminator-gate.md`](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) stays `status: ready`. References #2855, #2031.
- **No capability claim:** Zero milestone capability or M3 bar completion is claimed from the finite host controls alone.

## Cancel probe

- Probe command:
  ```sh
  ~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler::tests::memory_turn_discriminator_and_margin_gate_controls
  ```
- Before adding `StatementMargin` and `MemoryTurnGate` on base `b8aa3aefd`: `running 0 tests` (`resolve_op_policy`, `SavedCompiler::action`, and `SavedCompiler::table_action` had no score-gap margin or speaker-frame memory-turn discriminator, so under `OpPolicy::UnlessQuery` any non-relation turn where the op model emitted `Op: assert ...` or `Op: update ...` compiled directly to `CompiledAction::Assert` or `CompiledAction::Correct`).
- Deliberate-break check in `memory_turn_discriminator_and_margin_gate_controls`: replacing `MemoryTurnGate::default()` with `MemoryTurnGate::disabled()` causes the `#1718` small-talk and anaphoric controls (`"The weather has been unusually warm this week."`, `"I enjoy walking in the park after dinner."`, `"Actually, let us set that topic aside for another conversation."`) to compile to `Assert` / `Correct` instead of `Unresolved`.
- After adding `StatementMargin`, `MemoryTurnGate`, `has_memory_turn_cue`, `gate_compiled_action`, and `resolve_op_policy_gated`: `26 passed; 0 failed` (`relation_compiler::tests`) and `5 passed; 0 failed` (`nonrelation_prose_panel::tests`).

## Deliverables

1. **Integer / additive memory-turn discriminator and margin gate ([`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs)):**
   - [`StatementMargin`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1614-L1664): computes integer milli-unit score gaps (`relation_gap_milli`, `act_write_gap_milli`, `span_margin_milli`) directly from linear class scores via `saturating_sub`, `saturating_add`, and `Iterator::max` (zero squashing functions, zero dense matrix multiplications, zero discrete-path floats; D23/D25/D32 ratchets unchanged).
   - [`MemoryTurnGate`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1690-L1854), [`has_memory_turn_cue`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1964-L1973), [`gate_compiled_action`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1980-L2010), and [`resolve_op_policy_gated`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L2015-L2039) (plus integration into `SavedCompiler::action` and `SavedCompiler::table_action`):
     - Leaves `CompiledAction::QueryCurrent`, `CompiledAction::Query`, and `CompiledAction::Unresolved` untouched.
     - Rejects any candidate `Assert` or `Correct` that fails the additive score-gap margin thresholds ($\Delta_{\text{rel}} < 250$, $\Delta_{\text{act}} < 250$, $\Delta_{\text{span}} < 500$) or the structural speaker-grounded relation-frame check, returning `CompiledAction::Unresolved { reason }` and **never** converting a write into a read.
2. **Finite CPU verification ([`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L4224-L4806)):**
   - `memory_turn_discriminator_and_margin_gate_controls`: verifies additive score-gap thresholds, non-write passthrough, write-to-read prohibition on questions, and `#1718` + synthetic non-relation control rejection under `OpPolicy::UnlessQuery`.
   - `memory_turn_gate_preserves_all_world_v2_relation_statement_templates`: verifies `122 / 122` closed-relation `assert` and `update` templates across all `10` relations in `world=v1` and `world=v2` plus `3 / 3` open-relation templates (`0` false rejections).
   - `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`: grades all `50` frozen rows across all `6` strata of [`data/panels/nonrelation-prose-v1.txt`](../../../../../data/panels/nonrelation-prose-v1.txt) (`5,111 B`, `sha256: 7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`) under `grade_prose_panel` across all `10` closed relations and derived open relations (`1,100` adversarial candidate writes), achieving `0 / 50` false asserts and `0 / 50` false overwrites across all `22` sweeps.
3. **Lab note and `DRAFT` pre-registration sheet:**
   - [`docs/labs/m3-memory-turn-discriminator-gate/README.md`](../../../../labs/m3-memory-turn-discriminator-gate/README.md)
   - [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING - M3 #2031 m3-memory-turn-discriminator-gate`).

## Verification

- `~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler`: `26 passed; 0 failed`
- `~/.cargo/bin/cargo test -p uor-r4-training --lib nonrelation_prose_panel`: `5 passed; 0 failed`
- `uor-harness ratchet d23`, `d25`, `d32`: all `OK` (`0` sites added)
- `uor-harness loop gates --full`: `PASS`
