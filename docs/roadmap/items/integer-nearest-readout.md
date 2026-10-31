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

The project's first hurdle is one working model end to end, served from its integer artifact. The served model must decode as the trained one does. Claude-231 has built that readout on native and browser paths; end-to-end parity against the trained model remains to be proven.

- **The trained model decodes by nearest.** (r-drop) trained with `objective: "cell"`, and `in_space_long` decodes such a model as the whole table's nearest token: `decodes_nearest`, "Whether replies decode as the whole table's nearest token, not the writer's pick". [`one-decoder`](one-decoder.md) (claude-150) made whole-table nearest the float side's served decoder.
- **Before claude-231, the integer session decoded by the native writer.** It computed whole-table nearest only beside the writer, for timing. In [`point-key-integer`](point-key-integer.md)'s real run (claude-229, #3460), integer greedy's first token equalled claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 on p-w. The float's own first token equalled the reference on 129 and 179. The run's grade read nothing and ended at row 0.
- **Claude-231 built `head = "native_vector_nearest"` and its export** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md), #3463): 395 integer-library tests with one ignored, seven negative gates and full serving/WASM audits. The nearest branch serves `stack_whole_table_nearest` on native and browser paths, after memory and the Hamming read; the point-key override stays unchanged. An older engine refuses the new head, and an artifact without it decodes byte for byte as before.
- **The remaining gate is end-to-end greedy parity,** against claude-220's stored float greedy replies on DEV and p-w at seeds 1–3. Those replies are sealed data in the store, so no new float run is needed. The parity bar is registered before reading and must pass before the item closes.

## What is missing

- **End-to-end parity against the trained model,** which is NOT_RUN. Integer greedy, memory on, against claude-220's float greedy replies: `samples-memory.json` for DEV and `samples-p-w-greedy-memory.json`, per seed, in `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/out/r-drop-s{1,2,3}/` (claude-220's served run).
  - It covers the whole served model: trunk, Hamming read, turn memory and readout. Nothing has checked those on a trained model yet.
  - Those replies are sealed data in the store, so this item does not wait on [`one-model-end-to-end`](one-model-end-to-end.md), which made them.
- **The completed readout, for reference:** `head = "native_vector_nearest"`, with the whole table's `−L1` scan at Q12 and ties to the smaller id. The browser uses `IntegerStackSession::serve_step`, so FBS models no longer fall back to token 0. The nearest scan costs about 2.85× a token at 1,365 decode and 1,405 prefill tok/s; prefill-scan removal is a separate optimization, not parity evidence.

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
