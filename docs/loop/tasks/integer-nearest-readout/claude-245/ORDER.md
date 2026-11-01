# ORDER: integer-nearest-readout, claude-245

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `3e341672f`.

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

**This unit is the decision card that claude-244's sheet names as its next unit** ([REPORT](../claude-244/REPORT.md), the near-tie residual row with pooled r₊ ≤ 3%). It re-derives done-when 3's bar from the measured per-position residual, and states how the new bar treats p-w seed 3 (r₊ 3.09% alone) and the 151 pooled p-w disagreements at clear margins. It is a change to a bar (CHARTER §5.1, §5.2), so it needs my own fresh decision review, logged on #3462 for Casey, who can reverse it. It reads no new data and runs nothing: the bar it registers is read by the next unit, under a sheet with its own FIT.

## Cancel probe (run first)

The unit is cancelled if any check fails:
- **(a) The bar is still the old one.** The item is `ready` at the base, its done-when 3 still names the ⌊0.03 n⌋ first-token bar registered against claude-227's 0.33%, and no decision card after claude-243's exists in `docs/loop/tasks/integer-nearest-readout/`.
- **(b) The candidate quantity is unread.** No grader, REPORT or notebook row on the base counts how often R's teacher-forced pick, or F′'s, equals the reference token. A bar on a quantity already read on the data it is graded on is not a test.
- **(c) The inputs carry it.** claude-244's sealed forced decodes carry R's pick, runner-up and both scores at every position, and claude-243's sealed F′ rows carry F′'s nearest and the target at every position, so the next unit can read the bar on the laptop.

**Result (1 Nov): all three pass; the unit goes ahead.** The class is instrument: this unit changes how parity is read, not the model.
- (a) The item is `ready` at `3e341672f`. Its done-when 3 still reads "The bar is registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions)" (line 58). The only decision card in the task folder is claude-243's.
- (b) claude-244's grader reads the reference only for alignment checks: `grade-cell.jq:28` and `:40` compare prefixes, and `:42` checks that F′'s target equals the reference token (`targets_wrong`, 0 in every cell). It counts no pick against the target. claude-243's grader does not read the reference token at all. No m4-serving row counts it. The one count of picks equal to the reference is claude-205's, on the float side only: the float model's nearest is right on 10,079–11,513 DEV rows. That is F's accuracy alone, at an older build, never R's, and never paired.
- (c) Each forced decode row carries `picks`, `runner_up`, `score_q12`, `runner_score_q12` and `position`. Each F′ rows file carries `positions`, `nearest` and `targets` of equal length (49,251 each for p-w s1).

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

1. **The decision card** (`DECISION.md` in this folder) answers how done-when 3's bar is re-derived from claude-244's measured residual. It gives the question, what is measured, the options, the constraints, a recommendation, and what a reversal costs. It states how the bar treats p-w seed 3 and the 151 clear-margin disagreements, and how it treats the cut windows that claude-243's bar counted.
2. **The decision review:** fresh-context, at most two rounds, reading the whole chain (claude-227, claude-232, claude-243 and claude-244, the item, and the card). The card records each round. It is approved and logged on #3462 as `[claude] owner log:`.
3. **The item** is amended: done-when 3 names the approved bar and cites the card, and "What is missing" names the next unit, the sheet that reads it. The item stays `ready`. The roadmap is re-rendered.
4. **No reading of the registered quantity** is made in this unit: no pick of R or F′ is counted against the target. The bar is fixed before it is read.
5. **A REPORT,** negatives first. The unit has no result, so it adds no notebook row.

## Stop and ask if

The lab answers these itself and does not wait (CHARTER §5.1):

- **The review finds that the recommended bar passes a port with a real fault that the old bar would catch, and no change closes that.** Then the card recommends keeping the old bar, and records why.
- **The sealed targets are not the reference replies' tokens.** Then a quality comparison is not what the card says; the card is revised before approval.
- **The recommendation touches CHARTER §3 or the caps.** That is the owner's: one `[claude, for Casey]` note on #2028, and the item parks. (This card does not touch them.)

The lab makes every other call itself and records it in the REPORT (CHARTER §5.1).
