# Native Relation learning path — 17 October 2026

Owner-prioritized implementation of the missing learned-attention path, using
existing discrete geometry and the shared writer. This is source preparation;
**no learned attention result, model execution or hold release is recorded.**
Owner implementation/coordination direction is recorded on [M1
6229960779](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229960779),
following the [source assessment
6229756412](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229756412).
Codex/cf1 owns implementation; Kimi retains fitness, protected queue and execution
coordination; AGY has an independent source/math review handoff.

## Problem and implementation

The previous `SchedulePolicy` implemented only vector-then-read ordering.
At V4096/B4/C192, a 16,404-batch canonical Relation dose would visit all
16,384 vector coordinates and twenty combine gates, with **zero** of the twenty-five
relation parameters attempted. All-zero combine initialization also makes an
initial relation-only proposal invisible at the writer loss. These are source
reachability gaps, not measured negatives for relational attention.

The existing driver now implements `InterleavedReadEvery8V1`: every eighth batch
is a read proposal; other batches visit vector coordinates. Scheduling uses the
**absolute completed-batch ordinal**, including after cursor wrap and resume.
The metadata cycle is 16,404, independently of Relation's 16,429-coordinate
inventory. A full registered dose has 14,354 vector and 2,050 read attempts;
Relation's read family comprises 920 combine and 1,130 relation attempts.
Corpus order, identities, geometric carrier, shared writer and objective are retained.

`read_initialization: active_leaf_v1` is an explicit prospective configuration:
CUDA sets +1 at each payload block's leaf-combine gate and zero at every other
combine/relation gate. The existing omitted/default `zero_gates_v1` keeps its
old meaning and serialization. Initialization identity binds configuration,
lineage, model binding and snapshot; a canonical/zero checkpoint cannot silently
resume into the new experiment. GPU storage readback verifies the declared
initial pattern. Learned gates may subsequently change.

An active path supplies a learning opportunity, not a guaranteed gain. Snapping,
ties and writer exclusion may still hide a particular proposal's effect. No
answer-derived gate, source index, teacher, surrogate optimizer or host forward
is added. Treatment and the primary relation-disabled control use the same
Relation read inventory, leaf initialization and interleaver. The control's
explicit `relation_learning_policy: frozen_zero_v1` keeps all 25 relation trits
zero, observes the same scheduled counterfactual proposals, and refuses their
commits through the existing transition. The policy binds config/lineage and
checkpoint admission; default omitted `learn_v1` preserves older configuration
bytes. It is an experimental control, not another reader or optimizer.
Both arms therefore receive 920 combine and 1,130 relation slots. Refused slots
retain genuine hypothetical sensitivity observations and have zero accepted
updates; they are not reported as all-tied model observations. Flock remains a
secondary identity control with 2,050 combine proposals, a frequency difference
that prevents using it as the primary causal comparison. A Route-only comparison
cannot attribute relation benefit.

## Scoped executor and evidence

The attention executor reuses `PreparedDriver`'s CUDA transition and existing
release/checkpoint/lock machinery. Legacy A2 pilot/completion permissions remain
limited to canonical Route with zero-gate initialization; attention uses a distinct
arm/seed release scope. No D33 delegate can create an attention owner release.
The record is an operational safeguard on the shared OS account, not cryptographic
proof of the owner's identity. No agent invokes the owner-release command.

The new path records actual VectorBlock/GateCombine/GateRelation attempts,
candidate-cost sensitivity, declared relation refusals and accepted updates, checkpoints before first apply
and every 64 transitions/end, and refuses incompatible resume state. Final output
retains the trained checkpoint, full DEV rows and same-trained-state relation-off
ablation. A separately pinned transfer corpus is evaluated on CUDA after the
fixed training dose, without checkpoint or initialization selection on its labels.
The attention packet retains the existing minimum 2,000 transfer targets and
500 successor rows; successor coverage and held-out provenance require the
reviewed data packet. The generic transfer evaluator's 200-row D22 floor does
not replace those attention-specific requirements.
Different file hashes alone do not prove semantic independence; the reviewed
packet must establish held-out provenance and its fixed decision bars.

The existing successor-panel scorer can consume the per-row evaluations. Report
all 3,847 successor rows and the 10,440-row complement, actual target admission
and conditional rank, matched-arm comparison and same-checkpoint ablation. A
TRAIN-only successor control tests frequency explanations. A context-local rule
is tautological on the target-defined successor panel and is not learned evidence.

## Verification and execution status

| Check | Status |
| --- | --- |
| Schedule/cursor/default/lineage regression source | Implemented; execution NOT_RUN |
| Active initialization and matched-off CUDA control | Implemented as additional `qualify-relation` control; NOT_RUN |
| Exact-head metadata build/tests and Clippy | NOT_RUN |
| Exact-head CUDA check/Clippy and finite controls | NOT_RUN |
| Independent integrated source review | Pending |
| Scientific initialization/comparator amendment | PENDING FIT |
| Training / M2 / Stage B | HELD |
| Q4 | NOT_EXECUTABLE |
| Model updates, new pod leases, GPU cost | 0 / 0 / $0 |

The local disk was below the 30 GiB pod-admission floor at source preparation;
no pod or material build was started. Source preparation used a small sparse
owned worktree and formatter only. The shared model-time JSON was not available
at its historical path; this preparation adds no model-time charge. Live harness
receipts record the coordination; model/compile timing remains NOT_RUN.
The old 3.74-second batch projection is historical, not a current budget.
A future reviewed packet must price its actual source, full arm/seed dose,
construction, baseline/final/ablation/transfer evaluations, retries/resumes,
checkpoint/archive storage and resource margin from current measurements.

Line: M1 learned-attention link (Relation interleaved read), accepted by PM
[6229949186](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229949186).
No historical count is reset by this source delivery; the milestone owns count
disposition and its suspended 37/232 headline remains unchanged.

## Next dependency and interpretation

Finish exact-head source review and meaningful checks in this source unit, obtain
prospective fitness on the amended matched experiment, and obtain the distinct
applicable owner/PM execution releases and fresh resource admission. Source
preparation is not gated on another timing optimization programme. The existing
bounded timing closure and K16 draft remain separately preserved.

If relation proposals cannot affect loss, repair that particular credit path
before extending training. If relation helps conditionally but writer admission
collapses, preserve the read observation and isolate that writer interaction.
Only an attributable learned gain with the declared non-regression and fresh
transfer criteria earns a bounded learned-attention claim. No result here closes
a mechanism family, resets a line count or moves the suspended M1 headline.

The scientific sheet is developed in parallel in [PR
#2574](https://github.com/UOR-Foundation/uor-r4/pull/2574), not a second source
commission. Source/design reconciliation is recorded in [6230176389](https://github.com/UOR-Foundation/uor-r4/pull/2574#issuecomment-6230176389).
The real training transition makes one `Device::propose` call per batch; the
separate virtual-only `propose_next` API is not called by that transition.
A cost projection that adds both APIs as if both execute each batch doubles the
proposal component. Actual Relation throughput is still unmeasured; neither a
K16 diagnostic call nor older Route throughput is a current Relation budget.
