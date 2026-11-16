# REPORT: one-model-end-to-end, codex-28 (the relative-offset key is built and untried)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-27](../codex-27/REPORT.md).

## What was found

The mechanism the literature recommends — a question key over a **relative offset**, not absolute
positions — is already implemented end to end **and has never been trained**:

| Piece | Where | State |
|---|---|---|
| the carry in the trainer | `HammingReads::with_carry` (`flock_read_hamming.rs:1072`), `--hamming-carry-question H` | built (claude-335), off in the p recipe |
| the export | `format.rs:2815` writes `carry_rows`; `hamming.d{depth}.carry` gains | built |
| the served read | `AskImage` with carry, Q16 gains, `ASK_CARRY_MAX` (`hamming.rs:99-110`) | built |
| the recipe | `--hamming-place KIT --hamming-place-pull 1` | uses the **place** read instead |

The two are exclusive by construction: `--hamming-carry-question` refuses `--hamming-place`, because
both change the readout read's ask heads. So the project has been running the place arm and has never
run the relative-offset arm.

## Why it matters

codex-27's trace puts the failure at the readout depth's ask key: the depth-2 ask head keeps the
assert position, the readout's does not, and the first token is the format bigram. The carry is
exactly the unbuilt lever at that depth. It is the difference between "the model has no question
key" and "the model has a question key it was never given".

## What is needed to run it

- The next02 corpus (`tokens.u16`), which is a store object, not in this worktree.
- A CUDA build of `train-in-space` on a 4090.
- The 64-step smoke, then the export, then codex-27's trace as the gate, then the full arm
  (16,384 steps, ~1.4 h a seed).

## Checks run

- **The 64-step smoke trained, exported and parsed.** Corpus `next02/input/uor-next02-inputs-92d7c0b36`
  (4 MB, 28 MB extracted), the trainer built with `--features cuda` on the 4090, the arm at
  `--hamming-ask 1 --hamming-carry-question 2`, 64 steps in 10 s. `fbs-snapshot` then
  `fbs-export --legacy-artifact` produced a `/7` artifact whose record reads
  **`asks [True, True]`, `carry_rows 2`, depths [2, 6], heads 4, kept 16**, with the table
  **`hamming.d6.carry`** present. To our knowledge this is the first artifact in the project to carry
  the relative-offset question key.
- **The `turn-memory-tail 9` variant refuses the export at 64 steps**:
  `memory.reply.keys: entry 0 is not a key this engine reads`. The base recipe (no tail) exports. The
  tail-nine chain is therefore not trainable-then-exportable at this budget; it is a separate defect
  to inspect before arm rt's configuration is rerun.
- **The 64-step trace is uninformative, as expected**: the carry gain starts at zero, and every head
  keeps the same 16 positions at row 135 at this budget. The gate needs the trained arm.
- The full arm (seed 2, 16,384 steps, ~82 min on the 4090) is running from the same recipe; its
  artifact is the one the trace and the bare-value panel are read on.
- The `--decision-trace` path refuses an unsnapped artifact ("the snap is not a decision it
  records"), so the gate uses `--hamming-trace` alone, which steps the artifact and emits each head's
  kept positions per row.
