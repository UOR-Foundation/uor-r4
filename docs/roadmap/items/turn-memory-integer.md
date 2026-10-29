---
id: turn-memory-integer
title: Serve the turn memory from the integer artifact on the laptop
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [route-plus-memory]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, docs/harness/prep/turn-memory-integer*, docs/loop/tasks/turn-memory-integer/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/turn-memory-integer.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact…". This is the [pipeline map](../../model/pipeline-map.md)'s R5, for the memory.

[claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md)'s turn memory doubled reply openings, to `598.7 / 598.3` of 1,226, at no measured training cost. Its key and table are already exact integer ids. Only the move `n + w ⊙ (c − n)` is float, and the integer runtime has no memory.

**Both tables, since claude-194** (amended by claude-196's decision, logged on #3392):
- [claude-194](../../loop/tasks/turn-memory-reply/claude-194/REPORT.md) trained the memory's reply table, with cell A at both seeds: DEV `T̄` 11,526 / 11,415, against the opening table alone at 8,906 / 10,471.
- It also showed that the trained trunk depends on the memory. With the memory taken out, it falls to 4,101 / 3,807 hits, below the empty-table control's 10,079 / 11,171.
- So the served model needs both tables. Serving the opening table alone would serve a model that does not work.

## What is missing

- **The artifact** carries the trunk and the writer's table, not the memory's table or weight. The exporter refuses a model with a memory ("a memory, read selection or read router is not served", `fbs_export.rs`).
- **No per-row float reading of the memory.** claude-161 reports opening counts, not each opening row's decoded id. Parity needs the ids, so the float side writes them (the three float-train files in this item's scope).
- **The integer session** has no memory. It needs both tables' keys, their lookups, and their moves on the readout.
- **The keys, as the float trainer's own greedy decode forms them** (`greedy_reply_key`, `in_space_long.rs:1164–1173`):
  - **The turn.** The first 8 ids from the turn anchor: the position after the last `<|bos|>` or `<|eos|>` at or before the prompt's last token (`opening_slots` with `OpeningAnchor::Turn`, `flock_read_opening.rs:77–91`). Fewer, if the turn is shorter.
  - **The opening table's key** is the turn's ids alone, at the reply's first step.
  - **The reply table's key**, at each later reply step, is the turn's ids, `u32::MAX`, the reply's length so far, `u32::MAX`, and the reply's last id (`ReplyKey::TurnPosition(1)`, `in_space_long.rs:1230–1240`). It is named only where TRAIN's commonest next id holds a share of at least 0.5.
  - **The turn is in the token stream**, by `<|bos|>`/`<|eos|>`. The reply's start is not: training reads it from the response mask (`reply_rows`, `in_space_long.rs:1175–1212`). So the session takes one role signal from its caller, where the reply begins. A decode loop knows it, since every token it generates is reply.
  - **The window rule.** Training keys over what a 192-token window shows: a turn or a reply cut by the window's start gives a shorter key there. A session keys over its last `context` tokens, as a window of that length does.
- **The moves:** `n + w ⊙ (c − n)` on the readout point, the opening table's weight at the reply's first step and the reply table's after it, as `apply_readout` (`flock_read_memory.rs`). They have the Hamming readout move's form, and come after it in the step.
- **No parity:** the integer reply has not been compared with float's on the same checkpoint.

## Done when

1. **Export and session, for both tables.**
   - The exporter writes each table (keys and values as integer ids) and each weight (at the exporter's fixed point, rounded as it rounds).
   - The session takes the reply's start from its caller, keys as "The keys" above, and applies the opening move at a reply's first step and the reply move at each later one.
   - Each key is checked against a test-side reference of the float rule, including a turn and a reply cut by the window.
   - Both disassembly audits pass with the moves reached. The memory is off for an artifact without one.
   - A session restore with a memory is refused until its turn state is saved.
2. **Parity,** on DEV, against the float forward of claude-194's sealed (r) checkpoints at step 16,384, both seeds (`caseyallard/uor-r4-store:results/claude/turn-memory-reply-2026-10-29/`).
   - The share of reply rows (openings and later rows) whose integer and float decoded ids agree.
   - The integer `T`, `O`, `C` and `R` counts beside float's.
   - The float reference runs on the GPU (CHARTER §3.5). The integer side runs on the laptop.
   - The bar is pre-registered in a sheet with a fresh FIT.
3. **Cost on the M1:** `fbs-toks --stages` with the memory, timing its lookup and move beside the writer and the trunk, plus the artifact's added bytes.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-161's REPORT](../../loop/tasks/route-plus-memory/claude-161/REPORT.md) and [claude-194's](../../loop/tasks/turn-memory-reply/claude-194/REPORT.md);
- `crates/uor-r4-float-train/src/flock_read_memory.rs` (the float read, its `save`, both tables and `apply_readout`);
- `in_space_long.rs`'s `reply_rows`, `ReplyKey` and `greedy_reply_key`, and `flock_read_opening.rs`'s `opening_slots`;
- `fbs_export.rs` and `fbs-export`;
- `stack_step_fbs` in `crates/uor-r4-integer/src/stack/session.rs`, and the Hamming readout move before it.

**The order of units** (claude-196):
1. the export of both tables and weights;
2. the session's role signal and keys, against the reference;
3. the moves at the readout, with the audits;
4. the parity, under a sheet;
5. the cost.

`integer-hamming-read` and `fbs-serving-kernels` also edit `session.rs`, and the float-side items edit `in_space_long.rs` and `train-in-space.rs`. This item runs when none of those is claimed.

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction. The hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them; until then the move's products go through the tables. The lookup is a hash and a load.
- **Geometry only, no squashing** (§3.1–§3.2).

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
- [claude-194](../../loop/tasks/turn-memory-reply/claude-194/REPORT.md), and [claude-196](../../loop/tasks/turn-memory-integer/claude-196/ORDER.md)'s decision to serve both tables (#3392).
