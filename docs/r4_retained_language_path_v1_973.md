# #973 R4 retained language-path generalization V1

Status: **INDEPENDENTLY FROZEN / IMPLEMENTATION IN PROGRESS / RESULT NOT RUN**

Issue: [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)

Authoritative freeze:
[issue comment 5491166627](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5491166627)

Policy: `R4RetainedLanguagePathV1`

## Decision

The preceding CPU recovery established that the bounded retained state changes
unseen next-token logits beneficially, but its complete 3.17M-parameter decoder
memorized only 4,096 training decisions and failed aggregate validation loss.
This successor changes the language-path data/parameter regime while preserving
the qualified exact-H4 read-before-write retained-attention law.

One matched two-arm experiment is authorized:

1. exact-H4 120-address retained attention; and
2. ordinary strictly causal full-prefix Q/K/V attention with RoPE and stable
   softmax.

The ordinary arm is an offline scientific positive control, not the intended
deployed architecture. This rung does not test an approximate or runtime-native
softmax replacement.

## Exact model and budget

Both arms use vocabulary 4,096, width 48, two pre-norm decoder blocks, SwiGLU
width 128, four heads of width 12, context 120, tied embedding/output storage,
RMSNorm, no dropout, and initialization seed 9,738. Each head contains three
whole R4 coordinate blocks.

The retained arm uses the existing separate key/value fields, exact H4
transport, learned four-scale decay, learned delta-write gates, read-before-write
attention over 120 transported addresses, and no RoPE. The ordinary arm uses
full-prefix causal RoPE Q/K/V attention and active learned per-head score and
output gains. Those eight scalars per block match the retained decay/write
parameters without adding inert weights.

Each arm therefore has exactly 252,160 learned parameters. At full context,
each has a 23,040-f32-value incremental K/V state per sequence; the retained
arm additionally records 240 occupancy bits and the ordinary arm has the
corresponding 240 per-layer valid-position bits. Shared-shape learned tensors
must begin byte-identically.

This is an equal parameter, state-capacity, data, and optimizer-dose comparison,
not an equal-operation claim. Across one 120-token sequence, retained attention
scores 115,200 address pairs while ordinary causal attention scores 58,080
position pairs.

## Frozen nonsealed population

Only the already materialized #1019 training view is eligible. The #1019
sealed-confirmation directory and every #1017/#973 sealed or fitted model
artifact remain forbidden.

- #1019 training-view CID:
  `blake3:bb090c4b87fb62e71ce073c2e4df525745109e71e0db3e9846852a696af5501e`
- tokenizer CID:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`
- source train-store CID:
  `blake3:c2752553b0b855a75685bb8ed16e113221e9a93575771e20046e95224b347e79`
- source development-store CID:
  `blake3:16e81a98cee6075fe740b7c612a2ef101a0bd790c1e664cb69af3a43f5aad2ca`

Training begins at token offset 149,996,595, the beginning of #1019 capacity
story 734,500 / source story 815,766, immediately after #1017's last training
source story. The slice is exactly 5,285,280 token IDs with CID
`blake3:8efeef090f1d729ad7782cd9f14a52561438a6cf256b58c62240f3fab83ae118`.
It forms 43,680 nonoverlapping 121-token windows and 5,241,600 distinct causal
next-token decisions per arm. Windows are presented once without replacement
in the BLAKE3 order of seed 9,738 and window ordinal.

Validation is the first 249,986 token IDs of the fresh #1019 development split,
CID `blake3:75b8d841a580211d55a81df04eee54807fec80549504cabc4238e5bd883bdfb8`.
It forms 2,066 nonoverlapping 121-token windows and 247,920 decisions. It begins
at source story 47,299, after #1017 development ended at source story 47,293.

The training dose is 20.7868 distinct decisions per parameter, versus about
0.00129 in the failed predecessor. No teacher logits, inherited weights,
checkpoint, test partition, or heldout reveal participates.

## Frozen optimization

Each arm receives exactly one deterministic epoch: batch 16, context 120, and
2,730 optimizer steps. AdamW uses betas 0.9/0.95, epsilon `1e-8`, weight decay
0.1, gradient clip 1.0, a 100-step linear warmup to `3e-4`, then cosine decay
to `3e-5`. Arms receive identical batches and execute in isolated model/
optimizer processes where the selected compute plan requires it.

Only initialization and the final checkpoint are scored. There is no
development-selected checkpoint, sweep, alternate seed, changed learning rate,
extra epoch, continuation, or scientific retry. Checkpoints exist only for
same-run interruption recovery and never change the frozen trajectory.

## Admission and result gates

Before optimization, focused checks must establish exact parameter/state
counts, active ordinary matching gains, byte-identical shared initialization,
finite nonzero gradients, strict-prefix causality, retained full/incremental
parity, data/dose identity, deterministic replay, and zero forbidden reads.

The ordinary positive control must improve its raw validation NLL by at least
1.0 nat and top-1 by at least 5.0 percentage points from initialization. If it
does not, the terminal is `INVALID_LANGUAGE_RECIPE`; the retained arm receives
no scientific verdict and this run does not authorize a parameter or optimizer
sweep.

Subject to that control, retained generalization requires the same 1.0-nat and
5.0-point improvements. Retained state must remain load-bearing: turning its
attention contribution off at final validation must cost at least 0.10 nat and
1.0 percentage point, or 2,480 of 247,920 decisions. Of those decisions,
245,854 have reachable prior state; the first decision in each window starts
empty by construction.

A full language-path pass additionally requires retained final NLL no more than
0.20 nat behind ordinary attention and retained top-1 no more than 2.0 points
behind it, with all causal, replay, finite-gradient, and isolation conditions
passing.

The outcome branches are binding:

- both arms qualify and retained is competitive: preserve the geometric
  checkpoint and next run only a fixed five-prompt, 64-token autonomous
  generation smoke;
- retained generalizes and uses state but misses competitiveness:
  `GENERALIZES_BUT_NOT_COMPETITIVE`; repair only decoder conditioning/readout
  before any scale increase;
- ordinary qualifies while retained misses generalization or state use: retire
  this compact group-addressed language path, not attention generally or the
  wider geometric programme;
- ordinary fails: `INVALID_LANGUAGE_RECIPE`; do not interpret retained results;
  and
- a compute/resource contract cannot be met: `UNAVAILABLE_COMPUTE`, never a
  model failure.

H4-specific superiority is deliberately `NOT_EVALUATED`; a third trained
scramble arm would add cost without deciding this rung.

## Measured compute contract

Before the only fit, disposable exact-step probes compare deterministic Apple
Accelerate CPU with four threads, CPU with eight threads, two concurrent
two-thread CPU workers, and sequential deterministic MPS. Each uses one warmup
and five measured training steps plus representative evaluation. CUDA and
external GPU execution are forbidden. The plan with the lowest measured
projected aggregate wall is binding; hardware, BLAS, threads, workers, timing,
memory, and deterministic-equivalence evidence are recorded.

The fit launches only if the 1.25-safety projection, including final validation
and checkpoint work, is at most 7,200 seconds and projected memory is at most
80% of the device budget. Runs projected beyond 15 minutes must emit durable
completed/total progress, ETA, checkpoints, and resume instructions. Crossing
the two-hour whole-process wall stops with `UNAVAILABLE_COMPUTE` and completed
step evidence.

This rung cannot establish H4 superiority, coherent generation, reasoning,
correctness, exact/table lowering, browser readiness, or release readiness.
Those remain downstream of its result.
