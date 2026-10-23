# REPORT: loop-render-review-hardening, deepseek-46

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `loop-render-review-hardening`. Base: `0af044c7f`. Status: **in progress (draft PR #3048)**.

## What did not work, first

- **The item was unclaimable when carded** because its scope listed `loopprcheck.rs`, a shared registration file; fixed in #3045 before this claim. This unit then had to edit that shared file as an always-allowed path, with its test in `tests/loop_review.rs`.
- **The documented render workflow was refused by the gate.** `loop render` stamped HEAD, which is the merge commit after a merge-forward, while the gate requires the stamp to equal the merge base - exactly the failure Kimi hit on #3038. Implemented the default-stamp fix instead of only documenting `--at`.
- **The queue mirror could admit a fork named `kimi/render`** because `headRefName` carries no owner and `ReviewContext::of` took the sentinel on the branch name alone. Fixed by plumbing `isCrossRepository` through `Pr`, `PrGh`, `QueuePr` and requiring a known same-repository head.

## What changed so far

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/looprender.rs` | `default_at(root, branch)` returns `merge-base(origin/main, HEAD)` on `kimi/render`, HEAD elsewhere; `cli` uses it and skips the working-tree roadmap render when the base is used |
| `crates/uor-r4-harness/src/loopreview.rs` | `render_sentinel(head_ref, files, cross)` requires a known same-repository head; the review path uses it |
| `crates/uor-r4-harness/src/loopprcheck.rs` | `Pr.is_cross_repository`, `PrGh::is_cross_repository` (real + fake), `ReviewContext::of` sentinel via `render_sentinel(pr.is_cross_repository)`, pr-check `--pr` fills the flag |
| `crates/uor-r4-harness/src/loopqueue.rs` | `QueuePr.is_cross_repository` read from `gh pr view --json ...isCrossRepository` and set on the `Pr` |
| `crates/uor-r4-harness/tests/loop_review.rs` | offline temp-git test for the merge-base default; sentinel tests (fork, unknown, wrong branch, non-view path) |
| `docs/loop/tasks/infra-loop-render-review/deepseek-42/REPORT.md` | fixed the doubled `infra-infra-` prefix nit |

**Still to do for the item:** the `context()` fake-git fixture with its negative gates (the review's one remaining requirement); the roadmap-README decision is recorded here as **exempt with reason** (xtask renders the working tree, the views-only set keeps it out of the no-ORDER path, and the docs job gates it advisably); the loop-page wording is PM-owned and carded on #2028.

## Measurements

- `cargo test -p uor-r4-harness --test loop_review`: **34 passed** (32 existing + merge-base default + sentinel cases). `cargo check -p uor-r4-harness`: clean.
- No GPU; every test is offline and uses a temporary git repository or pure functions.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `cargo test -p uor-r4-harness`; results are recorded after the runs when the draft is opened for review.

## Skills used

- `solution-integrity`: fixed the workflow the gate refused instead of only documenting a workaround; refused the sentinel on unknown repository provenance rather than assuming same-repo.
- `scope-guard`: shared files edited additively; tests in the item's own test file; no owner path.

## Still open

- The `context()` fake-git fixture with negative gates (blocking the item's done-when).
- The loop-page wording (owner-only; carded on #2028).

## Plain summary

- Landed two of the five follow-ups (merge-base render default, queue-mirror fork check) with tests, fixed the REPORT nit, and recorded the README exemption reason; the fixture remains before the item can close.