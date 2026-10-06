# Native geometric vocabulary prediction — October 6

This is an executed CUDA learning connection and a mixed synthetic update,
not geometric attention or conversation qualification. PR #1792 is still a draft;
these changes are not yet delivered to main. The next capability experiment uses
ordinary response tokens and retrieval over the same full causal source-bank state.

## Implemented mechanism

### Delayed finite-state credit repair

A native intervention diagnosis at `e5072039c51a0d5f62b365d1c6edfaa5b125db19`
reproduces a causal omission in the older state-logit learning path. The second
token has a fixed identity action and zero state-dependent transition bases.
Changing its earlier retained state across all120 H4 roots changes its final
native utility, but the old backward gives the first token exactly zero transition
credit. This is an executed diagnosis, not a negative geometry verdict.

The repair adds a training-only hard-onehot retained-state utility channel. Its
backward retains120 values through time: action alternatives receive `U(old*a)`,
the existing local action-choice pullback runs once, and earlier-state alternatives
receive `U(r*factual_action)` plus the transition-score dependence on each root.
Arbitrary utility is not projected into four coordinates. Other lanes and the
carried action remain factual, so this is a declared local finite-choice surrogate,
not a global recurrent posterior or a derivative of argmax.

Generate replaces its old state-softmax attachment with this utility channel.
Factual coefficient and token-prototype gradients remain separate. Native forward
scores, state transitions and artifact formats are unchanged. The default Copy
consumers still use legacy credit. An opt-in prefix Copy path is described below;
contextual-readout and cue Copy credit remain legacy and unqualified.

At `f6973e0a6b0eedcd1e4ee92b743e59c53303de64`,33 focused checks pass on the
leased5090 pod:23 context checks,9 Generate checks and1 bank-binding check. Actual
CUDA tests cover delayed even-harmonic credit, CPU/CUDA parity, cross-lane dependence,
reset, and unchanged legacy channels. The Generate check independently enumerates
native conditional utilities and verifies unchanged parameter gradients without
duplicate state credit. Binary SHA256:
`6bc8563b2d5f037a590004d7380c4d405c701800d8645ed30a8d5bd9d1486ec3`.
Real-data CUDA admission and optimizer fits remain separate requirements.

The opt-in decoder learns H4 token-code tuples, directed relative unary fields,
ordered pair fields and token biases. Scores use the exact historical H4 frame.
Full120 conditional utilities carry local state/code credit, including table modes
that the older four-coordinate projection erased. The factual score is the native
integer score; offline Rust/CUDA supplies its declared surrogate adjoints. The
legacy channels retain their four-coordinate recurrence; Generate now has the
separate temporal utility channel described above. The current bank adapter
scores every Copy occurrence; the stopped selected-winner route belongs to the
older endpoint Period/Stop branch, which this vocabulary pool does not consume.
Differentiating that older route would not repair this path's temporal credit.

All legal fixed-tokenizer tokens are admitted by Generate, including EOS once.
Copy occurrence aliases join Generate through one positive native table-weight
marginal. There is no extra Null/Period/Stop action or hidden family bonus. Sparse
ID holes are excluded without collapsing identity. The common clip is ±8Q24;
reports retain clipping and raw-versus-clipped winner changes. The preallocated
numerical reducer is distinct from its allocating diagnostic trace.

The new target-free bank adapter consumes the final retained H4 state and full120
hard retained-state choices from the same full-bank causal context consumed by Copy. Query
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

The real-data CUDA admission subsequently completed at source
`e5acf3c87bcfdaabd2ea711856676e062ea274ef`, executable SHA256
`26a5587e64562bdba3b7f8afca1787e9c4f3bcb9a5f3037575b35a0aa05e137f`.
It processed84 prediction positions across8 conversations in12.85s of learning
work (19.21s complete admission), with finite nonzero gradients for all9 context
and4 Generate groups. Independently reloaded native artifacts and the checked
integer action pool passed. This was zero optimizer updates, and its eight-row
quality subset does not establish a fit or full512-panel result. The sealed report
is on the canonical pod volume at
`/workspace/uor-r4/codex/native-geometric-generate/real-b8-admission-seed1001-attempt2`.
The actual donor is4096 tokens with2 heads and4 lanes per head; dimensions now
come from authenticated consumer metadata. The earlier1024 inference was wrong
and caused an admission rejection before model work, not a model negative.

