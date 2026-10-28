# ORDER: fbs-cost-profile, claude-157

Lab: claude. Item: [fbs-cost-profile](../../../../roadmap/items/fbs-cost-profile.md). Base: `93a98e311`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". This unit measures where one training step and one served token spend their time, so every later choice is ranked by cost per token. The item's done-when reads:

> 1. **One training step, profiled by stage** on an RTX 5090, at claude-152's (a) recipe (cell objective, width 256, 4 Hamming heads), and at width 512 with no read:
>    - wall-clock per stage: embedding, each layer's recurrence and feed-forward, each read, the objective's table search, the backward, the optimizer;
>    - from CUDA events or the driver's profiler;
>    - summing to the measured step time within 10%;
>    - with the top three named.
> 2. **One served token, profiled by stage** on the M1 in `fbs-toks`: each trunk layer, the readout (writer and whole-table nearest, each timed), and the loop overhead, summing within 10%.
> 3. For each of the top three costs, one line on the cheapest change that would remove most of it, with its effect on prediction stated as measured or as a hypothesis.
> 4. One notebook row and a REPORT. A one-page sheet with a fresh FIT comes before the GPU part (CHARTER §4.5), because it runs training steps.

## The instruments

- **Training: a stage clock.** It is thread-local and off unless a run asks for it. When on, each stage boundary in the forward, the objective, the backward and the optimizer records a CUDA event on candle's stream, with the host's time beside it.
  - A stage's time is the gap between its two events. No synchronization is added between stages, so the profiled step runs as the unprofiled one does.
  - The host gaps beside the GPU gaps show whether a stage waits on kernel launches or on the GPU.
  - The steps are profiled after warm-up and alternate with unprofiled steps, whose time is the "measured step time".
- **Serving: a stage-timed twin of the FBS session step.** `IntegerStackSession::measure_fbs_step_stages` mirrors `stack_step_fbs`, as `measure_step_phases` mirrors the dense step, and leaves `step` untouched for the zero-multiply audit. It times:
  - the embedding;
  - each layer's recurrence and feed-forward;
  - the final norm;
  - the writer;
  - the bookkeeping.

  Beside these it times a whole-table nearest search over the token table, as the readout the cell objective decodes. That search changes no state. `fbs-toks --stages` reports the medians over the protocol's decode steps.

## Instrument checks (before any reading)

- **I-1.** With no new flag, every file a run writes is unchanged (new config keys are skipped when off). `fbs-toks` without `--stages` prints what it printed.
- **I-2.** A profiled step's loss equals the unprofiled run's loss at the same step and seed, exactly.
- **I-3.** The stage times sum to the GPU span of the profiled step. That span is compared with the unprofiled step time; a gap over 10% is reported as a miss, not hidden.
- **I-4.** Over the whole `fbs-toks` protocol, `measure_fbs_step_stages` gives the same tokens and the same session state as `step_native` (a host test, and the protocol's reply check at run time).

## Cancel probe (run first)

Run at `93a98e311` before any work:
- no open PR touches `in_space_long.rs`, `geometric_stack_native.rs`, `train-in-space.rs`, `fbs-toks.rs` or `session.rs`;
- no other lab's ORDER names `fbs-cost-profile`;
- the item is `ready`.

All three held.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `docs/harness/prep/fbs-cost-profile*`
- `docs/loop/tasks/fbs-cost-profile/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/fbs-cost-profile.md`

**Scope change (claude's decision review, 28 Oct).** This unit adds `crates/uor-r4-integer/src/stack/session.rs` to the item's scope.
- **Why:** the FBS step's stages are private functions in that file, so a stage-timed step can only live there.
- **Reversal:** the new function is additive, `step` and `stack_step_fbs` are unchanged, and deleting the function reverses it.
- **Evidence:** the dense step's twin, `measure_step_phases`, set the pattern.

## Done when

1. The four instrument checks pass, with their output in the REPORT.
2. The item's done-when 1 to 4 are met: two training recipes profiled on an RTX 5090 and one served token on the M1, each stage summed against the measured time, the top three named, one line each on the cheapest fix.
3. The run's outputs are sealed in the HF store and read back.
4. The PR is merged and verified on main.

## Stop if

- **I-2 fails.** The profiler changed the computation: an instrument fault. It is fixed before any reading is taken.
- **The pod or a lease would exceed the caps.**

A stage sum that misses by more than 10% is not a stop: it is reported with the gap named. No step waits on Casey.
