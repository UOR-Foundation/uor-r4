# Costs, measured (started 18 Nov 2026)

The audit the plan requires: one row per stage, measured at a named shape, so a planned run can be
refused when a known fix would cut it by more than 20 %. Every figure names its conditions; an
estimate is marked as one.

Model for all rows unless stated: **8,192 vocabulary, 512 width, 12 layers, 512 context**, on the
local RTX 4090 (24.5 GiB, **shared with other labs**), batch 24 × 512 = 12,288 tokens/step.

| stage | figure | conditions | status |
|---|---|---|---|
| training step | **0.60 s/step = 20.5 k tok/s** | card free, *before* the route fix (codex-54) | **superseded, re-measure** |
| training step | 1.22 s/step | *collapsed* route grid (one block per output) — the first, wrong attempt | recorded |
| training step | 0.42 s/step = 4.9 k tok/s | batch 4, card contended by 13 GiB from another lab | not comparable |
| evaluation | **60 s / 101,175 rows** | `--dev-tokens 200000` | measured |
| evaluation | 12 min / 2,131,471 rows | full donor dev | measured |
| export | ≈ 2 min, **8,558,848 B** | snapshot + `--legacy-artifact` | measured |
| serving (decode) | 25.5 tok/s | **debug** binary, 8 k table | not a serving claim |
| serving (decode) | 11,026 tok/s | narrow 1.1 M model, release, Apple M1 | other shape |
| corpus tokenize | ≈ 50 min / 385 M tokens | `prepare-chat-jsonl`, 225 shards | measured |
| tokenizer | ≈ 3 min / 407 MB text | `train-bpe`, 8,192 pieces | measured |

## Known costs not yet attacked

| where | size | the fix | expected |
|---|---|---|---|
| projection backward | the route + every sibling walked **per row** | **done** (codex-54): one block per (output, 256-row chunk), route in shared memory | unmeasured |
| writer candidate inventory | 512 rows × 8 cells × 32 slots × 512 lanes = **537 M elements/step** | score the cell centroid, or stream the inventory | unmeasured |
| per-stage quantiser | ~2 M values × 72 projections/step as a separate pass | fuse into the gather | unmeasured |
| served nearest scan | 8,192 × 512 = **4.2 M MAC/token** | shortlist by kept cells (≤ 16 × 32 = 512 rows) | 16–32× |
| evaluation | 2.13 M rows scored for a reading 100 k rows gives | fixed-row subsample (already capped to 101 k) | 21× |

## The rule this file exists to make checkable

A planned run is refused when a fix above would cut its wall clock by more than **20 %**. The 1 B,
0.89-epoch run at the current rate is `1e9 / 20500 ≈ 13.5 h`; at a 2× step it is 6.8 h; at 4×, 3.4 h.
That is the arithmetic the roadmap is ordered by.

## The layer is the cost (ablation, 18 Nov)

Same shape, batch 8 x 512 (1,024 rows a micro-batch chunk), `nearest` + fast-rivals, `--profile 16`:

| model | ms/step |
|---|---|
| 1 layer | **95.7** |
| 12 layers | **792.0** |

So **≈ 63 ms per layer**, and the fixed part (embed, writer, objective) is ~32 ms. A layer is six
projections plus gates over 1,024 rows; a dense 512→512 matmul of that size is ~0.3 ms, so the
geometric layer is **~200x a dense layer** — it is bound by per-element gather, quantise and gate
work at low arithmetic intensity, not by FLOPs.

**Batch amortises it.** Batch 8: 792 ms / 4,096 tokens = **193 us/token**. Batch 24 (card free,
same objective): 0.60 s / 12,288 tokens = **80 us/token** — 2.4x better. The queue run's 1.21 s/step
at batch 24 is the shared card, not the model: another lab's process was resident.

**What that points at.** The layer's cost is per element, so the win is (a) a larger batch while the
writer inventory fits, and (b) fusing the per-element passes — the q16 conversion and the gate —
into the gather, which is one pass instead of three over the same ~2 M values per projection.
The route walk was 3 % (measured, codex-54 revert).

## M2 landed (codex-55)

