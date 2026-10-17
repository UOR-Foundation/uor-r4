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
after the byte-unchanged production CUDA initializer, the opted-in constructor
uploads literal +1 at each payload block's leaf-combine gate and zero at every
other combine/relation gate. This is declared storage preparation, with no
target-dependent computation or learning apply; epoch remains zero. The
initializer ABI and default body remain unchanged. The existing omitted/default `zero_gates_v1` keeps its
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
| Host metadata library tests, including schedule/cursor/default/lineage/control | At `202855294`: 65 passed, 0 failed, 1 ignored; no model execution; final pin-only rerun recorded on PR #2575 |
| Active initialization and matched-off CUDA control | Implemented as additional `qualify-relation` control; NOT_RUN |
| Host `check --no-default-features --all-targets` | PASS at `202855294`; final pin-only rerun recorded on PR #2575 |
| Strict host Clippy | BLOCKED by existing dependency lints in unchanged integer crate; not a pass |
| Advisory host Clippy | Completed with warnings; no suppression added |
| Exact-head CUDA check/Clippy and finite controls | NOT_RUN; compiler lease stopped at the clean-tree preflight before Cargo |
| Independent integrated source review | APPROVE through `202855294`; AGY source/math APPROVE at `f6c7fa5e1`; subsequent initializer/fixture and canonical-sheet deltas independently approved; no runtime verdict |
| Scientific initialization/comparator design | FIT (design), PR #2574; no execution release |
| Training / M2 / Stage B | HELD |
| Q4 | NOT_EXECUTABLE |
| Model updates, compiler pod leases, lease-cost estimate | 0 / 1 / $0.067731111; no model execution |

The local disk was below the 30 GiB pod-admission floor at source preparation;
no pod was started. Source preparation used a sparse owned worktree. Subsequent
host metadata verification materialized about 90 MiB of tracked sources and
used a small isolated, regenerable Codex target directory, with debug information
and incremental output disabled. This is compiler/storage/schedule verification,
not CPU model/reference execution or pod admission. The shared model-time JSON was not available
at its historical path; this preparation adds no model-time charge. Live harness
receipts record the coordination; model/compile timing remains NOT_RUN.
The old 3.74-second batch projection is historical, not a current budget.
A future reviewed packet must price its actual source, full arm/seed dose,
construction, baseline/final/ablation/transfer evaluations, retries/resumes,
checkpoint/archive storage and resource margin from current measurements.

The first host invocation exposed missing tracked include files in the sparse
checkout; materializing those exact files fixed setup. The first test run then
found a stale source-inventory spelling after the guarded apply expression and
an absent ignored `local/` parent for an existing storage test. The inventory
still permits exactly the same two apply call sites; its expected expression
was updated, and the test's parent directory was created without changing its
source. The complete rerun passed. Strict host Clippy stopped on pre-existing
`native_geometry.rs` dead code and `geometric_span.rs` style in the integer
dependency; the source unit does not silently cap that result or expand into
unrelated lint repair. CUDA check/strict Clippy and finite behavior remain
separate, unexecuted requirements.

Line: M1 learned-attention link (Relation interleaved read), accepted by PM
[6229949186](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229949186).
No historical count is reset by this source delivery; the milestone owns count
disposition and its suspended 37/232 headline remains unchanged.

## Next dependency and interpretation

### First compiler lease: preflight blocked, no compiler result

The compiler-only release at milestone comments `6230657270`/`6230824049`,
reaffirmed by `6231087599`, admitted source `bf8c7e965`. The first local launch
refused a missing runner companion before allocation; restoring exact-source
`hot-set.txt`/`uor-store` passed independent packaging review and the existing
runner's dry-run preflight. No pod or charge followed that first refusal.

The actual lease `vast-56364467` then ran from 07:49:17Z to 07:53:11Z on
17 October, with provider-reported uptime of 232 seconds at $1.051/hour,
an estimated $0.067731111. The ledger up-to-down wall interval is 234 seconds,
a separate $0.068315000 estimate; neither figure is an invoice measurement.
SSH became ready after 148 seconds. Setup reports exit zero,
but the gate script reports exit one before printing the source head or
invoking any Cargo gate. All CUDA/host compiler and metadata results at this
head remain NOT_RUN; no model or qualification command ran.

The wrapper creates its output directory beneath `evals/` before requiring a
completely clean checkout, and the pod runner also creates its job files under
`jobs/codex/`. Neither path is ignored by this source's `.gitignore`. A local
Git-only reproduction reports both as untracked. This establishes a defect in
the wrapper's preflight composition. The sealed remote record does not contain
the rejected status listing, so attribution to that exact predicate is a source
inference rather than a captured remote diagnostic. The repair must retain
whole-tree tracked-source checks, admit only this session's generated output
and job paths, and print the source/status evidence before refusing. It does
not change model code, the scientific design or its bars.

The 336,896-byte archive
`icloud:UOR-R4/results/codex/attention-compile-gate1-archive.tar`, MD5
`afbaee141d854857c30a92cfa6b686e6`, passed local SHA inventories and complete
cloud MD5 readback before release/down; the fleet returned to zero. The failed
packet and transcripts are preserved. A corrected packet needs prospective
review and explicit disposition before another paid attempt; no on-lease patch
or automatic paid retry is authorized by this receipt. Cumulative Codex lease
estimate is $3.558663333, plus separate PM costs; storage/transfer remain
UNMEASURED. This is an operator preflight blocker, not a mechanism negative.
Independent outcome review accepts the operational blocker receipt and verifies
all 8 remote, 18 operator and 10 frozen packet SHA entries. A prospective
session-scoped guard repair passes eight Git-only fixture cases: owned runtime
paths pass; unrelated files, peer jobs, unstaged/staged source changes, tracked
changes within allowed paths, a staged addition removed from the working tree,
and wrong HEAD are refused. The index and working tree are checked separately. Those metadata checks
are not compiler/model execution or permission for another lease.

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

The scientific sheet was delivered in parallel in [PR
#2574](https://github.com/UOR-Foundation/uor-r4/pull/2574), merged at `fb4519226`
with FIT (design), not an execution release. Source is aligned to its unchanged
production initializer and three-part ACTIVE/INERT/INIT LOCK construction
contract. Source/design reconciliation is recorded in [6230176389](https://github.com/UOR-Foundation/uor-r4/pull/2574#issuecomment-6230176389)
and [6230362901](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230362901).
The real training transition makes one `Device::propose` call per batch; the
separate virtual-only `propose_next` API is not called by that transition.
A cost projection that adds both APIs as if both execute each batch doubles the
proposal component. Actual Relation throughput is still unmeasured; neither a
K16 diagnostic call nor older Route throughput is a current Relation budget.
