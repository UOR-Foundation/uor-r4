# REPORT: infra-loop-harness, claude-25 (feed the bench from every lab's work)

Lab: claude. Base: `a8ff5690e`. Order: [ORDER.md](ORDER.md), committed first (`d4a4437eb`). The cancel probe found that no harness command wrote bench-shaped files. `loop board` printed text or JSON only.

## What did not work, first

- **The first feed called AGENTS.md's whole opening paragraph "the project in one sentence".** On the real repository that pulled in three lines. It now takes the paragraph's first line (`d3c356de2`). The test that pinned the joined form was updated to the specified behaviour and now also asserts that the second line is absent (`072076e25`). I committed `d3c356de2` before that test's failure stopped me, and caught it on the next line of output.
- **The first column names and file lists matched the kits.** "Cancel if" and "Why / What was rejected" matched LoopKit, and the four page names in order matched CorvusWing: 3 and 2 eight-word runs. D44 keeps kit text out of the repository, so they are reworded (`272295b2e`). The bench finds a column by its first word (`cancel`, `reversal`), so nothing it reads changed.
- **The feed writes nothing from an owner checkout without `docs/harness/labs.json`.** It refuses at the lab check, so the live run used the claude-25 worktree as `--repo`.
- **The builder extended `doccheck.rs`'s flag list** (`--bench`, `--every`), a shared file it had not been handed. The ported-rules test required it.

## What changed

| Commit | What |
|---|---|
| `b17ea7e71` | `loopbench.rs` writes the five files atomically into DIR, from roadmap items, open lab PRs with their ORDER and last review record at the head, lab presence, `docs/integration/DECISIONS.md`, committed receipts and local gate ledgers. A DIR inside the tracked tree is refused (exit 2), and `--every` below 30 is refused. `INBOX.md` is never touched. GitHub is read through a read-only `BenchGh`, with a fake for tests. `loopgates.rs` sends `loop board --bench` here, and `loopboard.rs` is not edited. Six tests are in `tests/loop_bench.rs`. |
| `8f628cc02` | The harness README's `loop board` paragraph. |
| `9cbbedac9` | Merges `origin/main`, which carries claude-26's scope lines. |
| `d3c356de2`, `072076e25`, `272295b2e` | The fixes listed above. |

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean, and whole-crate clippy with `-D warnings` is clean. `loop_bench` 6 passed, and the library tests passed 662, with 1 ignored.
- Negative gate. With `item_status` set aside to always return `ready` (2 lines), `bench_lanes_follow_item_and_pr_state` fails at line 290. Restored, it passes, and the tree is clean.
- Live, on the real repository, `--bench /Users/casey.allard/uor-r4/local/bench` wrote five files (2 open PRs, 88 items):
  - Next: one row per lab PR, with its ORDER's done-when and stop bullets.
  - Blocked: the 4 owner decisions.
  - Tasks: dependency waits named.
  - State: every lab's presence.
  - Checks: 24 gate lines.

  `--every 120` keeps Casey's bench current.
- Full harness suite at `072076e25`: 1,010 passed, 0 failed. At `272295b2e`: 1,010 passed, 0 failed.
- Claim-wording finds nothing in the changed files. The copyright check finds 0 LoopKit and 0 CorvusWing eight-word runs.

## Skills used

solution-integrity: the feed is generated from the sources the loop already keeps, so the bench cannot disagree with the roadmap, and no hand-kept copy of the state exists.

## Still open

- The bench's Next panel shows four rows, so with five labs one lab's current task appears only in the count.
- The real GitHub path (`gh pr view` and the contents API) has run once, live. Only the fake is under test.
- `infra-loop-pr03-render` already scopes `STATE.md` and `HANDOFF.md` as rendered views. Whether the bench feed should become that renderer is a call for Kimi.

## Plain summary

The bench Casey installed now shows every lab's work: what each one is doing, every roadmap task in its lane, the decisions waiting on him and the latest checks. It is all generated from the project's own records every two minutes, so it stays current without anyone keeping notes by hand.
