# ORDER: turn-memory-reply, claude-192

Lab: claude. Item: [turn-memory-reply](../../../../roadmap/items/turn-memory-reply.md). Base: `54d8a3c36`.

## Goal


**This unit (claude-192): done-when 1 only, the probe.** Count the DEV reply rows that an exact memory of TRAIN could name, for each candidate key and each stored-share threshold. Split them into openings, copies and the rest, against claude-161's openings-only memory. This is a count over the pinned corpus on the laptop, with no model and no GPU (CHARTER §4.5).

**Decision rule, fixed before the probe runs.** The extended read (done-when 2) takes the key, and the stored-share threshold, with the most DEV hits on non-opening rows, among those whose precision on the rows they name is at least **0.83**. That is the higher of claude-161 (m)'s two seeds' rest-row accuracies, 7,512 / 9,094; a memory less accurate than that would on balance displace right answers. If no key reaches 0.83, lookup alone cannot carry the reply past its first word: the next unit is a decision unit on the key, not a GPU run.

The item's done-when, for reference:
The item's done-when reads:

> 1. **The probe:** for each candidate key, the DEV reply rows TRAIN's table names. It splits them into openings, copies and the rest, against claude-161's openings-only memory. This is a count with no GPU (CHARTER §4.5).
> 2. **The read, extended** to the best key from the probe, with host and CUDA tests in the style of claude-161's, and no change to any file a run writes when off.
> 3. **A pre-registered run** (a sheet with a fresh FIT): claude-161's (m) recipe with the extended memory, an empty-table control, two seeds. The bar is on top-1, copies and the rest rows against claude-161's (m), with openings held.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

Cancel if claude-161's memory already moves later rows, if the pinned splits are not at hand, or if the probe exists.
- **claude-161 moves only opening rows** (its REPORT: "The memory moves only opening rows"; "replies are still templated after their first word").
- **The pinned TRAIN (`mw-balp/train`) and DEV (`ft/dev`)** are copied from the store's `w2-inputs/2026-10-25/driver-data/` to `local/claude/data/`. DEV's `tokens.u16` sha256 begins `0919e90f953e988e`, the pin. The loader checks every pin at load.
- **No reply-row probe exists:** `grep -rn reply_bound crates/` finds nothing.

Result: **go.** An instrument, the corpus count; no mechanism is judged.

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

1. **`reply_bound` in `in_space_long.rs`, and `--reply-bound 1` in `train-in-space`.** For each reply row: its turn's prompt, the reply's earlier tokens, its target and its kinds (opening, copy, rest). For each key, and stored-share thresholds 0, 0.5, 0.8 and 0.95, it reports DEV rows, rows whose key TRAIN holds, and hits, per kind. The keys:
   - the reply's last 1–4 tokens alone (the no-turn control);
   - the turn's first 8 tokens with the reply's last 0–4;
   - the turn's first 8 with the reply position, with and without the last reply token.
2. **Host tests, pinned:**
   - the rows and their prompts and replies on a small corpus;
   - the table rule (the commonest target, ties to the smaller id);
   - the share threshold;
   - the per-kind counts.
   A negative gate shows one test fails with the tie rule reversed.
3. **The probe's run on the pinned splits.** Its JSON is committed under the task folder. The opening row of "first-8 + no reply token" at share 0 must name about claude-161's 607 with its fallback.
4. **The decision rule applied** in the REPORT, with one notebook row.

## Stop and ask if

- **The pinned splits fail their sha256 check.** Stop; nothing is counted on unpinned data.
- **The opening reference differs from 607** by more than the rows the fallback adds. Then the rows are wrong: fix the instrument, not the bar.
