# ORDER: turn-memory-reply, claude-192

Lab: claude. Item: [turn-memory-reply](../../../../roadmap/items/turn-memory-reply.md). Base: `54d8a3c36`.

## Goal

The item's done-when reads:

> 1. **The probe:** for each candidate key, the DEV reply rows TRAIN's table names. It splits them into openings, copies and the rest, against claude-161's openings-only memory. This is a count with no GPU (CHARTER §4.5).
> 2. **The read, extended** to the best key from the probe, with host and CUDA tests in the style of claude-161's, and no change to any file a run writes when off.
> 3. **A pre-registered run** (a sheet with a fresh FIT): claude-161's (m) recipe with the extended memory, an empty-table control, two seeds. The bar is on top-1, copies and the rest rows against claude-161's (m), with openings held.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

To fill in.

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

To fill in.

## Stop and ask if

To fill in.
