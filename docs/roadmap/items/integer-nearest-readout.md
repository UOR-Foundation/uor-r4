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

The project's first hurdle is one working model end to end, served from its integer artifact. The served model must decode as the trained one does. Claude-231 built that readout on native and browser paths, but claude-232's end-to-end parity check missed in every cell. Claude-243's, with both sides on the same rules, meets the bar on DEV and misses on p-w. The item stays ready while the p-w gap is localized.

- **The trained model decodes by nearest.** (r-drop) trained with `objective: "cell"`, and `in_space_long` decodes such a model as the whole table's nearest token: `decodes_nearest`, "Whether replies decode as the whole table's nearest token, not the writer's pick". [`one-decoder`](one-decoder.md) (claude-150) made whole-table nearest the float side's served decoder.
- **Before claude-231, the integer session decoded by the native writer.** It computed whole-table nearest only beside the writer, for timing. In [`point-key-integer`](point-key-integer.md)'s real run (claude-229, #3460), integer greedy's first token equalled claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 on p-w. The float's own first token equalled the reference on 129 and 179. The run's grade read nothing and ended at row 0.
- **Claude-231 built `head = "native_vector_nearest"` and its export** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md), #3463): 395 integer-library tests with one ignored, seven negative gates and full serving/WASM audits. The nearest branch serves `stack_whole_table_nearest` on native and browser paths, after memory and the Hamming read; the point-key override stays unchanged. An older engine refuses the new head, and an artifact without it decodes byte for byte as before.
- **Claude-232 ran end-to-end greedy parity** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md), #3464) against claude-220's stored float greedy replies on DEV and p-w at seeds 1–3. It missed every cell. First-token agreement is 291, 285 and 284 of 310 on DEV, and 911, 777 and 64 of 1,202 on p-w. Whole-reply agreement is 83, 163 and 127 on DEV, and 60, 113 and 36 on p-w.
- **The readout was necessary but not sufficient.** The gap is largest on exact-key misses, and p-w seed 3 over-concentrates on one first token. The trunk, Hamming read and readout remain suspects, not located causes. The parity bar must pass before the item closes.
- **Claude-243 re-ran the parity with both sides on the same rules** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-243/REPORT.md), #3488): the raw export (R) against the float reference re-decoded on the GPU with the read's tie rule on (F′). It is a **MISS** on p-w at every seed and meets the bar on DEV.
  - First-token disagreement is 4, 4 and 5 of 310 on DEV (bar 9), and 64, 47 and 78 of 1,202 on p-w (bar 36). Whole-reply agreement is 188, 247 and 201 on DEV, and 488, 751 and 547 on p-w.
  - The icosian snap was the larger difference behind claude-232's MISS: the snapped export against F′ agrees on 291/285/284 and 909/777/64 first tokens. F′ against F, the float code's change since `eb762e602`, moves at most 12 first tokens. By the code that change is the tie rule, but `train-in-space` has no switch to test it, so the reading is not credited to it.
  - The remaining p-w disagreements are diffuse; no first token is over-concentrated.
- **Claude-244 measured the bar's margin** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-244/REPORT.md)), under teacher forcing with the new `fbs-toks --forced`. R's disagreements with F′ sit where R's own top two are nearly tied: 88% of the pooled p-w disagreements are in R's lowest-margin decile, and the median margin there is 80 to 100 times below typical. The gap is per position, not special to the opening.

## What is missing

- **Re-derive the first-token bar from the measured residual, in a decision card.** Claude-244 ([REPORT](../../loop/tasks/integer-nearest-readout/claude-244/REPORT.md)) measured where R's disagreements with F′ sit under teacher forcing, against R's own top-two margin g.
  - On pooled p-w, 1,095 of 1,246 disagreements are in R's lowest-margin decile (c = 0.879 against q = 0.100; κ = 0.865). DEV reads the same (κ = 0.937).
  - The gap is per position, not opening-specific: r₀ is 4.12% and r₊ is 2.89%. That is the sheet's near-tie residual row with pooled r₊ ≤ 3%, and its next unit is this card.
  - The card needs my own fresh decision review, logged on #3462. It must say how it treats p-w s3 (r₊ 3.09% alone) and the 151 disagreements (12%) at clear margins.
- **Then locate what remains of claude-243's p-w gap on R against F′,** layer by layer: the trunk, Hamming read, memory and readout. Recheck end-to-end parity before rerunning p-x. Under a sheet; the leads:
  - **Trace the clear-margin disagreements** (g above G₁₀) first.
  - **Read F′'s own top two** at the disagreeing positions. Its sealed rows carry no scores, so claude-244's residual bound is one-sided.
  - **The cut windows' openings** disagree three to four and a half times as often as the counted ones (claude-244, a reading).
  - **Split the disagreements by exact-key status,** as claude-232 did on DEV s1. p-w is the panel where the memory names fewer replies.
  - claude-227's trunk parity on the raw export against the trained forward, on p-w windows, would give the residual layer by layer.
  - The two differences between claude-232's sides, the float reference's tie order (claude-238, claude-240) and the integer side's icosian snap (claude-233), are removed in claude-243's arms. Its sealed run is `results/claude/integer-nearest-readout-tie-parity-2026-11-01/`; claude-232's is `results/claude/integer-nearest-readout-parity-2026-10-31/`.
  - The float replies are sealed data, so this item does not wait on [`one-model-end-to-end`](one-model-end-to-end.md), which made them.
- **Making raw the default, or the snap faithful,** is a follow-up: S against F′ now measures what the snap costs.
- **The completed readout, for reference:** `head = "native_vector_nearest"`, with the whole table's `−L1` scan at Q12 and ties to the smaller id. The browser uses `IntegerStackSession::serve_step`, so FBS models no longer fall back to token 0.
  - On the snapped artifact (claude-231) it decodes at 1,365 and prefills at 1,405 tok/s; the nearest scan costs about 2.85× a token. Prefill-scan removal is a separate optimization, not parity evidence.
  - On the raw artifact (claude-243, R s1, done-when 4): 1,204 tok/s decode and 1,264 prefill (medians), 190.7 MB peak resident, under other load. The 4,096-row nearest is 65% of a decode step, so a cheaper exact nearest is the speed lead.

## Done when

1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
   - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
   - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
   - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
     - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
     - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
   - It is reported as first-token agreement and whole-reply agreement. The bar is registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions).
   - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
   - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
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
