# ORDER: infra-loop-harness, claude-74 (phase 0: record the single-lab rule)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `3ac553790`, then merged forward onto claude-73 (#3159), which puts `CHARTER.md` and `docs/rules/changes.md` in this item's scope.

## Goal

From 23 Oct 14:10Z to 24 Oct 13:43Z, the labs merged 70 PRs and closed no issue. Then every lab stopped behind posted asks that nobody answered: review requests, PM asks, and owner approvals waiting in one lab's own chat. On 24 Oct the owner answered the redesign questions in Claude's chat; his words are quoted below. This unit records them as the rules, with no tooling change:

- the single-lab rule: no step waits on another lab or on the owner;
- review inside the unit, with other labs auditing after the merge;
- the pre-run review in place of the owner release and the holds;
- the decision review in place of decision cards to the PM or owner;
- the owner's queue as a log;
- the finishing PR closes its issue;
- a supervisor that restarts idle labs.

The harness changes (gate, supervisor, queue, closing) are later units (phases 1–3 of the loop stall review).

## Cancel probe (run first)

At `3ac553790`, does the charter already let one lab finish a GPU run, a decision or a code PR without another lab or the owner? No. §5.1 holds Training, M2 and Stage B until an owner release given in the asking lab's chat, and says a pending owner question means wait. §5.2 sends bar, status, constraint and spend changes to the PM or owner as decision cards. §5.5 requires a code or model PR's exact-head review from a different vendor's lab (D46). The unit stands. Class: instrument (the rules), not a mechanism.

## Files in scope

- `CHARTER.md` (§5.1, §5.2, §5.5, §5.10 and §6 only; §3 and §4 are not touched)
- `AGENTS.md` (the summary lines that restate §5.1, §5.2 and §5.5; at most 80 lines)
- `docs/integration/DECISIONS.md` (rows D65–D71 appended; D62–D64 are left to Kimi's open #3149)
- `docs/rules/changes.md` (one dated entry)
- `docs/loop/tasks/infra-loop-harness/claude-74/**`

## Done when

- `CHARTER.md` states the single-lab rule, review inside the unit, post-merge audits, the pre-run review, the decision review, the owner's queue as a log, the closing PR and the supervisor. It quotes the owner verbatim, and §3, §4 and the GPU caps are unchanged.
- D65–D71 are appended to the decision table, one decision per row, with reason, alternatives turned down and cost to undo.
- `AGENTS.md` agrees with the charter and stays at 80 lines or fewer.
- `docs/rules/changes.md` has a dated entry naming who decided, quoting the owner, and listing the side effects on earlier rules.
- `loop report --check`, `loop pr-check --local`, `git diff --check`, the quick gates and the roadmap check pass at the head.
- The exact-head review is Claude's own fresh-context review. The gate at the base counts that for a doc-only PR, and it is also the review this unit records as the rule.

## Stop and ask if

A §3 constraint, a §4 evidence rule or the GPU caps would have to change, or any file outside the list above is needed.

## Owner approval

Owner paths changed: `CHARTER.md`, `AGENTS.md`, `docs/integration/DECISIONS.md`, `docs/rules/changes.md`.

Casey, in Claude's chat, received by 2026-10-24T21:00:26Z: "Yes, start phase 0: open the records PR with my words verbatim (single-lab rule, review inside the unit, pre-run and decision reviews, post-merge audits, owner queue as a log) and post the #2028 note with each lab's first move. I'll reopen the labs."

The words to record are Casey's answers earlier in the same chat, about 20:35–20:58Z on 2026-10-24.

- Owner queue: "One queue (Recommended)".
- Supervisor: "Yes, where headless (Recommended)".
- Closing: "The finishing PR (Recommended)".
- Held and GPU-gated work: "i never see these requests for released, so I would say do not park any work at all for any reason - if there was only one single lab running what would the process be? a single lab can't park and wait for another lab or me to review. It must be able to accurately judge whether or not, with an advesareal review, whether or not the GPU run is the correct test at that time, and whether the correct tools exist in either the repo or the huggingface repo that will make them successful. We don't want to be doing runs if they are not going to unlock further work, like when we finish a mechanism and are simply waiting on the gpu training or something to happen which unlocks further work, we want to do those as highest priority. It's all about removing blockers at this point."
- Read-back of the single-lab rule: "Accurate (Recommended)". The holds become runnable through the pre-run review, inside the cap.
- What still waits for the owner: "in theory the project structure should be build in the github issues so there should not be a point where anything is waiting on my decision - make sure before the decisions are made that the entire project is understood and that we understand from a whole project context what this step is for and what it is achieving and that it makes sense to do it - do not approve without larger context".
- Correction to the read-back's 30-minute review fallback: "Reviewers waiting 30 minutes for other labs to see it is the entire problem. Posting and waiting for an answer is where we lose."
