# ORDER: infra-loop-harness, deepseek-59 (sync: flip the three delivered-but-unflipped items)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item metadata). Base: `1cb89e81a`. `References #2904`.

## Owner approval

Casey approved these three flips in the deepseek chat on **2026-10-23T21:04Z**, answering the roadmap-sync question with the selected option **"Yes, flip them (Recommended)"**. The owner-facing audit with the merged-commit evidence for each item is on #2028 (`6374552436`). This unit changes only `status:` lines and the rendered roadmap; it changes no bar, hold, dependency, plan or result, and the PM's normal `roadmap sync` still owns the GitHub issue state.

## Goal

Flip the three items whose done-when is met and whose merged evidence is on main - `m2-fresh-heldout-panel` (#2849), `m6-reasoning-panel-freeze` (#2866), `m1-position-mechanism` (#2843) - to `status: done`, regenerate `docs/roadmap/README.md` with `cargo xtask roadmap render`, and change nothing else.

## Cancel probe (run first)

At base `1cb89e81a`, each item still reads `status: ready` although the delivery evidence is merged: #2849 -> PR #2985 `dfa28932e`, `20f618132`, PR #3039 `f0dfca2b0` + the owner's M2 fresh-panel bar; #2866 -> PR #2977 `a94ae87f` FIT, `2ec16d513`, `683e182ba`; #2843 -> `3e3967c7`, PR #3046 `aac963833` + the owner's 02:36Z bar registrations. `cargo xtask roadmap render` then moves ready 33->30 and done 28->31. Task stands.

## Files in scope

- `docs/roadmap/items/m2-fresh-heldout-panel.md`
- `docs/roadmap/items/m6-reasoning-panel-freeze.md`
- `docs/roadmap/items/m1-position-mechanism.md`
- `docs/roadmap/README.md` (rendered, never hand-edited)
- this task folder

## Done when

- The three item files differ from base only in the `status:` line (`ready` -> `done`); no other line changes.
- `cargo xtask roadmap render` (not hand editing) produces the README change; `cargo xtask roadmap check` and `roadmap render --check` agree (ready 30, held 26, decision 4, done 31).
- `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted review records the owner approval and evidence; REPORT cites the owner words and ISO time.

## Stop and ask if

- Any item other than the three needs a status change, or a bar/hold/result would change; a scope or issue sync is needed.