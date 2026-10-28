# ORDER: fbs-cost-profile, claude-157

Lab: claude. Item: [fbs-cost-profile](../../../../roadmap/items/fbs-cost-profile.md). Base: `93a98e311`.

## Goal

The item's done-when reads:

> 1. **One training step, profiled by stage** on an RTX 5090, at claude-152's (a) recipe (cell objective, width 256, 4 Hamming heads), and at width 512 with no read:
>    - wall-clock per stage: embedding, each layer's recurrence and feed-forward, each read, the objective's table search, the backward, the optimizer;
>    - from CUDA events or the driver's profiler;
>    - summing to the measured step time within 10%;
>    - with the top three named.
> 2. **One served token, profiled by stage** on the M1 in `fbs-toks`: each trunk layer, the readout (writer and whole-table nearest, each timed), and the loop overhead, summing within 10%.
> 3. For each of the top three costs, one line on the cheapest change that would remove most of it, with its effect on prediction stated as measured or as a hypothesis.
> 4. One notebook row and a REPORT. A one-page sheet with a fresh FIT comes before the GPU part (CHARTER §4.5), because it runs training steps.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/harness/prep/fbs-cost-profile*`
- `docs/loop/tasks/fbs-cost-profile/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/fbs-cost-profile.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
