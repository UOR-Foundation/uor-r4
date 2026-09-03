# Learned clause-role interface over the frozen compound core — #1077

## Frozen experiment contract (2026-09-02)

Issue [#1077](https://github.com/UOR-Foundation/uor-r4/issues/1077) follows
[#1075](r4_zoology_compound_r4_1075.md), delivered in #1076 at
`45320652a03984cfe52046484d188cf07fb693a2`. The preceding experiment preserved
all 46,080 measured compound-binding predictions under unchanged native R4
transport. The next limitation is that fixed grammar supplies semantic roles.
This experiment learns a bounded role interface while retaining that core.

**Objective:** learn owner/object/location selection from controlled clauses,
including a competing owner name, and carry the resulting soft mixtures through
the original compound Q/K/V operation to the full 4,096-token answer head.
Use one model, one fixed dose and one conditional evaluation/replay. No sweep,
replacement fit, core retraining, geometry change or R4 execution occurs here.
A qualified ordinary interface can receive a separate R4 qualification next.

## Fixed source and learned interface

Retain the #1073 model qualified by #1075: **286,976 parameters**, model file
`blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4`,
learned state
`blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59`.
Bind the three published/retained #1075 envelopes, their 282-file implementation
closure and the #1073 source/model/data lineage. Retain the native frame-tree
identity without loading, exporting or executing frames.

The new reader has **141,571 parameters**: a separate 4,096-by-32 input
embedding, a width-5 convolution from 32 to 64 channels with bias, GELU, and a
64-to-3 role score projection with bias. The convolution has radius two inside
each supplied clause. There are no absolute position embeddings, hard semantic
role positions, entity-type filters, equality masks or answer filters.

Inputs are four fact clauses followed by one question clause, tokenized and
padded to width 13 with their lengths supplied. These are **given clause
boundaries**, not learned segmentation. Zero padding embeddings before the
convolution, and mask only padding in the role softmax. Every valid token,
including punctuation, function words and both owner names, remains eligible.

**Definition — soft role readout.** For each clause and each of the three role
scores, apply a softmax over its valid tokens. Use that distribution to average
the original frozen core embeddings of those tokens. Pass the four fact
owner/object pairs through the original compound normalization and Wk, their
location mixtures through the original location normalization and Wv, and the
question owner/object mixtures through original normalization and Wq. Append
original null K/V, use ordinary scaled softmax across all five entries, then
original output projection/normalization and tied full vocabulary head.
Question-location pooling is computed but unused. No argmax role selection
enters inference; pointer argmax is only an evaluation diagnostic.

The new question boundary is the end of its complete supplied clause, including
`answer :`. The semantic oracle uses the old canonical position-37 readout.
Both precede the answer token; this interface changes the available prefix and
does not claim unchanged tokenwise readout access.

## Controlled wording and split

Reuse the canonical #1073 semantic worlds, answers and five-row groups:
base q0/q1, location-swapped q0/q1, and absent q0. Owner/object held-out pairs
retain their original partition. Development semantics were already observed
in #1073/#1075; new renderings do not make them fresh independent worlds.

Fact clause frames and owner phrases are independently specified:

- C0: `{R} put the X in the L.`
- C1: `in the L, {R} put the X.`
- R0: `O, not D,`
- R1: `not D but O`

Construction includes the two diagonal combinations C0/R0 and C1/R1;
withheld wording combines C0/R1 and C1/R0. Both local owner cues and both
clause frames occur in training. Each view uses the same combination for its
four clauses. Fact distractor D is owner index plus four modulo sixteen,
preserving the semantic pair residue partition while remaining distinct.

The question form is `where is the X owned by O, not D? answer:`. This one
question form is seen during training. For same-object contrasts, q0 and q1
exchange the two relevant owner names between O and D. Their fact clauses and
question token bags are identical, while their correct answers differ. This
requires contextual owner disambiguation; lexical category detection alone
cannot qualify. Other query distractors preserve the pair partition and are
recorded in the data policy. No distractor changes a semantic fact.

The input lexicon explicitly assigns `not`, `but`, comma, `owned`, `by` and
padding to six previously unused IDs. Existing entity/output IDs and the
original core weights/head stay fixed; the separate reader learns the new
input vocabulary. Object and location vocabularies remain disjoint, so a
positive result is not evidence that all three roles required syntactic parsing.

- Construction: **10,240 source rows x 2 seen views = 20,480 rows**.
- Conditional development: **1,280 source rows x 4 views = 5,120 rows**;
  views 0/1 use seen combinations, views 2/3 use withheld combinations.
- Each row supplies 12 fact-role labels and two question-role labels during
  training only. The unused question-location label is ignored.
- Preserve original five-row groups, both question families, supported/absent
  labels and source fact order. Do not add a fact-order campaign.

Preparation generates these files once, binds exact bytes and audits role
positions, partition/co-mention constraints, view counts, identical-bag paired
contrasts and split separation. Fit never loads development tensors. Validation
may hash their bytes without opening them as tensors. Development is scored
only after construction and its control qualify.

## Fit and execution plan

Fit only the reader, using role-pointer cross entropy over all valid labels.
There is no answer loss and no core model load during fitting. Freeze seed 123,
**512 AdamW updates**, batch **128**, learning rate **0.003**, betas
**0.9/0.999**, epsilon **1e-8**, weight decay **0.01**, gradient norm limit **1**.
Use a private permutation generator with seed 10771; consume whole batches,
reshuffling only after a full construction traversal. This is **65,536 row
presentations and 917,504 supervised role-label presentations**. Save only
reader tensors with their content identity and unchanged core binding.

The CPU plans are four or eight intra-op threads, one inter-op thread and one
process under the pinned Python 3.12/Torch 2.7.1 environment and Apple Accelerate.
Before candidate creation, seed-10770 synthetic role batches compare identical,
unupdated reader parameters under each plan: two warmups and four timed
forward/backward units. There are zero optimizer updates and no real data in
this calibration. Loss and gradient maximum differences must be <=1e-6.
Select the smallest median time, breaking ties by lower thread count, and
restore RNG before the sole candidate initialization.

**Resource cap:** 900 seconds cumulative for fit, evaluation and replay, with
4 GiB peak RSS. The clock includes live validation and calibration. A projected
512-step fit at twice the selected unit time plus 60 seconds for evaluation
must fit before launch. The first eight actual optimizer steps then provide a
second admission projection; they are retained, never discarded or repeated.
The pointer-only fit avoids the full decoder during training. Prior #1073's
larger 3,920-step end-to-end fit took 49.903 seconds; this smaller dose is
expected to cost tens of seconds, with measured admission binding the decision.

Write progress every 64 updates, retaining completed/total steps and block loss.
Exclusive preparation/fit/run/replay start markers prevent silent replacement.
A cap violation reports `INCOMPLETE_RESOURCE`; no renewed budget or successful
replay of an incomplete result is allowed. Model, data and source identities
are checked again after each completed phase.

## Reachability, criteria and conditional access

All final answers traverse fourteen learned role mixtures and the complete
five-entry compound-binding mixture. The fixed core previously scored 100%
on these canonical semantics; each interpreted primary view independently
requires a perfect canonical oracle before crediting interface behavior.
The role reader therefore has access to every required source token. The
structural paired-question instrument requires identical bags/facts and
distinct targets. A bag-only deterministic answer function has a zero ceiling
on *both answers correct* for such a pair; its per-answer ceiling is 50%.

**Empirical primary criteria**, separately in every admitted view:

- At least 95% supported accuracy and 95% absent/UNKNOWN accuracy.
- At least 95% complete supported quartets in each question family.
- At least 99% pointer accuracy overall and separately for owner, object and
  location; the unused question-location output is excluded.
- At least 95% complete same-object matched-owner answer pairs.
- Complete row populations, fourteen supervised role decisions per row,
  five binding score slots per row including null, and unchanged reader/core
  state, parameter identities, tied head and inference mode.

Only after both construction views pass, run the fixed value-cycle control in
both views: right-cycle the four projected fact V entries while Q/K and null
remain fixed. Require exact role and binding attention, equal work, at least
50-point loss on original supported targets, at least 95% accuracy on reassigned
supported answers and at least 95% UNKNOWN accuracy. Counterfactual target
construction occurs only in the evaluator. No control example trains the reader.

Only primary plus control qualification opens development. Evaluate its two
seen and two withheld-combination views separately against the same primary
criteria, without selection, retuning, additional updates or extra control arms.
One fresh process must reproduce the complete evaluation evidence exactly.
Report actual decoded predictions, full-head/attention digests, per-role
positions, group counts and syntax-pair behavior; pointer accuracy alone cannot
establish answer-level success.

Outcome actions are frozen:

- `LANGUAGE_INTERFACE_CONSTRUCTION_MISS`: retain the core, identify the failed
  construction role/answer criterion, and keep development closed.
- `LANGUAGE_INTERFACE_CONTROL_MISS`: retain measured role progress and diagnose
  the soft-mixture binding/control failure; keep development closed.
- `LANGUAGE_INTERFACE_HELDOUT_MISS`: retain measured role progress and compare
  seen versus withheld syntax failures before designing another interface.
- `LANGUAGE_INTERFACE_HELDOUT_PASSED`: retain the learned interface and next
  freeze its unchanged-R4 qualification. No R4 run occurs in this issue.
- `INCOMPLETE_RESOURCE`: preserve completed evidence and consumed clock without
  retrying or interpreting unmeasured views as a learning failure.

## Delivery and scope

Focused synthetic checks cover padding/role mixtures, frozen-core gradients,
controls, source lineage, data splits, paired syntax and conditional decisions.
Independent source and preparation review precede fitting. Broad workspace,
BDD, WASM, fuzz, audit and old mechanics QA remain dormant. Protected queue
statuses acknowledge transport only, not scientific or product qualification.

Append results and exact preparation/fit/result/replay envelopes here, reconcile
six current mirrors plus native trackers/milestone21, deliver through a protected
PR and close only #1077. Preserve earlier records. General English, learned
segmentation, unseen vocabulary, open-ended generation, correctness, reasoning,
chat, geometry advantage and softmax removal remain outside this experiment.
#973 stays open and #954 remains blocked.


## Published pre-fit preparation (2026-09-02)

Source freeze: `f52afce506897f8b477b1ebafd37f47272699410`. Sixteen focused checks passed in 0.107 seconds;
Ruff, claim wording and independent source/contract review passed. The sole
preparation generated and audited the bound renderings without constructing or
fitting a candidate. Calibration, fit, evaluation and replay remain unrun.

- [Preparation](r4_zoology_language_interface_1077_preparation.json): `blake3:0395b826049dbeed351a647960c7b66cc4d65fc19b65eb3c522fcdd807aaad69`.
- Implementation, 295 files: `blake3:baa22e6e908dbf3a7c694679f386815736ca65db444e98ce823d1b5a26dfdb83`.
- Data manifest: `blake3:dc349b9931556cb45604295382766c78d173d26a2dca8491deacefae49fbd0d1`.
- Data tree: `blake3:b8b51eccc1fb4b472a5a4c80b8d4f34acac462bba02272df26bad1f918888143`.

Actual audits confirm 20,480 construction rows and 5,120 development rows,
with 14 role labels each. Seen/held-out clause combinations and actual/negated
owner-object partitions remain separated as declared. There is zero exact input
overlap across construction/development and zero new semantic worlds.
Matched same-object questions retain identical fact inputs and query token bags
while requiring distinct answers. Actual preparation review is the final
pre-fit launch gate; the fixed calibration and retained-step admission then
govern the single fit.
