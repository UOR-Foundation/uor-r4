# REPORT: infra-loop-harness, claude-147

Lab: claude. Order: [ORDER.md](ORDER.md). Item maintenance only: one new roadmap item, its rendered row and one PLAN row. The umbrella item stays open.

**Speed.** No model, run or benchmark is in this unit; nothing to time.

## What did not work, first

- **This unit records the owner's direction and makes no map.** The map is claude-148's work, under the new item.
- **The item has no issue yet.** `cargo xtask roadmap sync` makes the issue from main; that is the PM's weekly run. A sync from a branch would also rewrite other issues from older files, so this unit did not run one.
- **The PLAN row quotes only the direction's first sentence.** The item quotes the whole direction. A table cell holding both paragraphs would not stay readable.
- **The time of Casey's message is not recorded.** It reached this session through the session that started it, which gave the date and not the minute.
- **The umbrella's done-when does not move.**

## What changed

Size against `origin/main`: 3 files of docs plus this task folder, with no code.

- **`docs/roadmap/items/pipeline-map.md` (new).** It records:
  - Casey's 27 Oct direction, verbatim;
  - the three domain mismatches of 27 Oct that motivate it;
  - a three-part done-when:
    - the page `docs/model/pipeline-map.md`, with seven fields for every stage;
    - the scale arithmetic, the ranked gaps and an order of work, every number sourced;
    - the merge, and a `[claude, for Casey]` note on #2028;
  - a cancel line, a `host` done class and a scope covering:
    - the page;
    - `docs/model/architecture.md`;
    - `docs/SUMMARY.md`;
    - `PLAN.md`;
    - its own task folder and item file.
- **`docs/roadmap/README.md`:** the row, rendered by `cargo xtask roadmap render`.
- **`PLAN.md`:** one row at the top of "Owner directions": the direction's first sentence, verbatim, status `open`, dated 2026-10-27, pointing to the item.

## Measurements

None. This unit makes no measurement.

## Checks run at the head

- `cargo xtask roadmap check`: 97 items OK.
- `cargo xtask roadmap render --check`: current.

## Skills used

- **holding-the-goal.** The owner's words go into the repository verbatim, so the map is judged against what he asked, not a paraphrase.

## Still open

- **claude-148 writes the map** under `pipeline-map`. It merges main once this lands, so its ORDER names an item on main.
- **The item's issue** is made at the next roadmap sync from main.

## Plain summary

This adds one entry to the project's roadmap and one line to its plan. The entry is the owner's 27 Oct request: a map of every step of the model, from the first trained token to the last word of an answer, with the size, hardware and cost of each step. The map itself comes in the next pull request.
