# REPORT: integer-hamming-read, claude-234 (the scope and method the read's trace needs)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). References #3385.

## Tokens per second

Not measured here: this unit changes the item's text and runs nothing.

## What did not work, first

- **The trace cannot start in this unit.** It needs the float read's own codes and kept positions, which exist only inside `flock_read_hamming.rs`'s `apply` and `apply_readout`. That file was outside the item's scope, and the guard reads an item's scope from `origin/main`, so a branch cannot widen its own. This unit carries the scope change alone, as claude-214 (#3422) did for one-model-end-to-end. The trace moves to the next unit.
- **The item's cancel condition was stale.** It read "The integer runtime already has a Hamming read on main". Its copy-head kernel came in with claude-189, and it has been served since claude-194, 195 and 200, but its parity is unmet (claude-233's row R). So the condition, read literally, would cancel an unfinished item. It now names the done-when instead. Class: instrument (the wording), not the item.
- **`roadmap render` rewrites two rows this unit did not touch:** the titles of `m1-close-saved-model` and `docs-status-refresh`. Kimi's #3436 changed both items without a render. This unit's own edit renders no change, so the README is left as main has it, and the drift is left to its item's owner.
  - **So done-when 3's "no diff" is not met as written.** It is read as "no diff from this unit's edit". This was decided under the ORDER's stop-and-ask clause, by decision review, docs-only: re-rendering other items' rows is outside this unit's scope. The decision is in the record on #3385.

## What changed

- **`docs/roadmap/items/integer-hamming-read.md`:**
  - **`scope:`** adds `crates/uor-r4-float-train/src/flock_read_hamming.rs`.
  - **Step 5** now records the trace:
    - the float side, a capture inside `apply` and `apply_readout`, off by default. It writes the query and key codes as well as the kept positions, so the two causes can be told apart;
    - the integer side, the same record from `stack/hamming.rs`;
    - the timing on the same laptop pass: decode tokens/s against context, with the read and without (step 6's measurement, taken early; Casey, 31 Oct 2026, in chat: "Yes, fold it in");
    - the comparison, kept-set share and top-1:
      - each stream-read set is judged on its own;
      - a readout set is judged only where the stream read agrees up to its position, and otherwise counted as downstream;
      - each judged set that differs is put down to a tie across the 16th place or a sign flip, and anything else is treated as an instrument fault;
    - two arms beside the plain D11 forward, one per lead (the float tie rule set to the integer's, and the read's parameters rounded as exported).
  - **Cancel if** names the done-when, and says why the read's existence is not a cancel.
- **The decision** (a scope change, CHARTER §5.1) is logged on #3385, with its fresh decision review.
- **The collision check.** Two other open items cover the file:
  - `one-model-end-to-end`, ready, unclaimed; claude-214 added the file to it;
  - `e8-orbit-memory-sigmoid-free-iteration`, held, unclaimed, through `crates/uor-r4-float-train/**`.

  No open PR touches the file, `stack/hamming.rs` or this item, apart from this one (#3466).

## Measurements

None.

## Checks run at the head

- `uor-harness loop report --check`
- `uor-harness loop pr-check --local`
- `cargo run -p xtask -- roadmap render`: this unit's edit renders no change (the two-row drift above is not this unit's).

## Skills used

None.

## Still open

- **The trace unit (next):** a prep sheet with its FIT and pre-run review, then one GPU pod for the float side's three arms with the capture, the integer side on the laptop, and the comparison. Does the gap close under the integer's tie rule, under the rounded parameters, or under neither? And what does the read cost per token as the context grows?
- **Kept for later:** the unsnapped re-export and claude-232's parity rerun under it (claude-233's diagnostic lead for the trunk's own export gap).

## Plain summary

The model's look-back read gives different answers in the integer version than in the float one, and the next step is to trace exactly where. That trace has to record what the float read picks inside its own file, which this item was not allowed to change. This unit allows it, writes down how the trace will tell the two likely causes apart, and fixes a cancel rule that would have closed the item by mistake.
