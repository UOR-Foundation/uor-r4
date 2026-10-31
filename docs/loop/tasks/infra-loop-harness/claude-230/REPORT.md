# REPORT: infra-loop-harness, claude-230 (record the integer-nearest-readout item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses while Kimi's #3436 holds `PLAN.md`.

This is a records unit: one new item, and one `depends_on` with a line in an existing item. No code.

## Tokens per second

No measurement. This unit records an item.

## What did not work, first

- **The fault this item records was found by a run, not by reading the code before it.**
  - claude-225 found the retention gap by reading the code before any run. This gap passed through claude-225's to claude-229's planning, although PLACE.md's served-against-trained table had listed it since 29 Oct ("the served session still reads the native writer").
  - It cost claude-229's run: the run's instruments held, and its grade read nothing (row 0, #3460).
  - The new item's parity is end to end against the trained model's own replies, so it would also catch a gap of this kind elsewhere in the served model.
- **The new item's float side is claude-220's stored GPU replies,** not a new float run. They cover DEV and p-w at seeds 1–3, at claude-220's prompts and settings. A parity on other prompts would need a new float run on the GPU.
- **The existing snapshots carry no objective.** The item leaves the build unit to choose between reading the run config at export time and re-snapshotting on a pod.
- **The first draft could close on a failed parity, and left the browser out** (this unit's review).
  - Its done-when 3 only reported agreement against a bar. It now requires the bar to pass, and the item stays `ready` on a miss.
  - `wasm_serve`'s step takes the session's logits and their `stack_argmax`, not the session's token, so a nearest readout inside `stack_step_fbs` alone would not reach the Studio. `wasm_serve.rs` and `wasm_bindgen_exports.rs` are now in scope, with a done-when check that the browser's token equals the session's nearest.
- **The float side's measure is now cited,** not assumed: `nearest_scores` (`in_space_long.rs`), the negative mean L1 to every table row, then `argmax`. It is the integer rule's measure.

## What worked

- **The item, `integer-nearest-readout`:** "Serve the trained model's readout in the integer artifact".
  - Its fields: research, serving, `needs_gpu: false`, notebook `m4-serving.md`, `depends_on: [integer-retention-rates, one-decoder]`.
  - **What is missing:** the readout field, read from the run's objective; the nearest step under it; and end-to-end token parity.
  - **Done when:** the field and its refusal; the nearest step with a reference scan, the audit and the WASM build; parity under a sheet with a fresh FIT; tok/s with the scan counted; one notebook row.
- **Its sources are checked against the code and the store:**
  - `decodes_nearest` (`in_space_long.rs`, `Nearest | Cell`);
  - `stack_whole_table_nearest`'s doc comment (`stack/session.rs`);
  - claude-220's float replies at `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/out/r-drop-s{1,2,3}/`, listed in the store;
  - (r-drop)'s `objective: "cell"`, from its run config.
- **`point-key-integer`** gains `integer-nearest-readout` in its `depends_on`, and its unit 5 names the item and adds `I-Greedy-Parity` to the rerun.

## What changed

- `docs/roadmap/items/integer-nearest-readout.md` (new).
- `docs/roadmap/items/point-key-integer.md`: `depends_on`, and unit 5.
- `docs/roadmap/README.md`: re-rendered (118 items).

## Measurements

None.

## Checks run at the head

- `xtask roadmap check`: 118 items OK.
- `roadmap render --check`: current.
- **The sync's dry run** plans exactly: create `integer-nearest-readout`, and update #3450's body. It is applied after the merge.

## Skills used

None.

## Still open

- **`integer-nearest-readout`'s build unit:** the field, the nearest step, the audit, the WASM build and tok/s.
- **Then its parity unit,** on the laptop under a sheet.
- **Then point-key-integer's rerun** on p-x, with `I-Greedy-Parity`.

## Plain summary

The converted model picked its words differently from the trained model, which is why the last test told us nothing. This records that as its own task: teach the converted model the trained model's way of picking words. Then check it word for word against answers the trained model already gave.
