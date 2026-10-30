# REPORT: reopen and rescope docs-status-refresh, kimi-1 with repair kimi-2

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). ORDER: [ORDER.md](ORDER.md), repaired by [kimi-2/ORDER.md](../kimi-2/ORDER.md). DECISION: [DECISION.md](DECISION.md). Draft PR: #3431. References #3380 and #2029.

## What did not work, first

- **The first exact-head review returned CHANGES / scope decision REJECT.** It correctly found that the cancel probe cited the older five-bar JSON without claude-211's separate bar-f evidence, omitted two M1 item inputs, included a dated log and prior task glob, and did not name the antigravity-held mechanism pages. It also found unclear owner-path authority, proposed-versus-actual reopen wording, incomplete claim-check coverage, an unreported render mismatch, and missing preservation details. That head was not queued; [kimi-2/ORDER.md](../kimi-2/ORDER.md) repairs each finding and the rejected outcome is retained in DECISION.
- The first baseline command used `/opt/homebrew/bin/cargo`, which is `cargo-ai`, not Rust Cargo, and refused `test` as an unknown command. This was an instrument collision, not a repository failure. `/Users/casey.allard/.cargo/bin/cargo` is Rust Cargo 1.97.1; the baseline then passed 211/211 `xtask` tests.
- The `cargo xtask` alias is unavailable here. The workflow-equivalent explicit form `cargo run -q --locked -p xtask -- ...` works and is used for all xtask checks.
- The initial claim-wording command supplied several paths, but the binary reads only its first argument. The repaired check runs `-- .` and covers the repository.
- Codex #3429 overlapped the owner-assigned refresh. It was closed as superseded; its unique work remains on `codex/docs-current-status-20261030` and `origin/pr/3429/head`, both at `ed54006e7615a5e14335a2b501c5c7ba14e14eee`. Its useful corrections will be credited in the content PR. Claude #3428 had already merged, so claude-217 is valid merged evidence.
- The previous refresh left known negatives: two pre-existing broken book links from `docs/research/notebook/m4-serving.md`, one unformatted `figures_charts.rs` test on old main, two pre-existing xtask clippy findings, and antigravity-113's `exact-memory.md`/`object-store.md` holds. They are named, not silently attributed to this unit; the two mechanism pages and dated `current-state.md` are outside the repaired scope.
- Disk is constrained. The user asked for active monitoring. Free space has ranged from 7.3 to 18 GiB; the worktree began at 744 MiB with a 218 MiB target. Builds stay narrow and no cleanup of unique or other-lab files is attempted.

## What changed

- Prepared `docs-status-refresh` for reopening after merge and dated it 30 Oct 2026. Its repaired scope now includes the M1 close and bar-f item inputs, `CONTRIBUTING.md`, all three figure sources, the living front door/book/owner-status records/milestone input/crate READMEs/one-model item/Studio files, and only the named Kimi task folders.
- Exclusions now cover dated evidence and logs, previous task folders, Claude's active files, antigravity-113's two mechanism pages, and harness-rendered views.
- Added the owner's 30 Oct Kimi direction verbatim to PLAN; recorded the owner-approved plan's status-only owner-path authority; marked the 29 Oct documentation direction done without pretending that refresh made later pages permanently current.
- Added the scope ORDER, decision card, rejected-review outcome and repair ORDER. The item and PLAN owner-direction rows are the only public prose changed in this PR; no figure, web, model, workflow, owner-path, bar or technical-rule edit is present.

Diff size after repair: six files (both ORDERs, DECISION, REPORT, item, PLAN), with no line cap (D48). No production code or dependency changed.

## Measurements and cancel probe

- #2029 is `CLOSED`.
- `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` records bars (a)–(e) as `m1_milestone_status: "CLOSED_PASS"`; it is not called six-bar evidence.
- `docs/labs/m1-heldout-phrasing-transfer/evidence/panel-score.json` separately records `bar_f_pass: true` and `outcome: "PASS_BAR_F_HELDOUT_PHRASING"`.
- The prior refresh item was `done`, so proposing to reopen it is new post-close work.
- `cargo run -q --locked -p xtask -- roadmap check`: **112 items OK** — ready 16, held 10, decision 1, done 85.
- `cargo run -q --locked -p xtask -- roadmap render --check`: **EXPECTED STALE**, ready 16 expected against ready 15 in the generated page. This is the item-input change and requires the post-merge render; it is not reported as a pass.

## Checks run at the repaired head

- `cargo test --locked -p xtask` with explicit Rust Cargo — **211 passed, 0 failed**.
- `cargo run -q --locked -p xtask -- roadmap check` — **PASS**, 112 items.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` — **PASS**, full repository.
- `git diff --check` — **PASS**.
- Codex preservation refs verified at `ed54006e7615a5e14335a2b501c5c7ba14e14eee`; GitHub #3380 remains CLOSED until the post-merge sync.
- A new fresh exact-head review is required before ready/queue. The earlier review is not reused as approval.

## Skills used

Brainstorming, working discipline, documentation co-authoring (freeform under auto mode), UOR research architect, solution integrity, scope guard, writing plans, executing plans, git worktrees, context guard, test-driven development, and a fresh delivery/scope review. No model experiment or GPU compute was run.

## Still open

- New exact-head review, ready/queue, merge verification on fresh `main`, then `cargo run -q --locked -p xtask -- roadmap sync --apply` to reopen/update #3380 and the separate generated-view render flow.
- The comprehensive content PR (kimi-3 after the scope repair), figure/book/web checks, and final generated-state verification.

## Plain summary

This PR proposes reopening the documentation item with boundaries that now cover the real M1 cleanup and exclude other labs' records. The first review rejected an incomplete scope; that rejection is preserved and repaired rather than bypassed. M1's older five-bar evidence and new bar-f evidence are cited separately, and no public milestone count or research rule changes in this scope PR.
