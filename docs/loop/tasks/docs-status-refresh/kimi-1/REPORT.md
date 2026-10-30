# REPORT: reopen and rescope docs-status-refresh, kimi-1

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). ORDER: [ORDER.md](ORDER.md). DECISION: [DECISION.md](DECISION.md). Draft PR: #3431. References #3380 and #2029.

## What did not work, first

- The first baseline command used `/opt/homebrew/bin/cargo`, which is `cargo-ai`, not Rust Cargo, and refused `test` as an unknown command. This was an instrument collision, not a repository failure. `/Users/casey.allard/.cargo/bin/cargo` is Rust Cargo 1.97.1; the baseline then passed 211/211 `xtask` tests.
- The `cargo xtask` alias is unavailable here. The workflow-equivalent explicit form `cargo run -q --locked -p xtask -- ...` works and is used for generator/roadmap checks.
- Codex #3429 overlapped the owner-assigned refresh. It was closed as superseded with its useful corrections retained for credit in the content PR. Claude #3428 had already merged, so claude-217 is now valid merged evidence.
- Disk is constrained. The user asked for active monitoring. The worktree began at 744 MiB with a 218 MiB target; free space was 7.7 GiB and later 18 GiB. Builds stay narrow and no cleanup of unique or other-lab files is attempted.

## What changed

- Reopened `docs-status-refresh` and dated it 30 Oct 2026. Its scope now names the living front door, book, owner-status records, milestone input, figures/generators, all living crate READMEs, one-model item, and Studio status/check files. It explicitly excludes dated evidence, Claude's active files and harness-rendered views.
- Added the owner's 30 Oct Kimi direction verbatim to PLAN; confirmed M1 #2029 closed at all six bars with the useful-chat limitation; marked the 29 Oct documentation direction done without pretending that refresh made later pages permanently current.
- Added this ORDER and a scope decision card. No public prose, figure, web, model, workflow, owner-path, bar or technical-rule edit is in this PR.

Diff size at this report: five files (ORDER, DECISION, REPORT, item, PLAN), with no line cap (D48). The scope expansion and separate decision card were intentional; no production code or dependency changed.

## Measurements and cancel probe

- #2029 is `CLOSED`.
- `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` contains `m1_milestone_status: "CLOSED_PASS"`; the prior refresh item was `done`, so reopening is new post-close work.
- `cargo roadmap check`: **112 items OK** — ready 16, held 10, decision 1, done 85.

## Checks run at the head

- Baseline with explicit Rust Cargo: `cargo test --locked -p xtask` — **211 passed, 0 failed**.
- `cargo run -q --locked -p xtask -- roadmap check` — **PASS**, 112 items.
- `cargo run -q --locked -p uor-r4-claim-wording -- PLAN.md docs/roadmap/items/docs-status-refresh.md docs/loop/tasks/docs-status-refresh/kimi-1` — **PASS**.
- `git diff --check` — **PASS**.
- Fresh exact-head scope/decision review: recorded on the PR before ready/queue; no approval is assumed in this report draft.

## Skills used

Brainstorming, working discipline, documentation co-authoring (freeform under auto mode), UOR research architect, solution integrity, scope guard, writing plans, executing plans, git worktrees, context guard, and test-driven development. The public documentation remains the deliverable; no model experiment or GPU compute was run.

## Still open

- Fresh exact-head review, ready/queue, merge verification on fresh `main`, then `cargo xtask roadmap sync --apply` to reopen/update #3380.
- The comprehensive content PR (kimi-2), figure/book/web checks, and the separate generated-view render flow.

## Plain summary

This small PR reopens the existing documentation item and gives it the right boundaries for a post-M1 cleanup. It records the owner's new assignment and M1's confirmed close, but does not itself change the public milestone count or any research rule. The content update follows next, with dated evidence and Claude's work preserved.
