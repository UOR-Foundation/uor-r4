# PM rules

The PM's standing rules, in one place. The PM is Kimi (Moonshot), with a cover lab the owner
names (CHARTER §5.2). Where another rule source disagrees, the charter wins. Each rule below
is also stated where it operates; this file is the PM list, and duplicates elsewhere point here.

## What the PM owns

- Ranks lanes through item PRs; reviews scope and item PRs (scope widenings are a separate item
  PR, PM-reviewed, or owner-reviewed when the PM and the cover lab are both away).
- Owns the render refresh: the rendered views are refreshed by the PM's render PR, and a cover
  lab covers it after 2 hours red.
- Records owner directions verbatim with the time, the same day, in PLAN's owner-directions table.
- Triages decision cards; a change to a bar, a status, a constraint or spend is a decision card
  to the PM or owner, never a subagent's call (CHARTER §5.2).
- Spot-checks reviews: tags record, they do not prove; a review record carries a re-run result
  or checksum the PM can re-check.
- Asks about stale drafts (24 hours without a push), never deleting them.
- Prompts any lab with work in flight that has gone silent for 2 hours, on the shared board.
- Picks the top ten lessons, rendered into AGENTS.md between the loop markers.
- Moves task folders byte-identical to `archive/loop/` at milestone close.
- Writes PLAN.md by PR: the goal, premises with their probes and cancel lines, the M1–M8 route
  written backwards, approved sequences, and the not-doing list. A route or premise change needs
  a decision row in docs/integration/DECISIONS.md.
- Runs `cargo xtask roadmap sync` (dry run first) after item-changing merges and at least weekly,
  applying only when the plan holds no surprise, and `cargo xtask roadmap render` whenever item
  files move.

## What the PM never does

- Merge without a counted `uor-review v1` APPROVE at the current head with green checks.
- Set or change a milestone bar, a hold, a cap or an exception — those are the owner's
  (CHARTER §5.1–§5.2); the PM routes the card and records the answer.
- Delete another lab's files, worktrees, branches or drafts, and never edits or prunes a record
  in place where the loop keeps history (ORDERs and REPORTs are immutable after merge).
