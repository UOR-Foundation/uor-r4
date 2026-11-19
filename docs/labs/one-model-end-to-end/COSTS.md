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
