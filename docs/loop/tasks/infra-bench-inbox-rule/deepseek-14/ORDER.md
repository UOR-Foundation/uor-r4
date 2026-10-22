# ORDER: infra-bench-inbox-rule, deepseek-14 (bench inbox rule in AGENTS.md)

Lab: deepseek. Item: [infra-bench-inbox-rule](../../../../roadmap/items/infra-bench-inbox-rule.md) (#2028). Base: `cbde25bd3`.

## Goal

Add the one line the item's done-when asks for to `AGENTS.md`'s "How to work" list: at session start a lab reads `local/bench/INBOX.md`, acts on the `waiting` messages addressed to it or `all`, replaces `waiting` with `read` or `answered` plus the time (an answer carries a one-line reply), deletes none, and never treats a bench message as an owner release.

## Cancel probe (run first)

Class **instrument** (docs only). At base `cbde25bd3`: `AGENTS.md` has no bench rule (`grep -n 'local/bench' AGENTS.md` finds nothing), while the item records Casey's 22 Oct choice "Addressed per lab (Recommended)" and `loop board --bench`/`local/bench/INBOX.md` are the bench's message path (Claude's open claude-25 / PR #2942). The rule is independent text and takes effect when the feature lands; if the feature's path changes, a one-line repair follows. The task stands.

## Files in scope

`AGENTS.md` (item scope) plus the item's `status:` line and this task folder. No other file is touched.

## Done when

- `AGENTS.md`'s "How to work" list has the bench line with the addressed-to-lab/`all` rule, the `waiting`→`read`/`answered`+time update, the reply line, the no-delete rule, and the "never an owner release" sentence.
- `AGENTS.md` stays at most 80 lines and the read path stays within budget.
- The item is set to `done`; `loop gates --full` GREEN; `report --check` and `pr-check --local` pass.

## Stop and ask if

- The bench feature's actual message path differs from `local/bench/INBOX.md` before merge (then the rule needs a repair, not a guess).
- A charter/owner path would change; GPU work becomes implied (it is not).