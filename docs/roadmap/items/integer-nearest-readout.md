---
id: integer-nearest-readout
title: Serve the trained model's readout in the integer artifact
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [integer-retention-rates, one-decoder]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/wasm_serve.rs, crates/uor-r4-integer/src/wasm_bindgen_exports.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, docs/roadmap/items/integer-nearest-readout.md, docs/harness/prep/integer-nearest-readout*, docs/labs/integer-nearest-readout/**, docs/loop/tasks/integer-nearest-readout/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The project's first hurdle is one working model end to end, served from its integer artifact. The served model must decode as the trained one does, and for (r-drop) it does not.

- **The trained model decodes by nearest.** (r-drop) trained with `objective: "cell"`, and `in_space_long` decodes such a model as the whole table's nearest token: `decodes_nearest`, "Whether replies decode as the whole table's nearest token, not the writer's pick". [`one-decoder`](one-decoder.md) (claude-150) made whole-table nearest the float side's served decoder.
- **The integer session decodes by the native writer.** The served step emits the writer's token (`stack/session.rs`, `stack_step_fbs`). It computes whole-table nearest only beside the writer, for timing (`stack_whole_table_nearest`: "the readout a model trained with the cell objective decodes, timed beside the writer").
- **So the integer artifact is not the trained model.** In [`point-key-integer`](point-key-integer.md)'s run (claude-229, #3460), integer greedy's first token equalled claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 on p-w. The float's own first token equalled the reference on 129 and 179.
  - The run's grade read nothing, and it ended at row 0.
  - claude-227's trunk parity compared the writer's token on both sides, so it could not see this.

## What is missing

- **The readout in the artifact:** a shape field saying whether the stack decodes by its writer or by whole-table nearest, read from the run's objective (`cell` or `nearest` means nearest), as claude-226 read `retain_rates`.
  - An artifact without the field decodes as today, byte for byte.
  - An engine without this item refuses a nearest artifact by its own `validate`.
  - **The snapshots claude-227 made** (store `results/claude/integer-retention-rates-parity-2026-10-31/out/full/`) carry no objective. So the field comes from the run's `config.json` at export time, or from a new snapshot. The build unit decides, and prefers the laptop.
- **The nearest step:** under the field, the served token is the whole table's nearest to the query, `−L1` at Q12 over every writer-table row, ties to the smaller id (`stack_whole_table_nearest`'s rule).
  - **The float side's rule is the same measure:** `nearest_scores` (`in_space_long.rs`), "The negative mean L1 displacement of each `query` row to every table row", then `argmax`. A mean against a sum leaves the order unchanged.
  - The point key's override of an opening stays as it is.
  - The scan is adds, absolute differences and compares only, so the serving audit holds.
  - **The browser path too:** `wasm_serve`'s step takes the session's logits and their `stack_argmax`, not the session's token, so the nearest readout must reach it as well.
- **End-to-end parity against the trained model.** Integer greedy, memory on, against claude-220's float greedy replies, which are stored and need no GPU: `samples-memory.json` for DEV and `samples-p-w-greedy-memory.json`, per seed, in `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/out/r-drop-s{1,2,3}/` (claude-220's served run).
  - It covers the whole served model: trunk, Hamming read, turn memory and readout. Nothing has checked those on a trained model yet.
  - Those replies are sealed data in the store, so this item does not wait on [`one-model-end-to-end`](one-model-end-to-end.md), which made them.

## Done when

1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
   - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
   - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
   - Integer greedy of (r-drop) step 16,384, memory on, against claude-220's stored float greedy on DEV and p-w at seeds 1–3.
   - It is reported as first-token agreement and whole-reply agreement. The bar is registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions).
   - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
   - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side is stored GPU output.
4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
5. **One notebook row, and a REPORT with the negatives first.**

## How to start

- **Read:**
  - `in_space_long.rs`: `PointObjective`, `decodes_nearest`, and the decode in `score_rows`;
  - `stack/session.rs`: `stack_step_fbs`, `native_token` and `stack_whole_table_nearest`;
  - `fbs-snapshot.rs` and `run_rate_reading` (claude-226), for reading the run config;
  - claude-229's [lab record](../../labs/point-key-integer/README.md).
- **Before claiming:** check that no open order holds `stack/session.rs` (`point-key-integer` and `turn-memory-integer` also edit it).

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction in the step.
- **Deterministic exact checks only** (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- claude-229's [REPORT](../../loop/tasks/point-key-integer/claude-229/REPORT.md) and the sealed run, `results/claude/point-key-integer-run-2026-10-31/`.
- `crates/uor-r4-float-train/src/in_space_long.rs`, `decodes_nearest`.
- `crates/uor-r4-integer/src/stack/session.rs`, `stack_whole_table_nearest`.
