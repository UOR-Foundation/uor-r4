# ORDER: in-space-trunk-to-text, claude-153

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `e2f15d3fc`.

## Goal

The item's done-when reads:

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/secant_descent.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `crates/uor-r4-float-train/src/flock_read*.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/spiralcore_fbs*.rs`
- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `Cargo.lock`
- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/labs/in-space-trunk-to-text/**`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/in-space-trunk-to-text.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
