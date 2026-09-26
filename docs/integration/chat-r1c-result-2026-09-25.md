# chat-v0 R1c result — throughput diagnosis and bounded curve: the width-576 vehicle does not reach the count margin

September 26, 2026. Branch `codex/canonical-address-routing-20260925`; task-packet base
`a212c45de43ca47de3aab5f24485f31cd697240a`, worktree HEAD at handoff `a212c45d` (this R1c work is
uncommitted; the lead commits). **Development curve only.** No model is promoted to serving, and no
capability, integer, geometry or energy claim is made. This is rung **R1c** of the
[CAR-LM direction update](car-lm-direction-update-2026-09-25.md#4-reordered-ladder-to-chat) (§4, §6).

**Verdict.** The throughput ceiling of R1b is diagnosed and partly fixed: the fastest clean
configuration is **CPU with Apple Accelerate at batch 24 → 804–841 sampled targets/s** (7.0 s/step),
versus the R1b smoke's 137–374 targets/s. That is a **4.3–6.1×** improvement, and it is real, but it
still gives **27.3 h per epoch** clean and **35.6 h** sustained in the bounded run. In the bounded
≤3 h run the width-576 (5,429,826-parameter) response-masked learner reached held-out NLL
**3.2324** at 6,592,512 sampled targets (7.99 % of one epoch) against the order-5 KN count reference
at **2.4656** — still **+0.7668 nats** above count. The fitted learning curve saturates at
**2.34–3.0 nats**, i.e. around the count level rather than 0.10 nats below it; the optimistic
power-law crossing is **232 h** and the realistic estimate is unreachable. Per the predeclared rule
the full width-576 full-R1 fit is **not justified**, and the recommended next action is a **vehicle
change** (concrete options and projections in §6).

## 1. Throughput diagnosis

Profile of `dialogue-fit` (width 576, context 256, 5,429,826 parameters). Every step is a 256-position
sequential unroll of the shared recurrent graph; the per-step phase breakdown is instrumented in the
training crate. Clean idle-host runs, explicit binary paths.

| Device / build | batch | step s | targets/s | backward s (share) | forward s |
|---|---:|---:|---:|---:|---:|
| CPU default (no Accelerate) | 8 | 3.99 | 459.6 | 3.28 (82 %) | 0.58 |
| CPU default | 16 | 5.80–5.91 | 644–656 | 4.70–4.80 (81 %) | 0.96–0.99 |
| CPU default | 24 | 7.62–7.80 | 742–761 | 6.10–6.26 (80 %) | 1.37–1.39 |
| CPU default | 32 | 13.14 | 591.5 | 10.77 (82 %) | 2.21 |
| **CPU + Accelerate** | 8 | 3.58 | 522.1 | 3.04 (85 %) | 0.40 |
| **CPU + Accelerate** | 16 | 5.29 | 727.2 | 4.43 (84 %) | 0.70 |
| **CPU + Accelerate** | 24 | **6.93–7.25** | **804–841** | 5.84–6.11 (84 %) | 0.94–0.97 |
| **CPU + Accelerate** | 32 | 9.39 | 835.3 | 7.97 (85 %) | 1.27 |
| Metal | 8 | 5.11 | 281.3 | 4.34 (85 %) | 0.56 |
| Metal | 16 | 6.35–6.64 | 509–581 | 5.38–5.55 (84 %) | 0.64–0.78 |
| Metal | 24 | 8.51–8.73 | 671–686 | 7.28–7.47 (86 %) | 0.77–0.80 |
| Metal | 32 | 10.79 | 727.8 | 9.15 (85 %) | 1.01 |
| Metal | 48 | 22.23 | 540.4 | 18.31 (82 %) | 1.51 |
| Metal | 64 | 37.88 | 426.4 | 32.42 (86 %) | 2.08 |
| CPU default, contended (D8 cycle) | 8 / 16 / 32 | 13.78 / 21.88 / 14.28 | 136 / 176 / 537 | 80 % | — |

**Where the time goes.** In every viable configuration **backward is 80–86 %** of step time, forward
13–16 %, optimizer 2–3 %, data loader ≈0 %. In the bounded run the totals were backward 8047 s, forward
1286 s, optimizer 170 s, data 9 s (backward 84.6 % of kernel time). The step is **not FLOP-bound**:
per-step wall time is nearly flat from batch 8 to batch 24 (op-count-bound over ~256 sequential
positions of tens of tiny ops each), so targets/s scales with batch until compute and thermal pressure
reverse it (CPU batch 32, Metal batch 48+). Only one core is busy (≈95 % of one core), while seven sit
idle.

**Device conclusion.** CPU+Accelerate at batch 24–32 is the fastest correct path. Apple Accelerate is
~10 % faster than candle's portable gemm at batch 24 and slower at small batch (BLAS call overhead on
many small matmuls). Metal peaks at batch 32 with 728 targets/s — below CPU — and its advantage is
memory (host RSS ~0.3 GiB vs ~5.6 GiB) rather than throughput. The R1b 137–374 targets/s figures were
batch-8 measurements taken while the concurrent D8 cycle saturated the host; the clean batch-8 rate is
460 targets/s. Sustained throughput in a real run (8.88 s/step, 644.5 targets/s) is ~20 % below the
short-burst clean rate because of periodic evaluation and sustained thermal/contention.

