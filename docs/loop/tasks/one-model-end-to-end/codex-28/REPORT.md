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

- None yet: the arm needs the corpus and a GPU build. The ORDER's step 1 is the smoke.
- The flag's prerequisites and exclusivity were read from the trainer's parser; the export and the
  served application from the code cited above.
