# ORDER: turn-memory-integer, claude-196 (decision: serve both tables, and how a session keys them)

Lab: claude. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md). Issue: #3392. Base: `10b16f654`.

## Goal

A decision unit (CHARTER §5.1, D67, D68). It changes the item's done-when, under the lab's own fresh decision review, logged as `[claude] owner log:` on #3392.

**Why.** The item was written on 28 Oct for claude-161's opening table alone. Since then, claude-194 (#3398, merged `10b16f654`) trained the reply table, with cell A at both seeds. Its REPORT shows the trained trunk depends on the memory: without it, (r) falls to 4,101 / 3,807 DEV hits, below the empty control's 10,079 / 11,171. So a served model needs both tables, and the item, as written, would serve a model that does not work. This advances step 2 of PLACE.md's order: serve the working model's parts in integers.

The amended item states:
1. **Both tables are served.**
   - The opening table at a reply's first step: the turn's first 8 prompt ids.
   - The reply table at every later reply step: turn-8 + `u32::MAX` + the reply length so far + `u32::MAX` + the last reply id.
   - Each moves the readout point `n + w ⊙ (c − n)` with its own weight, as `flock_read_memory.rs`'s `apply_readout`.
2. **How a session forms the keys**, as the float side forms them (the item's "The keys", corrected in the decision review):
   - **The turn** comes from the `<|bos|>`/`<|eos|>` anchor of `opening_slots(.., TURN_KEY, OpeningAnchor::Turn)`, which the opening table's training read and the greedy decode both use. The reply rows' mask-based turn agrees at openings as a measured fact, and a test row with an internal `<|eos|>` pins the session's behaviour.
   - **One role signal:** the caller marks where the reply begins, since a plain `step(token)` carries no roles. The turn is in the token stream.
   - **The window rule follows training:** a reply row past the window's view of its prompt keys with an empty turn, as training did. The greedy decode's difference there (it reads no reply table) is a measured item for the parity unit.
   - **The key window is the trained 192 tokens,** carried in the artifact, whatever the session's own `context` (decision review round 2).
3. **The parity checkpoint** becomes claude-194's (r) at step 16,384, `caseyallard/uor-r4-store:results/claude/turn-memory-reply-2026-10-29/`, at both seeds, in place of claude-161's (m). It has both tables, and its trunk is the one that depends on them.
4. **The order of units:**
   1. the export of both tables and their weights;
   2. the session's role signal and keys, against a test-side reference of the float key rule;
   3. the move at the readout, with the audits;
   4. the parity, under a sheet;
   5. the cost.
5. **What stays refused:**
   - a session restore with a memory, until its turn state is saved;
   - an artifact with a memory on a non-FBS shape.

No code changes in this unit.

## Cancel probe (run first)

- **Is the item already serving both tables?** No. At `origin/main` `10b16f654`, `docs/roadmap/items/turn-memory-integer.md`'s done-when names the opening table only. `fbs_export.rs` still refuses a memory ("a memory, read selection or read router is not served").
- **Is the reply table withdrawn?** No. turn-memory-reply is `done` with cell A (#3398).
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/turn-memory-integer.md`
- `docs/loop/tasks/turn-memory-integer/**`

(The item's full scope stays as it is for the later units.)

## Done when

1. The item file states 1 to 5 above, with each fact sourced (file and line, or REPORT), and keeps its rules.
2. **A fresh decision review** reads the whole chain first and passes it:
   - PLACE.md's order;
   - claude-194's REPORT;
   - claude-161's REPORT;
   - the item;
   - the float key code (`in_space_long.rs`, `flock_read_memory.rs`);
   - the integer session (`stack/session.rs`).
3. **The owner log** is posted on #3392, with the review record.
4. `uor-harness loop pr-check --local` passes, and the roadmap re-renders if the item's rendered fields change.
5. A REPORT. The item stays `ready`.

## Stop and ask if

Stop and decide (never wait) if:
- the decision review finds that serving both tables needs a change to §3 or a cap. Then it is the owner's: post once on #2028, and park the item;
- the float key cannot be reproduced at serving without the trainer's windows. Then the unit records the closest exact rule, and the train/serve gap it leaves, as a measured item for the parity unit.
