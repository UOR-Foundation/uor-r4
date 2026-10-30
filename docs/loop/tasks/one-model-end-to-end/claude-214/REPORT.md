# REPORT: one-model-end-to-end, claude-214 (the scope the loop test needs)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). References #3407.

## Tokens per second

Not measured here: this unit changes one scope line and runs nothing.

## What did not work, first

- **The test could not start in claude-214 itself.** The guard reads an item's scope from `origin/main` ("so a branch cannot widen its own scope or name its own notebook page"; `crates/uor-r4-harness/src/loopguard.rs:2823`). Edits to `flock_read_hamming.rs` were refused on the branch that adds it.
  - **The fix:** this unit carries the scope change alone, as claude-203 did for `m-world.rs`. The test moves to claude-215.
  - **Its code so far** (102 diff lines in `in_space_long.rs`: the `ReadScope` parameter on `decode_reply`, its cut, and `score_rows`' scoped greedy files) is kept as a patch in the lab folder, to be applied in claude-215.

## What changed

- **`docs/roadmap/items/one-model-end-to-end.md`:** `scope:` adds `crates/uor-r4-float-train/src/flock_read_hamming.rs`.
  - `roadmap render` leaves `docs/roadmap/README.md` unchanged, since the scope line is not rendered.
- **The decision** is logged on #3407 (6518346519).
  - **Its fresh decision review: PASS.** Chain, collisions, narrowness and risk were each checked with evidence.
  - **Its conditions** are in the ORDER and bind claude-215.

## Measurements

None.

## Checks run at the head

- `uor-harness loop pr-check --local`
- `roadmap render`: it leaves the rendered README unchanged.

## Skills used

None.

## Still open

- **claude-215:** the read's decode-time scope (`Full`, `Copy`, `All`), its grade, a new held-out panel, the sheet with its FIT and pre-run review, and the GPU decode at three seeds. Does holding the copy heads to the prompt end the loops?

## Plain summary

The loop test needs to change the file where the model's copy heads choose what to look back at, and that file was outside this item's allowed files. This unit adds it, with a logged and reviewed decision, so the test can run next.
