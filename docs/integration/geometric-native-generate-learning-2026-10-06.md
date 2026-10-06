# Native geometric vocabulary prediction — October 6

This is an executed CUDA learning connection and a mixed synthetic update,
not geometric attention or conversation qualification. PR #1792 is still a draft;
these changes are not yet delivered to main. The next capability experiment uses
ordinary response tokens and retrieval over the same full causal source-bank state.

## Implemented mechanism

The opt-in decoder learns H4 token-code tuples, directed relative unary fields,
ordered pair fields and token biases. Scores use the exact historical H4 frame.
Full120 conditional utilities carry local state/code credit, including table modes
that the older four-coordinate projection erased. The factual score is the native
integer score; offline Rust/CUDA supplies its differentiable adjoints. The earlier
recurrence still uses its four-coordinate approximation, and factual source-route
selection remains stopped. Those limitations have not been solved by this decoder.

All legal fixed-tokenizer tokens are admitted by Generate, including EOS once.
Copy occurrence aliases join Generate through one positive native table-weight
marginal. There is no extra Null/Period/Stop action or hidden family bonus. Sparse
ID holes are excluded without collapsing identity. The common clip is ±8Q24;
reports retain clipping and raw-versus-clipped winner changes. The preallocated
numerical reducer is distinct from its allocating diagnostic trace.

The new target-free bank adapter consumes the final retained H4 state and full120
POSTSTATE logits from the same full-bank causal context consumed by Copy. Query
and emitted prefixes are state inputs, not Copy candidates. An explicit ordinary
continuation path encodes actual causal IDs with zero Copy candidates; no Source
record or initial token is fabricated. Legacy terminal reductions remain discarded
preparation overhead. Whole-path multiplier/float/allocation and laptop cost
qualification remain outstanding.

## Executed update and attribution

Source `e5a5228030c032ce092c7f9b6c8ed2be78c462a2`, binary SHA256
`070e42bac909afdb8e08bbf2e672a6c8df06f33efe36df052bbe03f23a58352d`.
Host: Linux x86_64, pod `d68osfkekgamor`, one leased RTX5090 (CUDA cap120),
CPU8. One AdamW update, B8, seed1001, lr0.2, global clip1, zero weight decay.
This displacement instrument uses a synthetic4096-token vocabulary and fixed
next-symbol targets absent from the causal prompts. It is not language data.

The CUDA update completed exit0 in1.9587s, peak host RSS548564KiB. Snapshot/staging
0.8369s, forward with native oracle0.01065s, backward0.03337s, optimizer/projection
0.00148s. These scoped timings establish executed GPU work; there is no matched
fit-speed or energy claim. All four decoder families and three consumed context
transition families have finite nonzero CUDA credit. The six observation families
are deliberately frozen in this state-only instrument.

Native token/self/neighbor transition coefficients change5736/960/960 respectively.
Generate unary/pair coefficients change229/13080;2048 packed bias bytes change.
No hard token prototype changes. Both endpoints independently reload, and native
emitted-token feedback executes eight capped32-token rollouts. Those rollouts do
not establish successful replies.

A separate zero-update audit at source `74443d4dc06d13c65e2fa7ac2d9c9d5085620010`
loads the saved checkpoints and crosses context/decoder endpoints, with identical
causal prefixes and target labels inspected only after full-pool prediction.

| Native replay | Mean target CE | Exact first-position targets /8 |
| --- | ---: | ---: |
| Initial context / initial decoder |8.356155|0|
| Learned context / initial decoder |8.301136|0|
| Initial context / learned decoder |7.878171|4|
| Learned context / learned decoder |8.194799|0|

All16 retained lane states move, and a fixed decoder changes thousands of token
scores for every row. There is no active clipping or Copy alias in this instrument.
The joint endpoint improves parent CE0.161356nat but is0.316628nat worse than the
decoder-only cross. The interaction is adverse +0.371647nat. This establishes
native context sensitivity and discrete parameter learning, not beneficial joint
learning or learned token codes. Retain all endpoints and crosses; eight synthetic
rows do not retire geometry or select a permanent frozen-state architecture.

## Necessary checks and retained faults

Core4 focused tests execute at `7d3329b6`: exact frame/all-pair consistency,
full120 conditional replacement and independent artifact reload/tamper admission.
The later sparse-fixture correction changes no core numerics. At `0ad43b00`, pool5
and decoder8 tests execute, including actual CUDA parity, cached/reference gradients,
asymmetric pair adjoints, multiple Copy aliases, stale snapshot refusal and exact
raw-score binding. At `e5a52280`, target-free Copy parity1 and driver admission2
execute, followed by the actual CUDA update. The crossed audit tests/build/run
execute at `74443d4d`. New integrated bank/panel checks are pending and must not be
counted as passing from source review.

Retained execution defects include the inverse-frame type/assignment repair,
missing timing utility, offline dependency availability, invalid sparse BPE fixture,
JSON macro recursion limit and abbreviated-SHA fetch failure. These are instrument
faults, not measured geometric negatives. The Copy forward is anchored to the
once-converted checked native integer head sum; this avoids false exact-parity
rejection from differing floating-point summation order while preserving gradients.

## Next experiment and reachability correction

Live artifact inspection corrects an earlier parameter-count inference: the real
donor has4096 tokens and two heads ×four lanes. Its consumer metadata, not an
inferred parameter count or synthetic configuration, owns these dimensions. Use all-source raw input packets and a new sealed
prose-label panel with an independent input-derived answerability reference.
Ordinary response words must have Generate support outside Source spans. Reusing
exposed development inputs does not create held-out data. No serving parser or
formatter authors responses.

Compare paired output-only/joint arms with identical data, initialization and
Generate groups, and actual complete emitted-prefix replies as the primary
endpoint. Decoder/context crossings and threshold margins remain diagnostics.
Before fitting, measure complete real B8 cost and device/gradient admission.

An independent review derives a reachability bound for fresh Adam(.9,.999), zero
decay: summed normalized coordinate displacement over128steps is at most
227.47318 times the group's learning rate. Consequently context lr0.0003 cannot
cross a0.125 quarter threshold from exact-quarter masters; prototype lr0.003
cannot close an initial winner/alternative logit gap2 even with both moving
maximally. A fit under those conditions would not test hard geometry learning.
The prospective groups are coefficient lr0.003, prototype-choice lr0.01 and
joint-context lr0.002. These allow crossings in principle and do not guarantee
useful learning. Measure actual donor margins and report every native crossing.
Do not reuse this bound for resumed optimizer moments without rederiving it.

Shared normalizer/Stack/session work remains with the other labs. No change to
DeepSeek's softmax/tensor-core track is included.

## Evidence and costs

Durable reports remain on the canonical network volume under
`/workspace/uor-r4/codex/native-geometric-generate/`: `update-seed1001-attempt1`
and `crossed-audit-seed1001`, plus all numbered build/check logs and exact binaries.
Reports are sealed and verified. The pod's own area is36MiB and shared Codex target
1.5GiB at the last measured receipt; this is not a laptop download.

The existing work card precharges the complete6h continuation, cumulative
1433835028/1434600028ms, with maximum2h rental, CPU8/RSS12GiB/GPU12GiB,
local new≤32MiB, pod new≤6GiB and Codex area/cache≤16GiB,128MiB stop margin.
The initial failed standard bootstrap used host quota30 build threads; subsequent
own commands use8. Actual build/model/rental charges and remaining work must be
reported separately; precharge is not executed model time.
