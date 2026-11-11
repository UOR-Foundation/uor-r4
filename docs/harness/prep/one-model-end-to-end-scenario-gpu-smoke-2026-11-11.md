# Prep sheet: aligned-corpus GPU smoke (one-model-end-to-end, deepseek-273, 11 Nov 2026)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: deepseek-273, "aligned-corpus GPU smoke". This is a bounded
configuration probe after the aligned scenario readout: the current
FBS/Hamming model has not been trained on the aligned TRAIN.

TEST FITNESS: pending a fresh-context non-author review.

PRE-RUN REVIEW: pending.

## Why

The current arm-r artifact stops on most scenario replies, but its 10
development and
11 Fresh asked-value hits are all in the `copy` archetype. Its memorize,
paraphrase, selection, update, abstention and memory-control columns are zero.
The 16,384-step CPU rank-margin dose moved no scenario reading. The missing
next measurement is therefore a GPU-trained model on the aligned corpus,
served as the integer artifact, not another one-coordinate CPU update or
internal ranking panel.

This smoke does not claim to implement Claude's planned state-keyed read. It
tests whether the current best geometric path already learns useful scenario
behavior when the training text matches the serving questions. If it remains
copy-only, more aligned data and steps are not the next purchase.

## Question

After one short fresh-start training run on the 20,000-row aligned scenario
TRAIN, does the exported integer model produce the asked value on held-out
memorize, paraphrase, selection and update scenarios beyond the copy control,
while retaining the wrong-memory and no-memory abstention controls?

## Mechanism and base artifact

- **Mechanism under test:** the integrated FBS trunk plus four Hamming reads,
  latest-exact copy and the current turn-memory path. No new mechanism is
  added. This is a configuration probe for the existing model path.
- **Recipe:** arm r's geometry and batch semantics plus the explicit
  train-as-served contract and a bounded step count:
  `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0
  --train-as-served 1 --context 192 --width 256 --hamming-read 4
  --hamming-copy-latest 1 --turn-memory 8 --turn-memory-reply 1
  --turn-memory-dropout 500 --turn-memory-leave-copied 1 --batch 16
  --steps 1024 --eval-every 256 --samples 6 --reply-tokens 48`.
- **Source pin:** the pushed head of this ORDER's branch, whose full commit is
  asserted by the pod job and written into the REPORT before launch. The job
  script's SHA-256 is asserted separately.
- **Base comparison:** arm-r artifact
  `a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`,
  schema `/10`, stack-contract semantic hash
  `299a9a2f3297eede06376dab952f0330352531ec20bb56c5cc393f3368dbcf6f`,
  stored for this run at
  `results/deepseek/one-model-end-to-end-scenario-gpu-smoke-2026-11-11/input/arm-r-s2-c8256-bound.uorlut`.
  Its `cpu-scenario-grade` readings were:

| Split | EOS | Exact | Asked value | Copy asked value | Distractor | Abstention | Distinct |
|---|---:|---:|---:|---:|---:|---:|---:|
| development | 980/1,024 | 0 | 10 | 10 | 0 | 0 | 428 |
| Fresh | 957/1,024 | 1 | 11 | 11 | 0 | 0 | 536 |

Arm r itself was not trained with `--train-as-served 1`; this run adds that
contract deliberately so the training forward and the `/10` serving forward
share their rounding. The comparison is therefore a configuration probe, not a
matched causal arm.

The same evaluator's TRAIN previous-token count table produced **0/441**
grounded asked-value hits, **0/220** control abstentions, **0** distractors and
**0/110** copy hits on both development and Fresh. The base is a comparison, not
the model's initialization. The smoke starts fresh so every learned scenario
result comes from the registered TRAIN.

## Data split

The production corpus is sealed for this run at
`results/deepseek/one-model-end-to-end-scenario-gpu-smoke-2026-11-11/input/scenario-corpus-v1/`
and receipt-bound by:

