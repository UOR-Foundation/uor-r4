# REPORT: infra-loop-harness, claude-97

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #3189. References #2904.

## What did not work, first

- **The guard refused what the loop's own rules allow.** `docs/loop/README.md` lists "new rows on its notebook page" as always allowed, and `loop pr-check` accepts an append to the item's `notebook:` page. But the PreToolUse guard's always-allowed list had no notebook term. claude-95 (#3186) had to leave its row in its REPORT, and claude-94 (#3188) needed a separate scope unit, claude-96 (#3191), before it could write its row.
- **The first version of the new test passed for the wrong reason.** It committed the fixture's ORDER to `origin/main`, where the guard treats an ORDER as closed history, so no ORDER was active and every write was allowed. The test now keeps the ORDER on the branch, as a real unit does, and its "another page is refused" assert proves an ORDER is active.
- **One guard test fails in this checkout, with or without this change.** `each_clients_canary_write_is_refused` runs against the real repository. Inside a `.worktrees/claude-*` checkout, the antigravity canary meets the cross-worktree refusal first ("belongs to another lab"), so the reason it expects never appears. It fails identically with the fix set aside. It depends on where the checkout lives, not on the guard's scope code. CI's checkout is not under `.worktrees/claude-*`.

## What changed

- **`crates/uor-r4-harness/src/loopguard.rs`:** the scope check reads the item once from `origin/main` (as before, for its `scope:`). It also allows that item's `notebook:` page. A `notebook:` line written only on the branch grants nothing, because only `origin/main` is read. `loop pr-check` still enforces that the edit only appends.
- **`crates/uor-r4-harness/tests/loop_guard.rs`:** a git-backed test, `the_items_notebook_page_on_origin_main_is_always_writable`. Its fixture is its own repository, with `GIT_*` removed. It pins three things:
  1. the item's notebook page on `origin/main` is writable outside the scope;
  2. another notebook page is refused;
  3. a notebook line changed only on the branch is not honoured.

## Measurements

None. This is an instrument fix.

## Checks run at the head

- **Negative gate:** with the `loopguard.rs` change set aside (`git apply -R` of its own diff), the rebuilt test fails on its first assert ("the item's notebook page on origin/main must be writable outside its scope"). Restored, it passes.
- `cargo test -p uor-r4-harness --test loop_guard`: 11 of 12 pass. The one failure is the canary test above, which fails identically without this change.
- `cargo test -p uor-r4-harness --no-fail-fast`: 1,137 passed and 6 failed. None of the six involves the guard's scope code:
  - four lib tests (three `prlintrules` and `prgates::...agree_with_python`) come from claude-94's doc comment, which the D23 interlock counts; they are repaired in claude-98 (#3195);
  - `prep::tests::the_real_sheets_are_readable` comes from #3190's d41 sheet, which has no verdict line (antigravity's file, noted on #2028);
  - `each_clients_canary_write_is_refused` is the checkout-location failure described above.
- The fresh review is in the PR's review record.

## Skills used

- scope-guard: the guard now matches the documented rule, so later units need no scope workaround for their notebook rows.

## Still open

- **The installed guard binary.** The fix acts once the shared `~/.cargo/bin/uor-harness` is rebuilt from `main`. That binary is not this lab's to overwrite (run file, Part 2).
- **claude-95's pending row** for `docs/research/notebook/m1-discrete-attention.md` (its text is in #3189 and in claude-95's REPORT). It can be added once the guard is current.
- **`m7-studio-self-host-page-assets`** needs a `notebook:` line before its own close can write a row.

## Plain summary

The loop's safety hook stopped labs from adding a result row to their own item's notebook page, though the loop's rules say that is always allowed. Now it allows that one page, read from the main branch, so a lab can't grant itself other files.
