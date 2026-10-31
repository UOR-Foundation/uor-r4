# ORDER: integer-nearest-readout, claude-231 (the readout, its field and its export, on both serving paths)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `54f20ac7b`.

**Written by hand,** if `loop claim` refuses on the false overlap with #3436's `crates/*/README.md` (as it has for every unit since claude-222).

## Goal

The item's done-when 1, 2 and 4, on the laptop, with finite tests and no run. Parity (done-when 3) is the next unit's, under a sheet.

## Cancel probe (run first)

- At the base, the FBS step's token is the native writer's (`stack_step_fbs`, `stack_native_writer`), with `stack_whole_table_nearest` only on the timed path.
- `validate` refuses any `head` other than `"native_vector"`.
- For an FBS model, `IntegerStackSession::step` returns an empty logits slice, and `wasm_step` takes `stack_argmax` of it, which returns 0.
- No open PR touches the scope.

If any fails, stop and report.

Run on 31 Oct at the base:
- `session.rs` sets `b.native_token = stack_native_writer(...)` in `stack_step_fbs` and in its timed twin;
- `format.rs:425` refuses a `head` other than `native_vector`;
- `step` returns `&self.b.logits[..0]` for FBS, and `stack_argmax(&[])` returns 0 (`kernels.rs:1577`);
- no open PR touches the scope.

Result: **go**.

## The design

1. **The field:** a new `head` value, `"native_vector_nearest"`, beside `"native_vector"`.
   - `head_native()` is true for both, so every native-head path holds. A new `head_nearest()` is true for the new value only.
   - `validate` allows it, and an engine without this unit refuses it by its own `validate` (`format.rs:425`).
2. **The export:** `FbsTrunkSnapshot.readout_nearest: bool`, with `serde(default)`, not written when false. The FBS export writes `head` from it, so every earlier snapshot exports byte for byte as at the base. That is pinned by claude-226's base hash test.
   - `fbs_snapshot::run_readout_nearest(config_json)`: the run's `objective`, where `cell` or `nearest` is true, another known objective false, and absent false.
   - `fbs-export --run-config CONFIG SNAPSHOT OUT` sets the snapshot's flag from it, for the snapshots in the store.
   - `fbs-snapshot` sets it too, for later snapshots.
3. **The step:** under `head_nearest()`, `b.native_token = stack_whole_table_nearest(fbs, vocab, &b.query_q12)` in place of the writer, in `stack_step_fbs` and in its timed twin.
   - It reads the same final query (after the memory's move and the Hamming read), and the point key's override follows unchanged.
   - `native_kept` is zeroed, since no cells are kept.
4. **The browser path:** for an FBS model, `wasm_step`'s `next` is the session's native token, the writer's or the nearest by the head. That fixes the writer case too: today `stack_argmax(&[])` gives 0 for every FBS step.
   - Its `margin_q16` is 0 for an FBS model, documented as such, since the step has no logits.
5. **Tok/s** (done-when 4): `fbs-toks` on a synthetic nearest stack against the writer one, with the nearest scan counted (`--stages` already times it).

## Files in scope

- `crates/uor-r4-integer/src/{fbs_snapshot.rs, wasm_serve.rs, wasm_bindgen_exports.rs}`
- `crates/uor-r4-integer/src/stack/{format.rs, session.rs, tests.rs, synthetic.rs}`
- `crates/uor-r4-integer/src/bin/{fbs-export.rs, fbs-toks.rs}`
- `crates/uor-r4-float-train/src/{fbs_export.rs, bin/fbs-snapshot.rs}`
- `docs/roadmap/README.md`, `docs/roadmap/issues.json` (the sync's write from claude-230, #3462)
- `docs/loop/tasks/integer-nearest-readout/claude-231/**`

## Done when

- **Tests,** each with a negative gate:
  - the field: `validate` accepts the new head, and refuses an unknown one;
  - a snapshot without the flag exports byte for byte as at the base (claude-226's pinned hash);
  - a nearest stack's every served token is the whole table's L1-nearest, by a test-side reference scan, ties to the smaller id;
  - the point override still applies under nearest;
  - `wasm_step`'s token equals `step_native`'s, for a nearest and a writer FBS stack;
  - the config reading.
- **The suites:** `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib`, and the float crate's tests, pass. So do the serving audit, the WASM build, the stub-nvcc check and `cargo fmt --check`.
- **The REPORT** gives the negatives first, and a fresh review runs in slices of at most 600 lines.

## Stop and ask if

A file outside scope is needed, or the audit refuses the nearest scan in the served step.