| File | SHA-256 |
|---|---|
| scenario receipt | `4991326286c3ebe2f3a3e975c23e633fe4bb2662e6f6746333535cdff9f68604` |
| tokenizer.json | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| TRAIN tokens | `3412e1b6ccb46db986959aa08b60490d46b3a2a3524fad476cf1fedd838259d8` |
| TRAIN mask | `b0f745fbea8c11f7e9c2993ca03a4eb3ae5ee101b2b0ca0c23e544cac9cb92e0` |
| TRAIN manifest | `d660f4e695363bbf121391841d17b147e8aec30071d5e59de4008b646c57ae57` |
| TRAIN labels | `9799855ae6aaa5dcfdff514ca68bd142b11226be449538035a3576a2902d9c43` |
| development tokens | `612638bba7213f1960cc118a41e6e51ff36074953bb3c7a22f8a9445f1b11777` |
| development mask | `b42518ff047712e7966dbd2619ea314cd0c21dd460b79b1d9cf05491ec54de92` |
| development manifest | `42dfd893ec4defd6f6a53539b5c7c564dc886fe1fe430179716757111447baae` |
| development labels | `2d49834d597aed75bf452b1f4469eea29b7a1ad6f42838da366f26efd7b4a299` |
| Fresh tokens | `574558344f9626d665112183a62ec6d9ce703fd51d6dc5b1ead6328accdc178d` |
| Fresh mask | `8517e0aed1e0f6d695e63cd3d5ab10427911947296213b1b7670265de8df6667` |
| Fresh manifest | `a6b671bb48db8fec737a247e43e2b421d09ad2f9cebd6dbbafb177060b43f735` |
| Fresh labels | `43b6dc81d480270013b467dbafc4281cc17e764b321b341b919ed4625719faae` |
| trainer `ft/dev` tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| trainer `ft/dev` mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |

- **TRAIN:** 20,000 rows, 1,739,599 tokens, 373,321 response tokens.
- **development:** 1,024 rows, 99,767 tokens. Scored only after export; it is
  not the trainer's in-run DEV.
- **Fresh:** 1,024 rows, 99,329 tokens. Used only for the deciding transfer
  read.
- Entity names, scenario ids, fact-value tuples, question-template families and
  compositions are disjoint across splits by the corpus receipt.
- `--data-root` contains the pinned `tokenizer.json` and the trainer-pinned
  `ft/dev`, whose 42,856-token panel supplies the in-run curve;
  `--train-split` points at the scenario TRAIN with its own token and mask
  hashes. The scenario development and Fresh directories are passed only to
  the artifact grader.
- The complete input directory was uploaded, downloaded and checked against
  `SHA256SUMS` at
  `results/deepseek/one-model-end-to-end-scenario-gpu-smoke-2026-11-11/input/`.
  Its portable bundle
  `.../input.tar.gz` was also read back; its SHA-256 is
  `f00df4be476fa399396c9c5d1820d9f18fc897eb0913a5c611a88817c3e1586f`.

## Steps

1. Run the GPU-only Hamming/test subset needed by this recipe and require at
   least one passing test.
2. Run a 64-step smoke at batch 16 and score its checkpoint with an 8-sample
   pass. This is an instrument check; its readings are not used.
3. Train one fresh model for 1,024 steps at batch 16, evaluate every 256 steps,
   and stop on the sheet's failure conditions.
4. Snapshot `models/step-001024`, export `/10` with the new stack contract and
   tokenizer identity, and parse the artifact.
5. Score 1,024 development and 1,024 Fresh cases through
   `cpu-scenario-grade`, each with the previous-token count table.

The exact execution path is:

- after the final input/pin/review commit,
  `uor-pod up --lab deepseek --session deepseek-scenario-gpu-smoke
  --purpose "aligned-corpus geometric smoke" --hours 3 --count 1 --gpu 5090
  --ref CODE_PIN` creates a pod from that exact remote commit;
- `local/deepseek/scenario-gpu-smoke-2026-11-11/launch.sh POD RUN_ID` stages
  the sealed input after verifying the same pod commit, starts the watcher,
  and calls `uor-pod run`; `UOR_SMOKE_PIN` and `UOR_SMOKE_JOB_SHA256` must name
  the independently recorded code and job hashes;
- `.../job.sh` runs training, smoke scoring, final snapshot and bound export;
- the job seals and uploads its run root, downloads it back, verifies
  `SHA256SUMS`, and writes the terminal `COMPLETE` or `FAILED` marker;
- `.../score.sh RUN_ID` downloads the terminal-sealed artifact and runs both
  held-out reports, with the count table, on the D84 CPU-development
  workstation (AMD Ryzen 9 9950X, 16 cores / 32 threads, 60 GiB RAM); it writes
  and store-round-trips `decision.json`.

The CPU scorer is the D84 development instrument. It does not substitute for a
GPU qualification, and this pilot makes no milestone claim.

## Seeds and rows

