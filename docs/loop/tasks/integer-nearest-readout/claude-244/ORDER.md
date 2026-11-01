# ORDER: integer-nearest-readout, claude-244

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `d6a058c26`.

## Goal

The item's done-when reads:

> 1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
> 2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
>    - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
>    - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
> 3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
>    - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
>      - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
>      - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
>    - It is reported as first-token agreement and whole-reply agreement. The bar is registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions).
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

**This unit** does not meet done-when 3. It is the localization that claude-243's MISS row names, and its first lead: the bar's margin. It measures R's agreement with F′ at every reply position under teacher forcing, on DEV and p-w at seeds 1–3, with R's own top-two margin at each position. The question is whether the p-w gap is the export's numeric residual at near ties or a systematic difference, and whether it is special to the reply's opening.
- **The integer side** runs on the laptop, as product serving (§3.6), under the owner's 31 Oct allowance for this chain.
- **The float side** is F′'s sealed teacher-forced rows from claude-243's GPU run (`rows-*-memory.json`, its `nearest`), so no pod is needed.

## Cancel probe (run first)

Does the reading exist already?
- `fbs-toks` decodes greedily (`--replies`) and has no teacher-forced mode.
- `fbs-compare` compares only the snapped export, on synthetic inputs (claude-227).
- claude-181's raw parity is the trunk alone, on claude-126's snapshot, on the host. It did not cover (r-drop) with the memory and the read, on panel text.
- claude-243's REPORT lists the margin as not measured.

**Result: not measured, so go.** Class: instrument, a missing reading, not a mechanism question.

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/wasm_serve.rs`
- `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/harness/prep/integer-nearest-readout*`
- `docs/labs/integer-nearest-readout/**`
- `docs/loop/tasks/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **`fbs-toks ARTIFACT --forced PROMPTS_JSON --out FILE`.** It steps each prompt as `--replies` does, then feeds the reference's tokens in turn. At each offset it records the served pick, the runner-up, and both `−L1` scores at Q12.
   - Tests on the synthetic stack check four things: offset 0 equals `panel_reply`'s first token; where the reference is the greedy reply, the picks equal it; the top two match a reference scan; and the format is right.
   - There are negative gates for the feed and for the runner-up.
2. **A prep sheet,** `docs/harness/prep/integer-nearest-readout-margin-2026-11-01.md`, with a fresh FIT. It registers the readings and the decision table before any forced decode of R is read.
3. **The six forced decodes** (R at s1–s3, on DEV and p-w) run on the laptop and are compared with F′'s rows by position, under the sheet's instrument checks. The sheet's row is recorded.
4. **The records:** one notebook row, a REPORT with the negatives first, and the item's "What is missing" updated. The item stays `ready`.

## Stop and ask if

Each of these is stop and decide, never stop and wait.
- **R's forced picks differ from its greedy reply** where the two prefixes are equal. That is an instrument fault in the new mode: repair it, and count nothing.
- **F′'s rows differ from F′'s greedy reply** where the prefixes are equal. The rows are then not aligned to the prompts; record it and count nothing.
- **The reading calls for a change to the bar.** That needs a decision card and my own fresh decision review, logged on #3462. This unit changes no bar by itself.
