# REPORT: infra-loop-harness, claude-5 (D59 recorded: no lab review of the loop setup)

Lab: claude. Order: [ORDER.md](ORDER.md), committed before any edit, with its cancel probe: no D59 or similar exception existed.

## What did not work, first

- **The direction was missed the first time.** The owner's first statement ("none of the other labs are onboarded yet so having them review the onboarding will be problematic - anyway, go") was a direction. Claude read it as an open question and asked Kimi to review loop PR 1 (#2898). The owner corrected this. The requests were withdrawn on #2898 and #2028, and this PR records the direction as D59.
- **The scope was too narrow.** It did not cover the AGENTS.md decision-index label, which a new row makes stale. The ORDER was widened in a commit before that one-line edit, not after it.

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `CHARTER.md` | 1 sentence in §5.5, plus its citation | rules | Until the other labs are onboarded to the loop, the loop's setup PRs are reviewed by fresh-context reviewers of the authoring lab, never by the other labs (D59). |
| `docs/rules/changes.md` | a new entry | rules | 21 October 2026: the owner's two statements, verbatim, and what they decide |
| `docs/integration/DECISIONS.md` | row D59 | rules | the decision, its reason, the alternative turned down, and the cost to undo it |
| `AGENTS.md` | 1 label | rules | Decision index D0–D59 |
| `docs/loop/tasks/infra-loop-harness/claude-5/` | new | docs | this ORDER and REPORT |

## Measurements

None. This PR changes rules only.

## Checks run

The checks run at the head are listed in the PR body: claim-wording, `xtask links` on the changed files, `roadmap check`, the harness real-repo rule tests, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

D59 ends when the other labs are onboarded to the loop (PR 9). From then on, D46's cross-vendor review applies to every code and model PR.

## Plain summary

- The owner's direction is now written into the charter: the other labs don't review the loop's setup while they aren't running it.
- A fresh-context reviewer from the authoring lab reviews the setup instead.
