# REPORT: infra-loop-harness, claude-210 (the loop guard's false refusals; #3416 in the issue map)

Lab: claude. Order: [ORDER.md](ORDER.md). Setup row: 2. References #2028.

**Result: the loop guard stops refusing three kinds of legitimate work.** It is the owner's request: "change the harness to allow this … so we can move forward".
- **A heredoc's body** is data unless a shell runs it.
- **A variable the command sets itself** names its target.
- **An owner path** opens to the main session when the active ORDER's `## Owner approval` names it, with a time. That is the record `loop pr-check` already requires.

R10, the shared primary checkout, and every other rule are unchanged. The roadmap's issue map gains `m1-heldout-phrasing-transfer` → #3416.

**Speed.** None measured; this unit is tooling.

## What did not work, first

- **The harness suite is red on `main`, and was before this unit.** It has one failure: `preprules::tests::the_real_sheets_pass_the_shape_rule`.
  - **The cause is mine:** two prep sheets from claude-205 and claude-206, `one-model-end-to-end-overturn-2026-10-29.md` and `one-model-end-to-end-protocol-2026-10-29.md`. They lack the sheet rule's Bar, Base artifact, Data split, Steps and Minimum test power headings.
  - **It merged anyway.** The "harness rules and tests" job, which is not required, FAILED on #3411, #3412 and #3414. It was still running when each was queued, and was not read again.
  - **The fix is not in this unit.** The sheets are in `one-model-end-to-end`'s scope, not this unit's, and `loop claim` refuses a second claim while this one overlaps it. So the fix is the next unit, right after this merges.
  - **The rest of this unit's suite passed:** 676 of the lib tests, and every other target.
- **The roadmap sync ran in the wrong place.** `roadmap sync --apply` was run from the shared main checkout and wrote two files there. R10 refuses a lab's restore. The same two lines land here. The main checkout's copies equal them, and Casey can restore them with `git -C ~/uor-r4 checkout -- docs/roadmap`.
- **The installed guard is still the old one** until this merges and `uor-harness` is rebuilt. This unit was written around the old guard's refusals.

## What changed

- **`crates/uor-r4-harness/src/loopguard.rs`:**
  - `split_heredocs`, `heredoc_openers` and `feeds_a_shell`:
    - a heredoc's body leaves the command before tokenizing, and is checked as commands only when a shell or `eval`/`source` reads it;
    - an unterminated body stays in the command;
    - a `<<` inside quotes, and the here-string `<<<`, open nothing.
  - `record_assignments` and `expand_known_vars`:
    - standalone and `export` assignments are recorded per command, and `$NAME` / `${NAME}` in later redirect targets and arguments expand to them;
    - `$HOME` is known;
    - a command's own leading environment (`F=x cmd`) is not recorded.
  - Owner paths (`check_repo_rel_write`, step 2):
    - opened by `loopprcheck::owner_approval_missing` on the active ORDER, which needs the path named and an ISO UTC time;
    - never for `docs/harness/owner-paths.json`, and never for a subagent;
    - scope still applies.
- **`crates/uor-r4-harness/tests/loop_guard.rs`:** three tests, through `evaluate_guard` on the existing fixture:
  - `a_heredoc_body_is_data_unless_a_shell_runs_it`: 7 cases, 4 of them refusals;
  - `a_variable_set_in_the_command_names_its_target`: 6 cases, 3 of them refusals;
  - `an_owner_path_opens_with_the_orders_recorded_approval`: 5 cases, 4 of them refusals.
- **`docs/roadmap/issues.json` and `docs/roadmap/README.md`:** #3416, as the sync wrote it. `roadmap check` gives 112 items OK.

## Measurements

None.

## Checks run at the head

- **`cargo test -p uor-r4-harness --test loop_guard`:** 17 passed (3 new). `command_and_cross_lab_guards_refuse_violations` still refuses all of its commands. That includes `echo hacked > CHARTER.md` and `git checkout -- CHARTER.md`, whose fixture ORDER has no approval.
- **The negative gate:** with `loopguard.rs` set aside (`git apply -R` of its diff), all three new tests FAIL; restored and rebuilt, all three pass.
- **`cargo test -p uor-r4-harness --no-fail-fast`:** every target passed but the lib's `preprules::tests::the_real_sheets_pass_the_shape_rule` (676 passed, 1 failed), whose cause is above.

## What it unlocks

- **No more workarounds for the three refusals.** Labs stop writing files through the Write tool, `git apply` patches, or literal paths to get past them.
- **The next unit:** the two sheets' headings, which turns the harness suite green on `main`.

## Skills used

None was invoked this unit.

## Still open

- **The sheet-shape fix** (`one-model-end-to-end`), next.
- **Rebuilding the shared `uor-harness`** after merge, so the guard in use is this one: `cargo build` to a `.new` path, then `mv`.
- **Casey's restore** of the main checkout's two roadmap files.

## Plain summary

The safety checker that watches each command was refusing harmless work. It read the contents of files being written as if they were commands. It couldn't follow a file name kept in a variable. And it couldn't see that Casey had already approved an edit to the rule book. It now handles all three, and still blocks everything it blocked for a real reason. Separately, a check that two of my planning documents broke earlier today is still failing; fixing those two documents is the next task.
