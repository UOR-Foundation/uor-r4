# Persistent integer geometric attention — source implementation

Status: source implemented and statically reviewed; scoped compilation, loaded
comparison, allocation and compiled-instruction qualification are **NOT_RUN**.
This record does not promote the session, its mathematical adjoints, or the
surrounding language model. The active source card is
[#1512 comment5958637499](https://github.com/UOR-Foundation/uor-r4/issues/1512#issuecomment-5958637499).

## Implemented source and reason

The existing native composition caller recomputes event/context/span histories,
every occurrence payload, and every causal query row whenever a prefix is
extended. The new `integer::geometric_attention` session maintains those states
and caches exact occurrence addresses and composed payloads. A push updates the
current occurrence once and scores/reduces only the newest query row.

Initial scope is the retained H2/L4/width32, context at most128, rra/layer2 reader.
Each real token is a distinct occurrence. It remains in the denominator even
when its address is absent, its value is present-zero, or nonzero atoms cancel.
There is no token deduplication, support pruning, coefficient clipping, eviction,
new selector, new artifact format or optimizer dose.

The push stages independent event/context states, observes the OLD held span,
uses NEW context state/observation for the current value and NoRead, caches its
actual packets and signed-bank composition, includes self in causal support,
normalizes each head separately with its NoRead, and checks the integer head
sum. Only then are span/state/history/publication committed. Span COMMIT at t
is visible at t+1. Capacity failure rejects before mutation; private unpublished
scratch may be overwritten by a failed reduction but cannot extend public
history or replace the last successful output. Reset clears all logical state.

The training adapter joins the complete dependency metadata of the admitted
context/event/span/potential/value/NoRead/reducer/composition objects, their token
registry, exact signed H4 algebra and Q25 observation basis. Its immutable kernel
accessors introduce no per-token parsing, hashing, source-model call or Tensor.
Strict q4 context, potential, value and NoRead artifacts are required. Event
source coefficients and learned age biases retain their wider policies and are
explicit in the admission receipt.

The CPU Stack bridge runs one session per batch window, converts the final i64
Q16 head sum outside the integer session, and inserts it at the same residual
site that already bypasses `read.out`. Floating recurrence, MLP, residual stream,
normalization and vocabulary projection remain. It does not edit
`stack_grounded_session.rs` or replace Claude's Metal training path.

## Prepared decisive checks

The new comparison example loads the two unchanged, sealed strict-context
lineages from the completed [context construction](geometric-context-q4-2026-10-02.md).
It binds the exact parent report, sealed argument bytes, loaded context
metadata and retained episode hashes; no artifact is compiled or rebound during
evaluation. It compares each token's event/context
state/action, raw observations, old-held content, value packet/raw choices,
composed payload, every causal score/age, occurrence/NoRead weights, denominator,
maximum score, head output and checked sum with the native whole-window path.
It retains full reader traces and checks actual-position Stack logit bits plus
the prior saved answer logits/predictions on all original/stress rows.

A valid preceding value artifact with a different context lineage must be
refused. Capacity/invalid-token rejection and reset are checked on each loaded
episode. Separate same-shape Stack comparisons cover a short prefix and B2
with different retained sequences. Synthetic integer cases cover delayed span
visibility, repeated occurrences, absent/present-zero/cancellation, independent
head denominators and a late second-head error. The capacity fixture also
pushes all128 occurrences, checks every support length and independent weight
sum, and rejects the129th without publishing a new result. These tests are
authored and parsed, not executed yet.

The storage receipt counts actual inline layout and constructor-owned heap
elements, including padded address storage, payload cache, span buffers and both
private exponential tables. It excludes borrowed immutable kernels, allocator
overhead and temporary stack frames. It is not peak RSS or an energy measure.
Compiled successful-push/callee allocation and instruction inspection remain
necessary before claiming no allocation, float, multiplier or divide opcode.

A follow-up caller review found six complete legacy `ReadSource` initializers
that omitted the new optional residual field. All six now explicitly initialize
it to `None`; the other four literals use the default or set the injected
residual. This repairs a source-level compilation blocker across ordinary,
span, binding and composed callers. Direct parsing/formatting and diff checks
pass; actual compilation and behavioral checks remain NOT_RUN. No numerical
operator, learned artifact or learning policy changes in this repair.

## Execution admission and continuation

The source projection is240min total: source/review120, cold rebuild/checks45,
unchanged no-update comparisons30, delivery45. Each admitted build/worker uses
2threads/3GiB; one Cargo at a time. Loaded workers retain840-second internal and
900-second external bounds; research/archive each at most256MiB, regenerated
private build cache at most1.5GiB, physical stop margin128MiB. All preparation,
reviews, failures, waits and delivery remain cumulative ledger charges.

At the initial source checkpoint, actual peer workers declare6+4threads against the
shared ceiling8. No own Cargo/model worker has started. The live coordination
observation is on #820; the source is preserved in a draft PR pending scoped
execution. This is a capacity constraint, not attention-quality evidence.

After exact orchestration is exercised, event source width and age policy remain
the next reader coefficient decisions. Selected parameter/candidate access,
float trunk/output replacement, natural-input durable conversation and novel
coding/reasoning remain programme obligations. Offline donor/operator compilation
is retained. Neither this source implementation nor context's retained45 losses
establishes a reason to retire geometric attention.

## Learning policy: primary-literature applicability review

The independent mathematics review does not justify changing context learning
while this session is implemented. The native-conditioned forward chooses exact
actions deterministically. [Rao-Blackwellized straight-through Gumbel-Softmax](https://arxiv.org/abs/2010.04838)
reduces conditional sampling variance under a categorical/Gumbel sampling law;
that law is absent here. The [2023 counteranalysis](https://proceedings.mlr.press/v202/shekhovtsov23a.html)
also does not establish stability for this deterministic recurrent surrogate.
The retained large-gradient draws differ in input content and history; they are
not a controlled context-length or estimator-variance experiment.

Tangent projection is already the historical context adjoint. It can erase
useful finite-choice credit: at the positive identity, a radial gradient of
`softplus(alpha * x0)` projects to zero although the negative identity lowers
the loss for positive alpha. Reducing a norm by restoring that projection is
therefore not evidence of improved learning. The current finite-choice backward
already enumerates all120 local actions through a linearized downstream
adjoint; simply enumerating exact roots again does not supply actual nonlinear
suffix losses. This is a mathematical/source review, not model execution.

A distinct later diagnostic remains available if directional context credit
becomes the observed blocker: freeze one real transition site and evaluate
actual downstream loss after each exact action intervention, replaying the
hard suffix and affected neighbor/consumer states. With those120 losses `C_a`
held fixed, the local surrogate `S = sum_a softmax(z)_a C_a` has derivative
`p_i * (C_i - sum_a p_a C_a)`, bounded coordinatewise by one quarter of the
observed cost range. This is a finite local objective, not the deterministic
model gradient, a proof of shared-q4 realizability, or a learning guarantee.
A later work card would need to compare that ordering with the current credit
and state the decision its extra replay cost can change. No intervention run,
new estimator, tangent change, stochastic serving, temperature sweep or fit is
part of this card. Preserve the mechanism and finish the persistent integer
session first.