One seed (`273`) is run because the deciding Fresh split contains 1,024 fresh
cases, above the project's 200-row power floor. The four grounded archetypes
hold 441 cases in aggregate, and each individual archetype holds 110 or 111.
This is a first pilot: a pass can justify a replicated follow-up, while a miss
is provisional and may route the next unit but cannot retire a mechanism.

## Readings

`cpu-scenario-grade` is read at
`--max-cases 1024 --reply-tokens 64 --table-baseline 1`, separately on
development and Fresh. The **Fresh** split decides transfer; development is
reported beside it.

- **P1, grounded transfer:** Fresh `asked_value_present` summed over
  `memorize`, `paraphrase`, `selection`, and `update`. The arm-r and
  previous-token baselines are both 0/441.
- **P2, control safety:** Fresh `abstained` summed over
  `wrong-memory-control` and `no-memory-control`, and Fresh
  `distractor_present` over the exact 882-row non-copy set. The arm-r and
  previous-token baselines are both 0 abstentions and 0 distractors.
- **P3, positive pipeline control:** Fresh `asked_value_present` on `copy`.
  The arm-r baseline is 11/110; the previous-token baseline is 0/110.
- **Reported:** EOS, short-cycle, cap, exact-reference, and distinct-reply
  counts; every archetype; the development rows; the training curve;
  throughput; cost;
  and twenty non-exact grounded replies read token by token.

## Bars and noise

For a zero-hit split of 110 cases, the 95% upper bound is about 3/110; for 220
it is about 3/220, and for 441 about 3/441. The previous-token table measured
zero hits in every scored family. The model's bars sit outside those limits;
the one-seed training variance remains a reason to call a pass provisional
until replicated.

- **P1 pass:** at least **20 of 441** Fresh grounded asked-value hits, and
  **each** of memorize (110), paraphrase (110), selection (110) and update
  (111) has at least **5**.
- **P2 pass:** at least **10 of 220** Fresh wrong/no-memory abstentions,
  and no more than **10 of 882** Fresh non-copy rows with a distractor.
- **P3 positive control:** at least **11 of 110** Fresh copy
  asked-value hits.

The copy control separates a model that learned no usable reply behavior from a
model whose non-copy behavior is specifically absent.

### Preflight controls

The existing arm-r artifact and the TRAIN previous-token table were run through
the exact scoring command before this sheet's review:

- development report
  `a7ec4f34981b9f07906a0e2523964b205df3700aa2afa2e139c0d67a3eacb15c`;
- Fresh report
  `7429e6c4e4fe887d52ac58a0cc750d4fdb06d22ae83b3e6e492a4fb0c635c3f9`;
- sealed at
  `results/deepseek/one-model-end-to-end-scenario-gpu-smoke-2026-11-11/preflight-v3/`
  and read back with both checksums passing.

Arm-r reproduced 10 development and 11 Fresh copy hits. It reproduced zero
grounded hits, zero control abstentions and zero distractors. The previous-token
table produced one distinct reply in every selected case, stopped on EOS in all
1,024, and produced zero asked-value, abstention and distractor hits. Both
split reports selected all 1,024 rows with zero prompt-ineligible rows.

## What each outcome decides

Rows are checked in order; the first match owns the result.

| Row | Reading | Classification | Next |
|---|---|---|---|
| 0 | any job, hash, checkpoint, export, parse or scoring step fails; or P3 copy is under 11/110 | NOT_RUN or underfit instrument; not a mechanism result | keep the sealed failure, repair or rerun once under a new root within the cumulative budget; do not extend the step count |
| 1 | P3 passes, P1 passes, P2 passes | the aligned-corpus pilot shows grounded transfer with controls intact | replicate with a second seed before any wider claim, then compare against a matched no-state-keyed-read arm when it exists |
| 2 | P3 passes, P1 passes, P2 fails | value transfer appears but abstention or distractor control is unsafe | target control separation; do not extend this run |
| 3 | P3 passes, P1 fails, P2 passes | the current FBS/Hamming pilot remains copy-only on aligned data | re-read 20 grounded replies and route the next unit to a state-keyed or other context transform before more aligned steps |
| 4 | P3 passes, P1 fails, P2 fails | the pilot learned copied text only and did not preserve memory controls | perform the reserved 20-reply trace; a confirmed result is provisional pending a second seed |

Row 1 does not establish a frontier model, useful open-domain chat, or the
state-keyed read. It establishes only that this registration changed held-out
scenario behavior through the integer artifact.

