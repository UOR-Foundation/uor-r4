# Compound-binding preservation through unchanged R4 — #1075

## Frozen contract (2026-09-02; before fitted-model inference)

Issue [#1075](https://github.com/UOR-Foundation/uor-r4/issues/1075) continues
#973 after [#1073](r4_zoology_compound_binding_1073.md), delivered in #1074 at
`d7a369cb53a0311ff35111bb20a99c1aaba7dfde`. The compound model qualified at
100% supported/absent construction and held-out-combination development in
all four cyclic fact orders, and separately passed its value-only causal control. This
experiment preserves that exact learned behavior through the existing R4
gauge law. It performs zero training or new-population generation.

The source artifact is 1,148,672 bytes, with 286,976 unique parameters:
`blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4`.
Its learned-state identity is
`blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59`.
Bind all four published #1073 envelopes and the 270-file source closure.
Load no checkpoint, optimizer or saved RNG payload. Keep original data,
vocabulary, parameters, aliases, normalizations, projections and tied full
4,096-token head unchanged.

## Rectangular coordinate interface

Reuse the complete native #1059 frame export qualified by
[#1061](r4_zoology_exact_coherent_inference_1061.md). The frame bundle tree is
`blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.
Keep its 8,192-token map, 120 registered f64 matrices, exact multiplication
table and native prefix witnesses unchanged. Do not slice the map, rebuild
native geometry, change precision, or expand its reachable subgroup.

The old square-attention wrapper cannot directly accept this model. Add a
parameter-free rectangular wrapper while leaving historical source untouched.
Ordinary execution delegates to the frozen source model. Coherent execution
reuses the source's owner-object Q/K and location-V projection modules.

Frame assignment folds **only input positions 0 through 37**, using the
existing native ordered prefix law. Query frame is at position 37; fact K/V
frames are at positions 7, 15, 23 and 31. The learned null has no token source
and uses the atlas identity frame. Fixed grammar supplies these roles; no
target, equality matcher, answer filter or future token enters assignment.

**Definition — coordinate execution.** Treat each 64-dimensional vector as
16 consecutive four-lane blocks. For query frame Fq and true source frame Fj,
including the fifth/null entry:

```text
q_local = transpose(Fq) q
k_local_j = transpose(Fj) k_j
v_local_j = transpose(Fj) v_j
T_qj = transpose(Fq) Fj
score_j = f32(sum_over_16_blocks(q_local dot (T_qj k_local_j))) / 8
a = softmax_f32(score)
context = f32(Fq sum_over_all_5_entries(a_j T_qj v_local_j))
logits = original_head(original_output_LN(original_Wout(context)))
```

Frame operations, dot accumulation and weighted value transport use native
f64; softmax uses f32. Match #1073's arithmetic boundary: cast the completed
dot to f32 **before dividing by eight**, rather than prescaling its keys as
the older square adapter does. Decode to f32 before the original output
projection. Return actual attention `[batch,1,1,5]` and full-head logits.
Orthogonal coordinate cancellation motivates the comparison; finite-precision
behavior must satisfy the empirical criteria below.

Encode, transport, score and aggregate **all five entries**, including null.
In #1073, mean null attention was 54.64% on supported construction and 91.25%
on absent queries. Neither hard fact selection nor an argmax-null absence rule
would preserve the demonstrated mechanism.

## Population, exact reference and primary decision

Use the same 10,240 construction rows (8,192 supported, 2,048 absent) and
1,280 development rows (1,024 supported, 256 absent), in their original
canonical file order and right-cyclic fact rotations 0/1/2/3. These are
**46,080 decisions per arm**, comprising 36,864 supported and 9,216 absent.
The rotations are correlated views; development was already examined in
#1073. This is preservation on observed held-out combinations, not an additional
fresh or sealed generalization result.

Keep batch size **256**, eight CPU/Apple Accelerate threads and one inter-op
thread, matching #1073. Reuse its rectangular scorer and exact reduction/digest
ordering. All eight ordinary views must reproduce **every field** of their
published all/supported/unknown score records before any R4 forward. Those
records include counts, NLL and full logits/prediction/attention CIDs. An
ordinary mismatch stops with `COMPOUND_R4_REFERENCE_MISMATCH` and zero R4 or
control decisions.

**Empirical preservation criteria**, in every population/order and every
all/supported/unknown stratum:

- Identical top-1 predictions.
- Maximum absolute difference across the full vocabulary logits **<=0.005**.
- Maximum attention-weight difference **<=1e-5**.
- Absolute mean NLL difference **<=1e-5 nats**.
- Complete causal support, null participation and transport accounting; no
  future reads or future score slots. All group-level binding counts preserved.
- Unchanged model state, parameter count, tied head, eval mode, source/data,
  native frames and implementation identities before/after execution.

These are the existing #1059/#1061 preservation tolerances, not a replacement
learning threshold. Record decoded prediction IDs, group summaries, all
stratified scores/digests, differences and measured work.

## Conditional inconsistent-transport control

Only after **all eight primary views pass**, run one same-model, same-support
control. Encode each K/V in its true frame, but substitute source frame
`(j + 1) modulo 4` in the transport connection for the four facts. Leave the
null identity frame fixed. Do not combine this with #1073's value-cycle control.
Queries, original K/V, the five support entries, full softmax, parameters and
work remain unchanged; the resulting attention need not remain unchanged.

Require complete supported/absent decisions, equal coherent/control work,
causal reads, all four fact frame positions shifted and actual matrix changes
matching the preflight. Only a valid control with **at least 50 percentage
points of supported accuracy loss in every population/order** establishes
strong transport sensitivity. Report changed predictions, group behavior and
absent-query effects separately. UNKNOWN preservation belongs to the primary
comparison; it is not a broken-transport success condition.

- Primary miss: `COMPOUND_R4_PRESERVATION_MISS`; retain the source model and
  investigate only the adapter/numerical preservation failure.
- Primary pass, invalid control: `COMPOUND_R4_PRESERVED_CONTROL_INVALID`;
  retain preservation and repair the control's integrity separately.
- Primary pass, weak control: `COMPOUND_R4_PRESERVED_CONTROL_WEAK`; retain
  preservation and reconsider the intervention without claiming strong sensitivity.
- Primary and strong valid control pass: `COMPOUND_R4_PRESERVED`; retain the
  R4 binding path and next freeze a bounded language-interface step addressing
  fixed-position role extraction. No such follow-up fit runs in this issue.
- Resource interruption: `INCOMPLETE_RESOURCE`, with last completed progress
  and consumed clock retained. No replacement run, retuning or renewed budget.

## Reachability, cheap instrument and resource cap

All 46,080 decisions traverse the new interface. Each arm has **230,400**
fact/null score slots, including 46,080 null pairs; no triangular token mask is
needed. Each R4 row encodes 16 query blocks and 80 key/value blocks each,
transports 80 key/value blocks each, and decodes 16 output blocks. Per complete
R4 arm this is **737,280 query/decode blocks** and **3,686,400 key/value
encoding and transport blocks each**. Plain has the same support and zero
coordinate-transform counts.

The label-free frame preflight, before any model forward, completed in
**0.593 seconds** and passed. Every view reaches the same **24 of 120 frames**.
Actual changed source matrices across construction rotations are
**39,213 / 39,205 / 39,070 / 39,098** of 40,960; development counts are
**4,817 / 4,817 / 4,862 / 4,814** of 5,120. All supported rows have a changed
source frame except two construction rows in rotation 2. The resulting
supported-loss reachability ceilings are **100% everywhere except
99.9755859375% there**, above the 50-point criterion. These are opportunity
ceilings, not predicted model losses. Actual frame-matrix changes, rather than
merely shifted positions, remain part of the control integrity record.

Use **900 seconds cumulative for run plus replay**, **4 GiB peak RSS**, one
CPU process, eight intra-op threads and one inter-op thread. The clock includes
live source/frame/preflight validation. Retaining ordinary full-head logits for
all eight views uses 754,974,720 bytes (720 MiB). The run/replay pair at full
three-arm coverage has 276,480 decisions. Scaling #1073's 7.146-second,
87,040-decision evaluation by that volume and a conservative 4x adapter allowance
projects about **91 seconds**; the measured cap, not this estimate, is binding.

## Review, freeze and execution

Fourteen focused synthetic checks cover causal frame assignment, full-width
scaling/null aggregation, transport-control equations, source-state/RNG
preservation, historical binding/root relocation, exact-reference mapping,
per-stratum tolerances, work integrity, conditional control access, preflight
ceiling and complete question groups. Independent source review found no
blocker. Broad workspace, BDD, WASM, fuzz, audit, native rebuild/export and old
mechanics campaigns remain dormant. Queue statuses acknowledge transport only.

Publish source and the immutable preparation before the first fitted forward.
One run evaluates ordinary, coherent and conditional control; one fresh-process
replay must reproduce the complete deterministic evidence. Exclusive start
markers reject silent repetition. Source/model/frame bindings and learned
state are checked before and after. This offline floating-point experiment
does not change the deployed integer/table runtime or establish H4 superiority,
softmax removal, learned English parsing, free generation, reasoning or chat.

From `tools/r4-softmax-trainer` in the frozen offline environment:

```bash
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4 \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding \
  --frame-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1059-zoology-r4-inference/frames
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4
```

Append the observed result, reconcile six current mirrors and native trackers,
and close only #1075 after protected delivery. #973 stays open and #954 blocked.
