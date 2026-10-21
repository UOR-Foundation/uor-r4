# REPORT: infra-loop-pr07a-claim, claude-15 (loop PR 7a: loop claim, suborder and ask)

Lab: claude. Base: `32092fca6`. Order: [ORDER.md](ORDER.md), committed (`cdeca0e45`) before the work. The cancel probe found no `claim`, `suborder` or `ask` under `loop`.

Three fresh-context Opus builders each wrote one commit. The main session re-ran the gates and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **`loop claim --lab-topic` was refused by the top-level flag parser until commit 3.** `main.rs` only lets through the flags in its `SUBCOMMAND_FLAGS` and `SUBCOMMAND_VALUE_FLAGS` tables, and commit 2 did not add `--lab-topic` there. Commit 3 added it, along with the suborder and ask flags. **No test passes `--lab-topic` through the binary yet**, so nothing pins that fix.
- **The real GitHub paths are untested.** All tests use the fake: `git push -u` followed by `gh pr create --draft`, `gh pr close`, `git push origin --delete`, and `gh api …/contents/<path>?ref=<branch>`. Tests must not touch the network, and the fake records every write call in a log.
- **A squash-merged branch is never an ancestor of `main`.** Under commit 1's rule, a leftover worktree would have kept its item claimed for ever. Commit 2 now treats an order as merged when its ORDER file exists on `origin/main` with identical bytes.
- **An ORDER's path does not name its item.** For example, `claude-15` is for `infra-loop-pr07a-claim`. So the claim reads each open order's text to find its `Item:` line. On GitHub, that costs one `gh api` call per open ORDER.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopclaim.rs` (new) | code | 885 | `loop claim <item> [--lab-topic <topic>]`. Refusals, then writes, then race settlement, with GitHub behind the `Gh` trait. |
| `crates/uor-r4-harness/src/loopsub.rs` (new) | code | 248 | `loop suborder --parent <ORDER> --scope <globs>` and `loop ask --scope\|--blocked`. |
| `main.rs`, `lib.rs`, `loopgates.rs`, `doccheck.rs` | code | 39 | Dispatch beside `gates`. Flag tables. Doccheck `GROUPS` for `loop` is `ask, claim, gates, suborder`, plus eleven flags. |
| `crates/uor-r4-harness/tests/loop_claim.rs` (new) | tests | 569 | 17 tests over fixture repositories with a bare remote and the fake `gh`. `GIT_*` is stripped. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 21 | Usage, exit codes and the test-only fake. The loop page marks claim, suborder and ask as built in loop PR 7a. |

That is about 1,750 lines, excluding this ORDER and REPORT.

**Design calls.** These are the ORDER's calls, plus the builders' decisions:

- **When an item counts as claimed.**
  - An open order names the item. Open orders are:
    - the ORDERs in this lab's `.worktrees/<lab>-*` worktrees whose branch is not merged;
    - the ORDERs changed in open PRs, from `gh pr list --json number,headRefName,isDraft,files`.
  - A worktree's order counts as merged when its head is an ancestor of `origin/main`, or when its ORDER bytes are identical on `origin/main`.
- **Refusals.** Each one exits 2, and they run in this order:
  1. the item is not `ready`;
  2. the item is claimed;
  3. the item has no `scope:` line;
  4. the item's scope overlaps an open order's scope;
  5. the lab is at its `open_pr_limit` from `docs/harness/labs.json`.

  Overlap is decided by literal prefix. It is conservative: it may refuse a claim that would not really collide, but it never misses one. Globs under `docs/research/notebook/` never count. An open order whose own item has no `scope:` is refused rather than guessed at.
- **Writes, in the loop page's step-4 order.** They happen only after every refusal passes.
  1. `git fetch origin main`.
  2. `git worktree add .worktrees/<lab>-<topic> -b <lab>/<topic> --no-track origin/main`. `<topic>` defaults to the item id. An existing folder or branch is refused.
  3. Commit a skeleton ORDER at `docs/loop/tasks/<item>/<lab>-<n>/ORDER.md`. It carries `Item:`, `Setup row:` (for loop setup items) and `Base:`, the item's done-when quoted, and placeholders.
  4. Open the draft PR.
  5. Re-list.

  A failure before the PR exists removes our exact worktree and branch, and exits 2.
- **Race settlement.** If the re-list shows a lower-numbered draft PR whose ORDER names the same item, this claim lost. It closes our PR, deletes our pushed remote branch by its exact name, removes our exact worktree and branch, and exits 1 with "lost the race to #N". It never touches anything it did not create.
- **Suborder.** A child glob must be contained in a parent glob. With no wildcard, the parent must equal the child. With a wildcard, the parent's literal part must be a string prefix of the child's literal part. The suborder writes `local/<lab>/<topic>/orders/<n>.md` and prints the subagent's block: the parent ORDER, the scope, and the stop rules from the loop page.
- **Ask.** Exactly one of `--scope` (`--file --reason --lines --own-code`) or `--blocked` (`--limit --measured --command --state`) is given, plus `--order`. Each writes a card to `local/<lab>/<topic>/asks/<n>.md` and posts nothing.
- **The test-only fake.** `UOR_LOOP_GH_FAKE=<json>` names a file of PRs, with optional `orderTexts`, `nextPrNumber` and `afterCreate` (PRs that appear only after our create, which is how a race is staged). Write calls are logged to `<json>.calls.jsonl`.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran these at `e02dc4243`.

- `cargo test -q --locked -p uor-r4-harness --test loop_claim`: 17 passed.
- **Negative gates**, run by the builders. Each set aside only its own lines and restored a byte-identical copy afterwards.
  - With the overlap check forced to "no overlap", `an_overlapping_claim_is_refused` FAILED. This is the planted fault.
  - With the race-settlement block removed, `a_lost_race_closes_our_pr_and_removes_only_what_we_created` FAILED.
  - With the containment test forced to true, both suborder-outside tests FAILED.
- The lost-race test also checks that an unrelated worktree and branch survive.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 720 passed, 0 failed.
- `cargo fmt -p uor-r4-harness --check` is clean, and clippy reports no warnings in the touched files.
- The PR body lists claim-wording, `xtask links` on the changed files and the local copyright check.

## Skills used

None of the private kit's skills.

## Still open

- **`--lab-topic` has no test through the binary**; see above.
- **The real `gh` and `git push` paths are covered only by reading the code.**
- **No item has a `scope:` line yet.** So `loop claim` refuses every item today with "item has no scope: line". The PM's backfill of `scope:` (PR 2's keys) is what makes `loop claim` usable.
- **Claims recorded by the older `uor-harness team claim` in `docs/harness/claims.json` are not read.** The loop's claims are its open orders. Retiring `claims.json` belongs to PR 8.
- **Two small doc items.** A docs hook notes that no docs page lists `loopsub.rs` in its `sources`. The `FLAGS` doc comment in `doccheck.rs` still says "76 of them".

## Plain summary

`uor-harness loop claim` lets a lab take a roadmap item only when the item is ready, unclaimed, and scoped away from everyone else's open work, and only while the lab is under its PR limit. It creates the worktree, commits the ORDER and opens a draft PR. If another lab claimed the same item first, it undoes only its own steps.

`loop suborder` gives a subagent a narrower scope inside the parent's. `loop ask` writes the two stop-and-ask cards.
