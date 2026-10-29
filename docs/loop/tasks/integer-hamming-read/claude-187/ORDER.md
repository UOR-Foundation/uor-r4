# ORDER: integer-hamming-read, claude-187 (the item's scope, and the design the port follows)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `98a2f3b52`.

## Goal

The item's done-when 1 asks the integer runtime to run the whole of claude-146's Hamming read. **The item's scope cannot hold that work.** It lists `stack/hamming.rs`, `stack/mod.rs`, `stack/session.rs`, the two FBS bins and the float exporter. The read also needs:
- **`crates/uor-r4-integer/src/stack/format.rs`:** the stack shape, the schema choice, and the FBS exporter that writes the sections;
- **`crates/uor-r4-integer/src/fbs_snapshot.rs`:** `FbsTrunkSnapshot`, the exporter's input;
- **`crates/uor-r4-integer/src/stack/synthetic.rs`:** the synthetic FBS trunk the controls build on;
- **`crates/uor-r4-integer/src/stack/tests.rs`:** the stack's controls.

The session guard holds every write to the item's scope, and that scope is read from `origin/main`. So it cannot be widened inside the unit that needs it (claude-157's lesson).

**This unit, under one decision review (CHARTER §5.1):**
1. **The item's `scope:`** gains those four files.
2. **The item's "What is missing" gains the design** the next unit builds from. It comes from reading the float read (`flock_read_hamming.rs`) and the integer format in this session:
   - the tensor layout;
   - the quantization;
   - storing the rotations raw, and normalizing them at serving as `lut-stack/5`'s raw gates are;
   - the session refusing what it does not yet serve;
   - the tie rule, as a recorded decision.

No code changes in this unit.

## Cancel probe (run first)

Run at `98a2f3b52`:
- **The integer runtime has no Hamming read.**
- **The item's scope** omits the four files.
- **The float read** is on `main`, and its checkpoints are stored (`results/claude/hamming-read-2026-10-27/lang-6d8bac25/hamming-s{1,2}/models/step-*/`).
- **No open PR touches** `stack/format.rs`, `stack/synthetic.rs`, `stack/tests.rs` or `fbs_snapshot.rs`; the only open PR is antigravity's #3323.

A first attempt in this unit, a builder subagent, ran out of context before writing code: subagents here start near 140k tokens. Nothing was committed, and its worktree is removed. **Go. Class: instrument (the item's scope).**

## Files in scope

- `docs/roadmap/items/integer-hamming-read.md`
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/integer-hamming-read/**`

## Done when

1. **The item's scope** lists the four files, and its design note is in "What is missing".
2. **`cargo xtask roadmap render`** leaves the README current.
3. **A fresh decision review passed.** Its owner log is posted on #2028 (the item has no issue).
4. **`loop pr-check --local` and `loop gates --full` pass.** The PR is merged and verified.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **the review finds a design choice the float read contradicts:** fix the note, or leave that choice open for the build unit.
