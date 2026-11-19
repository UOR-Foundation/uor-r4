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
