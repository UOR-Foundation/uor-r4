# REPORT: infra-loop-harness, claude-7 (PR 6's review gate follows D59 for the setup PRs)

Lab: claude. Base: `80938b12e` (#2900 merged). Order: [ORDER.md](ORDER.md), committed (`daefaf8dd`) before any edit. Its cancel probe found no D59 in the PR 6 item. The only D59 mention on the loop page was in the build-plan note.

## What did not work, first

- **PR 6's spec was written before D59.** Built as specified, its gate would have refused a Claude-built setup PR carrying a Claude review. It would also have accepted a Kimi review of a Codex-built setup PR. D59, as amended in #2900, requires the opposite in both cases.
- **The other twelve loop items still tell builders to ask another vendor.** Items PR 2–12, except PR 6, still say "ask a lab of a different vendor for the exact-head review". Under D59, a builder following them could ask Kimi to review a setup PR. That is outside this ORDER's scope, so it is left for a follow-up.

## What changed

| File | Class | What |
|---|---|---|
| `docs/roadmap/items/infra-loop-pr06-review-records.md` | docs | **Done-when:** until the other labs run the loop, a setup PR passes only with an APPROVE from Claude's fresh-context reviewers, whichever lab built it. A Claude-built setup PR with a Claude `fresh-context` APPROVE passes. The exemption ends without a code change, and how the gate learns that is left to PR 6's ORDER. **Planted fault:** widened to cover a setup-PR record from a lab other than Claude. **Elsewhere:** the review steps, the delivery rule and the sources now name D59. |
| `docs/loop/README.md` | docs | **"The review record":** one D59 sentence. **PR 6's build-plan row:** the same planted fault. |
| `docs/loop/tasks/infra-loop-harness/claude-7/` | docs | This ORDER and REPORT. |

D46 outside the setup, D55 (identity by tags) and D59 are unchanged. The item's status, dependencies and title are unchanged, so the rendered roadmap does not move. The item is not yet synced to an issue (it is absent from `docs/roadmap/issues.json`), so nothing on GitHub needs updating.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head: claim-wording, `xtask links` on the changed files, `roadmap check`, `roadmap render --check`, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- The same D59 note is needed in the review steps of the other twelve loop items, PR 2–12.
- PR 6's ORDER chooses how the gate learns that the other labs run the loop. One candidate is a recorded switch in `docs/harness/labs.json`, so ending the exemption needs no code change.

## Plain summary

- When PR 6 builds the machine check for reviews, it will follow the owner's rule: during setup, Claude reviews every setup PR.
- The other labs' reviews start counting once they are running the loop.
