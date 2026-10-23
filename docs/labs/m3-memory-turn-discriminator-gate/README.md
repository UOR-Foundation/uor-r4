# Lab Note: M3 Memory-Turn Discriminator and Margin Gate (`#2855`)

- **Roadmap item:** [`m3-memory-turn-discriminator-gate`](../../roadmap/items/m3-memory-turn-discriminator-gate.md) (`#2855`)
- **Milestone:** M3 — Exact Memory & Grounded Sessions (`#2031`)
- **Mechanism:** [`exact-memory`](../../mechanisms/exact-memory.md)
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
   On the table/span path, classification previously picked the argmax relation and act without checking whether the additive linear score gap $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$, the write-act gap $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$, or the span boundary margin $\Delta_{\text{span}}$ clears a positive threshold, and without checking whether the turn's words outside `span` actually form a speaker-grounded statement for the target relation. `SavedCompiler::table_action_with_margin` now computes the real `StatementMargin` directly from the compiler's `relation_head`, `act_head`, and `span_head` linear logits on `source`, and `SavedCompiler` carries a configurable `gate: MemoryTurnGate` field (`with_memory_turn_gate` / `set_memory_turn_gate`) so the two-arm comparison (Arm A `MemoryTurnGate::disabled()` vs Arm B `MemoryTurnGate::default()`) runs without source edits.

---

## 2. Integer / Additive Gate Design (`StatementMargin` + `MemoryTurnGate`)

Implemented in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs) with zero squashing functions, zero dense matrix multiplications, and zero core floating-point operations (D23/D25/D32 ratchets unchanged):

- **[`StatementMargin`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1640-L1690):**
  Computes integer milli-unit score gaps from linear class scores via `saturating_sub`, `saturating_add`, and `Iterator::max`, wired on the production path via [`SavedCompiler::table_action_with_margin`](../../../crates/uor-r4-training/src/relation_compiler.rs):
  - `relation_gap_milli`: $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$
  - `act_write_gap_milli`: $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$
  - `span_margin_milli`: $\Delta_{\text{span}} = \text{start\_margin} + \text{end\_margin}$
- **[`MemoryTurnGate`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1715-L2150):**
  1. **Score-gap margin gate (`check_margin`):** Requires $\Delta_{\text{rel}} \ge 250$, $\Delta_{\text{act}} \ge 250$, and $\Delta_{\text{span}} \ge 500$ whenever a `StatementMargin` is provided.
  2. **Structural memory-turn discriminator (`check_turn_structure`):**
     - Passes `QueryCurrent`, `Query`, and `Unresolved` through untouched.
     - Rejects candidate writes on interrogative turns (ending in `'?'` or opening with question/request cues such as `"what"`, `"where"`, `"who"`, `"remind"`, `"tell"`).
     - Rejects negated statements (`"not"`, `"never"`, `"isn't"`, `"don't"`, etc.), conditional/hypothetical turns (`"if"`, `"whether"`, `"unless"`, `"suppose"`, `"were"`), and third-person reported speech (`"said"`, `"told"`, `"claimed"`, etc.).
     - Rejects candidate writes whose frame attributes the relation to a third-person subject via general syntactic rules: any third-person subject/possessive pronoun (`"his"`, `"her"`, `"their"`, `"he"`, `"she"`, `"they"`), any possessive noun `<noun>'s` outside the closed-relation slot possessives (`"name's"`, `"dog's"`, `"cat's"`, `"pet's"`, `"friend's"`), or a kinship/person noun modified by `"my"`/`"our"` (with zero panel-specific noun blocklist).
     - Requires the words outside `span` to match the speaker-grounded lexical frame of the target closed relation (`1..=10`, [`canonical_relation_name`](../../../crates/uor-r4-training/src/relation_compiler.rs); bare `"I'm {v}"` / `"I am {v}"` for `user_name` requires a capitalized proper-name token so lowercase small talk like `"I'm tired."` is rejected, and `code_word` requires present-tense `"code word is"` / `"code word be"` grounding so narrative `"The code word in the film was Rosebud."` is rejected) or a first-person copular open-relation phrase (`relation >= DERIVED_RELATION_ID_BASE`).
- **[`gate_compiled_action`](../../../crates/uor-r4-training/src/relation_compiler.rs) & [`resolve_op_policy_gated`](../../../crates/uor-r4-training/src/relation_compiler.rs):**
  Any candidate `Assert` or `Correct` rejected by `MemoryTurnGate` is converted to `CompiledAction::Unresolved { reason }` and is **never** turned into a read (`QueryCurrent` / `Query`).

---

## 3. Finite Host Verification Summary (Pre-Flight Controls; GPU Measurement Pending)

Verified on host via `cargo test -p uor-r4-training --lib relation_compiler` (finite host pre-flight against synthetic adversarial controls and generator templates; the pre-registered GPU evaluation on `compiler-save-op-v25-rawtable` + `op-model-v25` awaits non-author `TEST FITNESS: FIT` and owner release on `#2855`):

| Test | Scope | Result |
|---|---|---|
| `memory_turn_discriminator_and_margin_gate_controls` | Additive score-gap margin checks ($\Delta_{\text{rel}}, \Delta_{\text{act}}, \Delta_{\text{span}}$); preservation of `QueryCurrent`, `Query`, and `Unresolved`; write-to-read prohibition on questions; rejection of `#1718`, negation, conditional, reported-speech, lowercase `I'm <adj>` small talk, and narrative `code_word` controls under `OpPolicy::UnlessQuery` | **PASS** |
| `memory_turn_gate_preserves_all_world_v2_relation_statement_templates` | Iterates directly over `milestone_world_v2::relations()` across `Split::Train` and `Split::Development` (`79 / 79` generator `assert` + `update` templates across all `10` relations), plus `17` teacher-paraphrase templates and `3` open-relation templates (`>= DERIVED_RELATION_ID_BASE`) | **PASS (`79 / 79` generator + `17 / 17` paraphrase + `3 / 3` open preserved, `0` false rejections)** |
| `saved_compiler_action_wires_configurable_gate_and_real_margin` | Fits and reloads a real `SavedCompiler` (`Table` and `OpModel` with `OpPolicy::UnlessQuery`), verifying that `SavedCompiler::action` computes a real `StatementMargin` from `relation_head`, `act_head`, and `span_head` linear logits and honors `with_memory_turn_gate` | **PASS** |
| `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel` | Host synthetic-adversary check: all `50` frozen rows across all `6` strata of [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) swept across all `10` closed relations and derived open relations (`1,100` adversarial candidate writes proposing the last word of each row) | **PASS (`0 / 50` false writes across all `22` host sweeps)** |
