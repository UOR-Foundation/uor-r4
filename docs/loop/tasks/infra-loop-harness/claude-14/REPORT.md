# REPORT: infra-loop-pr03-render, claude-14 (loop PR 3: loop render for every view, --check at the stamp, determinism)

Lab: claude. Base: `32092fca6`. Order: [ORDER.md](ORDER.md), committed (`1c1f1a584`) before the work. Its cancel probe found no `loop render`, and only `docs/roadmap/README.md` rendered today.

Four fresh-context builders wrote the code, one per commit, plus a finishing round. The main session re-ran the gates and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **The item's done-when is met only in part, as the ORDER's architecture calls narrowed it.**
  - `docs/loop/baseline.json` is deferred to loop PR 5. The loop page lists it as rendered but does not define its contents, and its first consumer is PR 5's `pr-check`.
  - No rendered view is committed into the real repository. STATE replaces STATUS.md, and the fold-in to STATE, HANDOFF and PLAN is PR 8's. Committing STATE.md beside a hand-edited STATUS.md would give two status files that drift apart (lesson L-001).

  So this PR says "References #2906", not "Closes".
- **HANDOFF came out over the loop page's 24 KB read-path budget, twice.**
  - The first form was 29.5 KB. It listed held and decision items twice: in full under open items, and again in the blocked table.
  - After the dedup it was 26.4 KB, because the blocked table still repeated each item's first done-when sentence.
  - The blocked table is now `| Item | Title | Status | Owner |`, and HANDOFF renders at 22.2 KB on the real repository. All 87 items are still listed.
  - One commit message had claimed "HANDOFF within budget" before that was true. It was amended before any push.
- **The third commit's first builder ran out of context with the code written but uncommitted.** A second builder finished the tests on top of it.
  - Six older tests rendered in place, so they reached the new roadmap step. In the xtask-less fixture that step is REFUSED, as it should be, so those tests now render to `--out`.
- **No mistakes file has a lesson id yet.** Every file's `id:` is a kebab-case name such as `archive-is-not-deletion`, not `L-NNN`. LESSONS therefore lists all 23 under "Without a lesson id yet (assigned in loop PR 8)". The renderer never invents an id.
- **No item has a `Cancel if` section or a `size:` key yet.** HANDOFF shows `not recorded` for both, and names no "item that will find out". No such item exists, and inventing one would be a fabricated record.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/looprender.rs` (new), `lib.rs`, `loopgates.rs`, `main.rs`, `doccheck.rs` | code | 864 | `uor-harness loop render [--check] [--out <dir>] [--at <commit>]`. Dispatch beside `gates`. Doccheck registers `render` and the `--check`, `--out` and `--at` flags, with `--out` and `--at` as value flags. |
| `crates/uor-r4-harness/tests/loop_render.rs` (new) | tests | 559 | 15 tests over fixture repositories, with `GIT_*` stripped. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 15 | Usage. The loop page's "What is rendered" paragraph says what PR 3 builds. |

That is about 1,440 lines, excluding this ORDER and REPORT.

**Design calls.** These are the ORDER's calls, plus the builders' decisions.

- **Inputs are read at a commit.** Every input comes from `git show <commit>:<path>` or `git ls-tree`, and never from the working tree. So a view is a pure function of one commit.
- **Stamp.** Each view's first line is `<!-- rendered by uor-harness loop render from <40-hex>; do not edit -->`. `--check` reads that stamp, re-renders at the named commit, and needs a byte-exact match.
  - It exits 1 on a difference and prints the first differing line.
  - It exits 2 for a missing or garbled stamp.
  - `--check` with `--at` exits 2, because the stamp already names the commit.
- **STATE**, from inputs read at the stamped commit:
  - milestone bars and holds, from `docs/harness/milestones.json`;
  - item counts by status;
  - the newest dated row per research line, from `docs/research/notebook/*.md`;
  - the mechanism board's ACTIVE, READY and PARKED rows;
  - the lab registry.

  A file whose format would need a guess, such as missing frontmatter or no `| Date |` header, is an error rather than a guess.
- **HANDOFF.**
  - Open items are those not held, decision or done. Each row gives the first sentence of Done when and Cancel if, the size, and the status with `depends_on`.
  - The blocked table lists held and decision items once each, with their owner.
  - Not doing is `none recorded`.
- **LESSONS.** Lessons come from `docs/harness/mistakes/*.md`, excluding `README.md`.
  - An id counts only if it matches `id: L-` followed by digits.
  - The title comes from the first `# ` heading, then the frontmatter `title:`, then the file stem.
- **The AGENTS.md top-ten block.** It is written only between `<!-- loop:lessons-top-ten:start -->` and `<!-- loop:lessons-top-ten:end -->`.
  - It holds the first ten id-bearing lessons by id, or `no lessons with an id yet`.
  - Without the markers, AGENTS.md is untouched and the run notes it. `--check` then compares only the bytes between the markers.
  - PR 8 adds the markers, with owner approval.
- **The roadmap view.** It runs `cargo run -q --locked -p xtask -- roadmap render [--check]`, with cargo resolved as `loop gates` resolves it.
  - xtask exits 1 on a mismatch, which reads as "differs".
  - A spawn error, exit 101 or a signal is REFUSED, with exit 2.
  - With `--out`, the view is skipped with a note, because xtask writes inside the repository.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran these at `1cce08902`.

- `cargo test -q --locked -p uor-r4-harness --test loop_render`: 15 passed.
- **Negative gates**, run by the builders. Each set aside only its own lines and restored a byte-identical copy. In each case, the named test failed with the lines set aside and passed after restoring:
  - With `--check`'s byte comparison skipped, `a_hand_edited_state_fails_check` FAILED. This is the planted fault.
  - With LESSONS removed from the views, `a_hand_edited_lessons_fails_check` FAILED.
  - With the AGENTS comparison skipped, the AGENTS hand-edit test FAILED.
- **Real repository**: `UOR_LAB=claude uor-harness loop render --out <tmp>` wrote:
  - STATE: 4,802 bytes;
  - HANDOFF: 22,200 bytes;
  - LESSONS: 3,906 bytes.

  It noted the roadmap skip, and AGENTS.md stayed unchanged. A second render was byte-identical, and `--check --out` exited 0.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 714 passed, 0 failed.
- `cargo fmt --check` is clean. Clippy reports no warnings in the touched files.
- The PR body lists claim-wording, `xtask links` on the changed files and the local copyright check.

## Skills used

None of the private kit's skills.

## Still open

- **`docs/loop/baseline.json`** waits for loop PR 5.
- **PR 8 owns the live views and the inputs they depend on:**
  - committing STATE, HANDOFF and LESSONS;
  - retiring STATUS.md and NOW.md;
  - adding the AGENTS.md markers;
  - assigning `L-NNN` lesson ids.
- **A lab PR that touches a rendered view fails.** That check belongs to the policy gate in a later PR, together with the 2-hour freshness check.
- **HANDOFF has about 2 KB of headroom under its 24 KB budget.** As items gain `Cancel if` and `size`, it will grow. The read-path budget gate is still to be built.
- **The renderer's own child git calls do not strip `GIT_*`.** Its tests strip them for every child.

## Plain summary

`uor-harness loop render` writes the project's status pages: STATE, HANDOFF and LESSONS, the AGENTS.md lessons block, and the roadmap page. It reads only what is committed, at one named commit, and stamps that commit on each page. `--check` re-renders at the stamp and fails on any difference, so a hand edit to a rendered page is caught. The real repository does not switch to these pages until loop PR 8.
