# ORDER: infra-loop-harness, claude-248 (the scope Q5's trace instrument needs)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), the umbrella that edits roadmap items. Base: `bc4b4c959`.

Setup row: 2 (item-file maintenance: a scope addition to a gap item).

## Goal

This unit adds to [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md)'s scope (#3462) the five files that claude-247's [decision card](../../integer-nearest-readout/claude-247/DECISION.md) named. The guard enforces an item's scope as it stands on `main`, so the scope must merge before the code that needs it (claude-197's finding, claude-203's precedent).

**Why the item needs it.** Done-when 3 now waits on Q5 alone: an exact trace of 184 clear-margin disagreements, under claude-247's rule.
- That rule records every discrete decision of five kinds on F, T and R. It forces codes and rate buckets, and it writes each side's query at p.
- No instrument at the base does this (claude-247's ORDER, check d; the card's "What the instruments can and cannot do").

**The decision is already made.** claude-247's card states the scope change under Constraints, and it was approved after two fresh decision-review rounds (owner log on #3462, `6563736971`; merged as `c3952eae2`). This unit applies it and adds nothing to it.

The five files:
- `crates/uor-r4-integer/src/stack/hamming.rs`: the trace records each code's 64 coordinates, and a forced code replaces the computed one;
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`: the same for the float read, with a force switch beside its `TIE` and `TRACE`;
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`: the recurrence records each lane's rate score and entry, and a forced entry replaces `rate_select`'s;
- `crates/uor-r4-float-train/src/cuda_stack.cu`: the same for the GPU kernel;
- `crates/uor-r4-float-train/src/in_space_long.rs`: `score_rows` writes each traced position's query and top two, and runs on chosen rows with the trace and a forced set.

## Cancel probe (run first)

- **Are the files already in the item's scope?** At `bc4b4c959`, `integer-nearest-readout`'s `scope:` holds none of the five.
- **Does an open order hold any of them?** Every ORDER under `docs/loop/tasks/` that names one of the five has a REPORT, and no PR is open. Other items list them in `scope:`, by literal path (`one-model-end-to-end`, `training-step-cost`, `point-key-integer`, `turn-memory-integer`, `writer-admission`) and by wildcard (`crates/uor-r4-integer/**` in six ready `m*` items, both wildcards in a held item). No open claim holds any of them. A shared file is allowed: the claim's overlap check orders the units.
- **Do the files exist at the base?** All five do.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-248/**`

## Done when

1. **The item's `scope:`** lists the five files above, and nothing else is added. Its "What is missing" says the scope is in place.
2. **`cargo xtask roadmap check`** passes, and the render is current.
3. **A fresh exact-head review passes.** It checks the edit against claude-247's card, the item, and the other items' scopes and open claims. No new decision review is needed: the card's two rounds made this decision.
4. **`loop report --check`, `loop pr-check --local` and `loop gates --full`** pass.

## Stop and ask if

Stop and decide (never wait) if:
- the review finds the instrument needs a file the card did not name. Then that file needs its own decision, and this unit adds only the five;
- another open ORDER holds one of the five files.
