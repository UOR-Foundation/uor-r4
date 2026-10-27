# ORDER: one-decoder, claude-150

Lab: claude. Item: [one-decoder](../../../../roadmap/items/one-decoder.md). Base: `54be88b72`.

## Goal

The item's done-when reads:

> 1. A pre-registered run (its own prep sheet, FIT and pre-run review, CHARTER §4.5) trains the in-space trunk with `--objective voronoi`, past step 16,384 (for example 32,768 steps), at two seeds. Every checkpoint reads both decoders' top-1 and openings on DEV.
> 2. The sheet's bar decides one of two outcomes, and the REPORT records the decision:
>    - **the writer holds** (no collapse past step 16,384, and within the sheet's margin of whole-table nearest): the writer stays the served decoder;
>    - **it does not:** the decision is to serve whole-table nearest. The REPORT gives its per-token cost at 4,096 tokens (1,048,576 absolute differences) and the point at which a sublinear index is needed. Changing the serving session (`stack/session.rs`, `fbs-toks`) is then its own unit; this item's scope is the run and the decision.
> 3. One notebook row; the result on main.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `docs/harness/prep/one-decoder*`
- `docs/loop/tasks/one-decoder/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-decoder.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
