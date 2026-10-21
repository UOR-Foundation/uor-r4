# REPORT: infra-loop-harness, claude-7 (PR 6's review gate follows D59 for the setup PRs)

Lab: claude. Base: `80938b12e` (#2900 merged). Order: [ORDER.md](ORDER.md), committed (`daefaf8dd`) before any edit. Its cancel probe found no D59 in the PR 6 item. The only D59 mention on the loop page was in the build-plan note.

## What did not work, first

- **PR 6's spec was written before D59.** Built as specified, its gate would have refused a Claude-built setup PR carrying a Claude review. It would also have accepted a Kimi review of a Codex-built setup PR. D59, as amended in #2900, requires the opposite in both cases.
- **The other twelve loop items still tell builders to ask another vendor.** Items PR 2–12, except PR 6, still say "ask a lab of a different vendor for the exact-head review". Under D59, a builder following them could ask Kimi to review a setup PR. That is outside this ORDER's scope, so it is left for a follow-up.
- **The first head left the gate underspecified.** A fresh-context Claude review of `985d4e1bb` asked for changes. All the fixes below are in the next head, inside the ORDER's scope.
  - **Medium: setup PRs.** They were defined by the folder their ORDER sits in. That cannot tie a PR to a build-plan row, and it would let a Claude-built code PR filed there pass on a Claude self-review. *Fix:* a setup PR now names its row in a `Setup row:` line, and the gate checks that row against the build-plan table.
  - **Medium: the review record.** It has no field that marks a fresh context. *Fix:* the item now says the gate checks the `claude` tag only (D55), and that fresh context is the reviewing lab's word.
  - **Medium: the planted fault.** The item and the table row worded it differently. *Fix:* both now read "a same-vendor record on a code or model PR outside D59".
  - **Low-medium: the end switch.** It was left to the builder. *Fix:* the owner sets the switch and decides when it flips; PR 6's ORDER picks only where it lives.
  - **Low: two stale lines.** "Why it matters" and the Sources line. *Fix:* both updated.

## What changed

| File | Class | What |
|---|---|---|
| `docs/roadmap/items/infra-loop-pr06-review-records.md` | docs | **Done-when:** while D59 is in force, a setup PR passes only with an APPROVE tagged `claude`, whichever lab built it, so a Claude-built setup PR with a Claude APPROVE passes. The gate checks tags, not sessions (D55). A setup PR is one whose ORDER names a build-plan row in a `Setup row:` line. D59 ends by a switch the owner sets, with no code change. **Planted fault:** covers a setup-PR record from a lab other than Claude, and a same-vendor record on a code or model PR outside D59. **Elsewhere:** "Why it matters", the review steps, the delivery rule and the sources now name D59. |
| `docs/loop/README.md` | docs | **"The review record":** D59, the `claude` tag check and the `Setup row:` line. **PR 6's build-plan row:** the same planted fault, word for word. |
| `docs/loop/tasks/infra-loop-harness/claude-7/` | docs | This ORDER and REPORT. |

D46 outside the setup, D55 (identity by tags) and D59 are unchanged. The item's status, dependencies and title are unchanged, so the rendered roadmap does not move. The item is not yet synced to an issue (it is absent from `docs/roadmap/issues.json`), so nothing on GitHub needs updating.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head: claim-wording, `xtask links` on the changed files, `roadmap check`, `roadmap render --check`, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- **The same D59 note is still needed in two places:**
  - the review steps of the other twelve loop items (PR 2–5, 7a–7c and 8–12);
  - the parent item `docs/roadmap/items/infra-loop-harness.md`, which says "Every review comes from a different vendor than the author".
- **Each setup PR's ORDER now needs a `Setup row:` line** for PR 6's gate to treat it as a setup PR.
- **PR 6's ORDER chooses where the owner's end switch lives.** When the switch flips stays the owner's call.

## Plain summary

- When PR 6 builds the machine check for reviews, it will follow the owner's rule: during setup, Claude reviews every setup PR.
- The other labs' reviews start counting once they are running the loop.
