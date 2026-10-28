# ORDER: route-plus-memory, claude-161

Lab: claude. Item: [route-plus-memory](../../../../roadmap/items/route-plus-memory.md). Base: `e473455de`.

## Goal

The item's done-when reads:

> 1. **A memory read,** float for training with an exact integer form:
>    - the key is the turn anchor's first `k` token ids, exact, or a learned code of the bag as a second arm;
>    - the table is built from TRAIN only: per key, its commonest next token, or a few with counts;
>    - at a reply row, the stored token's table point moves the readout point through a learned weight that starts at zero, as the Hamming readout does.
>
>    Host tests pin the key and the table build. CUDA tests pin that zero weight is no read and that the move equals the stored point's pull. With no new flag, every file a run writes is unchanged.
> 2. **A pre-registered run** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
>    - (m) claude-152's (a) recipe plus the memory read;
>    - (m0) the same with an empty table, the no-memory control (L-010);
>    - (g) claude-155's bag, which holds the item `in-space-trunk-to-text`'s bar, as a second reference;
>    - two seeds each;
>    - openings read as the mean of the last three checkpoints;
>    - (m) barred against (a), (m0) and (g);
>    - the per-slot or per-key removal reading reported, as claude-154's I-5 did.
> 3. **The REPORT** leads with cost per token: training rate and the memory's bytes and operations per token. One notebook row, and the result on main.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read*.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `docs/harness/prep/route-plus-memory*`
- `docs/loop/tasks/route-plus-memory/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/route-plus-memory.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
