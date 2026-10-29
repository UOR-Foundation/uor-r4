# REPORT: one-model-end-to-end, claude-207 (the protocol's training instruments)

Lab: claude. Order: [ORDER.md](ORDER.md). Protocol: [one-model-end-to-end-protocol-2026-10-29](../../../../harness/prep/one-model-end-to-end-protocol-2026-10-29.md), instruments I-B and I-C. References #3407; the item stays `ready`.

**Result: the two training instruments exist, and their host-side rules are tested.**
- **`--turn-memory-dropout P`** drops the turn memory from a seeded share of training windows.
- **`--fbs-arm one-atom|identity|frozen`** picks the SpiralCore/FBS transport for training and scoring. `frozen` keeps the routing queries out of the optimizer.

**Speed.** This unit times nothing; the protocol's run measures it.

## What did not work, first

- **The GPU halves of both checks have not run.** The protocol's checks for these instruments need CUDA and the data, so they are the run's first probe-ladder steps on the pod, before any arm:
  - **I-B:** at P = 0, training is byte-identical to (r), the same `progress.jsonl` at step 256; at P = 1, the reply weight stays 0.0;
  - **I-C:** under `frozen`, every routing query equals its seed after 256 steps; under `one-atom`, training is byte-identical to today's default at step 256.

  What is tested here is every host-side rule that those GPU checks rest on (below). The wiring inside `run_long` is not executed on the laptop, so a fault there would show first on the pod.
- **Scoring with a transport arm covers three modes only:** `--score-rows`, `--inspect` and `--decode-speed`, the ones the protocol uses. `--reach`, `--grad-norms` and `--recall` still run `OneAtomActive`, and now refuse any other `--fbs-arm` (review round 1).
- **The arm cannot be read from a checkpoint copy.** A `models/step-N` folder's `config.json` is the model's, not the run's. claude-205's job fetched step folders alone. So the scorer takes `--fbs-arm` explicitly, and refuses a mismatch only when the run's `config.json` is beside the checkpoint (`check_run_arm`). A step folder copied alone is scored as asked, and the run's job must pass the arm it trained with.

## What changed

In `crates/uor-r4-float-train` (the item's scope):
- **`LongConfig` gains two fields:**
  - `turn_memory_dropout` (thousandths) and `fbs_arm`;
  - both are left out of `config.json` at their defaults, so a run without them writes the same file as before.
- **`memory_dropped(seed, step, b, batch, permille)`:**
  - one xorshift64* draw from its own stream, seeded by the run's seed and the window's place in the run;
  - it never consumes the batch order's draws;
  - 0 drops none, and 1,000 drops all.
- **In `run_long`:**
  - a dropped window names no opening row and no reply row;
  - the hook, the model build and the main optimizer's filter (`main_optimizer_updates`) follow `fbs_arm`. Under `FrozenQueries` the routing queries (`.spiralcore_fbs`) are not updated, as in the registered sheet's runner (`spiralcore_fbs_sheet.rs:497`).
  - Dropout without `--turn-memory` is refused.
- **`score_rows`, `inspect_run` and `decode_speed`** take the arm, set the hook with it, and call `check_run_arm`.
- **The binary:**
  - `--turn-memory-dropout P`, with values above 1,000 refused;
  - `--fbs-arm`, with any name but `one-atom`, `identity` or `frozen` refused;
  - the arm is passed to the three scoring modes, and the usage docs list both flags.
- **Tests, 5 new:**
  - `memory_dropout_drops_the_share_it_is_given`: 0 drops none, 1,000 all, 500 within 4σ of half on 1,600 windows, the same each time, and another seed draws another pattern;
  - `only_the_frozen_arm_keeps_the_routing_queries_out_of_the_optimizer`;
  - `the_transport_arm_is_named_on_the_command_line`;
  - `a_config_writes_dropout_and_arm_only_when_set`;
  - `scoring_refuses_another_arm_than_the_runs`: run folder and step folder, a model config ignored, a missing key read as the default.

## Measurements

None. This unit is code and its tests.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --no-fail-fast`: lib 139 passed, 0 failed (5 new). Every other target passed.
- **The negative gate:** with `main_optimizer_updates` reduced to the old filter (`!name.ends_with("rec.decay")`), `only_the_frozen_arm_…` FAILS; restored and rebuilt, it passes.
- **A stub-nvcc `cargo check --offline --locked -p uor-r4-float-train --features cuda --bin train-in-space --lib --tests`:** it builds.
- **At `779a55fa9`:** `loop report --check` PASS, `loop pr-check --local` PASS, `loop gates --full` 11 GREEN.
- **Review round 1, at `9d7271ee2`: APPROVE WITH NITS.** It confirmed the default path unchanged (dropout 0 draws nothing; one-atom reduces every arm branch to the old code; `config.json` unchanged), dropout's row naming, the frozen filter against the sheet's `is_query`, and old runs accepted. Taken: an unreadable run config is refused, not skipped (tested); the checkpoint path is canonicalized; `--reach`, `--grad-norms` and `--recall` refuse a non-default arm. Left: no host test runs the dropout filter inside `run_long`, whose gate is the pod's P = 1 check.

## What it unlocks

Two of the protocol's seven instruments exist. Next:
- **I-A:** `--score-rows --panel`, for P-dev.
- **I-E:** the Flock rank decode.
- **I-F:** the table hit rate.
- **Then I-D and I-G** on the laptop.
- **Then the run's pre-run review.** Its probe ladder opens with the GPU checks above.

## Skills used

None was invoked this unit.

## Still open

- **The GPU halves of I-B's and I-C's checks,** on the run's pod.
- **The transport arm in `--reach`, `--grad-norms` and `--recall`,** if a later sheet needs them (they refuse a non-default arm today).
- **Instruments I-A and I-D to I-H.**

## Plain summary

The training program can now leave its exact memory out of a chosen share of training examples. A model trained that way also learns to answer without the memory. The program can also swap the model's learned way of moving information between positions for two fixed stand-ins, which shows whether the learned version earns its place. Both are switches for the comparison run. Their tests on the laptop pass. Their checks on the GPU run first when that run starts.
