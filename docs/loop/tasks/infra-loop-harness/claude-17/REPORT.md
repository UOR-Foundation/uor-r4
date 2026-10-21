# REPORT: infra-loop-pr07b-session, claude-17 (loop PR 7b: loop next, arrive, report and close)

Lab: claude. Base: `8c2c76e42`. Order: [ORDER.md](ORDER.md), committed first (`4be9a7687`). The cancel probe found no `next`, `arrive`, `report` or `close` under `loop`.

Five fresh-context builders wrote one commit each: `report`, `arrive`, `next`, `close --session` and `close --verify`. The main session re-ran every builder's gates before accepting its commit. It updated the loop page and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **Step 13 is not followed literally, and the owner should decide.** The loop page says `close --verify` should "run `uor-hygiene --apply`". Hygiene's `--apply` removes every clean, merged worktree with its branch, other labs' included. That crosses the lab boundary: never touch another session's files, and never delete another lab's work. So `close --verify` prints hygiene's plan (`--dry-run`) and does not apply it. The loop page's rule text is unchanged; its command cell says so. Applying it, or limiting hygiene to the lab's own worktrees, is the owner's call. This is reversible.
- **Most of `loop next`'s sources do not exist yet.**
  - Owner directions and premise probes need PLAN (loop PR 8). GPU collections need the watchers (loop PR 10). No lab has a lane, and no item has `lane:` or `rank:`.
  - `next` names each missing source and the PR that builds it, and never guesses.
  - No item has a `scope:` either, so `any` finds nothing. Run against real GitHub, `next` answered "STOP held work: 26 item(s) held, none ready and claimable".
- **Step 11 is only half built.** `next` finds the lab's PRs that need repair (a conflict, a failed check, or a counted REQUEST CHANGES at the head). Writing the repair ORDER `<lab>-<n+1>` is still done by hand.
- **The first full suite run failed once (850 passed, 1 failed).** Loop PR 4's `watch_and_other_actions_are_refused` used `loop next` as its example of an action that is not built yet, and this PR builds it. `5cc3c63e5` switches the example to `loop drive` (loop PR 10). The specified behaviour changed; the test was not weakened.
- **The ORDER template had a dead link.** Its item link is relative to a task folder, so from `docs/loop/templates/` it resolved nowhere and `xtask links` caught it. `4282b9ff0` puts it in a code span as an example.
- **The binary's own commit is unknown.** No commit is baked into `uor-harness` (its build script explains why). So `arrive` checks the checkout against `origin/main` and the installed binary against the newest local build, not the binary against `origin/main`.
- **`arrive` found a real mismatch on its first run.** The shared git `pre-commit` hook differs from the one the current binary installs. It reports this as a WARN, and `uor-harness git install-hooks` fixes it. Client hooks (Claude Code settings) are N/A until loop PR 9.
- **GitHub reads and writes are tested only through fakes.** These are: the PR body and the ready state (`close --session`); the merge state (`close --verify`); and the PR list, comments, files and labels (`next`). The real `gh` paths are checked by reading the code.
- **`hygiene` cannot be cited in docs.** The harness's documented-commands rule does not list the `hygiene` subcommand, so the README names the `uor-hygiene` wrapper instead. This is outside scope and noted for the PM.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopreport.rs` (new) | code | 331 | `loop report --check`, `--new` and `--receipts`. |
| `crates/uor-r4-harness/src/looparrive.rs` (new) | code | 472 | `loop arrive`. In order: fetch, checkout, binary, hooks, client hooks, views, PICKUP, read path. |
| `crates/uor-r4-harness/src/loopnext.rs` (new) | code | 566 | `loop next`. The sources in the loop page's order, the stops, and a read-only `NextGh` trait with a fake. |
| `crates/uor-r4-harness/src/loopclose.rs` (new) | code | 1,048 | `loop close --session` and `--verify`, with a `CloseGh` trait whose fake logs every write. |
| `loopprcheck.rs`, `loopclaim.rs`, `loopgates.rs`, `doccheck.rs`, `main.rs`, `lib.rs` | code | 90 | Shared helpers: `report_file_problems`, `parse_review_record`, `size_line`, `find_order`, `main_checkout`. Also dispatch, doccheck's loop group and flags, and `--json` pass-through. |
| `crates/uor-r4-harness/tests/loop_{report,arrive,next,close}.rs` (new) | tests | 1,553 | 49 tests over fixture repositories with bare remotes and fakes. |
| `docs/loop/templates/{ORDER,REPORT,PICKUP}.md` (new) | docs | 76 | Templates in our own words. Template prompts are `<!-- fill: -->` lines, which `report --check` refuses. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 93 | Usage, and the steps marked built. |

That totals about 3,770 lines, excluding this ORDER and REPORT. The ORDER set out five commands across five builder commits, which is what was built.

