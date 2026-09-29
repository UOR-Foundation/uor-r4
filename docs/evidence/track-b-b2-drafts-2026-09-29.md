# Track B B2 draft preservation — September 29, 2026

This record preserves source engineering while the shared Candle model build is
blocked by library-load policy. All five drafts are unregistered. Only the
std-only analytic cost estimator was compiled and tested. Source preparation,
Candle transfer/hybrid/fit tests, fitting, independent reload, model parity,
generation and model-quality measurements from these drafts remain **NOT_RUN**.
No model gain or B2 acceptance is inferred from source review or cost fixtures.

## Frozen source identities

Paths below are under `crates/uor-r4-training/drafts/`. The fit and hybrid received
bounded independent source review; that review establishes no executed behavior.

| Source | SHA-256 |
| --- | --- |
| [track_b_source_data.rs](../../crates/uor-r4-training/drafts/track_b_source_data.rs) | `3c4dec321d7aa17ca4ac862f1cc3232a518bec2639e700b9029418ad4b0f335e` |
| [track_b_transfer.rs](../../crates/uor-r4-training/drafts/track_b_transfer.rs) | `ad3936b6fcaa9b9714ac1f45c2d4700638fe131cebc9c42c9673618408916cf5` |
| [track_b_fit.rs](../../crates/uor-r4-training/drafts/track_b_fit.rs) | `9ab9e219dc19674c9c6277407e2511c26a9a1547438ea13fe64a9eb75eac21e3` |
| [track_b_hybrid.rs](../../crates/uor-r4-training/drafts/track_b_hybrid.rs) | `fb69c1cdbf7b3b196853a0e7139c40f33e08960e8f601db90fb921dc75675871` |
| [track_b_cost.rs](../../crates/uor-r4-training/drafts/track_b_cost.rs) | `75b94a7e760280263c4e4f27c4d06aa54bb28b4df8e72653ef8820e2abc9978d` |

The [draft README](../../crates/uor-r4-training/drafts/README.md) records recovery
provenance and integration boundaries. The estimator binds its analytic model
and harmonic source snapshots in its header; its figures do not automatically
apply after those implementations change.

## Executed accounting check

The preserved [receipt](track-b-cost-check-2026-09-29/receipt.json) identifies the
cost source hash above, start `2026-09-29T21:59:29Z`, end
`2026-09-29T21:59:30Z`, compile exit 0 and test exit 0. The
[runner](track-b-cost-check-2026-09-29/run.sh) uses std-only `rustc --test`;
Cargo and model execution were not used.

| Phase | Result | Measured wall time | Maximum RSS |
| --- | --- | ---: | ---: |
| [Compile](track-b-cost-check-2026-09-29/compile.log) | PASS | 1.30 s | 121,520,128 B |
| [Accounting fixtures](track-b-cost-check-2026-09-29/tests.log) | 9 passed; 0 failed/ignored | 0.06 s | 2,375,680 B |

Fixtures cover packed dimensions, nine-head state ownership, quadratic scaling,
tied weight counting, degree versus GEMM width, batch scaling, hybrid history and
selection accounting, capture early exit, and invalid/overflow requests. They
do not exercise Candle tensors or validate gradients, numerical parity, training
or inference. The frozen estimator header's NOT_RUN label is superseded only by
this exact fixture receipt. These measured RSS/timing figures belong to the
accounting program, not a model.

Coordinator ledger checkpoint 6 records 1,338,090 ms for the surrounding work
interval and cumulative use of 789,197,650 / 1,130,000,000 ms. The measured 1.36 s
compile/test time is included in that charge, not added again; model work in
this interval is zero.

## Analytic costs

All tables use SmolLM2-135M geometry: width 576, nine query heads, three native KV
heads, head width 64, MLP width 1,536, vocabulary 49,152 and 30 layers. Batch is
one. One MAC means one multiply-accumulate, not one FLOP. MiB means 2^20 bytes.
All-layer figures describe a hypothetical complete replacement; the isolated
fit draft does not execute such a model.

**Pure harmonic recurrence.** For packed dimension F, persistent matrix-plus-mass
state is `4 * 9 * F * (64+1)` bytes per layer. The core is
`9 * F * (2*64+1)` MAC per layer per position: update/query M and query z. Mass
updates, feature construction, projection, normalization and output division are
additional. The key maps are independent after native GQA repetition, so dividing
state by three would change the implemented parameter sharing.

| d | L | F | State B/layer | State B/30 layers | Core MAC/layer/position |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 1 | 17 | 39,780 | 1,193,400 | 19,737 |
| 16 | 2 | 152 | 355,680 | 10,670,400 | 176,472 |
| 16 | 3 | 952 | 2,227,680 | 66,830,400 | 1,105,272 |
| 32 | 1 | 33 | 77,220 | 2,316,600 | 38,313 |
| 32 | 2 | 560 | 1,310,400 | 39,312,000 | 650,160 |
| 32 | 3 | 6,512 | 15,238,080 | 457,142,400 | 7,560,432 |

