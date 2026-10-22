# ORDER: infra-loop-pr03-render, kimi-7

Lab: kimi. Item: [infra-loop-pr03-render](../../../../roadmap/items/infra-loop-pr03-render.md). Setup row: 3. Base: `3e3967c76`.

## Goal

The item's done-when reads:

> `loop render` writes STATE, HANDOFF, LESSONS, the AGENTS.md top-ten block, `docs/roadmap/README.md` and `docs/loop/baseline.json` from committed inputs only, with no clock and no GitHub. `--check` re-renders at the stamp and needs an exact match, and two renders of one commit are byte-identical. The planted fault turns the check red: **a hand-edited STATE fails.**

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-harness/src/looprender*.rs`
- `crates/uor-r4-harness/tests/loop_render*.rs`
- `xtask/src/roadmap.rs`
- `STATE.md`
- `HANDOFF.md`
- `docs/harness/LESSONS.md`
- `AGENTS.md`
- `archive/state/**`
- `docs/loop/baseline.json`

## Done when

To fill in.

## Stop and ask if

To fill in.
