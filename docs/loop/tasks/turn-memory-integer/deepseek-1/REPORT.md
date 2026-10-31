# REPORT: turn-memory-integer, deepseek-1 (serve /7 turn memory, units 2-3)

Lab: deepseek. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392). Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Rounding the move's pull alone was wrong.** Forming `n + round(w*(c-n))` disagreed with the float readout at half ties; the move now forms the whole sum in i128 and rounds once (half away from zero), which the test-side f64 reference caught on 9 of 32 coordinates.
- **The first key test could not distinguish the opening table's k from the reply table's reply_turn.** It now uses an 8-id turn with k=4 so the reply key's turn is 8 ids; the set-aside gate turns it red.
- **The anchor and the float trainer's mask rule are not the same rule.** The session uses the item's turn anchor after the last bos/eos for both tables (claude-196's decision); the trainer's reply rows use the response mask, which can reach past a prompt token that is itself an eos. A test pins that gap instead of hiding it; the parity unit measures it.
- **A memory session cannot be restored yet.** save_state succeeds but does not carry the caller's reply start or the anchor ids, so restore refuses by text (the item allows this until that state is saved).
- **`cargo test -p uor-r4-integer` without `--features serving-wasm` is invalid on this tree** (pre-existing: the stack module is absent, so `stack_chat_reference.rs` fails to compile; verified identical on origin/main). All runs below use the feature.
- Limits: units 4 (DEV parity against the sealed claude-194 checkpoints under a pre-registered sheet; also blocked until integer-hamming-read step 4 puts the Hamming read in the exporter) and 5 (cost) are not run; the keys/moves are verified against the test-side reference and synthetic /7 artifacts, not a trained checkpoint.

## What changed

- `stack/memory.rs` (+233): the served tables and the key/move rules.
- `stack/session.rs` (+162): model field, parse, role signal, the two step paths, restore refusal, wasm-facing accessors, test hooks.
- `stack/hamming.rs` (+1/-1): `round_shift` made `pub(super)` for the move's rounding.
- `stack/synthetic.rs` (+81): a synthetic /7 artifact helper.
- `stack/tests.rs` (+381): test-side reference, move/order tests, six negative gates, updated round-trip test.
- `wasm_serve.rs` (+49) and `wasm_bindgen_exports.rs` (+26): /7 sessions plus `memoryInfo`/`markReplyStart`/`clearReplyStart`.
- `stack/format.rs`, `stack/mod.rs`: unchanged exports only as the helper needs.
- Records: item Progress section, notebook row, this ORDER/REPORT.

## Measurements

- Engine lib tests: 372 passed / 0 failed / 1 ignored (origin/main baseline 368 -> +4 new).
- Full crate: 578 passed / 35 failed vs baseline 574 / 35 — identical pre-existing missing-input failures in the same 7 binaries; no new failure.
- Wasm: 506,483 B sha256 `7b3e0bce…` at the engine head (+11,149 B over the /7-refusing 495,334 B bundle); audit exit 0, 7/7, step mul 26 / div 12 / float 0 (was 23/10), unclassified 0.
- Native stack audit exit 0: 44 ranges, D80/D81 mul/div 47/1 ranges and 109/12 call graph (was 102/10), 113 reachable functions, 0 violations.
- fmt clean on the changed crate. No GPU run, no trained-checkpoint parity (units 4-5 open).

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib` -> 372 passed / 0 failed / 1 ignored.
- `cargo fmt -p uor-r4-integer -- --check` -> clean (the workspace `fmt --all` diffs are pre-existing in untouched docs/labs instrument files).
- wasm build with `RUSTFLAGS='-C target-feature=-simd128' wasm-pack ... --features serving-wasm` -> 506,483 B sha256 `7b3e0bce…`.
- `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap` -> 7/7, float 0.
- native stack audit (`audit_zero_matmul_serving.py --stack`) -> exit 0, 0 violations.
- The six negative gates were each set aside once and turned their named test red (recorded in the engine unit's run).
- `loop pr-check --local` / claim-wording / git diff --check -> pass at the head; five required checks carry `policy: pass`.

## Skills used

working-discipline, scope-guard, solution-integrity, uor-research-architect; engine run details at the unit's session log.

## Still open

- **Unit 4:** DEV parity against the sealed claude-194 (r) checkpoints at step 16,384, both seeds, under a pre-registered sheet (GPU; blocked until integer-hamming-read step 4 carries the Hamming read in the exporter).
- **Unit 5:** the cost measurement (`fbs-toks --stages` with the memory) and the artifact's added bytes.
- The turn state is not saved yet, so a memory session's restore/export/import is refused; the Studio worker wiring (memoryInfo, markReplyStart, no restore) is a separate unit.
- No trained /7 artifact exists; the serving is verified on synthetic artifacts and the test-side reference.

## Plain summary

The integer engine now serves the `/7` turn memory: it builds both tables, takes the caller's reply start, forms the item's keys and applies the opening/reply moves after the Hamming readout, with a test-side float-rule reference, six negative gates and a wasm audit that sees the move. Units 4 (GPU parity) and 5 (cost) remain open, so the item stays ready; a memory session's restore is refused until its turn state is saved.
