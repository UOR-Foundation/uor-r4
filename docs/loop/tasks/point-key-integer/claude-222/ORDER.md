# ORDER: point-key-integer, claude-222 (the integer point table, its search and its record)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Base: `437d1ddf5`.

**Written by hand.** `loop claim point-key-integer` refused on a false overlap: it read Kimi's open docs-status-refresh glob `crates/*/README.md` (#3436) as covering `crates/uor-r4-integer/src/stack/session.rs`. No README is in this order's scope. The harness is not changed (Casey: no tooling work).

## Goal

The item's done-when 1, on the laptop with finite tests and no run:
- the integer point table and its search, served at `stack_step_fbs`'s first reply step where the turn memory's exact key misses;
- the artifact record that carries the table;
- the builder that makes the table from the integer trunk's own points.

The TRAIN export pass, the panels and the bar are the next unit's, under a prep sheet.

## Cancel probe (run first)

- No open PR touches `crates/uor-r4-integer/src/{stack,bin,wasm*}`, and no open order but Kimi's (docs only) holds these files.
- The base serves the turn memory at `/7`, with no point table:
  - `STACK_FBS_MEMORY_SCHEMA` is `uor-r4.lut-stack/7`;
  - `memory.apply` returns `MemoryMove { opening: true, stored: None }` on an exact-key miss and moves nothing;
  - `stack_step_fbs` drops `moved` outside tests.
- No `point_key.rs` and no `fbs-point-key` exist.

If any fails, stop and report.

Run on 31 Oct at the base:
- no open PR touches the integer stack, binaries or WASM files;
- `format.rs:65` is `uor-r4.lut-stack/7`;
- `memory.rs` `apply` returns `stored: None` on a miss and moves nothing;
- `session.rs:3165` and `:3320` drop `moved` outside tests;
- `point_key.rs` and `bin/fbs-point-key.rs` are absent.

Result: **go**. The probe reads the code, so it is an instrument check.

## The design

1. **`stack/point_key.rs`, the served table.**
   - It holds TRAIN's opening points at the writer query's Q12 (`n × width` i16) and their targets (u32).
   - **The search:** the least L1 distance over every row (the `-L1` of `stack_whole_table_nearest`, summed exactly in i64), then the commonest target among all rows at that distance, ties to the smaller id. That is float's `l1_named` rule.
   - It uses only adds, subtracts, absolute differences and compares, with no multiply.
   - A named BOS, or a table holding BOS or an id past the vocabulary, is refused.
2. **The step.** In `stack_step_fbs` and its timed twin, after `stack_native_writer`: where the opening read returned `MemoryMove { opening: true, stored: None }` and the artifact carries a point table, the search over `b.query_q12` names the token.
   - `moved` is kept in every build.
   - Nothing else in the step changes, and an artifact without the table decodes as before.
3. **The record.** An optional header record `point` (rows, width, the Q12 exponent) with sections `memory.point.points` (i16, little-endian) and `memory.point.targets` (u32, little-endian).
   - It needs the turn memory.
   - A stack with it is `uor-r4.lut-stack/8`, which the `/7` engine refuses by its schema check.
   - The writer writes it, and parsing checks every size and id.
   - The table round-trips byte for byte.
4. **The builder, `fbs-export --point-key`.** It reads a `/7` artifact and a TRAIN split, and writes the `/8` artifact.
   - It is a mode of `bin/fbs-export.rs`, not a new binary. The crate sets `autobins = false`, so a new binary needs a `Cargo.toml` entry outside the item's scope (amended at slice C).
   - Its logic is the library's `point_key_windows` and `build_point_key`, so the tests reach it.
   - It replays each window that holds an opening through the integer session, with the memory off. The windows and openings are float's `windows_of` and `opens_reply` rule, ported as integer position logic.
   - At each opening row it takes `b.query_q12` and the reply's first token.
   - Here it is tested on the synthetic stack with a tiny split only.
5. **Restore.** The point key keeps no session state. The test pins that a `/8` session is refused on restore exactly as a `/7` memory session is.

## Files in scope

- `crates/uor-r4-integer/src/stack/{point_key.rs (new), format.rs, memory.rs, session.rs, mod.rs, tests.rs, synthetic.rs}`
- `crates/uor-r4-integer/src/bin/fbs-export.rs` (the `--point-key` mode; the item lists it). It replaces a new `bin/fbs-point-key.rs`, whose `[[bin]]` entry would need `Cargo.toml`, outside the item's scope.
- `crates/uor-r4-integer/src/wasm_serve.rs` and `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`, only if `memory_info` or the restore refusal must name the table
- `docs/loop/tasks/point-key-integer/claude-222/**`
- `docs/roadmap/items/point-key-integer.md` (a progress note only)

## Done when

- **Unit tests,** each with a negative gate (the rule set aside, the test failing, then restored):
  - the search's least distance, its commonest-target pass and its tie to the smaller id;
  - an opening the exact key fires on is left to the memory;
  - the point key is served only at the reply's first step, and only on an exact-key miss;
  - BOS and an out-of-range id are refused;
  - the `/8` round trip is byte for byte;
  - the `/7` engine refuses a `/8` artifact;
  - the builder's points on the synthetic stack equal the session's own `b.query_q12` at each opening;
  - restore is refused.
- **The suites:** `cargo test --release --locked -p uor-r4-integer` passes. So do the stack's disassembly and zero-multiply audits, with the search reached, and the WASM build the CI runs.
- **An artifact without the table** decodes token for token as at the base (a test).
- **The REPORT** gives the negatives first. A fresh review runs in slices of at most 600 lines.

## Stop and ask if

A file outside scope is needed, or an audit needs a rule change (§3 is the owner's), or float's window rule cannot be ported without float code in the integer crate.
