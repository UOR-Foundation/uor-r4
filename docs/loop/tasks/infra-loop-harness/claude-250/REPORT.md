# REPORT: infra-loop-harness, claude-250 (the files F's and T's trace sides need)

Lab: claude. Order: [ORDER.md](ORDER.md). References #3462.

**Speed.** No serving change: a scope edit.

## What did not work, first

- **Nothing was built or measured.** Q5 is still unread. This unit only lets F's and T's units write where the guard allows.
- **`loop claim` could not claim this unit.** The umbrella's `docs/roadmap/items/*.md` overlaps every open item's own file, so the claim refuses while any other lab has an order open. It refused on Kimi's #3509, then on DeepSeek's #3511. The claim has no override.
  - The unit was claimed by hand under Casey's 1 Nov direction, quoted verbatim in the [ORDER](ORDER.md#how-this-unit-was-claimed). The hand claim made the same writes as `loop claim`, and passed every check but the overlap by hand.
  - No open PR changes `integer-nearest-readout.md` or any of the five files below, so the supersede takes nothing from either lab.
  - Noted for Kimi and DeepSeek on #2028 (`6565213467`).
- **The three added files are shared with other ready items,** by literal path (done items that list them are left out):
  - `flock_read_memory.rs` with `one-model-end-to-end` and `turn-memory-integer`;
  - `geometric_stack_native_cuda.rs` with `training-step-cost`;
  - `bin/train-in-space.rs` with `one-model-end-to-end`, `point-key-integer`, `training-step-cost`, `turn-memory-integer` and `writer-admission`.

  All of those items are ready. `crates/uor-r4-float-train/**`, which covers all three, is in the held `e8-orbit-memory-sigmoid-free-iteration`. Whichever unit claims first, the overlap check makes the other wait.
- **The two removed files stay in other scopes:** `wasm_serve.rs` and `wasm_bindgen_exports.rs` are still listed by `point-key-integer`, `turn-memory-integer` and `m7-studio-live-model-sources`. Removing them from this item only ends its overlap with the Studio item's open orders.

## What changed

- **`docs/roadmap/items/integer-nearest-readout.md`:**
  - **`scope:`** adds three files and removes two, and nothing else changes in it. Added, each for one of claude-247's [decision card](../../integer-nearest-readout/claude-247/DECISION.md) kinds:
    - `crates/uor-r4-float-train/src/flock_read_memory.rs`, for kind 3, the memory lookups. `TurnMemory::stored` and `stored_reply` read the table inside `apply_readout`, and the opening's key is built only there and not returned.
    - `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`, for kind 5, the rate buckets. `rate_select_fwd` launches the selection kernel, and the card forces its entries.
    - `crates/uor-r4-float-train/src/bin/train-in-space.rs`, for the `--score-rows` options: the trace and the forcing set.

    Removed: `crates/uor-r4-integer/src/wasm_serve.rs` and `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`. The rest of Q5 touches neither: F's and T's sides, the replay driver, the grader and the sheet.
  - **"What is missing"** names the change, and says a later unit that needs either removed file adds it back.
- **The decision is claude-247's,** logged on #3462 (`6563736971`). This unit adds no decision of its own; the scope follows the card's kinds and the code that runs them.
- **`roadmap sync`:** a dry run shows one update from this unit, #3462's body. It is applied after the merge, from fresh `main`. The same dry run also shows a #2035 checklist change and a warning on #3500; both come from Kimi's `studio-typed-shell-upgrade`, not from this unit.

## Measurements

None: a scope edit.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 133 items OK. The render is unchanged.
- The cancel probe: none of the three files was in the scope at `03800ffd2`, and all three exist there. No open order holds any of the five files.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: recorded in the review record at the exact head.

## Skills used

None.

## Still open

- **F's side of the trace** (next, under `integer-nearest-readout`): `train-in-space --score-rows --decision-trace` writes side F's `uor-r4.decision-trace/1` on the GPU, with the five kinds in R's order. With the trace on, the rows files must stay byte-identical.
  - **Record the buckets at `geometric_stack_native.rs:1449`,** where the score is clamped and `rate_select` called. That file is in the scope. The secant hook in `rate_select_fwd` (`on_rate_select_fwd_values`) would need `secant_descent.rs`, which is not (review LOW 2). `geometric_stack_native_cuda.rs` is for forcing.
- **Then T's side, the replay driver and grader, and the sheet,** under claude-247's card.
- **`loop claim` cannot claim `infra-loop-harness` while any lab has an order open.** Noted in Claude's friction log; no harness change here.

## Plain summary

The next piece of work records every choice the float model makes on the way to a word, and it has to change three files the project's guard did not yet allow for this task. This change allows them, and stops listing two browser files the rest of the work never touches. Another lab's open task was holding the change up without sharing any of its files, so, as Casey directed, Claude went ahead past that hold without touching the other lab's work.
