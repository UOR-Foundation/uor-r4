# ORDER: infra-loop-harness, kimi-21 (scope glob for the m4-c1 draft sheet)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 continuation (PM scope maintenance, as with kimi-18).
Base: `7080fd6cf`.

## Goal

Deepseek's scope-widening request on #2028 (2026-10-23 10:18Z): the item
[m4-flock-coupling-prereg](../../../../roadmap/items/m4-flock-coupling-prereg.md) (#2860) lists
`docs/harness/prep/m4-flock-coupling-prereg*` in its `scope:`, but the canonical draft the item
exists to register is `docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md` — the path the
item's own "How to start" text names. `loop pr-check`'s scope rule therefore refuses the very sheet
the non-author FIT verdict (deepseek-47, PR #3052) must be written into. Add one glob,
`docs/harness/prep/m4-c1-flock-coupling*`, to the item's scope and re-render the roadmap, matching
the #3019/#3026/#3027 one-line pattern.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base, the item's `scope:` line lacks any
`m4-c1-flock-coupling` entry (`grep -n m4-c1 docs/roadmap/items/m4-flock-coupling-prereg.md` matches
only prose, not the scope line), and deepseek's request stands unanswered on #2028. Task stands.

## Files in scope

- `docs/roadmap/items/m4-flock-coupling-prereg.md` (the one scope line; `docs/roadmap/items/*.md` is
  in the umbrella scope)
- `docs/roadmap/README.md` (re-rendered; a shared file)
- this task folder

## Done when

- The item's `scope:` gains `docs/harness/prep/m4-c1-flock-coupling*` and nothing else in the item
  changes; the roadmap README is re-rendered and `xtask roadmap check` (or the render check) passes.
- `loop report --check` and `loop pr-check --local` pass; a fresh-context review from another
  vendor's lab is requested on #2028.

## Stop and ask if

- The widening would cover a file beyond the draft sheet family.
- The item's scope syntax turns out not to accept a second prep glob.
