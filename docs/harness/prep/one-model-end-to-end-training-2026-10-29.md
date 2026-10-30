# Prep sheet: the training run of the model to ship (one-model-end-to-end, claude-211, 29 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-211](../../loop/tasks/one-model-end-to-end/claude-211/ORDER.md). Protocol: [one-model-end-to-end-protocol-2026-10-29](one-model-end-to-end-protocol-2026-10-29.md) (TEST FITNESS: FIT). This sheet is the run that protocol registers; it changes none of its arms, strata, bars or decisions.

TEST FITNESS: FIT. PRE-RUN: PASS. Both are from fresh non-author reviews at `129de2373a9fa0242c0e2b99bafdcb9a0e659463`:
- **Round 1** (`66cb08708`) was NOT FIT and FAIL. The job lacked four of the GPU checks claude-207 and claude-208 promised.
- **Round 2** resolved that, and raised three questions:
  - whether scoring restores each arm;
  - whether the byte checks are fair;
  - the row grep.
- **Round 3** found all three resolved, and ran the store listing and `uor-pod status`: 0 pods running, $0/h.

## The question

The protocol's: which trained recipe ships, which add-ons are candidates, whether learned transport earns its place, which decode rule ships, and whether the shipped recipe generalizes or memorized. This run produces the stored outputs those five decisions are made from.

## Base artifact: the arms

- **The base recipe** is claude-194's (r), at width 256 on the pinned TRAIN, with the fixed-order backward:

  `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --seed S`

