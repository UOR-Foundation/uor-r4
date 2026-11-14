# ORDER: infra-loop-harness, claude-371 (`loop next` returns the current route to the complete model)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Base: `origin/main` at `675f5eb43b99e5ee634cd45b5fd517f69a638c52`.

## Goal

**Make `uor-harness loop next` return the current step of the route to the complete model, in place of the 1 Nov CPU-training sentence it has returned to every lab since.**

## Owner direction

Casey, in Claude's chat, 14 November 2026:

> "please update loop next into our current roadmap for the current run you are working on and toward a complete model that trains and serves the same format and runs locally on the laptop with and/or in browser without matmul or other wasteful operations"

`PLAN.md` is the PM's file by standing arrangement. The owner asked Claude for this change directly, so Claude writes it and says so in the PR.

## Why the harness changes too

`loop next` takes its line from the first `open` row of PLAN's `## Owner directions` table and prints that row's Direction cell, which holds the owner's words. Two things follow.

- A direction is one sentence. It cannot say which step of a roadmap is the current one, so a new row alone would make `loop next` print the owner's request back to them.
- An open direction decides for every lab. The 1 Nov row was given to one lab and has been offered to all five. A new open row addressed to Claude would be offered to the other four in the same way.

So the plan gets a `## Route` table of ordered steps, each with the lab that holds it, and `loop next` reads it before the directions. The new direction row is recorded with status `routed`: open, and carried by the route.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a source in `loop next` that already returns a step of a multi-step plan, or a plan table that already holds the route.
- **Result (at base):** none. `crates/uor-r4-harness/src/loopnext.rs` reads two PLAN tables, `## Owner directions` and `## Premises`. PLAN's `## The route M2–M8` is prose with no status, and no lab's `loop next` reads it.

## Claim

Written by hand. No code of this unit exists at this order's commit.

## Files in scope

- `PLAN.md`
- `crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/tests/loop_next_sources.rs`
- `docs/loop/README.md`, `docs/harness/README.md` (the two shared pages that state `loop next`'s order)
- `docs/loop/tasks/infra-loop-harness/claude-371/**`

Nothing else. `docs/integration/DECISIONS.md` is the owner's and is not touched.

## Done when

1. `loop next` has a `route` source with tests that fail without the rule they pin:
   - the first open row of PLAN's `## Route` table is the decision, before any owner direction, for a lab its Lab cell names, and for every lab when the cell reads `any`;
   - a step held by another lab is named and passed over, no later step is offered in its place, and that lab gets the next open owner direction as before;
   - a plan with no route table, or with every step closed, prints what it printed before this unit;
   - a route with no Lab column, or a first open row with an empty Work or Lab cell, is named and offers nothing.
2. PLAN records the 14 Nov direction word for word, and the route: the current run (claude-370) first, then the steps to a model that holds a conversation, trains and serves in one format, runs on the laptop and in the browser, and uses no matmul.
3. Run against this branch's PLAN, `loop next` returns step 1 of the route for `claude` and the 1 Nov direction for another lab.
4. The two shared pages state the new order.

## Decision review (the top session's own, §5.1)

- **What is decided.** How the owner's sentence is laid out as steps, and a new status word, `routed`, in PLAN's direction table.
- **What is not decided.** No bar, milestone order, scope or stop rule moves. Each step's numbers stay in its unit's own sheet. The M2–M8 order is unchanged. The 1 Nov CPU-training direction stays open and is still what the other labs are offered.
- **What the owner can reverse in one line.** A step's order or lab (edit the row), and the reading of "trains and serves the same format" written under the route.

## Stop rules

- No model, GPU, pod or training run is part of this unit.
- No existing `loop next` output changes for a plan without a route table.
- No decision row is written: the decision index is the owner's file.
