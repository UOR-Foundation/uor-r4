# REPORT: point-key-integer, claude-222 (the integer point table, its search and its record)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refused on a false overlap with Kimi's #3436 glob `crates/*/README.md`.

This unit is the item's done-when 1, on the laptop, with finite tests and no run.

## Tokens per second

No measurement. The search's cost, the laptop's integer decode against context, and the table's resident memory are done-when 5, measured in the next unit on the real table.

The search scans every row at each keyed opening. That is one keyed opening a reply, and no other step.

## What did not work, first

- **Builder subagents cannot write in this worktree.**
  - The loop guard allows a subagent to write only under a suborder in `local/claude/claude-222/orders/`.
  - Each builder also starts at about 142k tokens of tool context.
  - The first builder stopped before writing anything, so all three slices were built in the main session.
- **The serving audit refused the first search.** `ServedPointKey::name` compiled to a vectorized L1 sum, which left through `fmov x8, d0`, a float-register move. `core::hint::black_box` on the query value keeps the sum scalar, as `stack_whole_table_nearest` already does. The audit then passed.
- **No new binary.** The crate sets `autobins = false`, so `bin/fbs-point-key.rs` would need a `[[bin]]` entry in `Cargo.toml`, outside the item's scope. The builder is instead a mode of `fbs-export`, which the item lists, and the ORDER is amended.
- **The stack builds only with `--features serving-wasm`.** A test filter without it runs none of the stack's tests, silently.

## What worked

- **The table and its search** (`stack/point_key.rs`).
  - `PointKeyImage` holds `rows × width` Q12 points and their targets. A table with BOS or an id past the vocabulary is refused.
  - `ServedPointKey::name` finds the least L1 distance over every row, then the commonest target among the rows at that distance, ties to the smaller id. That is float's `l1_named` rule.
  - It uses adds, absolute differences and compares only. Its scratch is a session buffer sized to the table's rows, so the step allocates nothing.
- **The step** (`session.rs`).
  - Where the opening row's move shows the exact key missed (`opening`, `stored: None`) and the stack carries a table, the key names the reply's first token from that row's own Q12 query. This happens after the native writer, in `stack_step_fbs` and its timed twin.
  - The memory's last move is kept in every build.
  - Every other row, an opening the key hits, and a session without the reply-start signal decode exactly as `/7`.
- **The record** (`format.rs`).
  - `uor-r4.lut-stack/8` carries a `point` record (rows, width, exponent −12; unknown fields denied) and the sections `memory.point.points` and `memory.point.targets`. The record needs the turn memory.
  - `add_point_key_uorlut01` appends the table to a `/7` stack, keeping every other byte, and reads the result back.
  - The `/7` engine refuses a `/8` stack by its schema check.
- **The builder** (`fbs-export --point-key ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH`).
  - `point_key_windows` ports the float sheet's `build_document_windows` and `opens_reply` as position logic.
  - `build_point_key` steps each window in a fresh session with no reply-start signal, so the memory moves nothing, and keeps the writer's Q12 query at each opening row. Those are the integer trunk's own points, as condition (b) requires.

## What changed

- `crates/uor-r4-integer/src/stack/point_key.rs` (new): the image, the served search, the window rule and the builder.
- `stack/format.rs`: `STACK_FBS_POINT_SCHEMA`, `POINT_KEY_EXP`, `StackPointKey` and its check, the header and container field, the schema choice, and `add_point_key_uorlut01`.
- `stack/memory.rs`: `MemoryImage::point` and `ServedTurnMemory::point`.
- `stack/session.rs`: the override in both steps, `last_move` in every build, and the `point_tied` buffer.
- `stack/mod.rs`: the module and its exports.
- `stack/tests.rs`: five tests.
- `bin/fbs-export.rs`: the `--point-key` mode.
- `ORDER.md`: the builder's file, amended at slice C.

## Measurements

- **The crate's library tests,** `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib`: 380 pass, 0 fail, 1 ignored. At the base: 372 pass. The 8 new tests are 3 in `point_key.rs` and 5 in `tests.rs`.
- **The serving audit,** `python3 scripts/audit_zero_matmul_serving.py local/target/release/uor-r4-stack --stack`: FULL PASS.
  - 0 forbidden instructions across 44 symbol ranges and 116 reachable functions. `ServedPointKey::name` is among them; claude-198 counted 109.
  - The allowed scalar multiplies and divides (D80, D81) are 47 / 1 in the ranges and 110 / 13 in the call graph.

## Checks run at the head

- **Eighteen negative gates,** each failing with its rule set aside, and passing when restored:
  - **the search:** ties to the smaller id; a nearer row resets the tied set; BOS refused; an id past the vocabulary refused;
  - **the record:** `/8` is the point stack's schema; the record needs the turn memory; the rows its targets hold; unknown fields denied; the old sections kept (a one-byte shift makes the `/8` stack unreadable);
  - **the step:** the override; only on a miss; only at the opening; the timed twin's override; a `/8` session's restore refused (the base's rule set aside);
  - **the builder:** an opening follows a prompt token; the prefix never crosses the document's start; read at the opening row; a mask byte past 1 refused.
- **The review's fixes** (`1c39f5a0b`):
  - the UORT token count is a checked multiply;
  - a mask byte other than 0 or 1 is refused;
  - the search's no-error invariant is noted at the step;
  - a test assertion that could not fail is dropped.

  The UORT check is in the binary, so no unit test reaches it.
- **The WASM library build:** `cargo check --target wasm32-unknown-unknown --features serving-wasm --lib` finishes.
- **Clippy:** no new warning in the changed files.

## Skills used

None.

## Still open

- **The next unit (claude-223),** under a prep sheet with a fresh FIT and a pre-run review, on the laptop's bit-exact integer engine, by the owner's 31 Oct allowance:
  - the export pass over TRAIN's openings, and its cost;
  - parity against claude-220's float served names (reported, with its classes fixed in the item);
  - the deciding bar on a fresh panel p-x at seeds 1–3;
  - tok/s and resident memory.
- **The artifacts that pass needs:** (r-drop)'s seeds exported to `/7`. This unit builds and tests only the synthetic stack.
- **The CUDA evaluator,** for speed: its own item, later.

## Plain summary

The float model learned to start its reply by looking up which training questions its internal state is closest to. This unit builds the same lookup into the shipped whole-number model.

It adds a table of the model's own states at the start of each training reply, and a search that picks the most common first word among the closest ones. The file format gains a new version that carries the table, and a tool builds the table from the training data.

Every piece is checked by tests, including the check that the shipped model's arithmetic stays whole-number only. Running it on the real model and the real questions is the next step.
