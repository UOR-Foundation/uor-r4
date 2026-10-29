# REPORT: infra-loop-harness, claude-162

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: item-file maintenance, no model run.

## What did not work, first

- **The first `writer-admission` title was 84 characters,** over the roadmap's 80. `roadmap render --check` refused it, and it is now "Make the native writer's admission hold at every seed".
- **Two figures in the first drafts did not match their sources,** caught by reading each source before the review:
  - `training-step-cost` gave the cell objective's share in (a) as 13% (claude-157's headline). Its table gives `13.11` ms, 14%. The item now quotes the table, in ms and shares.
  - `writer-admission` said no run had measured the writer under the cell objective. Nothing on main says that. It now cites the one checkpoint the map does record (`345` against `7,813`, stage 7).
- **The five items cannot all run side by side.** The three float-side items (`turn-memory-reply`, `writer-admission`, `training-step-cost`) share `in_space_long.rs` and `train-in-space.rs`. The two serving items share `session.rs` with each other and with `integer-hamming-read`. `loop claim`'s overlap check is the lock. PLAN says one float-side and one serving-side item can run at once.
- **No item here is on the M1 path.** All five are `milestone: research`, like the pivot's three. They follow the map's order of work (R1–R5), not a milestone.

## What changed

- **Five new items** in `docs/roadmap/items/`, each `status: ready`, with the owner's words, the measured gap with its source, a done-when, `How to start`, `Rules that apply` and `Sources`. Every scope file exists at the base, except `crates/uor-r4-integer/src/stack/memory.rs`, which `turn-memory-integer`'s first unit creates.
  - **`turn-memory-reply`** (the follow-up to R1): claude-161's memory moves only a reply's first row, and copies did not rise (`1,713 / 2,333` of 4,130). Done when a TRAIN-against-DEV probe ranks candidate keys for later reply rows, the read is extended to the best one, and a pre-registered two-seed run with an empty-table control is read.
  - **`writer-admission`** (R2): claude-150's writer held at seed 1 (`8,430` against nearest's `6,762`) and never learned at seed 2 (`493` against `6,909`). The target's cell was admitted on only `461`–`714` of 14,287 rows. Done when seed 2's failure is diagnosed from its checkpoints, admission is changed geometrically, and a two-seed run holds the writer within a pre-registered margin of nearest.
  - **`fbs-serving-kernels`** (R3, host): the six layers are 98.6% of a served token (3.73 ms, 265 tok/s, claude-158). Done when each layer is split into gathers, core, gated product and requantization, and a vector kernel for the largest part is exact against `step_native`, with tokens/s with and without it.
  - **`training-step-cost`** (R4): the backward is 62% of (a)'s 96.5 ms step (claude-157). Done when an `nsys` profile names the backward's top three kernels, and a cheaper rival search is run against (a) with a bar on prediction.
  - **`turn-memory-integer`** (R5, the memory's part): the memory's key and table are exact ids, and only the move is float. Done when the artifact carries the memory, the integer session applies it, its openings agree with the float forward's on claude-161's sealed checkpoint, and its cost is measured on the M1.
- **`PLAN.md`:** the owner direction "ok yes, my vote is to pivot to your diagnosed recommended new roadmap" is now `done`, with its note. The trunk line's pivot list now names the three merged items and the five next, in the map's order, with which can run side by side.
- **`docs/roadmap/README.md`:** re-rendered (ready 12 → 17).

## Measurements

None: no run. Every figure in the five items is quoted from a REPORT or the map, and each item names its source.

## Checks run at the head

- `cargo xtask roadmap check`: 107 items, ready 17, held 10, decision 1, done 79.
- `cargo xtask roadmap render --check`: the README is current.
- Every function and file the items name was found by `grep` at the head: `stack_recurrence_core`, `stack_fbs_gather`, `stack_step_fbs_timed`, `measure_fbs_step_stages`, `step_native`, `nearest_loss`, `best_positions`, `opening_bound`, `opening_rows`, `MarginHinge`, `address_scores`, `build_table`, `docs/mechanisms/pointer-key.md`.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The unit closes the owner's pivot direction on its own done-when (all three merged) and writes only the items the map's order of work names next.
- **verify-before-done.** Each figure was checked against its source before the review, which caught the two mismatches above.

## Still open

- **The other two 28 Oct directions stay `open`:** "I don't care what we serve at long as it is cheap" is the standing criterion these items carry, and the float-train, integer-serve direction is carried by `turn-memory-integer` and `integer-hamming-read`.
- **R6–R8** (deterministic backward, the text path, the scale probe) get items once R2–R5 report, as the map orders.

## Plain summary

Casey voted to pivot to three pieces of work: measure where time goes, revise the map, and test whether a memory of the training replies helps. All three are done, and the memory doubled how often a reply starts with the right word. This writes up the next five pieces of work so they can be picked up: carry the memory past the first word, make the cheap way of choosing a word reliable, speed up the laptop's integer code, cut the cost of a training step, and run the memory on the laptop.
