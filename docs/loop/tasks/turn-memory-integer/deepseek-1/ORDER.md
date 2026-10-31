# ORDER: turn-memory-integer, deepseek-1 (serve /7 turn memory, units 2-3)

Lab: deepseek. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392). Base: e16ada251. Follows claude-196's decision and claude-198's unit 1 (#3392), which packed the memory into `uor-r4.lut-stack/7` and left serving to units 2 and 3. Owner direction, 2026-10-30: "/7 turn memory stacks, please fix".

## Goal

Serve the turn memory in the integer engine: `IntegerStackModel::parse` builds the memory tables instead of refusing; `IntegerStackSession` takes the caller's reply start, forms the item's opening and reply keys (turn anchor after the last bos/eos in training's 192-token key block; reply key turn ids, MAX, window-visible reply length, MAX, row id), and applies the opening/reply moves after the Hamming readout, integer-only. A synthetic /7 artifact helper, a test-side reference of the float rule with the item's three rows (reply cut by the window, prompt with an internal eos, context not 192), the move/order tests and six negative gates. Restore of a memory session stays refused until its turn state is saved (the item allows it). The wasm surface exposes the session role signal and memoryInfo; the Studio worker wiring is a separate unit. Units 4 (DEV parity under a pre-registered sheet) and 5 (cost) stay open, so the item remains ready.

## Cancel probe

- Origin/main still refuses `/7` in `stack/session.rs` ("does not serve the turn memory yet") and `stack::memory::memory_image` already reads the tables (claude-198); the turn-memory item is ready and unclaimed in the ledger.
- The float rule is in `flock_read_memory.rs` / `in_space_long.rs`; no GPU run is in this unit.
If either changes, stop and report.

## Files in scope

`crates/uor-r4-integer/src/stack/{memory,format,session,synthetic,tests,hamming,mod}.rs`, `crates/uor-r4-integer/src/{wasm_serve,wasm_bindgen_exports}.rs`, `docs/loop/tasks/turn-memory-integer/deepseek-1/**`, `docs/roadmap/items/turn-memory-integer.md`, `docs/research/notebook/m4-serving.md`.

## Done when

- `cargo test -p uor-r4-integer --features serving-wasm --lib` passes (372 expected; the four new /7 tests and six negative gates included);
- the test-side reference covers the item's three rows and the move's float form; set aside gates each fail their test;
- a built serving-wasm bundle reports schemas /1-/7 with the memory served and passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7, float 0, the move reached: mul 26 / div 12 in the step range);
- restore of a /7 session is refused with the recorded reason; no trained-checkpoint or GPU claim is made;
- `cargo fmt` on the changed crate, claim-wording, git diff --check and `loop pr-check --local` pass; five required checks carry `policy: pass` at the head.

## Stop and ask if

The keys or moves cannot be made exact against the float rule, a test must be weakened, an owner path or private artifact would move, or a GPU run would be needed for the unit's done-when.
