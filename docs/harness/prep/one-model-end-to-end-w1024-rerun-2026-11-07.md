# Prep sheet: width 1,024 rerun, with only the variables' gradients kept (one-model-end-to-end, claude-303, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-303. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438).

TEST FITNESS: pending (fresh non-author review).

**This sheet amends [claude-302's](./one-model-end-to-end-w1024-2026-11-07.md).** Its Question, Data split, Arms, Readings, Bar, outcome map, Minimum test power, I-0, I-3, I-m, I-f, I-t and Stop rule all apply word for word, except where a numbered change below says otherwise. The seeds, pods, steps and caps are claude-302's.

**Why.** claude-302 (#3808) could not train width 1,024. Both of its seeds stopped at registered gates, and both stops point at instrument faults.
- **Memory.** At batch 16 the four I-e runs peaked as follows: whole 66,411 MiB, micro4 66,379, micro2 66,379, and read-off 21,131. So micro-batching did not lower peak memory at all. At batch 64, I-m's smokes at micro 16 and 8 both ran out of a 97,887 MiB card.
- **The cause, read in the source** (ORDER claude-303).
  - candle-core 0.9.2's `backward()` gives a gradient to every input of an op, through `or_insert`. That includes inputs that track none, and those gradients are never removed.
  - The Hamming read's copy heads gather a `(positions, 16, width)` payload per head from a detached table, so each copy head leaves a payload-sized gradient that no optimizer reads.
  - `add_grads` summed every micro-batch's store into the accumulator, so those dead gradients piled up across micro-batches until the optimizer step.
- **The fix** (`variables_only`, claude-303's commit `fd90aa1c16e2af35d7fa52d101726e7fe6927953`): after each forward's `backward()`, the trainer keeps only the gradients of the variables that the loss reaches. Every optimizer reads only variables' gradients, so the update is unchanged. A CUDA test checks that a detached input's gradient is dropped and that the variable's gradient is kept bit-equal.
- **The I-e miss:** at seed 2, micro2 differed from whole by 1.84 × 10⁻⁶ relative at step 4. Every other cell was at most 3.47 × 10⁻⁷. Change 3 below sets I-e's role at width 1,024.

## Changes from claude-302's sheet

1. **The pin** becomes claude-303's commit, `fd90aa1c16e2af35d7fa52d101726e7fe6927953`, on PR #3809. Its trainer differs from `ff5156bcc` in exactly two ways, `git diff ff5156bcc fd90aa1c -- crates/uor-r4-float-train`:
   - `variables_only` and its test (claude-303);
   - DeepSeek's #3803–#3806: `query_cache.rs`, the `fbs-query-cache` binary, and code in `score_rows` and `train-in-space.rs` that runs only under `--query-cache-out`. This sheet never passes that flag.
   The width lever already compares two builds (claude-302, Base artifact), so the REPORT names both differences, as it names the card.
2. **I-0** also runs `a_steps_gradients_keep_only_the_variables_a_detached_input_leaves_none` by name, with claude-300's three accumulation tests.
3. **I-e at width 1,024 checks that the micro-batched step runs and agrees; it does not prove exactness.**
   - **Why the old tolerance cannot do more here.** Four steps of loss move the trunk very little: at width 1,024, turning the read off moved step 4 by only 1.07 × 10⁻⁶ and 2.97 × 10⁻⁶. A 10⁻⁶ tolerance therefore sits inside the gap it would need to detect, and it stopped seed 2 on a difference smaller than the read's own effect.
   - **What carries exactness instead:** I-0's CUDA test `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step`. It compares each variable's gradient, not the loss, for whole against micro-batched steps, at this pin.
   - **Pass:** (micro4)'s and (micro2)'s `train_loss_mean` are each within **10⁻⁵** relative of (whole)'s at every step, 1 to 4. A dropped or doubled micro-batch moves its share of the scored rows, so at step 1 the mean is off by 1/4 or 1/8 of a loss that differs across windows. That is well over 10⁻⁵.
   - **Fail:** as before, the pod trains nothing.
   - **The control** is recorded, as before.
   - This change is registered before any run at this pin. Every claude-302 cell, the 1.84 × 10⁻⁶ included, would have passed it.
4. **A new reading, the diagnosis: peak memory of each I-e run,** from `mem.log` inside that run's own window, which the job now writes.
   - **The fix works if** (micro2)'s peak is at most half of (whole)'s.
   - If (micro2) and (whole) are still within 10% of each other, the cause above is incomplete. The run still goes on to I-m, which gates it.
   - This reading is recorded, not gated.
5. **The seal path** is `results/claude/one-model-end-to-end-w1024-rerun-2026-11-07/runs/s{1,2}/`. claude-302's seals are not touched (L-002).
6. **The cost:** claude-302's, with a cap of $70 for this unit, and watcher stops at 17 h and $34 a pod.

## What a pass and a fail unlock

- **I-m passes and the arm trains at both seeds:** claude-302's Readings, Bar and outcome map decide the next unit, word for word.
- **I-m fails again at both micro-batches:** (w1024) is NOT_RUN at that seed. The diagnosis reading (Change 4) then names what the next unit measures: a remaining dead gradient, if (micro2) stayed near (whole), or the activations of a micro-batch of 16, if it did not.
- **I-e fails at 10⁻⁵:** that is a real disagreement, not reduction noise. The pod trains nothing, and the next unit is the accumulation code at width 1,024.
