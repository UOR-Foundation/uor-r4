# REPORT: m3-policy-precedence-and-sieve-tests, deepseek-31 (owner-directed close-out)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `b00d9edad`. Item PR, docs-only.

## What did not work, first

- **The item kept being offered although its code merged a day-earlier cycle.** #2947 delivered the helper extraction, the two precedence tests, the store-miss/sieve test and descriptive docs; #2947's own REPORT left the item `ready` on a PM decision card (`#2028` 6335479675), and the owner then directed the lab to proceed without waiting (quoted in the ORDER).
- **A first attempt to claim the umbrella item for the notebook-field edit was refused:** `loop claim` blocks an order whose scope overlaps an open PR's item scope, and Kimi's PR 8 (#2982) holds `docs/roadmap/items/*.md`. So this PR cannot add the item `notebook:` field (scope is read at the merge base); that is recorded as a follow-up, and the PM card's Option A (row in the in-scope lab README) is what this close-out uses.
- **The coupling is recorded as current behaviour, not as a settled mechanism contract.** That is the PM card's Option A and the conservative choice: a future change to make `StoreRead::Absent` fall through to `log_recall` is a mechanism decision with its own tests, not a bookkeeping edit.
- **No code was changed.** `lib.rs` already gates both modules under `#[cfg(test)]` (verified by reading it, with the exclusion comment), so PM card point 1 needs no further work.
- **No receipts file in this PR on purpose.** It is a docs-only change; per D46 a doc-only PR may carry a same-lab fresh-context record, and adding a `receipts.jsonl` would make it not doc-only. The verification evidence is in this REPORT and the gate logs.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` | docs | 1 | `status: ready` → `status: done` |
| `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` | docs | +~12 | the PM-card resolution under the owner direction: module gating verified, coupling kept descriptive, notebook row in-README with the real-page follow-up |
| `docs/roadmap/README.md` | generated | render | counts and row move to done |
| `docs/loop/tasks/m3-policy-precedence-and-sieve-tests/deepseek-31/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** about 60 changed lines.

## Measurements

- **Done-when evidence (already on `main` from #2947):** `op_policy_precedences_and_derived_id_guard_resolve_disagreements` pins both `OpPolicy` arms including the derived-id guard, with two mutation gates recorded in #2947's REPORT; `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` pins the store-miss/sieve rule; both are documented descriptively.
- **Comment-gating evidence:** `crates/uor-r4-training/src/lib.rs` lines around the two modules show `#[cfg(test)]` plus "Compiled only under `cargo test`; excluded from the non-test native qualification graph."
- **No out-of-scope path:** item status-only edit (allowed), lab README inside the item scope, roadmap render, and the task folder.

## Checks run at the head

- `cargo run -p xtask -- roadmap render` then `roadmap render --check`: current.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the head.
- `loop gates --full`: GREEN; `--break all`: CAUGHT.

## Skills used

- **solution-integrity**: the close-out names exactly what is settled (tests, documented descriptive rule) and what is not (the mechanism contract, the real notebook page), instead of treating a bookkeeping flip as the work.
- **holding-the-goal**: the item only closes because the done-when is already delivered; the owner direction resolves the pending coordination, not the science.

## Still open

- **Move the notebook row to `docs/research/notebook/m3-durable-memory.md` and declare `notebook:` on the item** once PR 8 (#2982) merges and `docs/roadmap/items/*.md` is unblocked.
- **The store-miss/sieve coupling as a settled mechanism contract** remains a decision if anyone wants it declared; today it is the documented current behaviour.

## Plain summary

- **The item's work was already delivered; this closes the bookkeeping under the owner direction and the PM card's Option A.** The two precedence tests, the store-miss/sieve test and the descriptive docs are on `main`, the module gating is already `#[cfg(test)]`, and the notebook row is recorded in the in-scope lab README.
- **Nothing scientific was changed or claimed settled.** The coupling stays a documented current rule; the real notebook page row is a follow-up once the item-file scope unblocks.