## 2. Bounded curve

Declared wall limit **10,200 s** (≤3 h); one run at the fastest config (CPU+Accelerate, batch 24 =
6144 sampled targets/step, context 256). Held-out evaluation on the **matched 512-window population**
(stride 4, 84,328 supervised targets) every 50 steps, overlaid with the count lines.

| step | wall s | held-out NLL | | step | wall s | held-out NLL |
|---:|---:|---:|---|---:|---:|---:|
| 0 | 20 | 7.3563 | | 550 | 4925 | 3.8703 |
| 50 | 406 | 6.1683 | | 600 | 5384 | 3.7536 |
| 100 | 838 | 6.1345 | | 650 | 5807 | 3.6677 |
| 150 | 1262 | 5.9788 | | 700 | 6350 | 3.5792 |
| 200 | 1727 | 5.6776 | | 750 | 6996 | 3.5132 |
| 250 | 2309 | 5.1498 | | 800 | 7633 | 3.4649 |
| 300 | 2756 | 4.7675 | | 850 | 8112 | 3.3974 |
| 350 | 3202 | 4.5530 | | 900 | 8568 | 3.3473 |
| 400 | 3606 | 4.3576 | | 950 | 9025 | 3.3162 |
| 450 | 4040 | 4.1842 | | 1000 | 9504 | 3.2664 |
| 500 | 4470 | 4.0178 | | 1050 | 9997 | 3.2399 |

Train-batch masked NLL per 100-step block: 6.279, 5.905, 5.125, 4.550, 4.208, 3.845, 3.661, 3.485,
3.401, 3.307, 3.201 (steps 1–1073). Held-out tracks the train curve closely; no train/held-out gap.

| Quantity | Result |
|---|---:|
| Steps completed / sampled targets | **1073** / **6,592,512** (7.99 % of one epoch) |
| Wall | 10,228 s (10,200 s update deadline + final evaluation + save) |
| Mean step / sustained throughput | 8.88 s / **644.5 targets/s** (step-only 691.9) |
| Train masked NLL, first → last | 7.3953 → 3.2616 |
| Held-out response-masked NLL, initial → final | **7.3563 → 3.2324** (−4.1239) |
| Count (order-5 KN, matched 512 windows) | **2.4656** |
| Margin (count − learner) | **−0.7668 nats** (count wins) |

The initial held-out NLL (7.3563) and population reproduce R1b exactly.

## 3. Extrapolation

Fit on the logged held-out points, threshold = count − 0.10 = **2.3656**. The early transient makes
the all-point power-law fit degenerate (no identifiable floor). The three model families disagree, so
windowed fits are reported.

| Window | Power-law-with-floor | Log-linear | Exponential-with-floor |
|---|---|---|---|
| all (50–1050) | floor −55.0 (degenerate), 2182 steps → 5.4 h | 2105 steps → 5.2 h | floor **2.904** → unreachable |
| second half (550–1050) | floor **2.562** → unreachable | 2522 steps → 6.2 h | floor **2.988** → unreachable |
| last 8 (700–1050) | floor **2.342**, **94,047 steps → 232 h** | 2901 steps → 7.2 h | floor **2.936** → unreachable |

Every **identified floor** model saturates at 2.34–3.0 nats, bracketing the count reference itself
(2.4656); the threshold 2.3656 is not cleared. The only family that projects inside the budget is the
floor-less log-linear form (6.2–7.2 h), and it is the least physically appropriate because it lets NLL
fall without bound. Wall projections use the run's sustained 8.88 s/step; at the clean 7.0 s/step they
are ~20 % smaller and still do not rescue the floor models.

**Reading:** at width 576 the learner converges toward the count level, not 0.10 nats below it. The
binding constraint is therefore not only exposure; it is the model's capacity/mechanism at this width,
compounded by a throughput ceiling that puts one epoch at 27–36 h.

## 4. Predeclared decision rule and outcome

**Rule (frozen before the run, `chat-r1c-predeclaration-2026-09-25.json`).** The full R1 fit is
justified only if the primary power-law-with-floor fit projects the threshold 2.3656 within **≤12 h**
at the measured fastest clean rate; a fitted floor at/above the threshold, or a projection beyond the
budget, means a vehicle change.

