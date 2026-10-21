# REPORT: infra-loop-harness, claude-9 (three D59 wording leftovers)

Lab: claude. Base: `72071ac75` (#2902 merged). Order: [ORDER.md](ORDER.md), committed (`5ad8e1a77`) before any edit. Its cancel probe found all three leftovers still present.

## What did not work, first

- **#2902 left three wording gaps, and its fresh-context review named them:**
  - Line 45 of the loop page stated the review rule with no setup exception.
  - The parenthetical "(another vendor's, or Claude's …)" listed another vendor first. That could be read as letting another vendor approve a setup PR, which PR 6's gate refuses.
  - `AGENTS.md` item 6, the first delivery summary a new agent reads, named only "another vendor's lab".
- **`AGENTS.md` is an owner path.** The edit to it rests on the owner's message, quoted verbatim in the ORDER.

## What changed

| File | Class | What |
|---|---|---|
| `AGENTS.md` | rules summary (owner path) | Item 6 adds: "loop setup PRs, until the other labs run the loop: Claude's fresh-context reviewers, D59". It stays one line, and the file stays 70 lines. |
| `docs/loop/README.md` | docs | Line 45 adds that during the setup, Claude's fresh-context reviewers review the setup PRs instead (D59). Step 12 and the supervisor queue rule now read "Claude's for a setup PR under D59; otherwise another vendor's". |
| `docs/roadmap/items/infra-loop-pr07c-queue-board.md` | docs | The done-when's parenthetical has the same Claude-first order. The planted fault is unchanged and still matches its row. |
| `docs/loop/tasks/infra-loop-harness/claude-9/` | docs | This ORDER and REPORT. |

D46, D55 and D59 are unchanged. Every edit restates D59 where a reader meets the review rule.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head: claim-wording, `xtask links` on the changed files, `roadmap check` and `render --check`, the harness real-repo rule tests, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- **The roadmap sync.** The 14 loop items still need it to become issues.
- **The wording of CHARTER §6.** Three reviewers read "except for the loop setup PRs that Claude builds" as contradicting §5.5. It is accurate, because same-lab review survives only for Claude-built setup PRs, but clearer wording is the owner's call.

## Plain summary

The three places a reader could still miss the setup exception now name it, with Claude first.
