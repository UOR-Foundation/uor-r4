# Stratified complete-reply fitting qualification

The owner requested a direct learnability check after the frozen512 headline
remained 8/512. The preceding [full-panel one-pass result](../m2-reply-gradient-2026-10-10/README.md)
regressed 8→2 despite lower teacher-prefix CE. The subsequent source/data assessment
found that all canonical answers fit vocabulary/context/output limits, but no
learned artifact demonstrates full-panel realizability. The old 512 answerability
reference is explicitly not a native prediction result.

## Question and pre-registration

Can ordinary Potential/Generate training fit a prospectively fixed 24-row
complete-answer curriculum while retaining its original 8 successes? The causal
intervention combines a narrower training distribution with repeated exposure;
it does not isolate extra steps or prove architectural capacity. This is the
same ordinary-learning line, entering at count 1/3; the closed protected/discrete
constructor, attribution and solver line stays closed.

[Pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093892615)
and [exact text](preregister.md) precede compute. The [fixed row list](qualification-rows.json)
contains 8 original controls and 16 initially unsuccessful rows: length 4 reverse,
length 8 forward, update reverse and reassert forward. Both job/home and both
source-swap arms are present, with selected q0/q2/q3/q4 wordings. All selected
banks are 00: correlated exposed examples, not 24 independent tasks or transfer.

## Fixed mechanism and endpoint

Accepted Source48/Generate64 initializes fresh ordinary AdamW at 0.003 for
Potential and Generate coefficients. Context, Generate prototypes, bridge,
Cue, Prefix and exponential payloads remain fixed; no U or target-supplied source
selection. Source selection can still change through learned Potential.

Mode `reply_qualification` fixes seed 1001, 96 updates, batch 8 and one deterministic
24-row shuffle repeated 32 times: 768 episode draws,9,280 canonical answer/EOS
positions. Equal-nonempty-phase loss uses canonical prefixes during training.
One discarded 8-episode gradient admission is charged separately as preparation.
Checkpoints 24/48/72 are recovery only; checkpoint 96 is the sole final endpoint.
There is no intermediate score selection, constructor gate or CE-based cutoff.

The baseline must reproduce exactly the original 8 successful row indices.
Both baseline and final saved/reloaded native models are scored on the unchanged
full 512 panel, using actual emitted prefixes, 32 output tokens, 128 total admission,
eight retained H4 lanes and 4096 legal Generate actions. The subset report reads
those authenticated endpoint rows without another inference pass.

`qualified_fit` requires 24/24 complete replies with EOS, including all original 8.
`model_keep` separately requires all original 8 retained and at least one new
complete reply on 512; partial gain is not full subset qualification. Teacher
all-token correctness, entry/EOS, complete swap pairs and gained/lost IDs are
reported separately. The milestone remains 256/512 followed by fresh >=40%; no
held-out draw is part of this experiment.

## Outcome decision

Before opening the result PR: if the result came out the other way, would the
next step differ? Yes. A qualified fit supports a larger fixed qualification;
partial retained gains support preserving that artifact without claiming 24/24;
a regression/no gain rejects this candidate and stops this recipe without an
unchanged dose/rate/seed sweep. Failure is not a proof of all-family incapacity.
No successor starts before protected delivery and cleanup.

## Resources and validation

Full projection 120 min includes recovery, source, bootstrap/build/tests, model,
controls, full evaluation, retries, review, preservation, merge and hygiene.
A single RTX5090 is leased through `uor-pod` at $1.19/h; model 2 CPU threads and
build 4 threads (stock bootstrap 15 separately accounted). Model RAM <=6 GiB and
GPU <=4 GiB; task pod transient <=16 GiB. Local compilation is disabled to preserve
30 GiB + 128 MiB disk floor; local transient <=768 MiB, retained compressed <=512 MiB.
Standing same-class 60 min ledger extension changes limit 1474200028→1477800028ms,
retaining prior cumulative 1467760243 ms. Full elapsed session starts 04:40 UTC.

Exact source review at 30c7d018825dcd4b3f2d2e7791af66ab4c192635 passed with no
blocking finding. It covers strict schedule/ID admission, legacy 64 preservation,
active-only clipping/frozen families, saved native reload and full 512 scoring
before the subset decision. Optimized CUDA build passed in 232.170 s (peak 3,576,160 KiB); all 6 focused tests
passed with 0 ignored in 25.234 s command time. Bootstrap separately took 200 s
(build 132 s, parity 52 s: 37 pass / 3 ignored). Producer executable SHA256
`71fe01d312bd37549c68a03d9d54d01bf063817fc5c8b67aebe4b06f88a27713`.
Actual model outcome follows after execution; checks alone are not a result.

A concurrent local storage decrease crossed the floor before model launch. Hash-verified
redundant restored cloud components were removed, and Git sparse checkout omits only
the tracked research/ tree from this owned worktree; full native source/current docs
remain present and omitted tracked material remains on main. This routine reversible
workspace adjustment restores the floor without touching another lab's material.
The admission hygiene pass completed and preserved other labs' active folders.
