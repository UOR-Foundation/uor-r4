# REPORT: infra-loop-harness, claude-171 (item maintenance: the serving follow-ups get an item)

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: item-file maintenance, no model run.

## What did not work, first

- **The map could not be fixed in this unit.** `docs/model/pipeline-map.md` is in the scope of no `ready` item, and this umbrella's scope does not cover it. So the map's update is the new item's done-when 1, and its scope lists the map. Until that unit runs, gap 3 and R3 still name NEON kernels.
- **The new item's speed target is not set.** Neither follow-up is measured, so the item asks for "a faster exact change" with no bar on how much faster.

## What changed

- **`docs/roadmap/items/fbs-serving-vector-kernel.md` (new, `ready`, area serving, `done_class: host`):**
  - **Why it matters:** the owner's words; `fbs-serving-kernels`' result (270 → 3,042–3,044 tok/s, about 5.2 ns a value against 57); claude-169's profile (snap 32%, Hamilton 21%).
  - **What is missing:** the stale map; fewer golden candidates (the query's magnitude order, with the lowest-index tie rule kept); a NEON kernel within the audit's rules (no multiply-class instruction, no `fmov`, in the kernel list if outside `step`'s direct calls).
  - **Done when:**
    1. the map's gap 3 and R3 updated;
    2. a faster exact change, held equal to the old kernel, with byte-identical replies and the audit passing;
    3. its speed against the previous sealed binary;
    4. a notebook row and a REPORT.
  - **Scope:** the serving files, the audit script, the map, its task folder, the notebook and the item.
- **`PLAN.md`, the pivot list:** `fbs-serving-kernels` is done (claude-163 to claude-170) and its follow-ups are the new item. The side-by-side note now names the new item.
- **`docs/roadmap/README.md`:** re-rendered.

## Measurements

None: no run.

## Checks run at the head

- `cargo xtask roadmap check` and `render --check`: in the review record.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The unit meets claude-170's decision-review condition, which asked for a real record of the follow-ups.

## Still open

- **The new item's first unit:** the map's gap 3 and R3, then fewer golden candidates or a vector kernel.

## Plain summary

The work item for making the laptop model faster was closed with two ideas left over. They now have a work item of their own, so they won't be lost, and so will a fix to the project map, which still describes the old plan.
