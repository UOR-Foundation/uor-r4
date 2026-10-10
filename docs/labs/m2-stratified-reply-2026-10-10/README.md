# Stratified complete-reply fitting qualification

The owner requested a direct learnability check after the frozen512 headline
remained8/512. The preceding [full-panel one-pass result](../m2-reply-gradient-2026-10-10/README.md)
regressed8→2 despite lower teacher-prefix CE. The subsequent source/data assessment
found that all canonical answers fit vocabulary/context/output limits, but no
learned artifact demonstrates full-panel realizability. The old512 answerability
reference is explicitly not a native prediction result.

## Question and pre-registration

Can ordinary Potential/Generate training fit a prospectively fixed24-row
complete-answer curriculum while retaining its original8 successes? The causal
intervention combines a narrower training distribution with repeated exposure;
it does not isolate extra steps or prove architectural capacity. This is the
same ordinary-learning line, entering at count1/3; the closed protected/discrete
constructor, attribution and solver line stays closed.

[Pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093892615)
and [exact text](preregister.md) precede compute. The [fixed row list](qualification-rows.json)
contains8 original controls and16 initially unsuccessful rows: length4 reverse,
length8 forward, update reverse and reassert forward. Both job/home and both
source-swap arms are present, with selected q0/q2/q3/q4 wordings. All selected
banks are00: correlated exposed examples, not24 independent tasks or transfer.

## Fixed mechanism and endpoint

Accepted Source48/Generate64 initializes fresh ordinary AdamW at0.003 for
Potential and Generate coefficients. Context, Generate prototypes, bridge,
Cue, Prefix and exponential payloads remain fixed; no U or target-supplied source
selection. Source selection can still change through learned Potential.

Mode `reply_qualification` fixes seed1001,96updates,batch8 and one deterministic
24-row shuffle repeated32 times:768 episode draws,9,280 canonical answer/EOS
positions. Equal-nonempty-phase loss uses canonical prefixes during training.
One discarded8-episode gradient admission is charged separately as preparation.
Checkpoints24/48/72 are recovery only; checkpoint96 is the sole final endpoint.
There is no intermediate score selection, constructor gate or CE-based cutoff.

The baseline must reproduce exactly the original8 successful row indices.
Both baseline and final saved/reloaded native models are scored on the unchanged
full512 panel, using actual emitted prefixes,32 output tokens,128 total admission,
eight retained H4 lanes and4096 legal Generate actions. The subset report reads
those authenticated endpoint rows without another inference pass.

`qualified_fit` requires24/24 complete replies with EOS, including alloriginal8.
`model_keep` separately requires alloriginal8 retained and at least one new
complete reply on512; partial gain is not full subset qualification. Teacher
all-token correctness, entry/EOS, complete swap pairs and gained/lost IDs are
reported separately. The milestone remains256/512 followed by fresh>=40%; no
held-out draw is part of this experiment.

## Outcome decision

Before opening the result PR: if the result came out the other way, would the
next step differ? Yes. A qualified fit supports a larger fixed qualification;
partial retained gains support preserving that artifact without claiming24/24;
a regression/no gain rejects this candidate and stops this recipe without an
unchanged dose/rate/seed sweep. Failure is not a proof of all-family incapacity.
No successor starts before protected delivery and cleanup.

## Resources and validation

Full projection120min includes recovery, source, bootstrap/build/tests, model,
controls, full evaluation, retries, review, preservation, merge and hygiene.
A single RTX5090 is leased through `uor-pod` at$1.19/h; model2CPUthreads and
build4threads (stock bootstrap15 separately accounted). ModelRAM<=6GiB and
GPU<=4GiB; task pod transient<=16GiB. Local compilation is disabled to preserve
30GiB+128MiB disk floor; local transient<=768MiB, retained compressed<=512MiB.
Standing same-class60min ledger extension changes limit1474200028→1477800028ms,
retaining prior cumulative1467760243ms. Full elapsed session starts04:40UTC.

Exact source review at30c7d018825dcd4b3f2d2e7791af66ab4c192635 passed with no
blocking finding. It covers strict schedule/ID admission, legacy64 preservation,
active-only clipping/frozen families, saved native reload and full512 scoring
before the subset decision. Executed validation and model outcomes follow in
this record after completion; source review alone is not a model result.
