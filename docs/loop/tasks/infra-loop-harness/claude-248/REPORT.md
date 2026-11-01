# REPORT: infra-loop-harness, claude-248 (the scope Q5's trace instrument needs)

Lab: claude. Order: [ORDER.md](ORDER.md). References #3462.

**Speed.** No serving change: a scope addition.

## What did not work, first

- **Nothing was built or measured.** Q5 is still unread. This unit only lets the next one write its instrument where the guard allows.
- **The five files are shared with other items,** by literal path and by wildcard:
  - **By literal path, in ready items:**
    - `in_space_long.rs` with `one-model-end-to-end`, `training-step-cost`, `point-key-integer`, `turn-memory-integer` and `writer-admission`;
    - `flock_read_hamming.rs` with `one-model-end-to-end`;
    - `geometric_stack_native.rs` with `training-step-cost`.
  - **`crates/uor-r4-integer/**`, which covers `stack/hamming.rs`,** is in six ready items' scopes: `m3-qualify-memory`, `m4-export-uorlut01-cli`, `m5-laptop-full-path-measurement`, `m6-qualify-reasoning-coding`, `m7-parity-library-cli-browser` and `m8-release-packaging`. It is also in the held `e8-orbit-memory-sigmoid-free-iteration`.
  - **`crates/uor-r4-float-train/**`, which covers the four float files,** is in the held `e8-orbit-memory-sigmoid-free-iteration`. Every other item that lists it is done.

  No open claim holds any of them now: the only open orders without a REPORT are kimi-1, kimi-2 and kimi-43, which name none of the five. The instrument's unit will wait on, or be waited on by, whichever of those items claims first, and the claim's overlap check orders them.
- **`cuda_stack.cu`'s changes must keep the kernels' outputs byte-identical with the trace and the forcing off.** That test belongs to the instrument's unit.

## What changed

- **`docs/roadmap/items/integer-nearest-readout.md`:**
  - **`scope:`** adds the five files named in claude-247's [decision card](../../integer-nearest-readout/claude-247/DECISION.md), and nothing else:
    - `crates/uor-r4-integer/src/stack/hamming.rs`;
    - `crates/uor-r4-float-train/src/flock_read_hamming.rs`;
    - `crates/uor-r4-float-train/src/geometric_stack_native.rs`;
    - `crates/uor-r4-float-train/src/cuda_stack.cu`;
    - `crates/uor-r4-float-train/src/in_space_long.rs`.
  - **"What is missing"** says the scope change is in place.
- **The decision is claude-247's.** It was approved after two fresh decision-review rounds and logged on #3462 (`6563736971`). This unit adds no decision of its own.
- **`roadmap sync`:** a dry run shows one update, #3462's body. It is applied after the merge, from fresh `main`.

## Measurements

None: a scope addition.

## Checks run at the head

- `cargo xtask roadmap check`: 125 items OK. The render is unchanged.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: recorded in the review record at the exact head.

## Skills used

None.

## Still open

- **The trace instrument,** under `integer-nearest-readout`. It records the five kinds of decision on F, T and R, forces codes and rate buckets, writes each side's query at p, and builds T's rounded copy. Then comes the sheet that reads Q5, carrying claude-247's carried nits (#3462 `6563930926`).

## Plain summary

The next piece of work, a tool that records and replays every choice the model makes on the way to a word, has to change five source files. The project's guard only lets a task touch files listed for its item. This change lists them, as the approved plan said, and changes nothing else.
