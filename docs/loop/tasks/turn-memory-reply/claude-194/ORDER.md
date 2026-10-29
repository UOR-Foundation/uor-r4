# ORDER: turn-memory-reply, claude-194

Lab: claude. Item: [turn-memory-reply](../../../../roadmap/items/turn-memory-reply.md). Base: `c6e2e19bf`.

## Goal


**This unit (claude-194): done-when 3, the pre-registered run.** Seeds 1 and 2, three arms each, on two RTX 5090 pods:
- (m), claude-161's opening memory, re-run on today's code;
- (r), with the reply table (claude-193);
- (r0), both tables empty.

The bar, the instrument checks and the stop rule are in the sheet, [`turn-memory-reply-2026-10-29`](../../../../harness/prep/turn-memory-reply-2026-10-29.md), fixed before any GPU time. It takes a fresh non-author FIT and pre-run review first (CHARTER §4.5, D67).

The item's done-when, for reference:
The item's done-when reads:

> 1. **The probe:** for each candidate key, the DEV reply rows TRAIN's table names. It splits them into openings, copies and the rest, against claude-161's openings-only memory. This is a count with no GPU (CHARTER §4.5).
> 2. **The read, extended** to the best key from the probe, with host and CUDA tests in the style of claude-161's, and no change to any file a run writes when off.
> 3. **A pre-registered run** (a sheet with a fresh FIT): claude-161's (m) recipe with the extended memory, an empty-table control, two seeds. The bar is on top-1, copies and the rest rows against claude-161's (m), with openings held.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

Cancel if the read is not on `main`, if the run already exists, or if the caps or the store are not available.
- **The read is on `main`:** #3397 merged as `c6e2e19bf` (claude-193), with `--turn-memory-reply 1`.
- **No run of the reply table exists:** no `results/claude/turn-memory-reply-*` in the store, and no notebook row.
- **The caps and the store:** `uor-pod status` shows 0 pods running, $0/h of $8/h, and RTX 5090 offers at $1.106/h. The pinned splits are in the store at `w2-inputs/2026-10-25/driver-data/`.

Result: **go**, after the FIT. This probes an instrument (the run's preconditions); no mechanism is judged here.

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

1. **The sheet** has TEST FITNESS: FIT and PRE-RUN: PASS from a fresh non-author reviewer, with its notes applied.
2. **The run:** both pods run `job.sh` at this unit's head, and I-1 to I-5 pass. The outputs are sealed to the store and read back by sha256 before `down`.
3. **The REPORT:** the bar applied, the cell named, every arm's readings and costs, and one notebook row. The item is set `done` only on cell A, or on a cell whose next step is another item.

## Stop and ask if

- **An instrument check fails:** stop at that check, as the sheet's stop rule says. Classify instrument against mechanism before calling anything.
- **The lease would pass the caps:** stop, and seal what exists.