A source-input-selected cost probe at indices128,129,256,257,384,385,448,449
covers paired opposite queries for length4, length8, update and reassert histories.
This changes only the zero-update cost sample, not the sequential B8 fit schedule.
It is representative coverage, not a worst-case bound: sampled raw causal lengths
are44–78 tokens, versus93 at the panel maximum. Full-panel evaluation and optimizer
reserves remain necessary before admitting the128-update paired experiment.

Core4 focused tests execute at `7d3329b6`: exact frame/all-pair consistency,
full120 conditional replacement and independent artifact reload/tamper admission.
The later sparse-fixture correction changes no core numerics. At `0ad43b00`, pool5
and decoder8 tests execute, including actual CUDA parity, cached/reference gradients,
asymmetric pair adjoints, multiple Copy aliases, stale snapshot refusal and exact
raw-score binding. At `e5a52280`, target-free Copy parity1 and driver admission2
execute, followed by the actual CUDA update. The crossed audit tests/build/run
execute at `74443d4d`. Integrated bank/source-free tests subsequently passed at
`92b05273`, the source-realizer integration at `974b47ac`, and all512 input-only
panel positive controls at `013a5d6d`. Those checks establish instrument support
and answerability, not learned conversation.

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


## Opt-in prefix Copy temporal utility

At source `92587b03a48a826523cb1b273a86644b51d3a375`, the prefix transport can
attach its existing exact centered120-state conditional table utilities directly
to actual retained choices. This replaces its local-logit attachment. The context
operator owns the one choice pullback and temporal carry; no second softmax or
extra semantic feature is introduced. Candidate offsetj still consumes poststate
j−1, offset0 uses constant identity, and an empty response supplies no invented
context. Other endpoints and lanes remain factual.

The bank adapter and fitter expose `prefix_temporal_utility`, defaultfalse, with
an explicit report scope. The live paired fits use frozen source1698aa79 and the
old setting. Source review found no remaining blocker after correcting a stale
report label. The library/fitter compile and12 focused CPU checks pass on the Linux pod:
1 actual-context temporal regression,5 legacy transport controls,6 fitter tests.
The new regression preserves native scores while carrying even-harmonic credit
to an earlier transition, cuts credit at reset and future source positions, and
retains empty-response identity. Actual CUDA parity and enabled-bank admission
wait for a free GPU. This is not yet a prediction improvement.

The real-data paired fits completed their full512 initial evaluations and passed
prospective cost gates:4080.868s output-only and4165.737s joint against4500s per
arm. Each runs128updates with B8; seeds1001,1002,1003 are sequential per GPU.
Completion and actual generated replies remain required before a family verdict.
The prospective temporal-fit resource extension supersedes the original2h rental
ceiling with5.5h ($10.89 maximum at$1.98/h); the complete6h wall and storage
bounds remain. The scoped prefix CPU build/check reserves four additional CPU
threads and6GiB host RAM, total20CPU/30GiB while the two fits run, with300s
check wall. No extra rental or GPU job is added.


## Adversarial interpretation and next decision

The120-state carrier removes a real four-coordinate nullspace, but its suffix
actions and other lanes remain factual. A changed early state can change later
policy choices; the current local surrogate does not recompute that alternate
suffix. Native reload and complete replies, rather than nonzero credit, decide
whether the paired learner helps. If it does not, compare centered carried
utilities against bounded native120-way suffix replay at actual first-error
transitions, with all later policies and alias-normalized loss recomputed. Require
factual replay, then evaluate proposed shared Q4 edits on the complete trajectory
population. These are conditional next discriminators, not launched extra fits.

Retained [native route fits](geometric-native-route-fit-2026-10-04.md) and
[cell-crossing witnesses](geometric-native-route-crossing-2026-10-04.md) already
show why local utilities alone are insufficient: all32 retained crossing witnesses
improved local loss while worsening full-panel loss. Reuse complete-trajectory
acceptance mechanics if the new results demonstrate that interference; preserve
the architecture and negative candidates. A stochastic policy-gradient objective
is a separate prospective learning choice, not an exact derivative of hard
argmax serving. The current fits decide the next integration step first.