The writer's candidate inventory is now scored in **column chunks**: it no longer materialises
`(rows, kept_cells x slots, width)` — 537 M elements at 4,096 rows — and the 1 GiB pre-check and the
**8,192-row cap** are gone. The per-column score is independent, so the result is unchanged: at batch
24 the step-64 `train_loss` is **1.00246**, identical to before the change. Batch **48 (24,576 rows)
now runs**, which the old cap refused.

**The timing is not yet a reading**: while the long run holds the card, every measurement is
time-sliced — batch 24 and batch 48 both show 1.81 s/step, which a compute-bound step cannot do. The
batch sweep waits for a free card.

**Tool hygiene, again.** `NativeVectorWriter`'s source hash is part of a checkpoint's saved binding,
so every tool that loads a checkpoint must be rebuilt after any edit to `native_vector_writer.rs`.
M2 changed that file, so `fbs-snapshot` was rebuilt immediately; a checkpoint trained before the
change cannot be exported by tools built after it, and one trained after cannot be read by tools
built before. The artifact itself is unaffected — it carries the shape and the table, not the
writer's source hash.

## Two silent kills, and what the run now does about them (18 Nov, late)

**The run died at step 8,192 (0.22 epochs) with no error.** Not a crash signature and not resource
pressure: no OOM in the kernel log, 42 GB of host RAM free, 40 TB free on the pool, no panic line in
the log, and the last line was a normal step-8,192 report. Both queue scripts were gone with it. The
signature is an external `SIGKILL` of the process group, the same one that killed the earlier attempt.
**90 minutes of training were lost**, and with it every queued stage, because the queue lived in the
same process tree.

**Three defects in the recovery, found by testing rather than by reading:**

1. **`--init-from` was never validated.** It is the only continuation path, and the whole resilience
   design depends on it. Validated directly: a 30-step run reached `top-1 2 / nearest 30 / admitted
   197`, and a 2-step run started `--init-from` that checkpoint began at **exactly those numbers**
   (a fresh init gives `top-1 0 / nearest 6`). It restores weights; per the source it never reads
   optimizer moments or the schedule, so a continued run is a **documented approximation**, not a
   bit-identical resume.
2. **The supervisor's arithmetic was wrong.** `latest()` returns the zero-padded directory name, and
   `008192` is not a valid Python literal, so the computed learning rate came out empty, the CLI
   refused the empty `--lr`, and the script fell through to the queue with the long run never
   started. Arithmetic is now bash `10#` plus `awk`, with no Python in the loop.
3. **Checkpoints were 90 minutes apart.** There is no separate checkpoint flag: checkpoints are
   written at `--eval-every`. At 8,192 that is one checkpoint per 1.5 h, which is why the kill cost
   the whole interval. It is now **2,048**, so a kill costs at most ~22 minutes, at ~16 extra
   evaluations over the run (roughly 16 minutes).

**The supervisor now logs every stage start and exit code** to `/tmp/opencode/survive.log`, continues
from the newest checkpoint at the decayed rate rather than restarting from zero, and carries the
kept-cell arms, the batch sweep and the address A/B in its own process so a kill of one trainer does
not lose the queue. Twenty failed attempts are allowed before it gives up, and it records the last
log line when an attempt writes no checkpoint.

**Cost of this lesson: ~1.5 h of training plus one hour of forensics.** The check it teaches is the
cheap one: a resilience path that has never been executed is not a resilience path.

## The instrument exists: bits per token (19 Nov, codex-57)

The plan's decisive rung, C5, compares against published dense curves, which are in bits per token.
This line had no comparable axis because it has no likelihood. It now has one:

    bits = mean over rows of log2(rank(target) + 1)

computed in the dev eval from the scores it already has, so it costs nothing extra. It is an
**optimistic bound** — a rank code ignores the probability mass between rank 1 and rank 2 — and that
optimism is the point: a bound that is already worse than another model's *true* bits settles the
comparison.

**Instrument check, and it passes exactly.** On the ports checkpoint at step 4,096 the new line
reproduces the recorded one field for field — `top-1 430/101175 nearest 10942 admitted 2309 dev_loss
2.99537` — and adds `bits 7.2223`. `nearest` is derived from the same scores, so agreement is not a
tautology: it is the same computation under two different code paths. 445 lib tests pass.

