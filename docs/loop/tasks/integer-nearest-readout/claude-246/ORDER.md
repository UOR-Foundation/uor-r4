# ORDER: integer-nearest-readout, claude-246

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `83d54d168`.

## Goal

The item's done-when 3 reads, in part:

> - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
>   - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
>   - the same holds at the opening, on free decode (Q3);
>   - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
>   - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under its own card's rule.
> - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.

**This unit reads Q1 to Q3 of claude-245's bar,** the first of the two units the card names.
- It is a count on sealed files, on the laptop at $0: claude-244's forced decodes, and claude-243's greedy replies and F′ rows.
- Q4 is carried from claude-244's sealed grade.
- Q5 is not read here. Its rule needs its own card, so done-when 3 cannot be met by this unit whatever Q1 to Q3 read.

## Cancel probe (run first)

The unit is cancelled if any check fails:
- **(a) The bar is registered and unread.** At the base, the item is `ready` and done-when 3 names claude-245's card. No grader, REPORT or notebook row on the base counts how often R's teacher-forced pick, R's greedy first token, or F′'s equals the reference token. claude-245 read no data.
- **(b) The inputs are present and sealed.** The local copies must equal the store's seal, file by file:
  - claude-244's six forced decodes;
  - claude-243's six R greedy files, six F′ greedy files and six F′ rows files;
  - claude-232's six prompt files.
- **(c) Row 0's sampler check can run.** claude-243's record names the commit F′ was decoded at. `in_space_long.rs` is readable at that commit, so the sheet can register the code path by file and lines.
- **(d) Nothing else holds the item.** No open order or PR holds it.

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

1. **A prep sheet,** `docs/harness/prep/integer-nearest-readout-grade-2026-11-01.md`, registers claude-245's approved bar verbatim:
   - the positions, the counts a, b and V, and the row identity;
   - Q1, Q2 and Q3 with its six tested strata and its readings;
   - Q4, carried and cited;
   - the cut-window I-Align rule with its minimum;
   - Row 0, with the sampler check by file and lines;
   - the outcomes, LOSS and held, and what each unlocks.

   It also names its inputs by sealed path and hash, and holds the grader.
   - It gets a fresh non-author FIT and a pre-run review before any file is read for a reading.
   - The sheet changes no part of the bar. A defect in the bar goes to a new decision card.
2. **Row 0 runs first, in every cell.** The readings are computed only once every Row 0 check holds in all six cells.
   - If one fails, nothing is read. The instrument is repaired, inside this scope if it can be, and the failure is recorded.
3. **The readings:**
   - Q1 and Q2 in every cell and both pooled panels;
   - Q3 in its six strata, with the DEV strata reported as readings;
   - the I-Align decision on p-w's cut windows, and if they join, Q1 on them.

   The outcome is stated as LOSS, naming the failing strata, or as held. The outputs are sealed in the store with a readback.
4. **The record:**
   - the item's "What is missing" names the next unit;
   - one row in `m4-serving.md`;
   - a REPORT, negatives first.

   On a LOSS, the classification by trace is the next unit's. This unit does not classify it, and a LOSS of any class fails the bar for R.
5. **Done-when 3 stays unmet** in any case, since Q5 is outstanding. The item stays `ready`.

## Stop and ask if

The lab answers these itself and does not wait (CHARTER §5.1):

- **The FIT finds the registered bar undecidable on these files.** For example, a field the card names is missing. Then nothing is read, and a decision card revises the bar first.
- **A Row 0 check fails with a cause outside this scope.** Then it is recorded, and the repair is the next unit.
- **Anything touches CHARTER §3 or the caps.** That is the owner's: one `[claude, for Casey]` note on #2028, and the item parks. This unit does not touch them.

The lab makes every other call itself and records it in the REPORT (CHARTER §5.1).
