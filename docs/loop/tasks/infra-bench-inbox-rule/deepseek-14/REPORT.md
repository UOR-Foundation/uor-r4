# REPORT: infra-bench-inbox-rule, deepseek-14 (bench inbox rule in AGENTS.md)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `cbde25bd3`. Closes the item (its issue is the #2028 umbrella entry that records Casey's bench decisions).

## What did not work, first

- **The bench feature this rule names is still an open PR.** `loop board --bench` and `local/bench/INBOX.md` come from Claude's claude-25 (PR #2942, open at the base). The item is independent text that only takes effect when that feature lands; if the feature's path differs before merge, the line needs a one-line repair rather than a guess now. The ORDER names this dependency, and the line references the item's own recorded path.
- **The line had to fit two gates, not just the item.** `AGENTS.md` is capped at 80 lines by the harness check and the read path has its own budget; the file was 70 lines and is 71 now, with the line kept to one bullet.
- **The rule has four parts and a ban, and dropping any one changes behaviour.** The line states the addressed-to-lab-or-`all` filter, the `waiting`→`read`/`answered`+time update, the one-line reply on an answer, the no-delete rule, and that a bench message is never an owner release.
- **A same-lab review will not count here either.** The PR carries `receipts.jsonl`, which the gate classes `other`, so it is not doc-only and needs another vendor's exact-head APPROVE — the lesson from #2940 is applied in the PICKUP rather than repeated as a mistake.
- **No other file was touched**, and the item was set to `done` only after its line landed.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `AGENTS.md` | docs | +1 | the bench inbox bullet in "How to work": read `local/bench/INBOX.md` at session start, act on `waiting` messages naming the lab or `all`, mark `read`/`answered` with the time (and a one-line reply), delete none, never treat one as an owner release |
| `docs/roadmap/items/infra-bench-inbox-rule.md` | docs | +1/−1 | `status: ready` → `done` |
| `docs/loop/tasks/infra-bench-inbox-rule/deepseek-14/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** one content line plus the item status and task folder; against the item's one-line done-when, the unit is as small as it looks.

## Measurements

- **No numeric result is claimed.** The checkable measurements are: `AGENTS.md` line count 71 (gate: at most 80), read path inside budget, and the rule's five clauses present in the one bullet.
- `loop gates --full` GREEN; `report --check` PASS; `pr-check --local` PASS.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.
- `uor-harness check`: the AGENTS 80-line rule and read path pass with the new line.

## Skills used

- **scope-guard** for a one-line, single-file change that stays inside the item's `AGENTS.md` scope.

## Still open

- **The bench feature (claude-25 / PR #2942)** must merge for the rule to have a file to read; the wording is path-accurate as recorded in the item.
- **An exact-head APPROVE from another vendor is required** (the PR is not doc-only because of `receipts.jsonl`); requested on #2028.

## Plain summary

- **One line in `AGENTS.md` makes the bench's messages reach a lab.** Without it, a message written on Casey's bench page is read by nobody; the line says who reads what, when, how to mark it, and that it is never an owner release.
- **The change respects the file's limits** (71 lines against the 80-line cap) and touches nothing else; the item moves to `done`.
- **It takes effect when the bench feature lands** (Claude's open PR #2942); until then it is a rule about a file that will exist, and a path change would be a one-line repair.