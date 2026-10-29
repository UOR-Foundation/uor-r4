# REPORT: infra-loop-harness, claude-201 (the item for scoring one model end to end)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3407.

**Speed.** No serving change: a roadmap item and the assessment it rests on.

## What did not work, first

- **Revisions 1 and 2 of the assessment were wrong, and the fresh decision review failed both.** What revision 1 got wrong:
  - it overstated the diagnosis. It read a key match (1,194 / 1,226) as a right answer, and the ceiling is 607. It read the reply key's 0.914 as dialogue overlap, but a reply-only control already gets 8,089;
  - it missed the sealed Fresh V2 panel and the M1 fresh-transfer bar;
  - it had the parameter count wrong (1,112,576, not 5–10M);
  - it called the M1 bar a lab decision.

  Revision 2 named a control checkpoint that does not exist, (m0) where turn-memory-reply's control is (r0). It would also have used Fresh V2 for a decision, with no noise band. Revision 3 passed. Each failure is recorded in the assessment's header.
- **The diagnosis is still an inference until the item's step 2 runs.** The check is whether (r) falls below (r0) where no key matches. If (r) is ahead at both seeds, the co-adaptation fear does not hold on DEV's unnamed rows.
- **The M1 bar question is Casey's.** It is posted once, on #2028 (`6507594237`), and nothing waits on it.

## What changed

- **`docs/roadmap/items/one-model-end-to-end.md`** (#3407): `ready`, `needs_gpu: true`, depending on `turn-memory-reply`. Its done-when has four parts:
  - **The census** (laptop): the TRAIN size conflict, the TRAIN reply index, and which panel holds new templates. The K16 fresh slice stays unopened.
  - **The overturn check** (GPU, under a prep sheet): (r) against (r0) and memory-off on DEV's 3,438 unnamed rows, pooled. The band, `max(2·√(b+c), the two-seed spread)`, and the cells are fixed in advance. Fresh V2 and the greedy samples are descriptive only.
  - **The protocol** sheet for the training run: the transport controls, the Flock read and the type spin, and greedy against a Flock rank decode rule.
  - **Tokens per second first,** and a notebook row per result.
- **The item's scope:**
  - its lab, prep and task folders;
  - the float-training notebook;
  - `in_space_long.rs` and `train-in-space.rs`, for the GPU check's per-row scoring.
- **[ASSESSMENT.md](ASSESSMENT.md)**, revision 3. It holds the chain token → vector → vector → integer → token, with SpiralCore's transport as the router and the position, and the Hamming read as the point where vectors become integer codes. It also holds the data and panels, the literature, and the order of work.
- **`roadmap sync --apply`** created #3407. It also refreshed four issue bodies that lagged their item files on main (#3380, #3385, #3392, #3393). `docs/roadmap/README.md` is re-rendered.

## Measurements

None: an item and its plan.

## Checks run at the head

- `cargo xtask roadmap check`: 111 items OK.
- `roadmap render`: README updated.
- `loop report --check` and `loop pr-check --local`: PASS at this REPORT's commit.

## Skills used

- None was invoked this unit.

## Still open

- **The item's step 1, the census,** is next. The census instrument is a standalone Rust crate under `docs/labs/one-model-end-to-end/instrument/`, reusing the fresh-transfer instrument's readers where it can.

## Plain summary

The project has a new task: score one model on prompts it has not seen. It rests on a reading of the whole project, and of the published work on models like ours. That reading found our test set mostly checks whether the model recalls the training templates. So the first steps are a count of the data, and one small check that could prove that reading wrong, before any training.