**The first reading: 7.2223 bits per token** at 42 M tokens seen (4,096 steps x 24 x 512), on the
donor dev. For orientation, a 30 M-parameter dense model on web text sits near **5.3 bits**
(perplexity ~40), and GPT-2 small near 4.9. So the *optimistic* bound is ~2 bits worse than published
models of comparable size on a different, harder corpus.

**What that does and does not settle.** It does not settle C5, because the corpora differ and bits per
token is corpus-dependent. It does say that until a **same-corpus count baseline** is measured on the
**same rows**, we cannot distinguish "2 bits behind because the transport is weak" from "2 bits behind
because this dev is harder per token". That baseline is the next unit, and it must be computed inside
the eval so the row set is bit-identical rather than approximately equal.

## The queue landed (19 Nov): C1, the readout, the slope, and the batch curve

**C1, the 0.89-epoch run with the centroid address** (`runs/gen-long-cent`, 32,768 steps, 19,184 s):

| step | epoch | nearest | pick (top-1) | admitted | bits |
|---|---|---|---|---|---|
| 2,048 | 0.06 | 9,266 | 7,793 | 12,249 | 7.9712 |
| 8,192 | 0.22 | 12,422 | 8,733 | 12,381 | 7.3563 |
| 16,384 | 0.45 | 14,029 | 10,433 | 14,330 | 7.2384 |
| 32,768 | 0.89 | **15,888** | **12,018** | **15,994** | **6.9468** |

- **C1 fails its floor: `nearest` 15,888/101,175 = 15.70 %** against T2's **15.99 %** — 0.29 points
  below a bigram count table after 0.89 epochs (402 M tokens seen).
- **The readout is no longer the limiter**: pick 11.88 % against nearest 15.70 %, and admission
  15.81 % is *above* nearest. Before the address fix the same trunk picked at 0.4 %. So ~76 % of the
  trunk's information now reaches the served token, and the count of lost answers fell from 19 in 20
  to 1 in 4.
- **Bits are still falling**: 7.971 → 6.947 over 16x tokens, **~0.26 bits per doubling**, with no
  plateau. (Published dense curves fall ~0.18 bits per doubling, so the slope is not the weak part;
  the *level* is, and the level needs a same-corpus baseline before it can be judged.)
- **Batch curve** (256 steps each): 24 → **51.3 µs/token**, 32 → 48.6, 48 → 42.6, **64 → 40.8**.
  A **1.26×** win at batch 64, not the 2.4x that batch 8 → 24 gave, because the fixed cost was
  already amortised at 24. Adopt 64.
- **The kept-cells A/B is void**: `kc-8` and `kc-16` produced *identical* `admitted` (13,215 and
  12,859 at 2,048/4,096 with matching step-0) although `kc-16`'s config records `kept_cells: 16`.
  Either admission is evaluated with a fixed 8 cells or the flag does not reach the eval's writer.
  It no longer matters for capability — admission (15.81 %) already exceeds nearest (15.70 %) — so
  the A/B is retired rather than diagnosed, and the anomaly is recorded so nobody cites those arms.
- **The feedback arm is confounded**, as its own script made it: it trained at batch 48 for 4,096
  steps (100 M tokens, lr 0.001 decaying over 4,096) while the comparison ran batch 24. Its
  `nearest` 15,032 is not evidence that the centroid address hurt training. The clean readout
  comparison remains the evaluation A/B, where the trunk was held fixed.

## What the last number means for the decision

The model is **at bigram parity and 1.02 bits above its own start**, with a slope that has not
flattened and a readout that no longer throws the signal away. Against published dense models of the
same size (~5.3 bits on web text) the optimistic bound of 6.95 is ~1.6 bits behind — but that
comparison crosses corpora and cannot decide anything on its own. **The missing measurement is a
count baseline in bits on the same rows**, and then whether the gap to it widens or narrows with
tokens. Both are cheap and neither needs a new training run.

## The baseline on identical rows (19 Nov): the model is 2.6 bits behind a bigram