Prefix validation receipt: library test SHA256
`00fe97f7c02ad6331fea8b4e942f5196f94241ede3276c5c9b2095e20a790881`,
fitter test SHA256
`1ad691d80a149c30da209c0c3c3bbd43fcb5f006a02b018f0002b7d7eb85a48a`.
Final release build466s, regression0.01s, exit0. An initial incomplete-source
manifest attempt exits101; two300s compile limits exit124. All are retained
in separate logs and are not mechanism results. The empirically corrected retry
allows900s, with prospective total check wall1500s; it changes no overall6h
wall, rental or storage ceiling. All17116 source archive files match the cached
source tree after restoring its research files and applying the four changed
code files. Live fits keep their separate frozen binary.


## Observation-credit inquiry: avoid an equivalent reroute

A subsequent source/mathematical review corrects the earlier narrow migration
suggestion: lifting an existing readout state derivativev4 into120 values
`U(r)=dot(v4,root(r))` does not recover higher harmonics. Right quaternion
multiplication by a fixed H4 action preserves this linear function family. The
q4 context already uses finite-choice ambient Hamilton credit, without the older
tangent projection. An equivalent carrier reroute is therefore not justified as
a new information-recovery mechanism. No such arm is implemented or launched.

The substantive candidate is a conditional native-observation intervention.
Changing retained lanej can alter its own observation and the preceding lane's
neighbor-dependent observation. Recompute both packed root/category/absence
choices, shared endpoint identity and all affected candidate aliases before
scoring utility. At actual Copy failures, compare that120-state native utility
with the current lifted readout derivative, require factual replay, and determine
whether omitted or reversed directions improve actual target probability/margin.
A nonlinear residual alone does not justify a new fit. Preserving readout
coefficient adjoints while replacing only the selected input-state credit is a
prospective estimator decomposition, with double-credit and shared-packet risks.
This inquiry remains separate from fully recomputing later recurrence controllers.

## Seed1001: learning without first-token success

Both frozen1698aa79 fits complete128 updates, seal their reports and exit0.
Source/executable/data identities match across the pair; executable SHA256 is
`abc4b4453c7b326c408cb65e11df0b46b7bb8d6e4a72f6d73e678a8ec764b393`.
Output-only elapsed3384.095s, joint3538.719s. Native equal-episode CE is initially
8.185317103, then7.903411290 output-only and7.928527014 joint. Complete answers
remain0/512 in each arm. All first errors occur at position0, and the corrected
direct-schema audit confirms all512 first gold tokens are Generate-only. An
initial diagnostic mistakenly looked for a nested `native` field in generation
steps; its family classification is superseded by the direct-field audit.

Token-weighted canonical CE partitions (distinct from equal-episode selection):

| Gold family | Positions | Initial | Output-only128 | Joint128 |
|---|---:|---:|---:|---:|
| First Generate-only |512|8.330277|8.081684|8.208213|
| Later Generate-only |2024|8.325012|8.071992|8.208251|
| Copy-supported |4128|8.084647|7.779896|7.729049|

Thus improvement is not confined to Copy, but joint movement attenuates Generate
improvement while improving Copy-supported likelihood more. This is one seed,
not the final three-seed decision or generalization evidence.

Native exported prototypes and source-master argmax agree at both endpoints.
All32768 hard codes remain unchanged, with per-lane distinct-code counts
`[120,35,1,1,1,1,1,1]`. Minimum prototype shadow margin falls2→0.313015 in
output-only and2→0.441180 in joint; medians are1.327134 and1.655980. Shadow
movement is real, but it has not changed token geometry. Joint changes13072
packed context bytes; output-only changes0.

The seeded base120 arrangement and disjoint pair graph imply exact cancellation
of lanes2–7 from any raw generated-token score difference while codes remain
constant. Those lanes may influence recurrence and Generate/Copy calibration,
but cannot directly distinguish output tokens in this decoder. Distinct full
token tuples do not remove that restriction. An independent source/mathematical
review recommends exact native target-versus-winner factor replay before a
matched, label-free all-lane initialization intervention. No semantic metric,
gate or normalizer change follows from this finding.

Two alternatives were investigated and rejected as current canonical causes:
2536 Generate-required positions contain no same-state/different-target
conflicts within or across rows; first-position Copy maxima are3.842009 initial/
output-only and4.266907 joint, below both the8-nat clip ceiling and5.6875-nat
strict decoder theoretical ceiling. No first-position action clips. These audits
do not qualify arbitrary divergent prefixes or establish full shared-factor
separability. The narrow Rust replay will verify saved score hashes before using
gold labels for diagnostic margins; it does not train or supply runtime answers.
Seed1002/1003 configurations and live binaries remain frozen.

