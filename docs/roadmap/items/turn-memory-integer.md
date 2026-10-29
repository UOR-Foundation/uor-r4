---
id: turn-memory-integer
title: Serve the turn memory from the integer artifact on the laptop
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [route-plus-memory]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-float-train/src/fbs_export.rs, docs/harness/prep/turn-memory-integer*, docs/loop/tasks/turn-memory-integer/**, docs/research/notebook/float-training.md, docs/roadmap/items/turn-memory-integer.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact". This is the [pipeline map](../../model/pipeline-map.md)'s R5, for the memory.

[claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md)'s turn memory doubled reply openings, to `598.7 / 598.3` of 1,226, at no measured training cost. Its key and table are already exact integer ids. Only the move `n + w ⊙ (c − n)` is float, and the integer runtime has no memory.

## What is missing

- **The artifact** does not carry the memory's table or weight. `fbs-export` writes the trunk only.
- **The integer session** has no memory. It needs:
  - the key from the turn anchor's first 8 ids;
  - the lookup;
  - the move on the integer query at a reply's first decode step, with the weight's products through D11's tables.
- **No parity:** the integer reply openings have not been compared with float's on the same checkpoint.

## Done when

1. **Export and session.** The exporter writes the table (keys and values as integer ids) and the weight (at the exporter's fixed point, rounded as it rounds). The session applies the move at a reply's first step only. `step` and its zero-multiply audit are unchanged, and the memory is off for an artifact without one.
2. **Parity,** on DEV, against the float forward of claude-161's sealed (m) checkpoint at step 16,384 (`caseyallard/uor-r4-store:results/claude/route-plus-memory-2026-10-28/`).
   - The share of opening rows whose integer and float decoded ids agree, and the integer openings count.
   - The float reference runs on the GPU (CHARTER §3.5). The integer side runs on the laptop.
   - The bar is pre-registered in a sheet with a fresh FIT.
3. **Cost on the M1:** `fbs-toks --stages` with the memory, timing its lookup and move beside the writer and the trunk, plus the artifact's added bytes.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-161's REPORT](../../loop/tasks/route-plus-memory/claude-161/REPORT.md);
- `crates/uor-r4-float-train/src/flock_read_memory.rs` (the float read, its `save`, and its key);
- `fbs_export.rs` and `fbs-export`;
- `stack_step_fbs` in `crates/uor-r4-integer/src/stack/session.rs`.

`integer-hamming-read` also edits `session.rs`. The two run one after the other, not side by side.

## Rules that apply

- **Serving stays integer and multiplier-free** (D11, CHARTER §3.6): the move's products go through the tables, and the lookup is a hash and a load.
- **Geometry only, no squashing** (§3.1–§3.2).

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