The count baseline is now computed on the eval's **own rows** — `UOR4_DUMP_ROWS` writes each row's
(conditioning token, target, the model's rank of that target, copy flag), and the counts come from
the same 385,284,895-token TRAIN the model trained on. 101,175 rows, no row-set mismatch:

| ranker | bits (mean log2(rank+1)) | top-1 | implied mean rank |
|---|---|---|---|
| unigram | 7.3181 | 6.20 % | ~158 |
| **bigram** (tie-break: unigram) | **4.3128** | **16.47 %** | ~20 |
| model, 0.89 ep | **6.9468** | 15.70 % | ~123 |

**The model is 2.63 bits behind a bigram count table**, and 6.9468 is its *optimistic* bound. Both are
rank codes on the same rows, so the comparison is exact. **Top-1 hid this**: at 15.70 % against
16.47 % the model looks like parity, while in information terms it ranks the target ~6x further down
(~123 against ~20).

**It is not a memorisation effect.** Split by whether the row copies its prompt:

| subset | rows | model bits | bigram bits | gap |
|---|---|---|---|---|
| copies the prompt | 5.2 % | 7.374 | 4.227 | **+3.15** |
| does not copy | 94.8 % | 6.923 | 4.318 | **+2.61** |

The model is **2.6 bits behind on the 95 % of rows where copying cannot help**, and it ranks the target
above the bigram on only **18.7 %** of rows. That is an information-extraction deficit, not a
table-memorisation one.

**And scale cannot close it at the measured rate.** The gap narrows from 3.658 bits at 25.2 M tokens
(step 2,048) to 2.634 at 402.7 M (step 32,768) — **0.256 bits per doubling**, so closing the remaining
2.634 needs ~10.3 doublings, about **1,250x more tokens (~500 B)**. At the measured 40.8 µs/token
(batch 64) that is **~236 days of this card**, and 1,300 epochs over a 385 M-token corpus, which would
memorise rather than generalise. **Scale is refused by arithmetic, not by preference.**

## Where the deficit points

The token hinge's margin defaults to **1.0** while the score is `-mean|q-v|` over 512 lanes; the code's
own comment says a margin of 1.0 "can never saturate". The observed dev token hinge is **0.916**, i.e.
the margin is violated on nearly every row and its gradient is a constant push rather than a ranking
signal. A hinge that cannot distinguish "close rival" from "far rival" trains a model to put the target
in the top ~120 and no better — which is exactly the rank distribution measured.

That makes the margin the leading suspect, and it is a one-flag change (`--token-margin`). Running now:
two arms at 4,096 steps with reachable margins (0.10 and 0.25), against the margin-1.0 reference of
**bits 7.2223 at the same 50 M tokens**. Gate, fixed before the numbers: a reachable margin must reach
**below 7.02 bits** (0.2 better) at equal tokens, or the margin is not the deficit and the next suspect
is the transport.

## The read stack's cost curve, measured on a quiet card (20 Nov)

A cost measurement taken on this laptop's card said the conditioning reads were 6-10x (4.8 s/step
against 0.585). **It was contention**: two other labs' jobs held 10.7 GiB at 100 % utilisation, and
the same four configs on a dedicated Vast 5090 give a monotone, modest curve:

| configuration | ms/step | 16,384 steps |
|---|---|---|
| no reads | **459** | 2.1 h |
| opening 4 + register 1 | 589 | 2.7 h |
| opening 8 + register 4 | **843** | 3.8 h |
| opening 16 + register 16 (+ picks 4) | 1,259 | 5.7 h |

So the whole read stack at its maximum is **2.7x** the no-read step, and the per-slot cost is roughly
linear. The stage split at the top of the range puts 67 % in the backward, 5 % in the register's
forward — the reads are not the forward cost, the gradient through them is.

**What that changes:** the conditioning experiment does not need the maximum, and it does not need
16,384 steps. The comparison point is **8,192**, where two references already exist — 45 openings
with no reads, 78 with opening 4 + register 1. The middle config at that point costs 1.9 h instead of
5.7 h, and still answers the question (does more slot capacity help).

**The rule, now standing:** every cost reading is taken on a **quiet** card, and `nvidia-smi
--query-compute-apps` is checked before the run, not after the surprise. A timing taken while another
lab's job is resident is not a measurement, and optimising against it is worse than not measuring.
