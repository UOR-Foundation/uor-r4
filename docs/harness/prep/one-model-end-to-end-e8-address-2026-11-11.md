# Prep sheet: the E8 address memory's first test
(one-model-end-to-end, claude-346/deepseek-276, 11 Nov 2026)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Mechanism: [the E8 address memory](../../loop/tasks/one-model-end-to-end/claude-346/ORDER.md),
built in draft #3913. This sheet registers a run. It runs nothing.

TEST FITNESS: PENDING

## Question

On T0's 1,000-dialogue memorize set, does the E8 address memory let the geometric
trunk fit and reproduce replies that the same recipe without it did not fit?
Which control fails first if it does not: the E8 cell, the simpler sign cell, a
constant row, or a token hash with no state in the address?

## Why

T0 recorded this recipe at 4,096 steps and two seeds at row 3: 0 of 1,072
complete replies exact, token fit 0.602/0.607, whole replies fit 0/1,072. Its
sheet sent the next test to the address memory, the first of the aligned
design's two missing learned parts. This test changes one address mechanism at
a time and compares the result with T0's sealed negative.

## Base artifact

- **Code:** this unit's reviewed merge of `deepseek/e8-audit` with `main`. The
  concrete pin is written into the launch record only after that commit is on
  the remote; a local-only commit is not a pod pin. The current audit tree is
  `d792c77f2` plus this sheet.
