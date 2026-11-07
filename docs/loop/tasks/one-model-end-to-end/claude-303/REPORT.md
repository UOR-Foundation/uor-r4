# REPORT: one-model-end-to-end, claude-303

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-w1024-rerun-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-w1024-rerun-2026-11-07.md), which amends [claude-302's](../../../../harness/prep/one-model-end-to-end-w1024-2026-11-07.md) (FIT WITH CHANGES; D67 PASS WITH CHANGES).

## What did not work, first

- **There is still no width-1,024 reading.** Both seeds stopped at I-t, claude-302's wall-time gate, before any arm trained. The bar, the width lever and the spread are all unread. This is an instrument stop, not a mechanism negative (L-007): it closes nothing about width.
  - I-t measured 3.578 s a step (seed 1) and 3.636 s a step (seed 2) at micro-batch 16. Over 16,384 steps that projects 66,641 s and 67,618 s, against the bound of 57,600 s.
  - Each job exited 0 with a registered NOT_RUN.
- **The memory diagnosis is not the one the cause predicted.** Change 4 predicted that, with dead gradients gone, (micro2)'s peak would fall to H or below. It did not, at either seed:

  | seed | whole | micro4 | micro2 | read0 | H | Change 4's reading |
  |---|---:|---:|---:|---:|---:|---|
  | 1 | 38,397 | 29,661 | 29,629 | 12,221 | 25,309 | over H, under 90% of whole: **partly** |
  | 2 | 29,803 | 29,579 | 29,579 | 12,395 | 21,099 | within 10% of whole: **the cause is incomplete** |

  (MiB, each run's peak inside its own window of `mem.log`.) At seed 2, (whole) and (micro2) differ by 224 MiB. So micro-batching still barely lowers peak memory. About 17,000 MiB more than (read0) is held whatever the micro-batch, and this unit did not find what holds it. The sheet names two candidates: memory the read keeps across micro-batches, or a pool that cudarc's allocator keeps. `mem.log` cannot tell them apart.
- **Seed 1's (whole) is the odd reading.** It is 8,594 MiB above seed 2's, while every other run agrees across seeds within 224 MiB. This is one 2 s sample series, not a repeated measurement.
- **FIT Finding 4 was not applied as written.** That is a CUDA test comparing a weighted step's gradients with and without pruning. The run gives a different, end-to-end check of the same question instead (see the readings).

## The readings

- **The fix lowered memory everywhere** (claude-302 → claude-303, MiB):

  | run | claude-302 | claude-303 seed 1 | claude-303 seed 2 |
  |---|---:|---:|---:|
  | whole | 66,411 | 38,397 | 29,803 |
  | micro4 | 66,379 | 29,661 | 29,579 |
  | micro2 | 66,379 | 29,629 | 29,579 |
  | read0 | 21,131 | 12,221 | 12,395 |

  (claude-302's figures are its seed 1's; its seed 2 stopped at I-e before recording memory.)
- **I-m passed at micro-batch 16 at both seeds:** 38,621 and 38,571 of 97,887 MiB. At claude-302's pin, the same smoke ran out of memory.
- **The pruned step trains exactly as the unpruned one, end to end.** The first four steps' `train_loss_mean` values agree, at every printed digit, with claude-302's at the unpruned pin `ff5156bcc`. That holds for all four I-e runs (whole, micro4, micro2, read0), at both seeds: 32 values. Steps 2 to 4 are computed after updates, so equal losses mean the updates agreed too, to float32 print precision.
- **I-0 passed at both seeds:** 397 CUDA library tests, 0 failed. These include `a_steps_gradients_keep_only_the_variables_a_detached_input_leaves_none` and `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step`.
- **I-e passed at 10⁻⁵ at both seeds**, worst cell 1.84 × 10⁻⁶ (seed 2, micro2, step 4: the same cell as claude-302's). The read-off control was underpowered again (1.07 × 10⁻⁶ and 2.97 × 10⁻⁶ at step 4), recorded and not gated, as the sheet says.

## What this unlocks (sheet, outcome map)

I-m passed, so memory no longer blocks width 1,024. I-t stops it, and under claude-302's outcome map that is NOT_RUN: rerun. Two readings point the next unit at speed:
1. memory at micro-batch 16 is 38.6 GB of a 97.9 GB card;
2. (whole) and (micro2) now peak within a few hundred MiB of each other at seed 2.

So **the next unit measures I-t at micro-batch 32 and at the whole batch of 64** on one pod, with I-m at each. If either projects under 57,600 s with I-m passing, width 1,024 trains at that micro-batch under claude-302's sheet. The 17,000 MiB that micro-batching does not reduce is a second, separate question. It is worth one allocator-counter reading, but it no longer blocks the run.

claude-301's (b64L) recipe at width 256 stays the trunk's human-text recipe, "met; unseen ahead".

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:** `variables_only`, called after each forward's `backward()`, keeps only the gradients of the variables the loss reaches. Its CUDA test is `a_steps_gradients_keep_only_the_variables_a_detached_input_leaves_none` (`fd90aa1c16e2af35d7fa52d101726e7fe6927953`).
- **`docs/harness/prep/one-model-end-to-end-w1024-rerun-2026-11-07.md`:** the sheet. `41cc9d30` added the required block headings for `prep.sheet_shape`, with no change in content.
- **`docs/loop/tasks/one-model-end-to-end/claude-303/`:** the ORDER and this REPORT.
- **`docs/research/notebook/float-training.md`:** one row.

## Measurements

| seed | pod | $/h | result | spend |
|---|---|---:|---|---:|
| 1 | vast-59491082, RTX PRO 6000 | 1.789 | I-0, I-e, I-m PASS; I-t NOT_RUN (66,641 s) | $0.91 |
| 2 | vast-59491128, RTX PRO 6000 | 1.775 | I-0, I-e, I-m PASS; I-t NOT_RUN (67,618 s) | $0.86 |
| 1, 2 | vast-59490794, vast-59490799, 2 × PRO 6000 each | 3.522 | deleted before SSH, no job (00:38–00:41Z) | ≈ $0.35 |

- **The discarded pair.** `uor-pod up` asks for 2 GPUs when `--count` is not given. The first pair was therefore placed on 2-GPU hosts, and I deleted it within minutes. The pair above was created with `--count 1`.
- **Total spend:** about $2.12 of the $70 cap. Each watcher took its pod down after `seal=0` (L-004). 0 instances are running.
- **Seals,** each fetched and checked against its SHA256SUMS on the laptop:
  - `results/claude/one-model-end-to-end-w1024-rerun-2026-11-07/runs/s1`: `w1024r-s1.tar.gz` `f5271a8e09551b90e146f9316fe191d7ac967014623d48b036a7f3ce90f09e60`
  - `runs/s2`: `w1024r-s2.tar.gz` `8dc4b2643fa4d8270375ffbf48fc0cab91df1a2c81653ca2067498d56ac8ce90`
- **Notes on #3407:** the FIT and D67 review (6699714881), and the launch (6699799890).

## Checks run at the head

- Library tests on the laptop: 294 pass. The stub-nvcc CUDA check (`--features cuda --all-targets`) passes.
- The harness's `preprules::` tests: 17 pass, including `the_real_sheets_pass_the_shape_rule`.
- The job's gate tests on the laptop (`gt/test-gates.sh`): 51 of 51 before launch, with each of three mutations caught.
- On the pod: I-0's 397 CUDA tests, both seeds.
- The seals' SHA256SUMS check OK on the laptop, both.
- The 32 I-e losses were compared by hand against claude-302's sealed `ie.txt`.

## Skills used

None.

## Still open

- What holds the roughly 17,000 MiB above (read0) that micro-batching does not reduce. The next diagnosis reads the allocator's own counters.
- The per-variable comparison asked for by FIT Finding 4, as a CUDA test. The end-to-end loss match above covers four steps, not a weighted step's gradients.
- Seed 1's (whole) peak, 8,594 MiB above seed 2's.
