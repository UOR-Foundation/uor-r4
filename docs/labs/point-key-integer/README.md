# Lab record: point-key-integer

Item: [point-key-integer](../../roadmap/items/point-key-integer.md) (#3450). It serves claude-220's point key, form C, in the integer artifact on the laptop.

## The run (claude-229, 31 Oct 2026): row 0, an instrument fault

- **Sheet:** [point-key-integer-run-2026-10-31](../../harness/prep/point-key-integer-run-2026-10-31.md). It passed FIT in three non-author rounds and a pre-run review.
- **Where it ran:** on the laptop's integer engine (Apple M-series), under the owner's allowance, with no GPU.
- **Store:** `caseyallard/uor-r4-store:results/claude/point-key-integer-run-2026-10-31/`, 101 files under `SHA256SUMS`: the `/7` and `/8` artifacts, the prompts, the 24 decodes, the grade and the logs.

### What held: done-when 2's instruments and the files

- **The fresh panel p-x** (`heldout-panel ... seed=2131`): 1,200 conversations and 149,800 tokens, a second draw byte-identical.
  - It is stored at `inputs/claude/heldout-panel-p-x-2026-10-31/`.
  - Its census is [census-p-x-2026-10-31.json](census-p-x-2026-10-31.json): 4,212 openings, 0.000 of prompts in TRAIN, 0.627 verbatim replies, and 703 keyed.
- **`/7`,** one per seed, from claude-227's snapshots (each retained once, matching the store):
  - s1 `19174e5e…`, s2 `a3065ffb…`, s3 `22192fef…`;
  - 51,310,848 B each, `rec_gates` `geometric-retention`.
- **`/8`:**
  - s1 `47fb4633…`, s2 `7713ffbf…`, s3 `b06cd97a…`;
  - 161,202,592 B each, with 212,968 rows in 67,446 windows;
  - read back as built;
  - about 2,010 s each, the three in parallel. A 200,000-token slice took 51.7 s for 5,680 openings.
- **The prompts:** DEV, p-w and p-v at 310, 1,202 and 393 at every seed, as claude-224 found; p-x at 1,204.
- **The 24 decodes** all exit 0.
  - Every served opening matched the reference scan (`reference_checked` = `keyed`): 19 on DEV, 1,172 on p-w, 385 on p-v and 1,189 on p-x a seed, 8,295 in all.
  - Resident memory is about 190.6 MB for greedy and 410 MB for served.
- **The grade** refused nothing.

### What did not: the integer decode is not the trained model's

- **The grade's every reading is empty.** On every split and seed, templates and exact replies show no discordant pair: (0, 0). So `carries` is false everywhere, including p-w, where claude-220's float key read (50, 9), (44, 4) and (47, 5).
- **Integer greedy against claude-220's float greedy, the same prompts at seed 1** (memory on):

  | Split | Integer first token = float's | Float first = reference | Integer first = reference |
  |---|---|---|---|
  | DEV | 0 of 310 | 129 of 310 | 0 of 310 |
  | p-w | 2 of 1,202 | 179 of 1,202 | 4 of 1,202 |

  The first p-w reply shows it plainly. Float: `1155 302 311 14 …` against the reference `1155 302 311 16 1`. Integer: `341 3529 4057 353 …`, running to 64 tokens with no end.
- **The cause, named by the code (not yet tested by a run):** the readout.
  - (r-drop) trained with the cell objective (`objective: "cell"`), and `in_space_long` decodes such a model as the whole table's nearest token, not the writer's pick (`decodes_nearest`, `Self::Nearest | Self::Cell`).
  - The integer session serves the native writer (`stack_step_fbs`). It computes whole-table nearest only beside the writer, for timing (`stack_whole_table_nearest`, "the readout a model trained with the cell objective decodes").
  - claude-227's parity compared the writer's token on both sides, so it could not see this.
- **So this is row 0: an instrument fault.** Nothing is counted, and form C's bar is untested in integers.
- **p-x's reading under the fault carries no information about the mechanism.** Neither arm produced one template or exact match anywhere. The sheet's row 0 is "Repair in a new unit, then rerun", and p-x stays the deciding panel for the rerun.

### What is next

1. **Serve the trained model's readout in the integer artifact:** whole-table nearest for a cell-objective model, flagged in the artifact from the run's objective.
   - Its check is end-to-end token parity of integer greedy against claude-220's float greedy on DEV and p-w (memory on), which are stored and need no GPU.
   - That parity also covers the turn memory and the Hamming read, since neither has been checked on a trained model.
2. **Then rerun this sheet's steps 2–8 on p-x.** `/8`'s point table does not depend on the readout (the TRAIN windows are teacher-forced), so it may be reused if its header can be rewritten. Otherwise it is rebuilt.