- **Recipe:** T0's b64L recipe, word for word:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0
  --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64
  --micro-batch 16 --steps 4096 --eval-every 512`.
- **Address switch:** each arm adds only `--address-memory ARM`, where `ARM` is
  one of `e8`, `sign`, `collapse` or `token`. The band is that arm's registered
  default; no `--address-band` override is permitted in this test.
- **No-memory baseline:** T0's sealed run at `6fcf57f7`, read from the store.
  It is compared with, not rerun. Its counts remain the baseline for both
  seeds.

## Data split

TRAIN and the scored panel are T0's same set, on purpose:

- **Train/panel directory:**
  `results/claude/one-model-end-to-end-memorize-2026-11-09/t0-set/train/`.
- **Tokens:** 122,037 ids, SHA-256
  `c1c779611d4f6c571579fca39778880020c5f1e4df699f1fcd5c32671c5a6f90`.
- **Mask:** SHA-256
  `3658e61246a124db19d95b7a254bbea00bdad25924920aeb613049856fe4cea9`.
- **Manifest:** SHA-256
  `1000883dc160c5eff77d9991ee3aeb5caacf7e0782827be834cbecaaaeae67bd`.
- **Tokenizer:** T0's pinned `tokenizer.json`, SHA-256 `d36d3e87…0f89`.
- **Expected selection:** 1,133 usable TRAIN windows and 40,400 scored targets;
  1,072 complete replies in the deciding decode.

The set is synthetic, controlled dialogue written by `m-world-recall-corpus`.
This is a fitting test, not held-out language capability.

## Arms

| Arm | Cell of one 8D block | Cells | Question |
|---|---|---:|---|
| T0 | no address memory | none | T0's sealed negative |
| e8 | nearest E8 root | 240 | The mechanism |
| sign | the block's 8 signs | 256 | Is E8's root geometry needed? |
| collapse | one cell | 1 | Is a constant learned offset enough? |
| token | seeded hash of input token, layer and block | 240 | Is state in the address needed? |

All non-T0 arms run at seeds 1 and 2. Eight jobs, identical except for seed and
the one address switch.

## Steps

1. On an RTX 5090 pod, check out exactly the reviewed remote pin.
2. Run the host tests and the GPU preflight in **Instrument checks** before any
   training.
3. Check the T0 hashes and the 1,133 usable-window count.
4. Run a 32-step smoke at seed 2 of the four arms and the no-address control.
   Each address arm must write `address_memory.safetensors` and
   `address.jsonl`; the control must write neither.
5. Run the eight registered jobs: four pods, two jobs in sequence per pod.
   Jobs are paired as `e8/s1 + sign/s2`, `sign/s1 + collapse/s2`,
   `collapse/s1 + token/s2`, and `token/s1 + e8/s2`. Assignment fixes only the
   schedule, never the command.
6. On each completed checkpoint, run T0's deciding
   `--score-rows ... --panel t0=THE_SET --context 192 --reply-tokens 48
   --samples 1200 --retain-rates 1 --sample-opening last` pass.
7. Seal each run, read its `SHA256SUMS` back on the laptop, then release and
   destroy each pod.
8. Read rows from the top of **What each outcome unlocks**, using the lower of
   the two seeds for every bar.

## Readings

The T0 reading definitions are unchanged:

- **R1, exact replies:** complete greedily decoded last-opening replies,
  compared with the reference through its end marker.
- **R2, token fit:** teacher-forced nearest-token hits over all 40,419 scored
  rows.
- **R2r, reply fit:** complete samples whose every reference token is a
  teacher-forced hit.
- **Address census:** `address.jsonl` at each checkpoint: cells used, largest
  cell share, movement from the prior checkpoint, share inside the band, and
  mean absolute row value.
- **Recorded beside the bars:** training loss, DEV row, first exact-reply miss,
  stop-marker count, and per-arm wall time.

No integer export, serving score or open-text claim is part of this test. The
address memory has no integer runtime section yet.

## Bar

The E8 arm passes when **all three** hold at the lower seed:

- **R1 >= 0.90**
- **R2 >= 0.95**
- **R2r >= 0.90**

The control arms are classified by the same bars but do not replace the E8
bar. Fewer than 800 complete replies at a seed is an instrument failure.

## What each outcome unlocks

Rows are checked from the top; the first match owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | a preflight, hash, window count, smoke, checkpoint, score row count, seal or read-back fails; fewer than 800 complete replies | NOT_RUN | repair, then one rerun under `runs-2` |
| 1 | E8 passes all three bars at its lower seed | the E8 address memory fits this set | report the control bars; then test held-out composition and start the state-keyed read |
| 2 | E8 meets R2 and R2r but misses R1 | a decode-path miss, not a fit miss | localize the first free-running divergence before judging E8 |
| 3 | E8 misses R2 or R2r, and at least one control passes all three | E8's state-root address is not the useful form | name the passing control and replace this E8 configuration with its next unit |
| 4 | E8 and every control miss at least one bar | this 4,096-step table configuration does not make T0 fit | close this configuration only; the next unit changes table capacity or training dose before another mechanism run |

An outcome split between seeds is reported as mixed and cannot close a
configuration. A row-4 result is a configuration negative, never an E8-root
mechanism negative.

## Instrument checks

- **I-p:** clean checkout at the reviewed remote pin.
- **I-g, host:** after merging current `main`, `CARGO_TARGET_DIR=local/e8-host
  cargo test --release --locked -p uor-r4-float-train --lib` gives **388
  passed**. The E8 subset is **10 passed**, and the `train-in-space` suite is
  **12 passed**.
- **I-g, compile:** the stub-`nvcc` path is not a GPU result; the registered
  type check is `cargo check --offline --locked -p uor-r4-float-train --features cuda
  --profile test --lib --bin train-in-space`, exit 0.
- **I-g, GPU:** run
  `cargo test --release --locked -p uor-r4-float-train --features cuda --lib --
  e8_address` and expect **17 passed** (10 host, 7 tensor). Then run
  `cargo test --release --locked -p uor-r4-float-train --features cuda --lib --
  the_address_memory_steps_as_the_whole_window_does` and expect **1 passed**.
  Any failure is row 0.
- **I-3:** every corpus and tokenizer hash above matches; the trainer prints
  1,133 usable windows.
- **I-s:** all five smoke jobs complete; each address arm writes one finite
  `address.jsonl` line and its memory file, and the control writes neither.
- **I-r:** one unprofiled smoke step's projection for every arm must put 4,096
  steps inside 2.5 hours. Otherwise the long run does not start.
- **I-t:** each job exits 0 with `models/step-004096`, `config.json`, the arm's
  memory file, and an address line at every eval checkpoint.
- **I-address:** E8 and sign show nonzero movement after step 512; token and
  collapse may have zero movement, which is recorded rather than failure.
- **I-score:** every scored run holds 40,419 teacher-forced rows and at least
  800 complete deciding replies.
- **I-o:** the T0 config and this branch's no-address config differ in no field.
- **Seal:** each job's `run/` is sealed with a file list and SHA-256 sums under
  `results/deepseek/one-model-end-to-end-e8-address-2026-11-11/`, read back from
  the laptop before teardown.

## Risks

- Every E8 tensor path remains unrun until I-g; compile success is not a
  numerical result.
- T0's baseline predates this branch. I-o can show config equality but not
  repeated training bytes without a baseline rerun.
- One band default is used per arm; no band sweep is registered.
- The test can show fitting on its own TRAIN, not generalisation.
- No integer artifact can be produced until the address-memory export section
  exists.

## Minimum test power

Two seeds per arm, every bar read at the lower seed. Each deciding arm has
1,072 complete replies and 40,419 teacher-forced rows, above the project's
200-row floor. Four address arms distinguish the mechanism from three matched
address controls. A result below 800 complete replies is underpowered or an
instrument fault and is not read.

## Cost

Four Vast RTX 5090 pods, one GPU each, two jobs per pod, at no more than
$0.70/h each. Expected wall time is 3 to 5 hours per pod and total spend
$6 to $12. The ceiling is $14 and 6 hours per pod; the standing cap is four
pods and $8/h across all labs.

## Stop rule

No job starts until I-g, I-3, I-s, I-r, I-o and the sheet's FIT review pass. A
failed job is sealed and marked NOT_RUN; remaining scheduled jobs may continue
because they answer independent arms. A healthy run past a cost or time notice
is never cancelled by the watcher. No rerun beyond `runs-2`, and no GPU remains
up after a verified seal.

## TEST FITNESS

TEST FITNESS: PENDING

Pending an independent non-author review of the T0 comparison, the 18-test GPU
preflight and the arm/outcome table. No pod may start at PENDING.
