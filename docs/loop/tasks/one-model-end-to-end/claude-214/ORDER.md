# ORDER: one-model-end-to-end, claude-214 (the scope the loop test needs)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `f67149842`.

## Goal

A scope decision for this item. The guard enforces an item's scope as it stands on `main` ("so a branch cannot widen its own scope"), so the scope must merge before the code that needs it. claude-203 did the same for `m-world.rs`.

**Why the item needs it.** claude-213 (merged as `f67149842`) found that the shipped model's greedy replies loop, and that inside a loop the nearest token is confident: 95.9–99.6% of loop steps admit exactly one candidate. No draw rule can break such a loop.
- **The candidate cause is the Hamming read's copy heads.** At position `t`, a copy head finds an earlier `s` whose previous token is the current token, and brings back the token at `s`. In free generation it also sees the reply's own earlier tokens, so a repeated token brings back what followed it last time.
- **The test** (claude-215, the next unit) is a decode-time limit on which positions the read may choose, so the reply reads only the prompt. That choice is made only in `choose_sorted` (`crates/uor-r4-float-train/src/flock_read_hamming.rs:577-604`, with one non-test caller at `:438`).

**The decision.** Add one file to the item's `scope:`: `crates/uor-r4-float-train/src/flock_read_hamming.rs`.
- **Logged** on #3407 (6518346519), with a fresh decision review: PASS.
- **Its conditions** bind claude-215:
  - `choose_sorted` takes an explicit limit;
  - the default path is unchanged byte for byte;
  - the cut is `s < prompt.len() − start` in window coordinates;
  - the limit is set only around `decode_reply`'s forward and cleared before any `?`;
  - a negative gate covers the cut.

## Cancel probe (run first)

Result, 30 Oct (instrument; nothing blocks):
- **Is the file already in the item's scope?** No. At `f67149842`, `one-model-end-to-end`'s `scope:` holds `flock_read_memory.rs` but not `flock_read_hamming.rs`.
- **Does another item or open PR hold it?**
  - **Items:** `in-space-trunk-to-text`, `route-plus-memory` and `turn-memory-reply` held `flock_read*.rs`, and all three are done. `integer-hamming-read` only cites the file in prose.
  - **Open PRs:** none touches it.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/one-model-end-to-end.md` (the `scope:` line)
- `docs/loop/tasks/one-model-end-to-end/claude-214/**`

## Done when

1. **The item's `scope:` line on `main`** holds `crates/uor-r4-float-train/src/flock_read_hamming.rs`, and the rendered roadmap is unchanged (the scope line is not rendered).
2. **The decision is logged on #3407** with its review record.

## Stop and ask if

Stop and decide; never wait.
- **A collision appears:** another lab claims the file before this merges. Recheck the open PRs, and narrow or withdraw the change.