**Outcome: NOT JUSTIFIED. Vehicle change required.** The bounded run did not reach the threshold
(3.2324 vs 2.3656). On the bent second half the power-law floor is 2.562 ≥ threshold (no crossing) and
the exponential floor is 2.988; the optimistic last-8 power-law floor, 2.342, still requires 94,047
steps ≈ **232 h**. The only sub-12 h projection is the floor-less log-linear form. Under the
predeclared conservative reading the current vehicle cannot reach the R1 margin within a feasible
budget.

## 5. Serving-width gap (R3 dependency)

The retained integer loader `uor-r4-integer::JointConfig::validate` accepts only width 128 or 256.
The trained width-576 artifact is a **floating training checkpoint and is not loadable by the current
serving loader**. R3 needs a serving-side width extension before this candidate could be quantized.
The serving crate was not modified in this run.

## 6. Recommended next action

Change the R1 vehicle. The measured evidence motivates both a throughput change and a capacity change,
and they should be addressed together.

1. **Immediate, low-risk execution fix — reuse CPU gradient sharding.** The joint path already
   implements `cpu_gradient_shards`; the dialogue driver does not. At batch 24, 84 % of step time is
   backward on one core while seven are idle. Splitting the batch across four CPU workers (batch 6
   each), then forming the weighted mean before one AdamW update, is projected at ~2.4× (60 % parallel
   efficiency) → **~1600–2000 targets/s, epoch ~11–14 h** — the first configuration that makes a
   one-epoch exposure feasible. This does **not** change the fitted floor (2.34–3.0), so it is an
   enabler, not the fix.
2. **Recommended structural change — remove the sequential op-count bottleneck and raise capacity.**
   Replace the 256-position sequential unroll with a parallelizable sequence block that stays in the
   quantizable joint family (for example a chunked parallel-scan linear recurrence, or a chunked
   attention/convolution mixer): sequential depth falls from 256 to O(chunks), which targets the
   measured 84.6 % backward cost, and the same block can carry more capacity. First step is a bounded
   throughput probe of one block; no rate is claimed before measuring. **Target ≥2,500 targets/s** so
   one epoch fits in ≤9 h, then a fresh R1c-style curve. Escalate capacity toward the direction
   update's 5–30 M band (width 1024 = 13.78 M in the current graph) once the fast path exists.
3. **Objective/data change — not indicated.** The response-masked objective decreases monotonically,
   the read/copy gradient path is present, and held-out tracks train; the floor is capacity/mechanism
   limited, not objective limited. Retain the objective and the chat-v0 corpus.
4. A **smaller width is contraindicated**: it cannot be loaded by the integer session beyond 256, and
   it reduces capacity while the curve already saturates at the count level.

## 7. Source changes (training crate only; serving crate untouched)

`crates/uor-r4-training/src/dialogue.rs`:

- optional device argument `dialogue-fit CAMPAIGN NEW_ROOT [cpu|metal]`;
- optional campaign field `eval_every` (serde default `0` = historical behaviour) writing periodic
  held-out NLL to `heldout-curve.jsonl`;
- per-step phase timing (`data`/`forward`/`loss`/`backward`/`optimizer`) in the curve and report;
- batch validation widened from `1..=16` to `1..=64`;
- report now records `requested_device`, `actual_device`, `build_features`, phase means and
  `sampled_targets_per_second`.

Focused tests: `cargo test -p uor-r4-training --lib dialogue::` → **5/5 pass** (including the width-band,
window-boundary, mask-semantics and full-gradient-audit tests).

## 8. Exact commands

```sh
# builds (shared target cache)
UOR_BUILD_SOURCE_COMMIT=a212c45de43ca47de3aab5f24485f31cd697240a \
  ~/.cargo/bin/cargo build --release -p uor-r4-training --offline
UOR_BUILD_SOURCE_COMMIT=a212c45de43ca47de3aab5f24485f31cd697240a \
  ~/.cargo/bin/cargo build --release -p uor-r4-training --offline --features metal,cpu-accelerate

# focused checks
~/.cargo/bin/cargo test --offline -p uor-r4-training --lib dialogue::      # 5/5 pass

# throughput probes (8 optimizer steps, tiny 16-window eval, explicit binary)
$BIN dialogue-fit <probe>.json <new-root> {cpu|metal}

# bounded curve (<=3 h update deadline) at the fastest config
BIN=/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1c/bin/uor-r4-training-metalacc
$BIN dialogue-fit .../r1c/bounded.json .../r1c/bounded-1 cpu
$BIN verify .../r1c/bounded-1
```

## 9. Artifact identities