This is logical persistent state, not a peak-memory estimate. The detached
full-prefix implementation also receives the full causal mask and Q/K/V,
materializes projected sequences, and retains outputs before concatenation.
Its out-of-place update can hold old M, an outer product and new M simultaneously.
It is not an incremental constant-memory API. The estimator distinguishes
logical read/write passes from hardware traffic: eager core reads are `3*(M+z)`
and writes `2*(M+z)` per batch step; a one-read/one-write fused implementation is
a counterfactual, not implemented bandwidth evidence.

**Current quadratic forward.** One F32 `[1,9,T,T]` score tensor is `36*T^2`
bytes; the shared U8 mask is `T^2` bytes. The following core MAC counts are per
layer per processed token in a full-window forward, including full-square
GEMMs before causal masking. Dense uses `9*T*(64+64)`; projected harmonic
training uses `9*T*(d+64)`. Multiply by T for one complete window. The latter
is independent of L for GEMM cost, but polynomial
scalar work changes with degree. Training does not construct F packed features.

| Context T | One F32 score tensor | U8 mask | Dense core MAC/token | d16 core MAC/token | d32 core MAC/token |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 512 | 9,437,184 B (9 MiB) | 262,144 B | 589,824 | 368,640 | 442,368 |
| 2,048 | 150,994,944 B (144 MiB) | 4,194,304 B | 2,359,296 | 1,474,560 | 1,769,472 |
| 8,192 | 2,415,919,104 B (2,304 MiB) | 67,108,864 B | 9,437,184 | 5,898,240 | 7,077,888 |

One score tensor is not the number of live tensors or an autograd memory bound.
Full vocabulary logits additionally occupy 96/384/1,536 MiB at these contexts
for B=1; isolated post-WO capture avoids them. Native KV history across 30 layers
would occupy 22.5/90/360 MiB. Hybrid support correction also needs retained or
recomputed selected keys/values, beyond harmonic state.

**Retained full-width matrices.** Parameter counts below equal the indicated
forward matrix MAC counts per position; payload bytes are F32. These are logical
matrix visits, not measured DRAM reads.

| Matrix scope | Parameters / forward MAC | Stored payload B |
| --- | ---: | ---: |
| Q/K/V, one layer | 552,960 | 2,211,840 |
| WO, one layer | 331,776 | 1,327,104 |
| Three MLP maps, one layer | 2,654,208 | 10,616,832 |
| Vocabulary projection, once | 28,311,552 | 113,246,208 |
| All 30 layers plus vocabulary matrix | 134,479,872 | 537,919,488 |

The tied embedding/head is stored once. Including all RMS gains, frozen payload
is **538,060,032 B**. Rank-4 Q/K/V LoRA adds 10,752 parameters/MAC per layer;
independent Q/K maps add 18,432 at d16 or 36,864 at d32. Thus harmonic trainable
counts are 29,184 or 47,616 per layer. Parameters, gradients and two Adam moments
alone occupy 466,944 or 761,856 B; backend temporaries, proposed optimizer values,
checkpoint buffers and frozen weights are additional.

The streaming isolated fit recomputes teacher capture and base QKV per window.
At zero-based layer 15, B=1 and T=512, teacher capture alone costs analytically
**32,463,912,960 MAC/window**, before reconstructed QKV, student forward or
backward. The estimator deliberately provides no fixed-multiple estimate of
backward/optimizer cost. Future receipts must count capture calls/positions,
student forward positions, backward steps and evaluation positions separately.

## Implementation and qualification limits

The fit draft supports isolated dense-control and harmonic arms only. It uses
actual dense-teacher post-WO targets, frozen full-width matrices, rank-4/alpha-4
Q/K/V LoRA, explicit paired seed and at most 500 scheduled updates. Constructor
and evaluation documents are separated. After the fixed update schedule, the
draft reconstructs the initial parameter set and evaluates both it and the final
parameter set, without intermediate development-driven selection. A
positive harmonic baseline requires at least 50% reduction in aggregate
target-energy-normalized squared error; dense zero-baseline reduction is
non-applicable. This is an implemented decision rule, not a passed model gate.
The proposed zero/constructor-mean predictor and fixed norm-bin diagnostics are
missing; they do not create new owner acceptance gates.

The hybrid draft assigns raw `1/(rank+1)` mass to supplied ranked support and
replaces the selected harmonic contributions in both numerator and denominator.
It neither selects support nor integrates with the fit arm. The external
selector's full scan/sort, key/value history, host validation copies, and dense
quadratic helper allocations remain costs. A normalized sparse output cannot be
substituted for raw mass. No O(k) selection, indexed implementation, composed
model, cancellation stability result or recurrent/quadratic parity is established.

Remaining execution is **NOT_RUN**: source-token preparation; Candle compilation
and draft fixtures; shared-model parity after the library-load-policy block;
isolated fits and independent checkpoint reload/evaluation; integrated hybrid
and actual composed-student inputs; all-layer held-out NLL/KL and matched dense
comparison; model RSS, tokens/s, energy and end-to-end cost. Analytic tables
establish none of those outcomes. Resource admission, cumulative accounting,
real OS observation, hard watchdogs and exclusive report claim/seal remain
required in the future runner. This preservation record changes no policy,
owner gate, strategic goal, or resource allowance.
