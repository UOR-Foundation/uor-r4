# ORDER: infra-loop-harness, claude-221 (record the point-key-integer item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `c3d5cb684`.

Setup row: 2 (item-file maintenance; a records PR, as in claude-210, claude-212 and deepseek-75).

**Written by hand.** `loop claim infra-loop-harness` refused: its `PLAN.md` overlaps Kimi's open docs-status-refresh order (PR #3436). This order touches no file of that order's: Kimi's ORDER leaves `docs/roadmap/README.md` to the render workflow and adds no item file. Casey's direction for the one-model work is "screw kimi forge ahead as you were before with your plan for the single serving model".

## Goal

Record the point table's integer export as its own roadmap item, `point-key-integer`, and create its issue with `roadmap sync --apply`. claude-220's decision review set this as its condition (d): "the export needs its own item". claude-220 read row 2, so the export is the next unit ([REPORT](../../one-model-end-to-end/claude-220/REPORT.md), merged as `c3d5cb684`, #3443).

## Cancel probe (run first)

- No `docs/roadmap/items/point-key-integer.md` exists at the base, and no issue carries its marker.
- No existing item covers a served point table in the integer artifact.
  - `turn-memory-integer` serves the memory's two exact-id tables only.
  - `integer-hamming-read` serves the Hamming read.
  - `one-model-end-to-end` (#3407) is the float model's item, and its claim is locked by #3436.
- claude-220 merged with row 2.

If any fails, stop and report.

Run at about 04:40Z on 31 Oct.
- The item file is absent at the base, and `issues.json` has no `point-key-integer` entry.
- The three issues a word search matched (#3392, #3385, #3390) carry other items' markers (`turn-memory-integer`, `integer-hamming-read`, `route-plus-memory`). The sync's dry run settles the marker.
- claude-220 is merged as `c3d5cb684` with row 2.

Result: **go**. The probe reads the setting, so it is an instrument check.

## Files in scope

- `docs/roadmap/items/point-key-integer.md` (new, hand-written)
- `docs/roadmap/issues.json` (written only by `roadmap sync --apply`)
- `docs/roadmap/README.md` (written only by `roadmap render` or `sync --apply`)
- `docs/loop/tasks/infra-loop-harness/claude-221/**`

## Done when

- **The item passes `roadmap check`,** and `render --check` is clean.
- **The sync:** the dry run shows exactly one create and no unexpected update. Then `roadmap sync --apply` creates exactly one issue and writes `issues.json`.
- **Nothing else changes:** no owner path, bar, status, other item, or rendered view but the tool-rendered roadmap page, and no code.
- **The REPORT** records the commands, the counts, the sync's GitHub writes and the limits.

## Stop and ask if

A file outside scope is needed, the sync plan shows an unexpected create, update or close, or the item needs a decision row rather than an item file.
