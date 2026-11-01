# ORDER: integer-nearest-readout, claude-247

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `d305f2d2f`.

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
>    - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
>      - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
>      - the same holds at the opening, on free decode (Q3);
>      - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
>      - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under its own card's rule.
>      - First-token and whole-reply agreement stay reported as readings; claude-243's MISS on the old bar is recorded as it is.
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

**This unit is Q5's decision card,** the unit that claude-245's card and claude-246's REPORT name next. claude-246 read Q1 to Q3 of claude-245's bar on the sealed files, with Q4 carried, and every test held. So done-when 3 now waits on Q5 alone: "every clear-margin disagreement is also accounted for by an exact trace, under its own card's rule".
- claude-245's card left the rule to this card and set four requirements for it:
  1. the rule, and any bound it uses, is fixed before either side's values are read;
  2. it traces each position's whole causal prefix;
  3. it has its own Row 0;
  4. it states its writer, any scope change and its cost.
- This is a change to a bar (CHARTER §5.1, §5.2), so it needs my own fresh decision review, logged on #3462 for Casey, who can reverse it.
- It reads no trace value and runs nothing. The rule it registers is read by later units, under a sheet with its own FIT and pre-run review.

## Cancel probe (run first)

The unit is cancelled if any check fails:
- **(a) Q5 is unread and has no rule.** At the base, done-when 3's Q5 says "under its own card's rule", no card in the task folder registers that rule, and no REPORT or notebook row traces a clear-margin disagreement.
- **(b) The rest of the bar is met.** claude-246's REPORT on the base records Q1 to Q3 HELD with Q4 carried. If they had not held, a LOSS would come first and Q5's card could wait.
- **(c) The set Q5 covers is known from sealed data.** The set is claude-244's 151 p-w and 11 DEV counted disagreements at a margin above G₁₀. claude-246's cut-window I-Align held, so p-w's cut windows add theirs at every position. Each of these can be listed from claude-244's sealed forced decodes and F′'s sealed rows, by position, without a trace.
- **(d) The leading proposal's instrument exists in scope.** `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` is on the base and in the item's scope.
- **(e) Nothing else holds the item.** No open order or PR holds it.

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

1. **The decision card** (`DECISION.md` in this folder) registers Q5's rule. It gives the question, what is measured and what is not, the options, the constraints, a recommendation, and what a reversal costs. It meets claude-245's four requirements:
   - **The rule and every bound are fixed now,** before any trace value of R, T or F′ is read. That includes what "accounted for" means, what a fault is, and what each outcome unlocks.
   - **It traces each position's whole causal prefix:** every earlier position of the window and every layer, not the position alone.
   - **It has its own Row 0:**
     - the float side reproduces F′'s sealed `nearest`;
     - the integer side reproduces R's sealed forced picks;
     - a rerun matches.

     A position that cannot be traced is an instrument repair, not a fault.
   - **It states its writer, any scope change, and its cost.**

   It names the set it covers exactly, by source and count.
2. **The decision review:** fresh-context, at most two rounds, reading the whole chain:
   - claude-243, claude-244, claude-245 and claude-246;
   - the item and the card;
   - the instruments the rule relies on, at the base.

   The card records each round. It is approved and logged on #3462 as `[claude] owner log:`.
3. **The item** is amended:
   - done-when 3's Q5 cites the card;
   - "What is missing" names the next unit: the trace instrument, and the sheet that reads Q5;
   - the item stays `ready`;
   - the roadmap is re-rendered.
4. **No trace value is read in this unit.** The rule is fixed before it is read. Listing the set from sealed per-position margins is not a trace.
5. **A REPORT,** negatives first. The unit has no result, so it adds no notebook row.

## Stop and ask if

The lab answers these itself and does not wait (CHARTER §5.1):

- **The review finds no rule that meets the four requirements on any instrument the item can build.** Then the card says so, and recommends a revision of claude-245's bar in a new card, rather than a Q5 that cannot be decided.
- **The rule needs an instrument outside the item's scope.** Then the card states the scope change, and the next unit carries it in its own ORDER.
- **Anything touches CHARTER §3 or the caps.** That is the owner's: one `[claude, for Casey]` note on #2028, and the item parks.

The lab makes every other call itself and records it in the REPORT (CHARTER §5.1).