- **Each arm adds one flag** (claude-207's I-B and I-C, `4916350fe`, are on main):

  | Arm | Added flag |
  |---|---|
  | (r) | none |
  | (r-drop) | `--turn-memory-dropout 500` (thousandths: p = 0.5) |
  | (r0) | `--turn-memory-empty 1` |
  | (r-flock) | `--flock-read 7` |
  | (r-spin) | `--type-spin 1` |
  | (r-ident) | `--fbs-arm identity` |
  | (r-frozen) | `--fbs-arm frozen` |

- **Seeds:** 1, 2 and 3, so 21 runs.
- **The binary:** built on each pod from this unit's head, `--locked`, with its sha256 recorded before the first arm.

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code.
- **Fresh V2:** `inputs/fresh-v2-sealed/2026-10-26/`, the tar checked against its `SHA256SUMS` and its own sha. Descriptive only.
- **P-dev:** `inputs/claude/heldout-panels-2026-10-29/p-dev`, checked against that folder's `SHA256SUMS`. `tokens.u16` sha256 is `790b7e12934a241f07ec5c04fc6e929c86e970583ba5dee9a8646f7d235266cd`. This run opens it once, as the protocol registers.
- **N-u, fixed before the run** (the protocol: "set by I-G before the run"). U's rule is "the rows TRAIN's tables name none of": on DEV, (T) names exactly K's 10,849 rows, and its 3,438 others are U. On P-dev, (T) names 245 openings and 1,029 later rows. So:
  - **N-u** is P-dev's other **15,143** rows: the rows of `rows-p-dev-T.json` whose pick is 4294967295 (no table names them);
  - **N-k** is the **1,274** named rows.

  The row set comes from `train-in-space --data-root DATA --controls 1 --out DIR --fresh-v2 FRESH --panel p-dev=PDEV` at `008b40413` (I-D, below). Its file `rows-p-dev-T.json` has sha256 `2fded3280ede48117967cef5b0c2932a83dd4eb66c7cc5936479e45c71037ff6`, and is byte-identical across two runs and two builds. No model output was read to set it.

## Steps

One pod per seed. Each runs one job, in this order:

1. **Source and tests.** The source is pinned to this unit's head. Then the crate's CUDA tests (`cargo test --release --locked -p uor-r4-float-train --features cuda --lib`), and the build, with its sha256 recorded.
   - **I-E's GPU half is one of those tests** (`the_flock_rank_rule_at_k1_decodes_as_greedy_on_a_model`). On a model, the Flock rank rule at K = 1 decodes exactly the greedy reply, through the same top-K sort and gather the served K = 16 runs, and K = 16 leaves greedy's path. A failing test stops the pod.
2. **The data,** fetched and checked as above.
3. **The probe ladder,** on every pod, before its arms. Every probe is 256 steps at seed 1, `--eval-every 256`, keeping the checkpoints at steps 0 and 256. Each result is a line in `instruments.txt`:
   - **Each of the 7 arms:**
     - its losses at step 256 are finite;
     - its training time projects to under 1 hour for 16,384 steps, as `train_seconds` × 64. Start-up and evaluations are not in it, so claude-194's 37 s wall probe no longer projects 39 minutes for a 26-minute run.
     - An arm that fails either is skipped on that pod alone.
   - **I-B at P = 0 and I-C under `one-atom`:** each is (r)'s config, so each must be (r)'s run: the same `progress.jsonl` without its two timing fields, and the same step-256 checkpoint, byte for byte.
     - A second run of (r) is the control (`I-H0`). If it too differs, training is not reproducible on that GPU, and the two checks cannot be byte checks. They are then recorded INCONCLUSIVE, each with its DEV top-1 gap from (r) beside (r)'s own gap from itself.
     - If only the flag's run differs, the flag's wiring changes training.
     - **Why byte identity is a fair test:** every checkpoint file is written deterministically. `safetensors` orders its tensors by dtype, then name, and the turn memory sorts its entries before saving (`flock_read_memory.rs:387`). The model's `config.json` is its own, with no path or time in it. So `diff -r` differs only when training does.
   - **I-B at P = 1000:** a further run of (r) with `--turn-memory-dropout 1000`. Its reply weight stays 0.0 at step 256 (`--inspect`'s `memory_reply_weight_mean_abs`).
   - **I-C under `frozen`:** the arm probe's step-0 and step-256 checkpoints are kept, and every `.spiralcore_fbs` tensor must be equal between them. The grader makes this check on the laptop.
   - **I-A:** the (r) probe's step-256 checkpoint is scored with `--panel p-dev=PDEV --samples 8`. P-dev must score its census's 16,417 rows, with the memory on and out. A miss stops the pod, since every arm's N-u reading rests on it. Its sample prompts, each opening a reply, are checked by the grader in every arm's 256 stored samples.
   - **The disk:** 63 checkpoints (7 arms, steps 0 and every 2,048), at the probe checkpoint's size, must fit in `/workspace` with a tenth to spare, or the pod stops.
   - **A failed I-B, I-C or I-H0 line does not stop the pod.** The arms train, and nothing is read from the affected arm's comparisons until the fault is classified as instrument or object (CHARTER §4.1). The checkpoints are kept, so a repaired instrument re-scores them without retraining.
4. **The arms:** each of the 7, for 16,384 steps, one after another.
5. **The scoring of each arm's step-16,384 checkpoint:**

   `train-in-space --score-rows CKPT --out DIR --fresh-v2 FRESH --panel p-dev=PDEV --samples 256 --reply-tokens 64 --retain-rates 1 --seed S --fbs-arm ARM`

   This writes:
   - the rows on DEV, Fresh V2 and P-dev, with the memory on and out;
   - DEV and P-dev samples, greedy with the memory on and out, and Flock rank with it on;
   - their hit counts.

   **It passes no training flag, and needs none.** `load_reads` (`in_space_long.rs:380`) restores from the checkpoint every read the arm trained with, each to none when absent:
   - the Flock read with its neighbour count (r-flock);
   - the Hamming reads, with the type spin's `step_rotation` when present (`flock_read_hamming.rs:237`, r-spin);
   - the opening read and the prompt register;
   - the turn memory and its reply table, empty for (r0).

   `--fbs-arm` is the one setting a checkpoint does not carry. So it is passed for (r-ident) and (r-frozen), and `check_run_arm` refuses a mismatch with the run's `config.json`.
6. **I-H, reproduction.** (r) and (r0) at seeds 1 and 2 are compared with claude-194's sealed step-16,384 DEV readings: 11,513 / 11,361 and 10,079 / 11,171.
   - A miss is an instrument finding: training is not reproducible across builds. It is reported first. It does not void this run's comparisons, which all use one binary.
7. **The seal.** Everything under the job's output folder goes off the pod, then into the store at `results/claude/one-model-end-to-end-2026-10-29/training-<sha8>/s<S>/`, with a `SHA256SUMS`, and is read back. Only then is the pod released and `down`.

**The job** is `local/claude/claude-211/job.sh` in this unit's worktree, sha256 `45c384e643ed5da3199752c5e73824751b3af2d8b1dc595282a1b7a2fe6f0ad5`, shipped to each pod as `/workspace/job.sh` and run as `job.sh <head> <seed>`. `job.log` carries row counts and exit codes, never a hit count.

**I-D is built and checked** (`008b40413`, `--controls`, on the laptop in 13 s):
- **(T):** on DEV, it names exactly 1,194 openings and 9,655 later rows, and gets the census's 8,822 of the later rows, as the protocol requires.
- **(S):** it gets 10,251 of DEV's 14,287 rows, above T2's 6,300, so no explanation is owed.

**One change to the protocol's order, made here and named for the FIT.** The protocol asks every instrument to exist before this run's pre-run review. One does not yet: **I-G**, the grader's comparisons and decisions, with I-F's full count over DEV's later rows.
- **Why it can wait:** it only grades stored outputs, on the laptop. Its rules are fixed in the protocol's text, and its strata are fixed above, N-u included. The stored outputs do not depend on it.
- **So it is built while the pods train,** within this unit, and passes its own tests, each with a negative gate, before any decision is read.
- **Its N-u row set** is the one committed above, before any pod started.

## Bar

The protocol's, unchanged: its band and cell (`seed_cell` over three seeds), and its five decisions, in order. This sheet adds no bar.

## Minimum test power

The protocol's band: the floor `⌈2·√(b+c)⌉`, two paired standard errors, and the spread over three seeds. A cell inside it is underpowered, never a negative.

## Stop rule

- **One arm stops alone** if its 256-step probe projects past 1 hour for 16,384 steps. The others go on, and the REPORT names it.
- **A pod stops** if any CUDA test, data check, I-A or the disk check fails on it.
- **The budget is $20 across the pods.** This unit's laptop watcher (`local/claude/claude-211/watch.sh`) adds each pod's hourly price over the time it was up, until it went down. Past $20 it stops and wakes this session.
  - **A healthy run is not cancelled** for a budget it set itself (AGENTS.md). The budget is extended, the extension is noted on #3407, and the watcher restarts.
  - **A run that is not healthy** is sealed and its pods taken down. Not healthy means a pod has stopped, a probe line failed for an arm still training, or training is not progressing.
- **Each pod's EXIT** is 0 only when all 7 arms trained and scored. Otherwise it is 5, with the missing steps in `ARMS_FAILED`, or 9 after a stop.
- **A small miss on a bar** is "keep and iterate", not a stop.

## Cost

- **Training.** claude-194 trained three arms in about 1.7 hours on one RTX 5090, about 34 minutes each. So 7 arms take about 4 hours per pod.
- **Scoring.** At 256 samples per split under three decodes, about 98,000 forward passes per arm. At claude-194's 112–176 tok/s that is about 11 minutes per arm, so about 1.3 hours per pod.
- **The probes:** 11 runs of 256 steps and one short scoring, about 15 minutes on each pod.
- **In all:** about 5.5 hours per pod, 3 pods in parallel.
- **The lease:** 3 pods of 1 × RTX 5090 at $0.56–1.0/h, about $9–16 in all. That is inside the caps (4 pods, $8/h across labs); `uor-pod status` showed 0 running.
- **Keeping the run alive:** watchers end on every terminal state, and a renew loop keeps each lease.

## After the run

- **On the laptop:**
  - I-D's controls;
  - I-G's grading, including I-C's tensor check and I-F's full count;
  - the five decisions, applied as the protocol fixes them.
- **The REPORT** leads with tok/s, gives negatives first, and adds one notebook row.
