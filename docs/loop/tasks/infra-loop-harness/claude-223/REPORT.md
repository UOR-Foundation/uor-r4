# REPORT: infra-loop-harness, claude-223 (point-key-integer's scope for its grade, and its unit order)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refuses while Kimi's #3436 holds `PLAN.md`.

## Tokens per second

No measurement: this unit amends an item.

## What did not work, first

- **The sync was not applied.** Its dry run planned two updates:
  - #3450's body, this unit's;
  - #3440's, DeepSeek's `m7-studio-live-model-sources`, whose item file changed on main in `d2f0354a9` without a sync.

  The sync has no single-item option, and this ORDER stops on anything but #3450's update. So #3450's issue body follows at the next sync (the PM's weekly run, or the next lab's). The item file, the source of truth, is amended here.

  This is the ORDER's stop rule ("the sync plans anything but #3450's update") acting. The lab settled it by its own decision, with no post-and-wait: writing another lab's issue is not this order's to do. Done-when bullet 2 is therefore deferred to the next sync.
- **The item's "everything runs on the laptop" was wrong in one step:** `fbs-snapshot` loads the float checkpoint on CUDA. That step is now named as a brief pod step.

## What worked

- **[`point-key-integer`](../../../../roadmap/items/point-key-integer.md)'s scope gains `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs`.** The integer arms are graded there, beside claude-220's `served_grade`, by the same reference reading.
  - The float crate links `uor-r4-integer` without `serving-wasm`, so it cannot decode the stack.
  - The decode is a `fbs-toks` mode, which the item already lists.
- **"The units"** records unit 1 as done (claude-222, #3452, `f4beaaf79`). It sets out unit 2 (the panel decode and the grade, on the laptop, with finite tests) and unit 3 (the run, under a prep sheet with a fresh FIT and a pre-run review).
- **"Which run goes where"** now splits the integer artifact's runs (the laptop) from the snapshot export (a pod, briefly).

## What changed

- `docs/roadmap/items/point-key-integer.md`: the scope, "Which run goes where", and a new section, "The units".
- `docs/roadmap/README.md` and `issues.json`: unchanged. The render is current, and no issue was created.

## Measurements

- `roadmap check`: 116 items OK.
- `render --check`: current.
- `roadmap sync` (dry run): 2 updates, #3440 and #3450. Not applied, as above.

## Checks run at the head

- The roadmap check and the render check, as above.
- **Nothing else changed:** no owner file, bar, status, other item or code.

## Skills used

None.

## Still open

- **#3450's issue body,** at the next sync.
- **Unit 2, the panel decode and the grade:** its own claim (hand-written while #3436 is open).

## Plain summary

This unit updates the plan for the integer lookup. The check of the integer model's answers belongs in the same place as the float model's, so that file is added to the plan's list. The plan now spells out its next two steps: build the tools that run and grade the integer model on test questions, then run them on the trained model.