An additional effective-state audit groups canonical Generate-only targets by
the first two retained lanes, since the six constant token lanes cannot affect
raw Generate ordering. Initial/output-only has6 conflicting first-position
groups (12 positions), and186 conflicting groups across2536 canonical
Generate-only positions (399 positions). Joint128 still has6 conflicting first
groups and226 across the canonical set (480 positions). Holding these factual
states and hard codes fixed, even arbitrary ranking per first-two-lane state
can satisfy at most506/512 first gold labels and2334/2536 initial/output-only
canonical labels (2295/2536 joint). These are optimistic bounds for raw Generate
ranking on saved canonical states, not serving-completion bounds or limits on
future learned states/codes. They demonstrate a real ignored distinction while
accounting for only a fraction of the observed0/512 failure.

The new `geometric-generate-margin-audit` example at numerical heada84bf2e1
has an independent exact-head source review. It verifies sealed inputs, native
checkpoint and row identities, full-vocabulary score SHA replay, canonical/
generation empty-prefix parity, and exact integer factor sums before diagnostic
gold indexing. Its pod build and2048-position maximum paired replay are admitted
within unchanged total wall/rental ceilings (4 CPU threads,6GiB additional host
RAM,900s build plus60s per arm). Build/arithmetic execution is now complete;
source review is recorded separately from execution. It does not modify learning
or runtime prediction. Existing fit binaries remain separate and frozen.

The actual pod release build and2 focused checks exit0. Both paired audits
complete and seal with exit0 at a84bf2e1, executable SHA256
`56e17e070a0253171faf715409dcf7945a1be1f5cd1bd257f817168abfa1f382`.
All2048 first-position full-vocabulary score hashes match the retained native
traces. Every unary/pair/bias decomposition sums exactly to the authoritative
integer score, and the six higher lanes/three higher pairs cancel from every
target/best-Generate margin. At128 updates, output-only has472 targets strictly
below best Generate and40 ties; joint has495 below and17 ties. Deterministic
Generate selection finds the gold in21 output-only cases and8 joint cases,
but all of those lose the final pooled decision to Copy-supported winners.
Initial505 strict deficits and7 ties select no gold. Mean raw target-minus-best
Generate margin moves−0.376221→−0.280640 output-only/−0.327515 joint nats.
This demonstrates a decoder-ranking deficit separately from the remaining
Copy competition. It supports a matched label-free all-lane initialization
intervention, subject to the after-step adversarial review; it does not qualify
prose, attention transfer or chat, and it does not justify changing normalization.

Independent PI review reads both actual audit reports, verifies their SHA256
identities and traverses all2048 rows: all three factor decompositions, both
margin vectors, code ranges, pair order, higher-factor cancellation and saved
Copy/Generate mass sums agree. That review does not independently rerun inference
or score-vector/BLAKE3 hashing. The executed auditor owns those latter checks.

## Next single-change control: balanced initial token geometry

At numerical head2a5bbd09 the fitter exposes default-false
`balanced_token_geometry`; the old `seeded` API retains its original digit
assignment. The opt-in assigns token `t=a+120b` to lanej using
`(a+j*b+existing_offset_j) mod120`. Adjacent pairs have determinant1 over
`Z/120`, so each pair recovers `(a,b)` and remains injective for4096 tokens.
Each lane contains all120 codes with34/35 counts. The arrangement changes
label-free root indices, not semantic distances. All coordinates remain a
deterministic embedding of token identity, not eight independent semantic axes.

Energy RNG/order, pair graph, q4 coefficients/biases, prototype winner gap2,
optimizer/objective and runtime operator costs remain fixed. Initialization mode
is explicit in admission, optimizer design, checkpoint receipt and final report;
exported artifacts contain the actual codes. The first pair already carried
4096 identities, so the intervention activates additional state-conditioned
token distinctions rather than adding identity information. Its different
initial score distribution must also be reported before attributing benefits.
Independent exact-head source review finds no blocker. Release compilation and
focused legacy/export/balance checks are running on the pod; CUDA admission,
balanced fit and any predictive improvement are NOT_RUN. Existing six frozen
runs continue. The next fit is a matched output-only comparison after their
complete decision and full prospective cost admission.
