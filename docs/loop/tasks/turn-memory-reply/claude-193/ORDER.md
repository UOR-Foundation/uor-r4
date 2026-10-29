# ORDER: turn-memory-reply, claude-193

Lab: claude. Item: [turn-memory-reply](../../../../roadmap/items/turn-memory-reply.md). Base: `c6384ea80`.

## Goal


**This unit (claude-193): done-when 2, the read extended to the probe's key.** claude-192 chose turn-8 + position + reply-last-1 at stored share ≥ 0.5 (#3396). That key names 9,655 non-opening DEV rows and gets 8,822 right. This unit builds it into the turn memory as a second table with its own weight, keeping claude-161's opening table as it is. Host and CUDA tests are in claude-161's style. No file a run writes changes when it is off. The pre-registered two-seed run is done-when 3, the next unit.

The item's done-when, for reference:
The item's done-when reads:

> 1. **The probe:** for each candidate key, the DEV reply rows TRAIN's table names. It splits them into openings, copies and the rest, against claude-161's openings-only memory. This is a count with no GPU (CHARTER §4.5).
> 2. **The read, extended** to the best key from the probe, with host and CUDA tests in the style of claude-161's, and no change to any file a run writes when off.
> 3. **A pre-registered run** (a sheet with a fresh FIT): claude-161's (m) recipe with the extended memory, an empty-table control, two seeds. The bar is on top-1, copies and the rest rows against claude-161's (m), with openings held.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

Cancel if the read already exists, or if the probe's key cannot be computed at a training, evaluation or decoding row.
- **No reply table exists:** `flock_read_memory.rs` holds only the opening table, read at the opening rows the caller names.
- **The key can be computed at every row it is needed:**
  - at training and DEV rows, from the windows and the split's response mask, exactly as `reply_rows` computes it;
  - at a greedy decoding step, from the prompt's turn anchor slots, which equal the probe's first-8 key at every DEV and TRAIN opening (claude-154's anchor check), and the reply generated so far.

Result: **go.** An instrument (the read's plumbing); no mechanism is judged here.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read*.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `docs/harness/prep/turn-memory-reply*`
- `docs/loop/tasks/turn-memory-reply/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/turn-memory-reply.md`

## Done when

1. **`flock_read_memory.rs`:**
   - an optional reply table, keyed as `ReplyKey::TurnPosition(1)` (turn-8, reply position, last reply id), built from TRAIN's non-opening reply rows at share ≥ 0.5;
   - its own per-coordinate weight, starting at 0;
   - the move `n + w_r ⊙ (c − n)` at the reply rows the caller names with their keys.
   It is saved and loaded with the memory; when absent, the saved file is byte-identical to today's.
2. **`in_space_long.rs` and `bin/train-in-space.rs`:** `--turn-memory-reply 0|1`, which needs `--turn-memory`. The empty control empties both tables. Training batches, DEV readings and greedy replies all name the reply rows. The reply table's TRAIN size and DEV hits are logged. The config key is written only when the flag is on.
3. **Host tests:**
   - the reply table's rule, with a fixture exactly at share 1/2;
   - the training rows' keys equal the probe's, including a document that spans two windows;
   - the greedy key equals the corpus key at the same row;
   - only named rows hit;
   - the config keys;
   - the save and load round trip, and the absent case unchanged;
   - the probe test gains a row that both opens and copies.
   **CUDA tests,** extending claude-161's: zero reply weight is no read, bit for bit; the reply move equals the stored point's pull, with its gradient. They compile here and run on the pod in done-when 3's run (its I-1).
4. **A negative gate:** with the reply move set aside, the zero-weight-equality host test or a named-rows test fails.

## Stop and ask if

- **The probe's key and the read's key would differ at some row.** Stop and decide which is right before building on either; never let the two drift silently.
- **The change would alter what an off run writes.** Restructure until it does not.
