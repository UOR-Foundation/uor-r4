# REPORT: reopen and rescope docs-status-refresh, repairs kimi-1 through kimi-3

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). ORDERs: [kimi-1/ORDER.md](../kimi-1/ORDER.md), [kimi-2/ORDER.md](../kimi-2/ORDER.md), [ORDER.md](ORDER.md). DECISION: [kimi-1/DECISION.md](../kimi-1/DECISION.md). Draft PR: #3431. References #3380 and #2029.

## What did not work, first

- **Two exact-head reviews returned CHANGES; neither head was queued.** The first rejected the scope for incomplete six-bar evidence citation, missing M1 inputs, dated-record overbreadth, unclear owner-path authority and several check/report defects. The second approved the repaired scope but found that `roadmap sync` does not reopen closed issues and that the active repair ORDER lacked its own correctly headed REPORT. It also flagged overstated claim-wording coverage and missing harness checks. The rejected outcomes are retained, and the repairs are split across kimi-2 and this ORDER rather than hidden.
- The first baseline command used `/opt/homebrew/bin/cargo`, which is `cargo-ai`, not Rust Cargo, and refused `test` as an unknown command. `/Users/casey.allard/.cargo/bin/cargo` is Rust Cargo 1.97.1; the baseline then passed 211/211 `xtask` tests.
- The `cargo xtask` alias is unavailable here. The explicit form `cargo run -q --locked -p xtask -- ...` works and is used for all xtask checks.
- The initial claim-wording command supplied several paths, but the binary reads only its first argument. The repaired `-- .` check covers the gate's documented Markdown surface (`docs/**/*.md`, `crates/*/README.md` and root `README.md`), not every public file in the repository. The content unit adds a separate scan for its other changed public prose.
- Codex #3429 overlapped the owner-assigned refresh. It was closed as superseded; its unique work remains on `codex/docs-current-status-20261030` and `origin/pr/3429/head`, both at `ed54006e7615a5e14335a2b501c5c7ba14e14eee`. Its useful corrections will be credited in the content PR. Claude #3428 had already merged, so claude-217 is valid merged evidence.
- The previous refresh left known negatives: two pre-existing broken book links from `docs/research/notebook/m4-serving.md`, one unformatted `figures_charts.rs` test on old main, two pre-existing xtask clippy findings, and antigravity-113's `exact-memory.md`/`object-store.md` holds. They are named, not silently attributed to this unit; the two mechanism pages and dated `current-state.md` are outside scope.
- Disk is constrained and below the 30–70 GiB guidance at several checks. Free space has ranged from 7.3 to 28 GiB; the worktree began at 744 MiB with a 218 MiB target. The user asked for active monitoring. Builds stay narrow, free space is rechecked before heavy work, and no cleanup of unique or other-lab files is attempted.

## What changed

- Prepared `docs-status-refresh` for reopening and dated it 30 Oct 2026. Its repaired scope includes the M1 close and bar-f item inputs, `CONTRIBUTING.md`, all three figure sources, the living front door/book/owner-status records/milestone input/crate READMEs/one-model item/Studio files, and only the named Kimi task folders.
- Exclusions cover dated evidence and logs, previous task folders, Claude's active files, antigravity-113's two mechanism pages, and harness-rendered views.
- Added the owner's 30 Oct Kimi direction verbatim to PLAN and recorded the owner-approved plan's status-only owner-path authority. The content ORDER will also carry the answer `Approved Plan` at `2026-10-30T15:30:07Z` before any owner-path edit.
- Corrected the reopen sequence everywhere: after this scope PR merges, Kimi manually runs `gh issue reopen 3380`, then roadmap sync. Sync itself does not reopen closed issues.
- Added the scope ORDER, decision card, two repair ORDERs and this active REPORT. The item and PLAN owner-direction rows are the only public-facing status prose changed; no figure, web, model, owner-path, bar, technical-rule or production-state edit is present. The item/PLAN status rows are the reversible workflow change.

Diff size after the second repair: seven files (three ORDERs, DECISION, this REPORT, item, PLAN), 185 insertions and 35 deletions (220 changed lines) against the approximately 180-line estimate (about 1.22×), with no line cap (D48). No production code or dependency changed.

## Measurements and cancel probe

- #2029 is `CLOSED`.
- `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` records bars (a)–(e) as `m1_milestone_status: "CLOSED_PASS"`; it is not called six-bar evidence.
- `docs/labs/m1-heldout-phrasing-transfer/evidence/panel-score.json` separately records `bar_f_pass: true` and `outcome: "PASS_BAR_F_HELDOUT_PHRASING"`.
- The prior refresh item was `done`, so proposing to reopen it is new post-close work.
- `cargo run -q --locked -p xtask -- roadmap check`: **112 items OK** — ready 16, held 10, decision 1, done 85.
- `cargo run -q --locked -p xtask -- roadmap render --check`: **EXPECTED STALE**, ready 16 expected against ready 15 in the generated page. This is the item-input change and requires the post-merge render; it is not reported as a pass.

## Checks run at the head

- `cargo test --locked -p xtask` with explicit Rust Cargo — **211 passed, 0 failed**.
- `cargo run -q --locked -p xtask -- roadmap check` — **PASS**, 112 items.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` — **PASS**, supported Markdown surface only; not a whole-repository scan.
- `git diff --check` — **PASS**.
- `uor-harness loop report --check` — **PASS**.
- `uor-harness loop pr-check --local` — **PASS verdict**. ORDER, REPORT, owner-paths, banned-presence, rule-sources, rendered-lock, read-path and FILEMAP pass. Only D76-advisory scope and commit-order warnings remain for this scope PR.
- Codex preservation refs verified at `ed54006e7615a5e14335a2b501c5c7ba14e14eee`; GitHub #3380 remains CLOSED until the manual post-merge reopen.
- A new fresh exact-head review is required before ready/queue. Earlier reviews are not reused as approval.

## Skills used

Brainstorming, working discipline, documentation co-authoring (freeform under auto mode), UOR research architect, solution integrity, scope guard, writing plans, executing plans, git worktrees, context guard, test-driven development, and two fresh delivery/scope reviews. No model experiment or GPU compute was run.

## Still open

- Harness report/policy checks and a new exact-head review, then ready/queue and merge verification on fresh `main`.
- Manual `gh issue reopen 3380`, roadmap sync, the separate generated-view render flow, and final verification.
- The comprehensive content PR (kimi-4 after the delivery repairs), figure/book/web checks, and owner-approval section before owner-path edits.

## Plain summary

This PR proposes reopening the documentation item with boundaries that now cover the real M1 cleanup and exclude other labs' records. Two reviews caught evidence, scope and delivery defects; those findings are preserved and repaired rather than bypassed. After merge, the issue is reopened manually before sync, and the active ORDER has its own correctly headed REPORT. No public milestone count or research rule changes in this scope PR.