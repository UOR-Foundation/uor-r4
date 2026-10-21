# ORDER: infra-loop-harness, kimi-2 (repair: un-serialize the scope lines)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `fa28f9339`. Repair of kimi-1 (PR #2929), after the counted CHANGES record at `fa28f93392c83dc22abd52a1022e9a80302bad58` (claude, uor-review v1, fresh context).

## Done in kimi-1 (history)

- Claimed the item on #2904 and wrote the first ORDER; commit `add1b4ea7`.
- Added a `scope:` line to all 51 ready items and the `Closes` step to seven loop PR items; commit `952a6e6ba`.
- Wrote the REPORT, opened draft PR #2929, requested the D59 review; commit `fa28f9339`.

## Goal

Address every point of the CHANGES record so the scopes let labs work in parallel:

1. HIGH 1 — narrow the loop items: drop `docs/loop/**` (the task folder is always allowed), narrow `crates/uor-r4-harness/**` to each PR's own modules and tests, drop `docs/harness/**` and `docs/integration/DECISIONS.md` where the item does not edit them, and leave the shared registration files (`src/lib.rs`, `src/main.rs`, the dispatcher, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`) out of every scope pending Casey's decision, which Claude is requesting.
2. HIGH 2 — replace `docs/labs/**` with `docs/labs/<item-id>/`, `docs/harness/prep/**` with `docs/harness/prep/<item-id>*`, and drop `docs/integration/NOW.md` (and the analogous shared `docs/integration/project-track.md`) from the non-loop items.
3. MEDIUM — add the files pr03's and pr08's done-when write (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`, `AGENTS.md`, `archive/state/**`; `STATUS.md`, `PLAN.md`, `archive/**`), and say in the items that keep owner paths (`AGENTS.md`, `docs/integration/DECISIONS.md`, `.github/workflows/release.yml`) that changing one needs the owner's approval recorded in the ORDER.
4. LOW — correct the kimi-1 REPORT (`src-tauri/` and `models/` are new folders, not existing paths) and put the real issue numbers in the seven step sentences (pr03 #2906, 7c #2912, pr08 #2913, pr09 #2914, pr10 #2915, pr11 #2916, pr12 #2917).

## Cancel probe (run first)

Is the counted CHANGES record at `fa28f9339` still the last counted record on PR #2929, with its points unaddressed in the branch? Result at `fa28f9339`: yes — `loop review --check --pr 2929` reports "the last counted record is CHANGES". The repair stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/*.md` — replacement `scope:` lines, the seven issue-number sentences, three owner-path notes.
- `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md` — the LOW factual correction.
- `docs/loop/tasks/infra-loop-harness/kimi-2/**`

## Done when

- Every CHANGES point is answered in the branch, and the replacement scopes no longer overlap through `docs/loop/**`, `docs/harness/**`, `docs/labs/**`, `docs/harness/prep/**`, `docs/integration/NOW.md` or `crates/uor-r4-harness/**`.
- `xtask roadmap check` and `xtask roadmap render --check` pass; `loop pr-check --local` fails only `scope` (expected until a scoped base lands).
- The repair REPORT is written, and a new review request naming the new head is posted on #2028.

## Stop and ask if

- Casey's answer on the shared registration files (being asked by Claude) arrives mid-repair and changes what the scopes should say.
- Any file outside the list above needs to change.
