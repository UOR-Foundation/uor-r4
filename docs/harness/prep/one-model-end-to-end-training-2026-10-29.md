# Prep sheet: the training run of the model to ship (one-model-end-to-end, claude-211, 29 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-211](../../loop/tasks/one-model-end-to-end/claude-211/ORDER.md). Protocol: [one-model-end-to-end-protocol-2026-10-29](one-model-end-to-end-protocol-2026-10-29.md) (TEST FITNESS: FIT). This sheet is the run that protocol registers; it changes none of its arms, strata, bars or decisions.

TEST FITNESS: PENDING. PRE-RUN: PENDING. (Filled after a fresh non-author review.)

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

## Steps

One pod per seed. Each runs one job, in this order:

1. **Source and tests.** The source is pinned to this unit's head. Then the crate's CUDA tests (`cargo test --release --locked -p uor-r4-float-train --features cuda --lib`), and the build, with its sha256 recorded.
2. **The data,** fetched and checked as above.
3. **The probe ladder,** seed 1's pod only, before its arms:
   - **What runs:** a 256-step run of each of the 7 arms, `--eval-every 256`. Its readings are reported and checked:
   - **Throughput.** An arm projecting past 1 hour for 16,384 steps stops alone.
   - **Finite losses** in every arm.
   - **I-B at P = 1000:** a further 256-step run of (r) with `--turn-memory-dropout 1000`. Its reply weight stays 0.0 (`--inspect`'s `memory_reply_weight_mean_abs`).
   - **I-C under `frozen`:** the step-0 and step-256 checkpoints are kept, and every `.spiralcore_fbs` tensor must be equal between them. The check is made on the laptop, by the grader unit's tensor comparison.
4. **The arms:** each of the 7, for 16,384 steps, one after another.
5. **The scoring of each arm's step-16,384 checkpoint:**

   `train-in-space --score-rows CKPT --out DIR --fresh-v2 FRESH --panel p-dev=PDEV --samples 256 --reply-tokens 64 --retain-rates 1 --seed S --fbs-arm ARM`

   This writes:
   - the rows on DEV, Fresh V2 and P-dev, with the memory on and out;
   - DEV and P-dev samples, greedy with the memory on and out, and Flock rank with it on;
   - their hit counts.
6. **I-H, reproduction.** (r) and (r0) at seeds 1 and 2 are compared with claude-194's sealed step-16,384 DEV readings: 11,513 / 11,361 and 10,079 / 11,171.
   - A miss is an instrument finding: training is not reproducible across builds. It is reported first. It does not void this run's comparisons, which all use one binary.
7. **The seal.** Everything under the job's output folder goes off the pod, then into the store at `results/claude/one-model-end-to-end-2026-10-29/training-<sha8>/s<S>/`, with a `SHA256SUMS`, and is read back. Only then is the pod released and `down`.

**One change to the protocol's order, made here and named for the FIT.** The protocol asks every instrument to exist before this run's pre-run review. Two of them do not yet exist:
- **I-D:** the controls (T) and (S);
- **I-G:** the grader's strata and decisions, and I-F's full count over DEV's later rows.

Both only grade stored outputs, on the laptop. So they are built while the pods train, within this unit, and each passes its own check before any decision is read. No decision is made before they exist, and the stored outputs do not depend on them.

## Bar

The protocol's, unchanged: its band and cell (`seed_cell` over three seeds), and its five decisions, in order. This sheet adds no bar.

## Minimum test power

The protocol's band: the floor `⌈2·√(b+c)⌉`, two paired standard errors, and the spread over three seeds. A cell inside it is underpowered, never a negative.

## Stop rule

- **One arm stops alone** if its 256-step probe projects past 1 hour for 16,384 steps. The others go on, and the REPORT names it.
- **The whole run stops** at $20 across the pods, or if any CUDA test or data check fails on a pod.
- **A small miss on a bar** is "keep and iterate", not a stop.

## Cost

- **Training.** claude-194 trained three arms in about 1.7 hours on one RTX 5090, about 34 minutes each. So 7 arms take about 4 hours per pod.
- **Scoring.** At 256 samples per split under three decodes, about 98,000 forward passes per arm. At claude-194's 112–176 tok/s that is about 11 minutes per arm, so about 1.3 hours per pod.
- **Seed 1's probes:** about 15 minutes.
- **In all:** about 5.5 hours per pod, 3 pods in parallel.
- **The lease:** 3 pods of 1 × RTX 5090 at $0.56–1.0/h, about $9–16 in all. That is inside the caps (4 pods, $8/h across labs); `uor-pod status` showed 0 running.
- **Keeping the run alive:** watchers end on every terminal state, and a renew loop keeps each lease.

## After the run

- **On the laptop:**
  - I-D's controls;
  - I-G's grading, including I-C's tensor check and I-F's full count;
  - the five decisions, applied as the protocol fixes them.
- **The REPORT** leads with tok/s, gives negatives first, and adds one notebook row.
