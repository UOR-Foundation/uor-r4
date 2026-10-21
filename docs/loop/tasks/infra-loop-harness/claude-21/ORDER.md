# ORDER: infra-loop-pr07c-queue-board, claude-21 (loop PR 7c: loop queue and loop board)

Lab: claude. Item: [infra-loop-pr07c-queue-board](../../../../roadmap/items/infra-loop-pr07c-queue-board.md) (#2912). Setup row: 7c. Base: `7864b0516`.

## Goal

The item's done-when:
- `loop queue` queues a PR only when all four of these hold:
  - the policy check is green at the head;
  - the APPROVE that PR 6's gate accepts is at the head (Claude's for a loop setup PR under D59; otherwise another vendor's);
  - no later CHANGES that the gate counts exists;
  - no owner or PM comment came after the approval.
- It refuses required checks that did not come from the policy job, and `.github/**` changes made without the owner's approval.
- `loop board` is a read-only view of the five labs.

The planted fault: **a later CHANGES record blocks queueing, unless it comes from a lab other than Claude on a setup PR (D59).**

## Cancel probe (run first)

Do `loop queue` or `loop board` exist? Result at `7864b0516`: no. Both print "not built yet". The task stands. Class: instrument.

## Architecture calls (Claude, D52)

1. **`loop queue --pr <n> [--dry-run] [--json]` runs `pr-check` on the PR's real head, then adds one rule.**
   - It reads the PR from GitHub (head, branch, state, draft), fetches `pull/<n>/head`, and refuses if the fetched commit differs from the head GitHub reports.
   - It runs every `pr-check` rule in `--pr` mode against `origin/main`. The rule list is factored into a callable function, not copied. Any FAIL refuses. That covers three of the conditions:
     - `review-record`: the gate at the head, which already ignores a CHANGES that does not count;
     - `required-checks`: each required name must carry `policy: pass` from GitHub Actions, and no commit status may use that name;
     - `owner-paths`: `.github/**` needs a recorded owner approval.
2. **The after-approval rule.** Every comment after the last counted APPROVE record is judged as follows:
   - a `uor-review v1` record, valid or not, is left to the gate;
   - a comment from a `[bot]` account does not count;
   - a comment that starts with the tag of a registered lab whose role is not `pm` does not count;
   - every other comment counts as possibly the owner's or the PM's, and refuses the queue. All labs post from one account (D55), so an untagged comment is the owner's as far as the tool can tell.
3. **Writing.** The only GitHub write is `gh pr merge <n> --match-head-commit <sha>`, through a trait with a fake. `--dry-run` writes nothing. Exit codes: 0 queued (or would queue), 1 refused, 2 cannot run.
4. **`loop board [--json]` is read-only.** For each lab in `labs.json` it shows:
   - vendor, client and role;
   - presence from the live tier (state, age, doing, next);
   - the number of live claims;
   - its open PRs: those whose title starts with `[<lab>]`, each with number, draft or ready, and short head.

   Open PRs with no lab tag are listed apart. GitHub is read through a trait with a fake. No hook canary yet: that is loop PR 9.

## Files in scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**`
- `docs/harness/README.md` and `docs/loop/README.md`: usage, and what is built.
- `docs/roadmap/items/infra-loop-pr07c-queue-board.md` (status) and `docs/roadmap/README.md` (rendered).
- `docs/loop/tasks/infra-loop-harness/claude-21/**`

## Done when

- Fixture tests show that queueing is refused for each of these:
  - a later counted CHANGES (the planted fault);
  - a policy check that is not green, or a required name posted as a commit status;
  - no counted APPROVE at the head;
  - an untagged comment after the approval, and a PM-tagged one;
  - a `.github/**` change without owner approval;
  - a fetched head that differs from the reported one.
- These cases queue: a later CHANGES from a lab other than Claude on a setup PR under D59, and a later comment tagged by a non-PM lab.
- `loop board` renders five labs from fakes, and makes no GitHub write.
- The full harness suite passes, as do claim-wording, links and the copyright check. `pr-check --local` fails only `scope`.

## Stop and ask if

- Queueing would need a GitHub write other than the merge.
- `pr-check`'s rules would have to change their verdicts, not only be made callable.
- Any file outside scope needs to change.
