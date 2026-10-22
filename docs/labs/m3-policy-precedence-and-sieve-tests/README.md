# Policy precedence and store-miss vs log-sieve tests (antigravity-6)

Status: **implemented and tested on host** (`crates/uor-r4-training`).

## 1. `OpPolicy` precedence and derived-ID guard (`relation_compiler.rs`)

`resolve_op_policy` (called by `SavedCompiler::action` in `crates/uor-r4-training/src/relation_compiler.rs`) combines `self.table_action(source)` and `self.op_action(source)` under four policies when `self.op.is_some()`:

| Table action (`table_action`) | Op action (`op_action`) | `OpPolicy::Op` | `OpPolicy::Table` | `OpPolicy::TableStatements` | `OpPolicy::UnlessQuery` |
|---|---|---|---|---|---|
| `QueryCurrent` / `Query` | Statement (`Assert` / `Correct`) | Op's statement | Table's query | Table's query (`!statement(&table)`) | Table's query (`matches!(table, QueryCurrent \| Query)`) |
| Statement (`Assert` / `Correct`), closed ID (`< DERIVED_RELATION_ID_BASE`) | Another statement (`Assert` / `Correct`) | Op's statement | Table's statement | Op's statement | Op's statement |
| Statement (`Assert` / `Correct`), closed ID (`< DERIVED_RELATION_ID_BASE`) | Non-statement (`QueryCurrent` / `Unresolved`) | Op's action | Table's statement | Table's statement | Table's statement |
| Statement (`Assert` / `Correct`), derived ID (`>= DERIVED_RELATION_ID_BASE`) | Statement (`Assert` / `Correct`, closed ID) | Op's statement | Table's derived statement | Table's derived statement (`!table_derived`) | Table's derived statement (`!table_derived`) |
| `Unresolved` ("table says nothing") | Statement (`Assert` / `Correct`) | Op's statement | `Unresolved` | `Unresolved` (`!statement(&table)`) | Op's statement (table did not read a query) |

Note on where `TableStatements` and `UnlessQuery` differ:
- When the table reads a query (`QueryCurrent` or `Query`) and the op model emits a statement, **both** `TableStatements` and `UnlessQuery` keep the table's query (`TableStatements` because `!statement(&table)` is true; `UnlessQuery` because `matches!(table, QueryCurrent | Query)` is true).
- The two policies differ when the table reads `Unresolved` ("table says nothing") and the op model emits a statement: `TableStatements` keeps `Unresolved` (trusting the table's non-statement classification), whereas `UnlessQuery` takes the op model's statement.
- Under both `TableStatements` and `UnlessQuery`, a table statement whose relation ID is derived (`*relation >= DERIVED_RELATION_ID_BASE`) is protected by `!table_derived` so the op model (which emits closed labels only) cannot overwrite an open-relation derived store address with a closed label.

Pinned by unit test `op_policy_precedences_and_derived_id_guard_resolve_disagreements` in `crates/uor-r4-training/src/relation_compiler.rs`.

## 2. Store-miss (`StoreRead::Absent`) vs `log_recall` sieve (`stack_grounded_session.rs`)

In `resolve_recall_line` (called by `GroundedSession::emitter_suffix` in `crates/uor-r4-training/src/stack_grounded_session.rs`):
- `MemoryEffect::Read { read: StoreRead::Absent }` returns `(Some("Memory: none.".to_owned()), RecallDisposition::Absent)` directly and **never** calls `log_recall`.
- `MemoryEffect::Unresolved if read` consults `log_recall` and, when the sieve returns a value, emits `(Some(format!("Memory: {value}.")), RecallDisposition::LogValue)`.

This coupling is intentional:
- `StoreRead::Absent` means the compiler resolved the query turn to a specific relation ID whose store slot has never been written (proven absence in the store). Falling back to the prior-turn text sieve (`log_recall`) on `StoreRead::Absent` would mask a missed statement write by recovering raw text from the dialogue log when the store slot itself is empty.
- `MemoryEffect::Unresolved` means the compiler declined to map the turn to any store action; only then is the optional `log_recall` sieve consulted.

Documented on `MemoryEffect`, `RecallDisposition`, and `resolve_recall_line`, and pinned by unit test `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` in `crates/uor-r4-training/src/stack_grounded_session.rs`.

## Sources

- Item: [m3-policy-precedence-and-sieve-tests](../../roadmap/items/m3-policy-precedence-and-sieve-tests.md) (#2857).
- `crates/uor-r4-training/src/relation_compiler.rs` (`OpPolicy`, `DERIVED_RELATION_ID_BASE`, `resolve_op_policy`, `SavedCompiler::action`).
- `crates/uor-r4-training/src/stack_grounded_session.rs` (`MemoryEffect`, `RecallDisposition`, `resolve_recall_line`, `GroundedSession::emitter_suffix`).
