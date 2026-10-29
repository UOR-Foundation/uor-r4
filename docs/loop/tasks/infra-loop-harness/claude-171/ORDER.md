# ORDER: infra-loop-harness, claude-171 (item maintenance: the serving follow-ups get an item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `2cc61c247`.

Setup row: 2 (item-file maintenance: a new gap item and PLAN's pivot list).

## Goal

claude-170 (#3358) set `fbs-serving-kernels` done. Its exact structured kernels took the M1's served token from 270 to 3,042–3,044 tok/s. Its fresh decision review approved on a condition: the open follow-ups need a real record, not a line inside a done item (L-001). The follow-ups are a NEON kernel the audit allows, and fewer golden candidates in the snap.

The map's gap 3 and R3 also still name NEON kernels as the change, and R3's "pass unlocks" claims a per-value rate near the hardware's, which is not shown. The served step now costs about 5.2 ns a value.

This unit:
- adds the item `fbs-serving-vector-kernel` (`ready`). Its scope holds the serving files and `docs/model/pipeline-map.md`, and its done-when 1 updates the map's gap 3 and R3. The map is outside this umbrella's scope, and in no open item's.
- updates PLAN's pivot list: `fbs-serving-kernels` is done, and the new item is its follow-up.

## Cancel probe (run first)

Run at `2cc61c247`:
- **No item** named `fbs-serving-vector-kernel` exists.
- **No open PR** touches `PLAN.md` or `docs/roadmap/items/`.
- **The map** is in the scope of no `ready` item: only `pipeline-map` and `pipeline-map-revision` list it, and both are done.

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `docs/roadmap/items/fbs-serving-vector-kernel.md` (new)
- `PLAN.md` (the pivot list's serving lines)
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-171/**`

## Done when

1. The new item holds:
   - the owner's words and the measured gap (claude-169's profile, the 5.2 ns a value);
   - what is missing, with sources;
   - a done-when whose first step updates the map's gap 3 and R3;
   - `## How to start`, `## Rules that apply` and `## Sources`;
   - a checked scope and `status: ready`.
2. PLAN's pivot list names `fbs-serving-kernels` done and the new item.
3. `cargo xtask roadmap render --check` passes. The PR is merged and verified.

## Stop and decide if

- **A scope file does not exist and is not a file the item's units create.** Then leave it out.

No step waits on Casey.
