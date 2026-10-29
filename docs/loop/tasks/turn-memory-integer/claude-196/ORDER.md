# ORDER: turn-memory-integer, claude-196

Lab: claude. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md). Base: `10b16f654`.

## Goal

The item's done-when reads:

> 1. **Export and session.** The exporter writes the table (keys and values as integer ids) and the weight (at the exporter's fixed point, rounded as it rounds). The session applies the move at a reply's first step only. `step` and its zero-multiply audit are unchanged, and the memory is off for an artifact without one.
> 2. **Parity,** on DEV, against the float forward of claude-161's sealed (m) checkpoint at step 16,384 (`caseyallard/uor-r4-store:results/claude/route-plus-memory-2026-10-28/`).
>    - The share of opening rows whose integer and float decoded ids agree, and the integer openings count.
>    - The float reference runs on the GPU (CHARTER §3.5). The integer side runs on the laptop.
>    - The bar is pre-registered in a sheet with a fresh FIT.
> 3. **Cost on the M1:** `fbs-toks --stages` with the memory, timing its lookup and move beside the writer and the trunk, plus the artifact's added bytes.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/memory.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/turn-memory-integer*`
- `docs/loop/tasks/turn-memory-integer/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/turn-memory-integer.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
