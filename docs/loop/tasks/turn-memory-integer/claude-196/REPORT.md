# REPORT: turn-memory-integer, claude-196 (decision: serve both tables, and how a session keys them)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3392. The item stays `ready`: this unit changes its done-when and its order of units, not its code.

**Speed.** No serving change: a decision unit.

## What did not work, first

- **The item, as written on 28 Oct, would serve a model that does not work.** It served claude-161's opening table alone. claude-194 has since shown that the trunk trained with both tables depends on them: without the memory, its DEV hits fall to 4,101 / 3,807, below the empty-table control's 10,079 / 11,171.
- **The first key rule was wrong twice, and the decision review caught both.**
  - **Round 1:** I had the session key over its last `context` tokens "as a window does". Training and the trainer's greedy decode disagree there. Training moves reply rows whose prompt has left the window, with an empty turn. The greedy decode reads no reply table there.
  - **Round 2:** keying over the session's own `context` matches training only at 192, and the decode lengths in use are 16, 64 and 192.
  - **Round 3:** training's windows are fixed 192-token blocks from a document's `<|bos|>`, not a sliding window.
  - The amended rule follows training's blocks, whatever the session's `context`, carried in the artifact. The two differences that a session cannot reproduce are measured items for the parity unit: a document's last, partial chunk, and the greedy decode's sliding window.
- **The turn agreement is measured, not built in.** The opening table's read uses a `<|bos|>`/`<|eos|>` anchor. The reply table's training rows find their turn from the response mask. These agree at the corpus's openings by claude-154's measurement, and a prompt with an internal `<|eos|>` could part them. The item now says so and requires that test row.
- **The review took three rounds, against the conventions' two.**
  - A decision has to pass its review before it is logged (D68), and each round found a different, specific gap in the key rule.
  - The third round was a narrow check of the second round's fix.
  - It passed the decision on two wording fixes, which are applied at `64ed1ff35`.

## What changed

- **`docs/roadmap/items/turn-memory-integer.md`:**
  - **Why it matters:** both tables, since claude-194, with its numbers.
  - **What is missing:**
    - the keys as the float side forms them, each sourced to its lines:
      - the turn anchor, and its measured agreement with the mask-based turn;
      - the opening and reply keys, and the share rule;
      - one role signal, where the reply begins;
      - training's 192-token blocks as the key window, carried in the artifact;
      - the two measured items for parity;
    - the moves, after the Hamming readout, as in the float trunk.
  - **Done when:**
    - export of both tables, their weights and the key window;
    - the session's keys, checked against a reference of the float rule, with rows for a reply cut by the window, an internal `<|eos|>`, and a session at a `context` other than 192;
    - both audits;
    - the restore refused;
    - parity against claude-194's (r) at step 16,384, both seeds, pinned to its five files per seed, in place of claude-161's (m).
  - **The order of units:** the export, the keys, the moves, the parity, then the cost.
- **`docs/loop/tasks/turn-memory-integer/claude-196/ORDER.md`:** the same rule, kept consistent with the item through each round.
- No code. The rendered roadmap is unchanged: the item's title and status did not change.

## Measurements

None: a decision. The facts it rests on are cited in the item, each to a file and lines.

## The decision review

Fresh-context reviewers. Each read the chain first: PLACE.md's order, claude-161's and claude-194's REPORTs, the item, the float key code, the integer session, and CHARTER §3 and §5.1.
- **Round 1, at `b41750ff6`: FAIL as written.**
  - The decision itself was judged sound and not the owner's call: no §3 rule or cap is touched, the moves are element-wise with no softmax and no matmul, and parity stays on the GPU.
  - The findings, all fixed: the window rule was misstated; the turn agreement was not stated as measured; the role signal differed between the ORDER and the item; the parity checkpoint was not pinned to files; two line citations were off.
- **Round 2, at `276dc4e9b`: FAIL.** It confirmed every round-1 fix, and found that the session's `context` need not be 192. Fixed: the keys use the trained window, carried in the artifact.
- **Round 3, at `2dbece59e`: PASS once two wording fixes are made.** The two fixes, applied at `64ed1ff35`:
  - name one window rule, training's blocks;
  - have the exporter write the key window.

The owner log is on #3392.

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`, at the head before the review record.
- The store listing of the parity checkpoints: 5 files each under `s{1,2}-fc66ed74/r-s{1,2}/models/step-016384/`.

## Skills used

- None was invoked this unit.

## Still open

- The item's five units, in the order above. The first is the export of both tables and their weights.

## Plain summary

The plan for running the model's memory in whole-number arithmetic now covers both of its tables, the one for a reply's first word and the one for every word after it. The model was trained to need both. It also says exactly how a running conversation forms the lookup keys, so they match what the model saw in training.
