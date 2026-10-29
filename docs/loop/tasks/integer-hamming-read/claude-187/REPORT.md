# REPORT: integer-hamming-read, claude-187 (the item's scope, and the design the port follows)

Lab: claude. Order: [ORDER.md](ORDER.md). The item is not done. It has no GitHub issue; its claims and merges are noted on #2028.

**Speed.** No serving change in this unit.

## What did not work, first

- **The unit's first plan, building the read's artifact format and its copy heads, could not be done under the item's scope.**
  - The session guard holds writes to the item's `scope:`, read from `origin/main`.
  - That scope omits `stack/format.rs` (the shape, schema and exporter), `fbs_snapshot.rs` (the snapshot), `stack/synthetic.rs` and `stack/tests.rs`, all of which the port needs.
  - claude-157 met the same limit: a scope cannot be widened inside the unit that needs it.
  - So this unit widens it, and the build is the next unit.
- **A builder subagent, launched for the build, stopped before writing code.**
  - In this session a subagent starts near 140k tokens, mostly the tool definitions loaded at startup. It reached the 150k hand-back after three reads.
  - Nothing was committed, and its worktree and branches are removed.
  - **For this session, the build runs in the main session or in agents with a trimmed tool set.** Reviewers, which only read, still fit.
- **The float read's tie rule is not defined.** `choose_sorted` sorts with `arg_sort_last_dim(true)`, and its doc says "Ties may come out in another order". The integer rule is therefore a decision (below), and float parity on exact ties is a question for the GPU parity unit.

## What changed

- **`docs/roadmap/items/integer-hamming-read.md`:**
  - **`scope:`** gains `crates/uor-r4-integer/src/stack/format.rs`, `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/stack/synthetic.rs` and `crates/uor-r4-integer/src/stack/tests.rs`.
  - **"What is missing"** gains the port's design:
    - the tensor layout;
    - the artifact's `lut-stack/6`, with read-free exports byte-identical;
    - the rotations stored raw and normalized at serving by the raw gates' exact path;
    - codes per vocabulary entry from `writer.table`;
    - the tie rule, as a recorded decision;
    - the session refusing whatever it does not yet serve;
    - the order of the next six units.
- **This ORDER and REPORT.** No code changes.

## Measurements

None: no run. The design's facts are quoted from `crates/uor-r4-float-train/src/flock_read_hamming.rs` (`HammingRead::new`, `save`, `load`, `choose_sorted`, `pack_signs`, `rotate`) and from `crates/uor-r4-integer/src/stack/format.rs`, `session.rs` and `fbs_snapshot.rs`, at `98a2f3b52`.

## Checks run at the head

At `bbee17d59`, and then the review's fix at the head (docs only):
- **`cargo xtask roadmap render`:** the README is unchanged; the scope is not rendered there.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **The fresh decision review (D68), at `bbee17d59`:** APPROVE. It checked every design claim against the code, the four files' need, and no open PR touching them.
  - **Medium:** it could not confirm that the copy heads' table is `writer.table` on the float side. I checked it: `geometric_stack_native.rs:1142` gives both reads `embedding.weight`, which `fbs_export.rs:279` writes as `embed`, which is `writer.table`.
  - **A rounding the note missed,** found at the same call site: the stream read's output goes through `q16` and the readout's does not. It is added to the note.
  - **Lows:** the epsilons, 1e-12 against 0, agree at zero, so a zero quaternion is added to the round-trip test plan. Byte identity is a claim the next unit's test pins.

## Skills used

- **holding-the-goal.** The unit stopped on the scope limit instead of writing around the guard. It records the limit, widens the scope through a reviewed decision, and leaves the build to a unit that can hold it.

## Still open

- **Every part of the item's done-when.** The next unit is the artifact (step 1 of the design's order).
- **Float parity on exact distance ties** depends on candle's CUDA sort, which is unchecked. That is for the GPU parity unit.

## Plain summary

This unit is preparation. The task is to make the integer version of the model run the memory lookup that so far exists only in the trained float model. But the task's list of files it may change left out four that the work needs, and the rules don't allow widening that list in the same piece of work that uses it.

So this unit widens the list, and writes down the design the next piece of work will follow: exactly what the lookup stores, how its rotations are computed in whole numbers, how equal distances are ordered, and that the model refuses any part it can't run yet. No code changed.
