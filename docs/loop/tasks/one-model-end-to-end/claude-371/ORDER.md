# ORDER: one-model-end-to-end, claude-371 (native-space inventory and first parity contract)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `675f5eb4334a5d4f87a2b5fa45ec8a46e0b81fc4`.

## Goal

**Write one inventory card of every state field and operator a served native-space model carries, from token to emitted id. Then write one executable parity contract that starts the training-mode simulation and the integer serving runtime from byte-identical bytes, compares raw words, and plants one fault for each category.** No training run, no capability claim, no speed claim.

**Why now.** DeepSeek, which runs the CUDA drivers and training on `wick`, handed this track to Claude through Casey (13 Nov). The rule it states: training and serving are one state transition. Addresses, carry, route choices, memory keys and reads, learned state, updates, optimizer state, and emitted ids with stop state all use one declared integer representation and the same typed operators from the first training step to serving. An export that quantizes a different F32 master is not this path.

**Sources the hand-off names that are not on `main`.** D85, a native-space skill, `native-space.json`, a native-space roadmap item and a Codex branch were all named. None is on `origin/main`, GitHub or this laptop. The rule above is taken from the hand-off as Casey pasted it, and is stricter than CHARTER §3.7, not against it. This order files under #3407 so that it does not duplicate an item Codex may hold unpushed. It does not touch `local/worktrees/codex-native-space-cycle`.

## Cancel probe (run first)

- **What would make this unit unnecessary:** an inventory of the served path's state with widths and wrap rules, plus a test that compares training-mode and serving words with planted faults per category.
- **Result (at base):** none whole.
  - `crates/uor-r4-cpu-train/src/discrete_route.rs` has `export_parity`. It compares read-vector words, admitted cells, top-8 and the predicted token between the CPU proposal scorer and `IntegerDiscreteModel`. It plants no faults, has no route-choice, flock-choice, touched-word, stop or save words, and has no CUDA leg.
  - `crates/uor-r4-integer/src/discrete_d11.rs` tests compare serving with hand-copied CUDA qualification witnesses, not with a CUDA run on the same bytes.
  - `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` is never compiled (`#![cfg(any())]`).

## Claim

Written by hand, as claude-359 to claude-370 were: `uor-harness loop claim` refuses while this lab's worktrees hold the item. No card or test of this unit exists at this order's commit.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-371/**` (ORDER, INVENTORY, PARITY, REPORT)
- `crates/uor-r4-cpu-train/src/discrete_route.rs`, its `#[cfg(test)]` module only (the parity contract test)
- `docs/research/notebook/float-training.md` or the discrete line's notebook, one row

## Done when

1. `INVENTORY.md` has one row per state field or operator. The columns are: field or operator; numeric domain, width and scale; lifetime; CUDA training; integer serving; CPU oracle; WASM; current parity evidence; gap. It covers every area the hand-off lists, each with a file and line from the code at base. Each width, mask, signedness and wrap rule is read from the code, not assumed.
2. `PARITY.md` gives the fixture: the start bytes, the words compared for each category, the planted faults, and the CUDA leg's design for `wick`.
3. One test in `discrete_route.rs` runs the contract's CPU training-mode to serving leg. Each category's planted fault is either caught by a named check, or named as not caught, with the reason. The test fails if a check that should catch its fault stops catching it.
4. A REPORT, negatives first, names the source files, representation choices, backend identities, failing controls and the smallest next training dependency.

## Stop list

- No GPU run, no training run, no pod. The CUDA leg is designed here and runs in a later unit on `wick`'s RTX 4090 (no rented GPU).
- No change to any kernel, serving code, export or artifact format.
- No harness, loop-kit or bookkeeping change.