| Artifact | SHA-256 / blake3 | Bytes |
|---|---|---:|
| `bounded-1/dialogue-fit.json` | `f3c5cee8a8e20ea60fc93ac20f3ee11e53c2c12a35783c2182aeb55021749056` (blake3) | 12,689 |
| `bounded-1/heldout-curve.jsonl` | `1aa6be007b3c78a9ec05a4717691eea443a7084dde1809079194a95a14fa85c2` (blake3) | 2,696 |
| `bounded-1/learning-curve.jsonl` | `ba04c4f39734c3f6bc719379802cbdf325a576d2dd6a002d393e097737d5b074` (blake3) | 1,460,861 |
| `bounded-1/parameters.f32` | `7f0938af379cd4a6037bd723b57ca5caeedde087cc0ad6799c087efc94dc02e2` (sha256) | 21,719,304 |
| CPU-default binary | `518d1a9cdba46193fca73189f845679cff69ffb58ca0bd3e86cc6dd42c18fd7c` | — |
| CPU+Accelerate binary used for the curve | `8f2985aafdbf2a2187b6f38f49cf0b9beb0443de57e9f7c1884a197d243703b0` | — |
| count model `count-3/model.ng5` | `90f0a6574b9d27a196af0040be96e8006923055b2cd68d4a499804aed01b6859` | 402,186,967 |

Full documents and the 22-row throughput table are embedded in
[`docs/evidence/chat-r1c-2026-09-25.json`](../evidence/chat-r1c-2026-09-25.json).

## 10. Corrections to the R1b full-fit specification

The [R1b §7 projection](chat-r1b-result-2026-09-25.md#7-full-r1-fit-specification-for-the-lead-to-launch)
(61–166 h/epoch; 137–374 targets/s) is **superseded** by the corrected clean throughput: the smoke's
figures were batch-8 under the concurrent D8 cycle. Corrected full-fit table (batch 24 = 6144
sampled targets/step, 13,433 steps/epoch):

| Exposure option | Steps | Projected wall | Sampled targets | Epoch share |
|---|---:|---:|---:|---:|
| Fastest clean (CPU+Accelerate, 7.0 s/step) | 6,171 | 12.0 h | 37,915,000 | 45.9 % |
| Sustained bounded (8.88 s/step) | 4,865 | 12.0 h | 29,890,000 | 36.2 % |
| One full epoch (clean) | 13,433 | **27.3 h** | 82,529,690 | 100 % |
| One full epoch (sustained) | 13,433 | **35.6 h** | 82,529,690 | 100 % |

A 12 h budget therefore reaches only 36–46 % of one epoch at the fastest measured configuration, and
the bounded curve shows the learner saturating around the count level, not below it.

## 11. Limitations

- **Development curve only** (7.99 % of an epoch). Not an R1 pass, not a language, chat, geometry,
  integer or energy result. The mechanism and objective are not retired; the negative is scoped to
  this width, recipe and exposure.
- **Extrapolation uncertainty.** The three model families disagree; the floor is only weakly
  identified. The decision relies on every identified floor model agreeing that the threshold is not
  cleared within budget, and on the in-run fact that 3.2324 ≫ 2.3656 at 8 % of an epoch.
- **Sustained vs burst throughput.** Headline clean rates were re-measured on an idle host, but the
  bounded run's sustained 8.88 s/step is ~20 % slower; both are reported.
- **Contention.** Early probe rows ran while two joint-fit processes were active; they are retained as
  `default-contended` and are not used for the headline table.
- **Cross-backend reproducibility** is not claimed: Accelerate BLAS changes floating-point summation.
- The training-crate change is uncommitted; the artifact hashes bind the executed binaries, and the
  `dialogue.rs` SHA-256 is recorded in the evidence file.

## 12. Cost

| Item | Value |
|---|---:|
| Release build, default features | 33 s (incremental, shared target) |
| Release build, `metal,cpu-accelerate` | 3 m 13 s (incremental) |
| Focused tests | 17 s executed |
| Throughput probes (~22 short fits) | ~25 min wall |
| Bounded curve | 10,228 s wall |
| Peak RSS, bounded run | ~5.6 GiB host |
| New storage | parameters 21.7 MB + curves ~1.5 MB + probes <100 MB |
| External / paid compute | none; no CUDA; serving crate untouched |

## 13. Next decision

Do not launch a full width-576 full-R1 fit. Adopt the vehicle change in §6: first wire CPU gradient
sharding into the dialogue driver and measure it, then replace the 256-position sequential unroll with
a parallelizable, higher-capacity block that keeps a documented quantization path into the integer
session, and re-run a fresh bounded R1c curve on the new vehicle. The sealed chat-panel `fresh` array
stays closed until design selection.