**Prediction, written before FIT:** row 3. The copy control should pass because
the scenario includes direct copy prompts; memorize may gain a small number of
hits from latest-exact copying, while paraphrase and selection remain near
zero.

## Controls

- No neural comparator, dense control, transformer control or generated
  teacher.
- The copy archetype is the model-path positive control.
- Wrong-memory and no-memory abstention are safety controls; the distractor
  count is reported beside them.
- The Fresh split decides; development cannot rescue a failed Fresh bar.
- The exact base artifact and previous-token count table are re-scored on the
  sealed rows with the same evaluator before the new run is classified.

## Instrument checks

- **I-p:** clean tracked checkout at the exact pushed head on the pod; the job
  fails unless the commit and job-script SHA-256 equal the launch values.
- **I-h:** every tokenizer, corpus receipt, split manifest, label, token, mask
  and trainer-`ft/dev` hash equals the table above before model work; the
  trainer logs the usable TRAIN window count.
- **I-g:** the current Hamming/recipe GPU tests named by the job pass, or the
  exact unavailable test is recorded and the smoke alone decides whether the
  recipe executes.
- **I-s:** a 64-step smoke with the registered command ends in a checkpoint;
  its 8-sample scoring command runs. Smoke numbers are not read.
- **I-r:** the final run ends exit 0 with a finite loss and
  `models/step-001024`.
- **I-e:** `fbs-snapshot` and bound `/10` `fbs-export --run-config` produce a
  snapshot and an artifact that `cpu-scenario-grade` parses; the artifact
  records stack contract
  `fbs-deepseek-scenario-smoke` and semantic hash
  `cfbc7af2f0b9f4807647b667ccb5d4b3848b140eecc466d3e96165f9b539990c`.
  The artifact, snapshot and checkpoint SHA-256s are recorded.
- **I-c:** `config.json` names objective `cell`, width 256, four Hamming reads,
  context 192, batch 16, no micro-batch, 1,024 steps, seed 273,
  `train_as_served: true`, `hamming_copy_latest: true` and
  `turn_memory_leave_copied: true`; the job asserts every field.
- **I-d:** both scenario reads contain 1,024 case rows with zero
  prompt-ineligible rows and include the previous-token table rows.
- **Seal:** the pod's run folder is sealed with a file list and SHA-256 sums,
  uploaded to the HF store, and downloaded/read back before deletion. A run is
  never written into an existing root (L-002). An exit trap seals failures too.

## Minimum test power

The deciding fresh split is 1,024 cases, above the 200-row floor. The four
grounded families total 441 cases; the copy and control bars use 110 and 220
Fresh cases. A one-seed result is a pilot: a pass requires replication and a
miss cannot retire a mechanism.

## Compute and stop rule

- Status at preflight: **0 running pods**, **$0/h**, no live lease; the Vast
  listing showed 17 verified RTX 5090 offers, cheapest `$0.563/h`. This check is
  repeated immediately before `uor-pod up`; the captured review-time output is
  `local/deepseek/scenario-gpu-smoke-2026-11-11/pod-status-before-review.txt`.
- One Vast RTX 5090 pod at no more than `$1.60/h`, one GPU only, leased for
  three hours and renewed by the watcher while the job moves.
- Expected: about 10 minutes of build/tests, 5 minutes of smoke, 20 to
  40 minutes for 1,024 steps, and 20 minutes for snapshot, export and scoring.
- The default unit ceiling is cumulative across smoke, final run and any rerun:
  three hours and `$5` total. Notices fire at two hours and `$3`; the watcher
  asks the job to seal and stop at three hours, and any extension is set
  explicitly before launch and recorded rather than left to a silent abandoned
  watcher. A stalled job is stopped by process group after 15 minutes with no
  GPU process and no log progress (L-004).
- Stop immediately on non-finite loss, a missing checkpoint or export, an input
  or artifact hash mismatch, or a stalled job with no GPU activity.
- The watcher is armed before the job ships (L-004). Release and destroy the pod
  only after the seal is read back and no GPU process remains.

## Cost

Expected under `$3`; at the offered `$0.563/h`, the card's compute is about
`$0.56/h`. The cumulative unit ceiling is three hours and `$5`, inside the
standing four-pod and `$8/h` caps. A row-0 rerun is allowed only by the same
job on a new run id and only inside the remaining cumulative time and cost.
