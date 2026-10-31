# ORDER: infra-loop-harness, claude-230 (record the integer-nearest-readout item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `856edac67`.

Setup row: 2 (item-file maintenance; a records PR, as in claude-221, claude-223 and claude-225).

**Written by hand.** `loop claim infra-loop-harness` refuses while Kimi's #3436 holds `PLAN.md`. This order touches none of that order's files.

## Goal

Record the fault that stopped `point-key-integer`'s run (claude-229, #3460, row 0) as its own item, `integer-nearest-readout`, and make the rerun depend on it.

**The fault,** found by claude-229's run and named by the code:
- **The float model's decode:** (r-drop) trained with `objective: "cell"`. `in_space_long` decodes such a model as the whole table's nearest token: `decodes_nearest`, "Whether replies decode as the whole table's nearest token, not the writer's pick", `matches!(self, Self::Nearest | Self::Cell)`.
- **The integer session's decode:** the served step emits the native writer's token (`stack_step_fbs`, `b.native_token`). It computes whole-table nearest only beside the writer, for timing (`stack_whole_table_nearest`, "the readout a model trained with the cell objective decodes, timed beside the writer").
- **So** integer greedy's first token equals claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 on p-w.

## Cancel probe (run first)

- No item serves nearest in the integer stack session (`grep` over `docs/roadmap/items` for serving nearest and the native writer).
- The roadmap sync's dry run plans nothing.

If any fails, stop and report.

Run on 31 Oct at the base:
- among the items mentioning nearest and integer, none serves nearest in the stack session. Only point-key-integer's units name the fault, from claude-229;
- the dry run plans nothing ("nothing to change").

Result: **go**. The probe reads the records, so it is an instrument check.

## Files in scope

- `docs/roadmap/items/integer-nearest-readout.md` (new) and `docs/roadmap/items/point-key-integer.md` (`depends_on`, and its unit 5)
- `docs/roadmap/issues.json` and `docs/roadmap/README.md` (written only by `roadmap sync --apply` and `render`)
- `docs/loop/tasks/infra-loop-harness/claude-230/**`

## Done when

- `xtask roadmap check` passes, `render --check` is clean, and the sync's dry run plans exactly the new issue and #3450's body update.
- The REPORT gives the negatives first, and a fresh review runs on the diff.
- After the merge, `roadmap sync --apply` creates the issue and updates #3450.

## Stop and ask if

The item needs a decision only the owner can make.
