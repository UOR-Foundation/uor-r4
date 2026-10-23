# Lab Note: M3 Memory-Turn Discriminator and Margin Gate (`#2855`)

- **Roadmap item:** [`m3-memory-turn-discriminator-gate`](../../roadmap/items/m3-memory-turn-discriminator-gate.md) (`#2855`)
- **Milestone:** M3 — Exact Memory & Grounded Sessions (`#2031`)
- **Mechanism:** [`exact-memory-store`](../../mechanisms/exact-memory-store.md)
- **Source:** [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs)
- **Draft pre-registration sheet:** [`preregistration-DRAFT-2026-10-23.md`](preregistration-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`)

---

## 1. Exact Decision Boundary Where Non-Relation Prose Compiles to `Assert` or `Correct`

Inspection of [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs) and the `#1718` (`world=v2`) session evidence locates two coupled boundaries where non-relation turns become false memory writes:

1. **`OpPolicy::UnlessQuery` in [`resolve_op_policy`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1572-L1604):**
   In the `#1718` configuration (`compiler-save-op-v25-rawtable` + `op-model-v25`, `op_policy = unless_query`), when the table head returns `CompiledAction::Unresolved` on a non-relation turn, `resolve_op_policy` delegates to `op()`. Because `op-model-v25` was trained predominantly on closed-world relation turns (`Op: none` is a small minority of training rows), it frequently generates `Op: assert <closed_relation> <word>` or `Op: update <closed_relation> <word>` using a surface word from the prompt. [`parse_op`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1465-L1516) locates `<word>` at a word boundary in `source` and emits `CompiledAction::Assert` or `CompiledAction::Correct`, which `UnlessQuery` accepts unconditionally.
   - Example (`#1718` turn 1): `"The weather has been unusually warm this week."` $\rightarrow$ false `Assert`.
   - Example (`#1718` turn 2): `"I enjoy walking in the park after dinner."` $\rightarrow$ false `Assert`.
   - Example (`#1718` turn 3): `"Actually, let us set that topic aside for another conversation."` $\rightarrow$ leading `"Actually,"` triggers `update`, producing a false `Correct`.
2. **Table / span classification boundary (`SavedCompiler::table_action`):**
   On the table/span path, classification picks the argmax relation and act without checking whether the additive linear score gap $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$, the write-act gap $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$, or the span boundary margin $\Delta_{\text{span}}$ clears a positive threshold, and without checking whether the turn's words outside `span` actually form a speaker-grounded statement for the target relation.

---

## 2. Integer / Additive Gate Design (`StatementMargin` + `MemoryTurnGate`)

Implemented in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs) with zero squashing functions, zero dense matrix multiplications, and zero core floating-point operations (D23/D25/D32 ratchets unchanged):

- **[`StatementMargin`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1614-L1664):**
  Computes integer milli-unit score gaps from linear class scores via `saturating_sub`, `saturating_add`, and `Iterator::max`:
  - `relation_gap_milli`: $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$
  - `act_write_gap_milli`: $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$
  - `span_margin_milli`: $\Delta_{\text{span}} = \text{start\_margin} + \text{end\_margin}$
- **[`MemoryTurnGate`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1694-L1832):**
  1. **Score-gap margin gate (`check_margin`):** Requires $\Delta_{\text{rel}} \ge 250$, $\Delta_{\text{act}} \ge 250$, and $\Delta_{\text{span}} \ge 500$ whenever a `StatementMargin` is provided.
  2. **Structural memory-turn discriminator (`check_turn_structure`):**
     - Passes `QueryCurrent`, `Query`, and `Unresolved` through untouched.
     - Rejects candidate writes on interrogative turns (ending in `'?'` or opening with question/request cues such as `"what"`, `"where"`, `"who"`, `"remind"`, `"tell"`).
     - Rejects candidate writes whose frame attributes the relation to a third-person kinship or institutional subject (`"my grandfather"`, `"my sister's"`, `"my brother's"`, `"our neighbor"`, `"her cousin"`, `"the team's"`).
     - Requires the words outside `span` to match the speaker-grounded lexical frame of the target closed relation (`1..=10`, [`canonical_relation_name`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1667-L1682)) or a first-person copular open-relation phrase (`relation >= DERIVED_RELATION_ID_BASE`).
- **[`gate_compiled_action`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1943-L1973) & [`resolve_op_policy_gated`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1978-L2002):**
  Any candidate `Assert` or `Correct` rejected by `MemoryTurnGate` is converted to `CompiledAction::Unresolved { reason }` and is **never** turned into a read (`QueryCurrent` / `Query`).

---

## 3. Finite CPU Verification Summary

Verified via `cargo test -p uor-r4-training --lib relation_compiler` (`26 / 26` passed):

| Test | Scope | Result |
|---|---|---|
| `memory_turn_discriminator_and_margin_gate_controls` | Additive score-gap margin checks ($\Delta_{\text{rel}}, \Delta_{\text{act}}, \Delta_{\text{span}}$); preservation of `QueryCurrent`, `Query`, and `Unresolved`; write-to-read prohibition on questions; rejection of `#1718` and synthetic non-relation controls under `OpPolicy::UnlessQuery` | **PASS** |
| `memory_turn_gate_preserves_all_world_v2_relation_statement_templates` | All `122` closed-relation `assert` and `update` templates across all `10` relations in `world=v1` and `world=v2` plus `3` open-relation templates (`>= DERIVED_RELATION_ID_BASE`) | **PASS (`122 / 122` closed + `3 / 3` open preserved, `0` false rejections)** |
| `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel` | All `50` frozen rows across all `6` strata of [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) swept across all `10` closed relations and derived open relations (`1,100` adversarial candidate writes) | **PASS (`0 / 50` false writes across all `22` sweeps)** |
