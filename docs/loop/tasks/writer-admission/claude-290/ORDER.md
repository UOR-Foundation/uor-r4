# ORDER: writer-admission, claude-290

Lab: claude. Item: [writer-admission](../../../../roadmap/items/writer-admission.md). Base: `4ea097661`.

## Claim check

- **Plan step:** P6, its second unit (the bound is 2 units). Writer admission, #3394, is on Casey's call of 2 Nov: "Yes, add both".
- **What it changes in the model:** the shared vector table, which is the input embedding, the writer's table and nearest's table at once. Under a new training option, the table's two port blocks are held at each token's cell roots. The writer reads its 8 cells from those same blocks. So nearest's own objective trains admission, and a writer that holds lets serving drop whole-table nearest. That is 484 of 744 µs a step at short context (claude-288's parts).

## Goal

Done-when 2 of the item, and only that. The run is done-when 3, in the next unit.

> 2. **A geometric change to admission,** for example a margin the 16 roots' spacing allows, cells kept by the target's own ports, or a kept count chosen by the margin. It ships with host tests, and with CUDA tests where tensors are involved.

**What claude-289 found** ([REPORT](../claude-289/REPORT.md)): at seed 2, both query ports fell to the roots' centre. Their mean |x| went from `2.2e-5` to `5.4e-7`, while the rest of the query grew to about 4. There all 16 roots tie and admission is a lottery. So the change must be one the tie point cannot satisfy.

**The change: anchored ports** (`--writer-anchor`, off by default).
- Token `t` lives in cell `c = t mod 256`. The writer's address score reads port A (coordinates `0..8`) against root `c / 16`, and port B (coordinates `W-8..W`) against root `c mod 16`.
- With the option on, the table's two port blocks for token `t` are set to exactly those two SpiralCore roots. This happens once when the model is built, and again after every optimizer step: a projection, so weight decay and gradients cannot move them. The other `W - 16` coordinates stay learned.
- **Why the tie point no longer satisfies the objective:**
  - With anchored ports, the writer's address score for cell `c` is the ports' share of the mean L1 distance from the query to every token in `c`. Up to the factor `8 / W`, it is the part of nearest's own score that the cell fixes.
  - A query whose ports sit at the centre is equally far from every cell's roots, so its ports add nothing to the target's margin over any rival in another cell.
  - A trunk that already ranks the target first by nearest gains margin by moving its ports toward the target's roots. Seed 2's trunk did (6,909). The recipe's own token objective (`cell` or `nearest`) therefore pulls the ports there on every row it can inform.
  - Unlike claude-150's `voronoi`, no separate address hinge is left to lose. And the table's ports cannot fall to the centre together with the query's.
- **What it does not claim:** for a trunk that carries no information about the target, the L1 pull still has the centre as its minimum, the caution in claude-289's Still open, item 1. The run in done-when 3 decides whether it holds.
- **Refusals:**
  - at width 8 the two ports are one block (`query_ports` aliases), so the option needs width ≥ 16;
  - the option refuses a model without the shared `embedding.weight`.
- **Recorded:** the option goes in `config.json` and is left out when off, so every existing run writes the same file as before.

## Cancel probe (run first)

1. No change to the writer's admission on `origin/main` since claude-289's evidence base: `git log 074d54b2..origin/main -- crates/uor-r4-float-train/src/native_vector_writer.rs` is empty, and no `writer_anchor`, `anchor_ports` or `port_anchor` appears in the float crate.
2. The item is still `ready`, with done-when 2 open, and no other open order holds its files (the claim checked overlap).
3. Serving still reads whole-table nearest, so the writer's cost case stands (claude-288 #3717, its parts).

All three are instrument checks of the project's state, not mechanism results.

**Result, 5 Nov ~13:25Z, at `origin/main` `4ea097661`: proceed.**
1. The log is empty: 0 commits touch `native_vector_writer.rs` since `074d54b2`. 0 lines in the float crate's source name `writer_anchor`, `anchor_ports` or `port_anchor`.
2. The item reads `status: ready`, and the claim's overlap check passed (draft PR #3727).
3. `fbs-export.rs` lines 23–24 still serve a `cell` or `nearest` run with the whole table's nearest readout.

## Files in scope

- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `docs/harness/prep/writer-admission*`
- `docs/loop/tasks/writer-admission/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/writer-admission.md`

## Done when

1. **The anchor.** `native_vector_writer.rs` gives each token's two port roots as a table-shaped tensor and a column mask, and the projection that writes them into a table. Both are elementwise, CUDA-only like the rest of the writer, and refuse width < 16.
2. **The option.** `LongConfig::writer_anchor` and `train-in-space --writer-anchor`:
   - applied at model build and after every main optimizer step, in both training loops;
   - recorded in `config.json` only when on.
3. **Host tests:**
   - the cell-to-root layout, for every cell, against the writer's own address order (cell `16 a + b`);
   - the width-8 refusal;
   - a `config.json` without the field reads as off;
   - with the option off, the field is not written.
4. **CUDA tests** (on a pod, finite kernel tests; no pre-registration needed, CHARTER §4.5):
   - after one training step with the option on, the port columns equal the roots exactly, and some learned column moved;
   - a query equal to token `t`'s anchored vector puts `t`'s cell first, with address score 0;
   - each has a negative gate: with the projection's call set aside, the port test fails.
5. **Gates at the head:**
   - the float crate's CPU suite with `--no-fail-fast`;
   - the stub-nvcc `--features cuda --all-targets` check;
   - the CUDA tests' log on the pod, sealed in the store;
   - `uor-harness check`, `loop gates --full`, `loop report --check`, `loop pr-check --local`.
6. **The record:**
   - a REPORT, negatives first;
   - the item's "What is missing" updated;
   - the item stays `ready` (done-whens 3 and 4 remain).

## Stop and decide if

- **The projection needs a file outside the scope** (`geometric_stack_native.rs`, the exporter): keep the projection in the training loop, which is in scope, and note what the exporter reads. Never widen the scope by hand.
- **Train-as-served quantizes the table so the roots are not exact:** project after the quantizer, or refuse the option with train-as-served, whichever keeps the served values equal to the roots; record which.
- **The fresh design review finds a way the tie point still satisfies the objective for an informed trunk:** fold its fix into the design before building, or record why not.
- **The CUDA tests need more than one short pod** (about $0.30): note it and keep going.
