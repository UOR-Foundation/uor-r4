# REPORT: integer-nearest-readout, claude-231 (the readout, its field and its export, on both serving paths)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap.

This unit is the item's done-when 1, 2 and 4, on the laptop, with finite tests and no run. Parity (done-when 3) is the next unit's, under a sheet.

## Tokens per second

`fbs-toks --stages` on (r-drop) s1's `/7`, exported both ways from claude-227's snapshot, on an Apple M1 (4 performance cores). The protocol is a 128-token prefill and 64 decode steps, with 1 warmup and 20 timed runs.

| Head | Prefill (context) | Decode | Step median | Whole-table scan |
|---|---|---|---|---|
| `native_vector` (writer) | 4,107 tok/s | 3,889 tok/s | 258 µs | 486 µs, beside the step |
| `native_vector_nearest` | 1,405 tok/s | 1,365 tok/s | 730 µs | 485 µs, inside the step |

The nearest readout costs about 2.85× a token: the scan reads 4,096 rows of 256 values, and stays scalar for the serving audit. Prefill pays it at every position, though only the last token is served (Still open).

## What did not work, first

- **The browser path served token 0 for every SpiralCore/FBS model.**
  - For an FBS model, `IntegerStackSession::step` returns an empty logits slice; its choice is read by `step_native`. `wasm_step` took `stack_argmax` of that empty slice, which is 0 (`kernels.rs`).
  - So an FBS stack loaded in the Studio (which accepts `uor-r4.lut-stack` artifacts and steps them through `wasm_step`) predicted token 0 at every step, with a margin of top1 − top2 over no values.
  - This is fixed here for both heads (`serve_step`). It was found while reading the path, not by a test of the Studio, and it was not checked in a browser.
- **`StackShape` has no `deny_unknown_fields`.** So a new field would be ignored by an older engine, not refused. The readout rides on `head`'s value instead, which `validate` already checks (as claude-226 rode on `rec_gates`).
- **`fbs-compare`'s readouts are still the writer's.** For a nearest artifact, its token parity would compare the integer nearest with the GPU writer. `fbs-compare` is outside this item's scope; the item's own parity reads claude-220's stored float greedy replies instead.
- **`cargo fmt -p uor-r4-float-train`** also rewrites four float files outside this order's scope (`in_space_long.rs`, `train-in-space.rs`, `flock_read_hamming.rs`, `flock_read_memory.rs`), which carry fmt drift from before this unit. They are not committed. The guard refuses to restore them, so they stay unstaged in this worktree until it is removed.
- **The point-key override under nearest has no negative gate of this unit's.** No line of this unit guards it; its test pins the behaviour as a regression check.
- **The browser margin is 0 for an FBS model,** documented as such: the step has no logits. A distance margin (the second-nearest gap) is not computed.

## What worked

- **The field:** `head = "native_vector_nearest"`.
  - `head_native_vector()` is true for both heads, so every native path holds. `head_nearest()` is true for the new value only.
  - `validate` allows it. An engine without this unit refuses it by its own `validate` ("the head is not native_vector"), which is pinned on an unknown head.
- **The step.** Under the field, `stack_step_fbs` and its timed twin serve `stack_whole_table_nearest(fbs, vocab, &b.query_q12)` in place of the writer, and zero `native_kept`.
  - It reads the same final query, after the memory's move and the Hamming read.
  - The point key's override follows unchanged.
  - The rule matches the float side's `nearest_scores` (negative mean L1, then `argmax`).
- **The export:**
  - `FbsTrunkSnapshot.readout_nearest`, not written when false.
  - `run_readout_nearest(config)`: `cell` and `nearest` read nearest, `sheet` and `voronoi` the writer, and a config without an objective the writer. Anything else is refused.
  - `fbs-export --run-config CONFIG` sets the flag for claude-227's snapshots, and refuses a config whose `retain_rates` disagrees with the snapshot's. `fbs-snapshot` sets it for later ones.
- **Byte identity on a trained model:** (r-drop) s1's snapshot exported without the flag at this head gives `19174e5e…`, claude-229's `/7` from before this unit. With `--run-config` it gives `0064b00a…`, head `native_vector_nearest`.
- **The browser path:** `IntegerStackSession::serve_step` is what `wasm_step` serves. It returns a dense stack's argmax and margin, or an FBS model's native token (writer or nearest) with margin 0. A native test exercises exactly what the browser runs.

## What changed

- **`crates/uor-r4-integer/src/stack/format.rs`:** `validate`, `head_native_vector`, `head_nearest`, and the FBS export's `head` (shape and source block).
- **`stack/session.rs`:** the nearest branch in `stack_step_fbs` and its timed twin, and `serve_step`.
- **`wasm_serve.rs`:** `wasm_step` forwards to `serve_step`.
- **`fbs_snapshot.rs`:** `readout_nearest` and `run_readout_nearest`.
- **`stack/synthetic.rs`** and the float **`fbs_export.rs`:** the field.
- **`bin/fbs-export.rs`:** `--run-config`.
- **The float `bin/fbs-snapshot.rs`:** the run's objective.
- **`stack/tests.rs`:** four tests, and two assertions added to claude-226's byte-identity test.
- **`docs/roadmap/{README.md, issues.json}`:** claude-230's sync write, with #3462 numbered.

## Measurements

- **The integer crate's library tests** (`--features serving-wasm`): 394 pass and 1 is ignored, against 390 at the base.
- **The float crate:** 186 pass.
- **The serving audit:** FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 118 reachable functions (116 at claude-228; the served step now reaches the scan).

## Checks run at the head

- **Six negative gates,** each failing at its own assertion with its rule set aside, and passing when restored:
  - **G1:** `validate` accepting any head, at `tests.rs:5596`;
  - **G2:** the served step's nearest branch, at `tests.rs:5633`;
  - **G3:** `serve_step`'s FBS branch, at `tests.rs:5637`;
  - **G4:** `cell` reading nearest, at `tests.rs:5672`;
  - **G5:** the export writing the nearest head, at `tests.rs:5591`;
  - **G6:** the writer case's bytes against claude-226's base hash, at `tests.rs:5576`.
- **The WASM library build** is clean, with no warnings; the old dead-code warning on the scan is gone.
- **The stub-nvcc CUDA check** of the float crate (`--all-targets`) finishes. Its only warnings are in vendored candle.
- `cargo fmt --check -p uor-r4-integer` is clean.

## Skills used

None.

## Still open

- **Parity (done-when 3),** under a prep sheet with a fresh FIT. Integer greedy of the nearest `/7`, memory on, against claude-220's stored float greedy on DEV and p-w at seeds 1–3. The bar is registered before reading.
- **Speed:** skip the scan on prefill steps, whose tokens are not served, which would bring context back near the writer's rate. A faster scan within the audit's allowances is a separate step.
- **`fbs-compare`'s readouts** are the writer's; a nearest artifact needs nearest readouts there.
- **A note to DeepSeek** about the Studio's FBS path, once this merges.

## Plain summary

The converted model now picks its next word the way the trained model does: the closest word in its whole vocabulary. Before, it used a different picker.

The browser version had a separate bug for this kind of model: it always answered with the same word, number 0. Both are fixed here.

Picking from the whole vocabulary is about three times slower for now. Whether the converted model now answers like the trained one is checked next, against answers the trained model already gave.
