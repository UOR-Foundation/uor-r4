# D8 bounded admission: one integrated continuation

Prospective September 25, 2026, before any new model execution. Base
`196f77d5b9ab75ec3b74ea4eb75811831b2eee9d` (PR #1393). This implements the next
dependency in the [canonical ladder](project-track.md); it does not replace the
transformerless geometric goal, D0-b or terminal D5 parameter sparsity.

## Question and fixed mechanism

Can both retained learned-code recurrent models preserve their measured behavior
when the same bounded admission rule is present in learning and incremental
generation? Does the indexed complement have measurable utility beyond recent32?
This is an access/retention question, not a geometric-advantage or language claim.

`joint_admission.rs` maintains exact occurrence IDs separately from addresses.
The fixed `orthant64` proposer unions recent32 with two sign tables over read
coordinates 0..4 and 4..8. Each table has 16 buckets, depth8 FIFO, and two probes:
the query's bucket and its least-absolute-coordinate neighbor, ties low index.
At most32 posting entries plus32 recent entries enter duplicate removal; at most64
unique earlier events are ranked by the retained full64-dimensional learned
energy, true occurrence age and NoRead. Writes follow reads. Missing an index
posting never proves global evidence absence. These read coordinates are dense
learned projections, not designated R4 state lanes or an H4 codebook.

The proposer is stopped-gradient. Conditional language gradients reach selected
queries, keys, values, recurrent state and output. There is no gradient estimator
for moving a missing event across a bucket boundary. Indirect shared-parameter
changes can alter future membership; success cannot be described as learned
optimal admission. Untied query/key maps, magnitude-sensitive dot products,
partial coordinates and bucket saturation are concrete recall risks.

The shared recurrent core uses selected keys/values and exact source tokens.
The bounded incremental path stores individual event tensors, without full-history
concatenation or a full copy scan. Offline batch learning/evaluation may retain
contiguous differentiable histories and expand masses for the existing evaluator.
That offline layout is not an inference cost measurement. Dense parameter access,
floating arithmetic, allocations and the model's 256-token context ceiling remain.

Controls use the same loaded weights: full context; recent64; recent32 (remove
the indexed complement); exact observed-token follower cache plus recent32, at
the same maximum64 scored candidates; and NoRead. Exact cache files each observed
event under its preceding observed token. It sees no target labels. Index storage
and actual query/insert work are reported; equal candidate caps do not imply equal
memory or arithmetic. All control structures are currently maintained per lane.

## Fixed learning and evaluation

Enter from both `fit-*-rounding-3/packed-model` artifacts in
`/Users/casey.allard/uor-r4-investigations/learned-rounding-20260925`.
Preserve both parents and all prior negatives. First verify fresh full-context
replay and measure the frozen orthant entry with the unchanged evaluator.

Train one bounded continuation per transport from actual packed code values:
**256 updates, B16/T256, two CPU sequence shards, data seed240924, next sampler
counter8860**. Use a new, explicitly recorded AdamW optimizer: learning rate0.0001,
weight decay0, existing beta/epsilon/clip defaults. Frozen dyadic grids and
full-strength STE remain; clamp shadows to their representable ranges after each
update. This is a new whole-model bounded-mask objective, not another alpha,
scale or rounding schedule search. Original alpha/optimizer parents remain intact.
The old quantizer clock and new optimizer/data clocks are bound separately.

Evaluate final actual packed exports after reload on all249,856 existing
development targets (233,472 comparison-tail targets), the same five generated
continuations and32 source responses. No source-panel answers enter training.
Run the fixed access controls on final weights; do not select a checkpoint or
threshold on their outcomes. One paired seed and exposed development stay explicit.
Full-context references get no matching new training, so this is a retention
comparison, not an equal-additional-compute superiority result.

### Frozen decisions

Each transport's **bounded retention** requires all of:

1. Comparison NLL no more than **+0.05** above its own retained learned-code parent
   (2.1142261687 /2.1108797998), and below count/cache2.3917861789.
2. Whole-prefix NoRead penalty at least half the parent's measured penalty:
   **0.2476913317 /0.2485550743**. This guards against retaining loss by suppressing
   the context pathway.
3. Five actual continuations nonconstant and without short token-cycle collapse.
   This does not qualify semantic coherence.
4. At most two previously correct first-noun responses lost against the immediate
   parent, and at most two against rung1. Report every complete-answer loss/gain
   separately against the intermediate parents; gains never cancel losses.
5. Packed reload probability delta exactly zero, seeded token/distribution parity,
   finite normalized outputs, exact causal occurrence identity, selected gradients,
   and batch/incremental probability agreement within2e-6 on the focused CPU check.
   Actual selected counts never exceed64 and query traversal is independent of
   history length for the fixed index parameters.

**Indexed complement utility** is separate: versus recent32 on the same final
weights, at least2/32 additional complete source answers or at least0.02 nats
improvement on comparison positions64..255, while remaining within+0.05 of
recent64. Report exact-cache results and actual older-event accesses. This does
not establish an advantage over all ordinary or random-projection indexes.

If retention passes, keep that candidate for quantized transport work. If only
the indexed-utility discriminator fails, preserve bounded behavior but do not call
the index useful. If retention fails, retain the negative and identify the actual
read/source failures; the parent remains accepted. No automatic hash/width/scale
sweep follows. A resource stop is a checkpoint, not a model-quality decision.
Quantized transport and integer execution remain subsequent dependencies.

## Independent review and primary sources

RDC ran concurrent DeepSeek source/mathematics and Rust index implementation work;
the principal implemented the shared graph, continuation and evaluation. Both
received the live source, history, complete question, limits and artifact paths.
Their raw handoffs remain in the local investigation root; no Kimi model was used.

Accepted objections: hard membership lacks direct credit; saturated postings can
hide valuable old events; partial sign agreement is not maximum-inner-product
recall; a natural-likelihood pass does not establish useful indexing or language.
Principal corrections: an individual missed event has no immediate selected-path
gradient, but indirect shared learning can move its future code; permanent
unlearnability was not proved. Recent32 need not equal NoRead because state itself
can carry information. Dense read coordinates are not R4 lanes. The older
611,814-access count belongs to another native artifact and cannot be assigned to
this joint learner. A-series local-chooser failures motivate care but do not
determine this full recurrent graph's outcome. The reviewer's extra random-index,
multi-seed and full-mask diagnostic list is not adopted as a gate for this narrow
retention step; no geometry-superiority claim will be made.

[Reformer](https://arxiv.org/html/2001.04451v2) and its
[primary implementation](https://github.com/google/trax/blob/master/trax/layers/research/efficient_attention.py)
motivate indexed similarity access, but use shared query/key projections and
sorting, unlike this untied bounded FIFO index.
[Routing Transformer](https://aclanthology.org/2021.tacl-1.4/) learns content
clusters with a different complexity and training mechanism. Neither supplies
our recall guarantee, a Hamiltonian, constant whole-model cost, or a serving
backbone. Their transformer models remain research references.

## Resources and delivery

The [prospective budget](../evidence/bounded-admission-budget-2026-09-25.json)
starts at17:08:39UTC and permits120 minutes for the complete cycle, including
preparation, both experts, build/failures, training, comparisons and protected
delivery. The standing extension was recorded before work exceeded the existing
allowance: cumulative use535,193,179ms; limit540,000,000→547,200,000ms.
The initial fit projection is35 minutes; dose/time changes require a prospective
reason and preserve all charges. Two model processes, two sequence workers each,
nested backend threads1, one Cargo process. Aggregate model RSS≤8GiB, individual
≤4.5GiB, incremental allocated storage including the worktree/build≤2.5GiB;
15GiB physical reserve plus128MiB stop and64MiB closeout margins.

Use existing shared build output. Preserve unique source/artifacts/worktrees,
checkpoint at guards, and charge unique chronological wall once. No external
training hardware, destructive cleanup, or physical-energy claim. Deliver source,
actual result and changed current-state/roadmap claims through a protected PR;
verify its actual merge and tree. #973/#820 remain open until full acceptance.
