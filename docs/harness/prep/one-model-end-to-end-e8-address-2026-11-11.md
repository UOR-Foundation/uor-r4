# Prep sheet: the E8 address memory's first test
(one-model-end-to-end, claude-346/deepseek-276, 11 Nov 2026)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Mechanism: [the E8 address memory](../../loop/tasks/one-model-end-to-end/claude-346/ORDER.md),
built in draft #3913. This sheet registers a run. It runs nothing.

TEST FITNESS: PENDING

## Question

On T0's 1,000-dialogue memorize set, does the E8 address memory let the geometric
trunk fit and reproduce replies, and does it do so beyond a same-pin no-address
control? Which of the four registered addresses reaches the bars: E8, the
simpler sign cell, a constant row, or a token hash with no state in the address?

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
- **Same-pin no-address control:** the control runs at the reviewed pin at seeds
  1 and 2 with T0's exact command and no `--address-memory`. Every address arm
  is compared with it. T0's sealed counts at `6fcf57f7` are reported beside it
  and are not the control.
- **Fresh starts:** every job starts fresh; no job takes `--init-from`.

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
- **Tokenizer:** T0's pinned `tokenizer.json`, SHA-256
  `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **DEV tokens:** SHA-256
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- **DEV mask:** SHA-256
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **Expected selection:** 1,133 usable TRAIN windows, 40,400 trained targets
  and 40,419 scored rows; 1,072 complete reference replies in the deciding
  decode.

The set is synthetic, controlled dialogue written by `m-world-recall-corpus`.
This is a fitting test, not held-out language capability.

## Arms

| Arm | Cell of one 8D block | Cells | Question |
|---|---|---:|---|
| T0 | no address memory at `6fcf57f7` | none | T0's sealed negative, reported only |
| B | no address memory at this pin | none | Same-pin control |
| e8 | nearest E8 root | 240 | The mechanism |
| sign | the block's 8 signs | 256 | Is E8's root geometry needed? |
| collapse | one cell | 1 | Is a constant learned offset enough? |
| token | seeded hash of input token, layer and block | 240 | Is state in the address needed? |

The four controls and E8 differ in table size (sign 256 cells a block,
collapse 1) and band (e8 0.141, sign 0.104, collapse and token 0, so no wall
gradient). No other field moves. B and all four address arms run at seeds 1 and
2: ten jobs.

## Steps

1. Before anything ships: `uor-pod status`, a lease on each used GPU, and a
   completion watcher armed for each pod. Pods only through
   `scripts/pod/uor-pod`.
2. On an RTX 5090 pod, check out exactly the reviewed remote pin.
3. Run the host tests and the GPU preflight in **Instrument checks** before any
   training.
4. Check the T0, tokenizer and DEV hashes and the 1,133 usable-window count.
5. Run a 32-step smoke at seed 2 of B and the four address arms. Each address
   arm writes `address_memory.safetensors` and `address.jsonl`; B writes
   neither.
6. Run the ten registered jobs. B/s1 and B/s2 run first. Each job's full
   command line, with `--seed`, `--data-root` and `--out`, is in the launch
   record before its pod starts. Jobs use four pods when four slots are free,
   and otherwise run in sequence on available slots; the four-pod cap is never
   exceeded.
7. Each job writes its own `<arm>-s<seed>/runs-1/` root, sealed before another
   job starts on that pod.
8. Score `models/step-004096` with T0's deciding
   `--score-rows ... --panel t0=THE_SET --context 192 --reply-tokens 48
   --samples 1200 --retain-rates 1 --sample-opening last` pass. Scores at
   earlier checkpoints are recorded and never decide a row. T0's second pass
   (`--sample-opening first --samples 128`) is omitted; it is not a bar here.
9. Seal each result, read its `SHA256SUMS` back from the laptop, then release
   and destroy the pod.
10. Read rows in the order fixed under **What each outcome unlocks**, using the
    lower of the two seeds for every bar.

## Readings

The T0 reading definitions are unchanged:

- **R1, exact replies:** exact / 1,072, from
  `samples-t0-greedy-memory.json`. A sample is complete when its reference fits
  the window and ends with the end marker; decoding never sets the count.
- **R2, token fit:** teacher-forced nearest-token hits over all 40,419 scored
  rows, from `rows-t0-memory.json`.
- **R2r, reply fit:** all-hit / 1,072, from the same complete samples and
  teacher-forced rows.
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
bar. Fewer than 800 complete reference replies, or a missing denominator, is an
instrument failure.

T0 ran at `6fcf57f7`. A pass shows that E8 fits at this pin. A gain over no
memory is claimed only against the same-pin B control.

## What each outcome unlocks

Rows are checked in the order 0, 5, 1, 2, 3, 4; the first match owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | a preflight, hash, window count, smoke, checkpoint, score row count, arm identity, address movement, seal or read-back fails; fewer than 800 complete references; any arm-seed still NOT_RUN after `runs-2` | NOT_RUN | repair; one attempt remains under `runs-2` if not already used; otherwise stop with rows 1–4 unread |
| 5 | an arm's two seeds land in different rows below | MIXED | closes nothing; run a third seed |
| 1 | E8 passes all three bars at its lower seed | E8 fits this set; if any control also passes, the E8 geometry is not shown necessary | report every control; then test held-out composition or the count that differs |
| 2 | E8 meets R2 and R2r but misses R1 | a decode-path miss, not a fit miss | report which controls pass; localize the first free-running divergence |
| 3 | E8 misses R2 or R2r, and B or at least one address control passes all three | this E8 configuration is beaten by the named control at this dose | it does not state what form the E8 address should take; name the next configuration |
| 4 | E8, B and every address control miss at least one bar | this 4,096-step table configuration does not make T0 fit | close this configuration only; change capacity or dose before another mechanism run |

A row-4 result is a configuration negative, never an E8-root mechanism
negative. A miss within 0.02 of any bar is recorded as “keep and iterate,” not
as a mechanism result.

## Instrument checks

- **I-p:** clean checkout at the reviewed remote pin.
- **I-g, host:** at the reviewed pin, `CARGO_TARGET_DIR=local/e8-host cargo test
  --release --locked -p uor-r4-float-train --lib` exits 0 with 0 failed. The E8
  subset and the `train-in-space` suite also exit 0 with 0 failed. Counts are
  reported beside the gate, not used as a pass condition.
- **I-g, compile:** the stub-`nvcc` path is not a GPU result; the registered
  type check is `cargo check --offline --locked -p uor-r4-float-train --features cuda
  --profile test --lib --bin train-in-space`, exit 0.
- **I-g, GPU:** run
  `cargo test --release --locked -p uor-r4-float-train --features cuda --lib --
  e8_address` and expect **17 passed** (10 host, 7 tensor). Then run
  `cargo test --release --locked -p uor-r4-float-train --features cuda --lib --
  the_address_memory_steps_as_the_whole_window_does` and expect **1 passed**.
  Any failure is row 0.
- **I-3:** TRAIN's three hashes, the tokenizer, and DEV's two hashes above
  match; `--panel t0=THE_SET` resolves to the exact T0 directory; the trainer
  prints 1,133 usable windows.
- **I-s:** all five smoke jobs complete; each address arm writes one finite
  `address.jsonl` line and its memory file, and the control writes neither.
- **I-r:** one unprofiled smoke step's projection for every arm must put 4,096
  steps inside 2.5 hours. Otherwise the long run does not start.
- **I-t:** each job exits 0 with `models/step-004096`, `config.json`, the arm's
  memory file, and an address line at every eval checkpoint.
- **I-arm:** each job's `config.json` records `address_memory` equal to its arm,
  except B which records none, and records no band override. Cells used are at
  most 240 for e8 and token, at most 256 for sign, and exactly 1 for collapse.
  Otherwise that job is row 0.
- **I-address:** E8 and sign must show nonzero movement after step 512 and a
  finite `address.jsonl` line. Zero movement or a non-finite line is row 0, not
  a mechanism result. Token and collapse may have zero movement; it is recorded.
- **I-score:** every scored run holds 40,419 teacher-forced rows and at least
  800 complete deciding replies.
- **I-o:** B's config and T0's sealed config differ in no field.
- **Seal:** each job's `run/` is sealed with a file list and SHA-256 sums under
  `results/deepseek/one-model-end-to-end-e8-address-2026-11-11/`, read back from
  the laptop before teardown.

## Risks

- Every E8 tensor path remains unrun until I-g; compile success is not a
  numerical result.
- T0's baseline predates this branch; only B's same-pin seeds provide the
  numerical control. T0's counts are reported beside B.
- One band default is used per arm; no band sweep is registered.
- Whether `memory.rate()` is identical across arms is read from the launch
  record; this sheet does not assume it.
- The test can show fitting on its own TRAIN, not generalisation.
- No integer artifact can be produced until the address-memory export section
  exists.

## Minimum test power

Two seeds per arm, every bar read at the lower seed. Each deciding arm has a
denominator of 1,072 complete reference replies and 40,419 teacher-forced rows,
above the project's 200-row floor. Five arms distinguish the mechanism from a
same-pin no-address control and three matched address controls. A result below
800 complete references is underpowered or an instrument fault and is not read.

## Cost

Four Vast RTX 5090 pods, one GPU each, ten jobs distributed across them, at no
more than $0.70/h each. Expected total spend is $8–14. The ceiling is $17:
four pods × 6 hours × $0.70 = $16.80. At the ceiling the run is extended, the
lease renewed and the extension noted. The standing cap is four pods and $8/h
across all labs.

## Stop rule

No job starts until I-g, I-3, I-s, I-r, I-o and the sheet's FIT review pass.
Every GPU is leased and has a completion watcher before its job ships. A failed
job is sealed and marked NOT_RUN; remaining scheduled jobs may continue because
they answer independent arms. A healthy run past a cost or time notice is never
cancelled by the watcher; its lease and budget are extended and the extension is
noted. No rerun beyond `runs-2`, and no GPU remains up after a verified seal.

## TEST FITNESS

> **[claude] Non-author TEST FITNESS review, 11 Nov 2026.** Full review:
> [#issuecomment-6807335261](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6807335261).
> Reviewed: this
> sheet at `3276ba71ff23242e5eb060feaaf8916cb59413c6` on
> `deepseek/e8-audit`, with T0's sheet, T0's REPORT, `deepseek-276/REPORT.md`,
> and the E8 code on the branch. Four fresh Opus reviewers, one per slice,
> each given the sheet and the rules and no plan; their blockers were re-read
> against the sheet's lines before this text was written. Laptop reads only:
> nothing was built, no pod was touched, the branch was not edited.
>
> **Blockers**
>
> 1. **The no-memory baseline crosses code pins.** T0 ran at `6fcf57f7`; this
>    branch changes the trainer by about 1,244 lines. A change could lift R2
>    with no address at all, and collapse does not cover that because it adds a
>    learned offset (L-010). **Repair:** a no-address control at the reviewed
>    pin, seeds 1 and 2, with T0's exact command; every arm is compared with it
>    and T0's counts are reported beside it.
> 2. **A seed split had two answers.** **Repair:** row 5, MIXED, is checked
>    first and closes nothing; its next step is a third seed.
> 3. **Row 3 made a mechanism claim from one configuration. Repair:** row 3
>    says only that this E8 configuration is beaten by the named control at
>    this dose and does not prescribe the E8 form.
> 4. **No rule covered an arm-seed still NOT_RUN after `runs-2`. Repair:** if
>    any exists, rows 1–4 are not read and the test is NOT_RUN.
> 5. **I-address had no outcome row. Repair:** zero movement after step 512 or
>    a non-finite address line for E8 or sign is row 0.
> 6. **Nothing checked each job's arm. Repair:** I-arm compares `config.json`
>    and cell counts with the named arm and refuses a band override.
> 7. **No step leased a GPU or armed a watcher. Repair:** Steps now call
>    `uor-pod status`, lease each GPU, arm a completion watcher and use only
>    `scripts/pod/uor-pod`.
>
> **Majors**
>
> 8. **R1 and R2r lacked denominators and files. Repair:** R1 = exact / 1,072
>    from `samples-t0-greedy-memory.json`; R2r = all-hit / 1,072; complete
>    means the reference fits the window, so decoding cannot shrink it.
> 9. **The deciding checkpoint was not fixed. Repair:** only
>    `models/step-004096` decides; earlier scores are recorded.
> 10. **A passing control was not classified. Repair:** row 1 says E8 fits but
>     its geometry is not shown necessary when a control also passes.
> 11. **Four pods could consume the whole cap. Repair:** when fewer slots are
>     free, pods run in sequence; the cap is never exceeded.
> 12. **40,400 trained targets and 40,419 scored rows were conflated. Repair:**
>     the sheet states both units and their fields.
> 13. **The claim against T0 was not separated from the same-pin control.
>     Repair:** T0 is reported history; a gain over no memory is claimed only
>     against B.
> 14. **The question asked which control fails first without a reading.
>     Repair:** the question now asks which registered address reaches the
>     bars; no order-of-failure claim remains.
>
> **Minors**
>
> 15. Cost arithmetic is corrected to $8–14 expected and a $17 ceiling.
> 16. The full tokenizer hash and both DEV hashes are written.
> 17. Table size and band changes are stated for every control.
> 18. Every job writes its full command line, seed, data root and output root
>     before its pod starts, and starts fresh.
> 19. Each job gets its own `<arm>-s<seed>/runs-1/` root, sealed before reuse
>     of a pod.
> 20. Host counts are reported beside a 0-failed gate, not as the gate.
> 21. T0's second decode pass is explicitly omitted; row 2 reports controls;
>     a miss within 0.02 of a bar is keep-and-iterate.
>
> **Checked and holding:** T0's hashes, counts, steps, seeds and readings; the
> training flags; the four address arms' other fields; the exact 18-test GPU
> preflight; mechanical row coverage when every arm is valid; seal read-back;
> retry roots; and no cancellation of a healthy run. The tensor code has never
> run, no integer export exists and no held-out reading is registered.
>
> **Not verified by the reviewer:** the CUDA build, the host test counts, the
> behaviour of the 1,244 changed lines with the address off, the `--panel`
> resolution, T0's sealed command lines, the market price and per-job hours.
>
> TEST FITNESS: NOT_FIT

### Changes after the first review

All seven blockers and all seven majors are applied above. Minors 15–21 are
also applied. The same-pin no-address control raises the run from eight to ten
jobs. Because those changes alter the registered configuration, the sheet is
back to PENDING for a second non-author review.

TEST FITNESS: PENDING

Pending a second independent non-author review of the same-pin control, MIXED
row, per-arm identity and the revised outcome table. No pod may start at
PENDING.