**Design calls.**
- **Exit codes.** `next` exits 0 for GO, 1 for STOP and 2 when it cannot run. The others exit 0 when ok, 1 when a check fails or the command refuses, and 2 when they cannot run.
- **`report --check`** reuses `pr-check`'s section checker. It also refuses a leftover `<!-- fill:` line, so a bare template never passes. `--receipts` copies this head's results from `local/<lab>/gates.jsonl` into the task folder, as raw lines, latest per gate.
- **`arrive`** renders the views from `origin/main` into `local/<lab>/views/`. A committed view that disagrees with its inputs FAILs: the repository wins. A PICKUP of 8 KB or more FAILs, and so does a standing reference that cites no §, D-row or L-id.
- **`close --session`** refuses, before any write, when:
  - the REPORT is incomplete;
  - the PR head is not local HEAD;
  - the PICKUP would be 8 KB or more.

  It then sets the body, with the size line first and `References #<issue>` from `docs/roadmap/issues.json`. It marks a draft ready and writes PICKUP.
- **`close --verify`** deletes nothing until four proofs pass. Then each deletion has its own guard. Another lab's PR is verified but never cleaned up. The proofs:
  1. GitHub reports MERGED.
  2. The `(#N)` squash commit is on `origin/main`.
  3. Main's copy of every changed file equals the PR head.
  4. `loop gates --full` passes in a throwaway worktree of `origin/main`.

  The deletion guards:
  - the worktree must be clean, under `.worktrees/<lab>-*`, and is never force-removed; a change to the old `claims.json` alone is reverted first;
  - the local branch must be at the PR head;
  - the remote delete only succeeds if the branch is still at the PR head.

## Measurements

None. This is a host tool.

## Checks run at the head

The main session re-ran each builder's tests at its commit, then ran these at `3841803b1` with this REPORT in place.

- Targeted suites: `loop_report` 6 passed, `loop_arrive` 12 passed, `loop_next` 12 passed and `loop_close` 19 passed. `loop_prcheck` still gives 27 passed, after the shared parser and size helpers moved.
- **Negative gates.** These were run by the builders. Each set aside only its own check, ran the test (which FAILED), then restored the file, confirmed byte-identical with `cmp`. The checks and their tests:
  - **The planted fault:** `report --check` ignoring the section problems; `a_report_missing_a_section_is_refused`.
  - **`arrive`'s PICKUP citation check:** the uncited-reference test.
  - **`next`:** the STOP-file check, and repairs ranking ahead of reviews; `a_stop_file_stops_on_the_kill_switch` and `repairs_come_before_reviews`.
  - **`close --session`:** the REPORT check before writes, and the push-first head check.
  - **`close --verify`:** the MERGED proof, the content proof, the own-lab check, and the remote lease (swapped for a plain delete).
- **The full suite**, `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: FULLSUITE_PLACEHOLDER.
- **`loop pr-check --local --base 8c2c76e42`** on this branch: `order`, `commit-order`, `report`, `owner-paths`, `rule-sources` and `read-path` (28,257 B) pass, `rendered-lock`, `filemap`, `review-record` and `required-checks` are N/A, and `scope` fails because the item has no `scope:` at the base, as for every loop PR until scopes are backfilled.
- **Real-world runs** in this worktree:
  - `loop arrive` exited 0, with a WARN for checkout and hooks;
  - `loop next` ended in STOP held work (exit 1), reading GitHub without writing.
- `cargo fmt -p uor-r4-harness --check` is clean, and whole-crate clippy `--all-targets` is clean. Claim-wording passed; `xtask links` finds 0 broken links in the changed files (after `4282b9ff0`); the local copyright check finds 0 matching 8-word runs, the templates included.

## Skills used

None of the private kit's skills.

## Still open

- **Owner call: hygiene in step 13.** Should it be applied, or limited to the lab's own worktrees? See the first bullet above.
- **The repair ORDER.** Writing `<lab>-<n+1>` for step 11 is not built.
- **Missing sources for `next`.** PLAN (PR 8) will add owner directions and probes. Watchers (PR 10) will add GPU collections. Lanes need a `lane:` key and a PM ranking.
- **Real `gh` paths.** They are covered only by reading the code. `next` judges checks by FAILURE and TIMED_OUT only, so an ERROR status does not trigger a repair.
- **PICKUP.** The standing references in a written PICKUP are left empty for the lab to fill.

## Plain summary

These commands carry a lab through one session:
- `loop next` says whether to work and on what. It stops on the kill switch, prefers repairs to the lab's own PRs, then reviews owed, then a ready item it can claim. It names the sources the project has not built yet.
- `loop arrive` refreshes the checkout and checks that the tools, hooks, rendered notes and reading budget are sound.
- `loop report --check` refuses a REPORT with a missing or unfilled section.
- `loop close --session` turns the REPORT into the PR description, marks the PR ready and leaves a note for the next session.
- After merge, `loop close --verify` proves the merge landed exactly as reviewed. Only then does it delete the lab's own branch and worktree, and it deletes nothing it cannot prove belongs to that lab.
