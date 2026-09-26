# chat-v0 R1b result — scaled dialogue learner, response-masked objective, and count screen

September 25, 2026. Branch `codex/canonical-address-routing-20260925`; task-packet base
`0b7b4c970e43176f909657247ebde5749a3558d0`, worktree HEAD at handoff `6cc050bc` (the lead
committed the R1a corpus pipeline during this run; it does not touch these files). This R1b work
is uncommitted on `6cc050bc`. All binaries were built with
`UOR_BUILD_SOURCE_COMMIT=0b7b4c970e43176f909657247ebde5749a3558d0`; each report's
`executable_sha256` binds the actual binary. **One development screen.** No model is promoted
to serving, and no capability, integer, geometry or energy claim is made. This is rung **R1b**
of the [CAR-LM direction update](car-lm-direction-update-2026-09-25.md#4-reordered-ladder-to-chat)
(§3–§4, §6) and consumes the R1a corpus ([corpus preparation](chat-v0-corpus-prep-2026-09-25.md)).

**Verdict.** The response-masked dialogue objective is implemented and its gradient reaches every
learned path, including the read/copy path. A 30-minute smoke fit at **5,429,826 parameters**
reduces held-out response-masked NLL from **7.3563 to 6.2190** over 113 steps / 231,424 sampled
targets. That is a real, decreasing learning signal but it is **3.753 nats worse than the order-5
Kneser–Ney count reference (2.4656)** on the exact same held-out window population, so the R1b
screen is **negative at this exposure**. The decisive secondary result is throughput: the scaled
joint-graph dialogue learner trains at **137–374 sampled targets/s** (measured under a concurrent
D8 cycle and in a lighter probe), so the direction update's projected 7–13 h/epoch is invalidated
for this width; a full epoch is **61–166 h** and cannot be reached on the declared machine budget.
Extending this run cannot close the margin, so R1's vehicle/scale is a decision for the lead.

## 1. Architecture decision and the serving-width boundary

The dialogue learner is the existing shared recurrent-memory graph (`JointModel`), because R1
keeps the **read/copy path** and R3 must quantize the result into the retained integer session
(`uor-r4-integer`), which executes exactly this graph. A dense transformer would satisfy the
"dense dialogue model" wording but cannot be quantized into that session, so it was rejected.

Reaching 5–15M in this graph is a width change, not an architecture change: the graph already uses
`config.width` everywhere. The parameter count is `9w² + 4235w + 4482` (vocab 4096, `read_width`
64, context 256). The chosen configuration is **width 576 → exactly 5,429,826 parameters**
(5,198,482 at width 560 would already clear 5M; 576 is the smallest round multiple of four above
5M). Width 1024 would give 13,778,306 and still fit the band.

The retained serving loader `uor-r4-integer::JointConfig::validate` accepts only width 128 or 256.
This task forbids modifying the serving crate, so the training crate gained a narrow
`JointModel::new_dialogue` constructor that skips that serving-side validation while leaving
`JointModel::new` (and the whole serving contract) unchanged. **The produced artifact is a
floating training checkpoint with 5,429,826 parameters and is not loadable by the current serving
loader.** R3 therefore needs a serving-side width extension before this candidate can be
quantized; that is a reported dependency, not a silent bypass.

`crates/uor-r4-training/src/dialogue.rs` (new) owns the campaign, objective, sampling, evaluation
and report paths. `crates/uor-r4-training/src/joint_model.rs` gained the `new_dialogue` constructor
and an `initialize` split; the serving crate is untouched.

## 2. Response-masked objective

`tokens.u16` is read through `MmapCorpusReader` (the canonical `UORT` container) and
`response_mask.u8` is read in lock-step; a length mismatch or a non-binary mask is rejected. For
every sampled lane, inputs are `tokens[start..start+256]`, targets are
`tokens[start+1..start+257]`, and the loss weight for target index `start+1+j` is
`response_mask[start+1+j]`. Loss is `sum(-ln p(target_t) · mask_t) / sum(mask_t)`, i.e. next-token
cross-entropy supervised on assistant content and its terminating EOS only. Read mode is enabled;
the vocabulary/copy mixture is untouched.

Verified at width 576: after one masked backward pass **all 21 named tensors are present, finite
and have at least one nonzero coordinate**, including `read.query/key/value`, `read.age`,
`read.no_read`, `update.*` and `copy.gate.*`. Focused tests cover the mask semantics, the
window/target boundary, the width band and a full-gradient audit (`cargo test ... dialogue::`,
5/5 pass). Train and held-out stores are distinct files (distinct SHA-256, asserted in code); the
fit samples only the train split and the held-out pass never calls `backward`.

## 3. Config and parameter count

| Field | Value |
|---|---|
| Vocabulary / tokenizer | 4096, shared byte-level BPE (unchanged) |
| Context | 256 |
| Width / read width / transport | **576** / 64 / quaternion |
| Parameter count | **5,429,826** |
| Optimizer | AdamW, lr 1e-3, weight decay 0.01, abs limit 1e6 |
| Batch / sampled targets per step | 8 lanes × 256 = 2048 |
| Data seed / model seed | 20260926 |

Width 576 is the minimum band point that is a round multiple of four above 5M. The exact count was
confirmed from the constructed model (`model.parameter_count()`), not only the shape formula.

## 4. Smoke fit (development screen)

Declared wall limit **1800 s**; one run on the train split.

| Quantity | Result |
|---|---:|
| Steps completed | **113** |
| Sampled targets | **231,424** (2048/step) |
| Train-batch masked NLL, first → last | **7.3953 → 6.2436** |
| Train-batch masked NLL, mean | 6.2911 |
| Held-out response-masked NLL, initial → final | **7.3563 → 6.2190** |
| Held-out change | **−1.1373 nats** |
| Mean / median step | 14.83 s / 14.94 s (range 10.85–19.30 s) |
| Throughput | **138 sampled targets/s** (loaded); 374 targets/s in the 5-step probe |
| Peak RSS | **2.30 GiB** (2.52 GiB peak footprint) |
| Wall, including constructor + two 512-window evaluations | 2008 s |

The loss curve decreases monotonically in aggregate; the per-step curve is
`fit-1/learning-curve.jsonl`, and the report is `fit-1/dialogue-fit.json`. The final parameters are
retained as `fit-1/parameters.f32` (21,719,304 B, SHA-256
`fcb5aa806ec2c1133d7c75658fb84ef718d6a2490e25cfdc872d2cab548a9b0e`).

The step time is measured with the shared host also running the D8 cycle; the 5-step timing probe
(much lighter load) measured **5.48 s/step**. Both are reported and both are used in the
projection below.

## 5. Count reference

Order-5 interpolated Kneser–Ney fitted on the chat-v0 train tokens (the same `KneserNey5Gram`
implementation as the existing `ngram-fit`/`ngram-evaluate` tooling; those commands are bound to
the evaluator-v2 population and cannot consume the chat corpus, so `dialogue-count` is a
response-masked driver over the same implementation — **NLL only, no decoder**).

| Quantity | Result |
|---|---:|
| Fit time / model size | 8.3 s (re-run 17.0 s) / 402,186,967 B |
| Retained types (orders 2–5) | 989,022 / 4,000,000 / 8,000,000 / 8,000,000 |
| Discount selected on **train** windows | **0.5** (grid 0.5/0.75/0.9; train NLL 1.8696) |
| Matched held-out response-masked NLL | **2.465643** over **84,328** targets |
| Full held-out response-masked NLL | 2.458183 over 3,942,189 targets |

The matched population is 512 windows spread across the whole held-out split (57 consecutive
wide-window samples would have stayed inside the first, response-heavy `smol-magpie-ultra.test`
region and mis-stated the split; the spread sampler was corrected before the retained run). The
matched figure is used for the margin because it is the population the learner also scores.

## 6. Screen result and margin

| Arm | Held-out response-masked NLL |
|---|---:|
| Count (order-5 KN) | **2.4656** |
| Dialogue learner (30-min smoke) | **6.2190** |
| Margin (count − learner) | **−3.7533 nats** (count wins) |

The R1 acceptance ("held-out response NLL beats count by a predeclared margin") is **not met at
this exposure**, so no greedy-output check or promotion follows. This is a development screen on a
0.28 %-of-an-epoch exposure; it does not retire the objective or the mechanism. It does show that a
few thousand float steps cannot close a 3.75-nat gap, and that exposure — not the objective — is
the binding constraint at this scale.

## 7. Full R1 fit specification (for the lead to launch)

> **Superseded throughput (2026-09-26).** The 61–166 h/epoch and 137–374 targets/s figures below were
> batch-8 measurements taken while the concurrent D8 cycle loaded the host. The [R1c throughput
> diagnosis](chat-r1c-result-2026-09-25.md#1-throughput-diagnosis) re-measured the same width 576 on an
> idle host: the fastest clean configuration is CPU + Apple Accelerate at batch 24 → 804–841
> targets/s (7.0 s/step), 27.3 h/epoch clean and 35.6 h/epoch sustained. R1c also shows the bounded
> curve saturating around the count level and does not justify the full fit. The table below is kept
> at its original artifact/budget scope.

Predeclared from the measured throughput. `batch 8 × context 256 = 2048` sampled targets/step;
one epoch over the 82,529,690-token train split is **40,298 steps**.

| Exposure option | Steps | Projected wall | Sampled targets | Epoch share |
|---|---:|---:|---:|---:|
| Bounded 6 h (fresh-load throughput, 5.48 s/step) | 3,942 | 6.0 h | 8,072,000 | 9.8 % |
| Bounded 6 h (loaded throughput, 14.83 s/step) | 1,457 | 6.0 h | 2,984,000 | 3.6 % |
| Bounded 13 h (fresh-load) | 8,540 | 13.0 h | 17,490,000 | 21.2 % |
| One full epoch (fresh-load, 5.48 s/step) | 40,298 | **61.3 h** | 82,529,690 | 100 % |
| One full epoch (loaded, 14.83 s/step) | 40,298 | **166.0 h** | 82,529,690 | 100 % |

- **The direction update's 7–13 h/epoch projection does not hold at width 576.** It was derived
  from ~3–4 k targets/s at width 256; the measured dialogue throughput is 137–374 targets/s.
- **RAM:** 2.30 GiB peak RSS at batch 8, inside the 12 GiB ceiling. Batch 16 would roughly scale the
  retained recurrent histories; it was not measured and is not pre-authorized by this run.
- **Checkpoints:** two `parameters.f32` writes are configured in the smoke campaign (21.7 MB
  each); for a long run, checkpoint every 500 steps (≈1.0 h fresh-load) and retain at most 8
  (≈174 MB).
- **Stop margins:** the 128 MiB physical storage stop margin is preserved; the SSD held ≈198.5 GiB
  free at run time. `max_process_seconds` stops new updates; final evaluation and parameter save
  need ~0.5–2 min of closeout.
- **Decision.** Because even a 13 h bounded run reaches only ~21 % of an epoch and the smoke
  trajectory is 3.75 nats behind count, extending this width-576 joint run is unlikely to change
  the R1 verdict. The lead should choose one of: (a) accept a bounded exposure as a partial R1 and
  measure the curve; (b) build the R1 vehicle as a configurable **dense** learner with far higher
  target throughput (as the direction update's "dense dialogue model" wording suggests) and keep
  the joint graph as the R3 quantization target; or (c) change scale. This report does not choose
  among them.

## 8. Panel reconciliation

The corpus template is `System:`/`User:`/`Assistant:`; the frozen panel uses
`<|user|>`/`<|assistant|>`. `dialogue-panel` renders the **38 development rows only** (fresh array
not opened) into the training template:

- single-turn: `User: {turn}\nAssistant: `
- memory: `User: {t1}\nUser: {t2}\nUser: {t3}\nAssistant: ` — the panel stores no intermediate
  assistant replies, so turns are emitted as consecutive `User:` blocks. This is recorded in the
  file's `note` and must be revisited before R4 memory scoring.

File: `docs/integration/chat-r1b-panel-requests-dev-2026-09-25.json` (SHA-256
`28c5ce327387b4eff90b7b36dd585fddb89c12bbb7ce7bfa5eff3b0e63e122c3`), 38 rows (core 20 / memory 10 /
refusal 8), panel file SHA-256 `10dfc0e7c529feecfa61e393b22a404ffee561339281abf5ea03f53964c6cfd2`.

## 9. Exact commands

```sh
# build (release, source-bound)
UOR_BUILD_SOURCE_COMMIT=0b7b4c970e43176f909657247ebde5749a3558d0 \
  ~/.cargo/bin/cargo build --release -p uor-r4-training --offline

# focused checks
UOR_BUILD_SOURCE_COMMIT=0b7b4c970e43176f909657247ebde5749a3558d0 \
  ~/.cargo/bin/cargo test -p uor-r4-training --offline --lib dialogue::   # 5/5 pass

BIN=/Users/casey.allard/uor-r4/target/release/uor-r4-training
LAB=/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1b

# count reference (order-5 KN, response-masked NLL; retains model.ng5)
$BIN dialogue-count $LAB/r1b-count-campaign.json $LAB/count-3

# declared 1800 s smoke fit (count_report bound to count-3 via r1b-fit-campaign.json)
/usr/bin/time -l $BIN dialogue-fit $LAB/r1b-fit-campaign.json $LAB/fit-1

# development panel requests in the training template (no fresh rows)
$BIN dialogue-panel <worktree>/docs/integration/chat-panel-v0-2026-09-25.json \
  $LAB/panel-requests-dev.json

# report seals
$BIN verify $LAB/count-3 && $BIN verify $LAB/fit-1 && $BIN verify $LAB/probe-fit-1
```

## 10. Artifact identities

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `fit-1/dialogue-fit.json` | `ba227de08cb9dc65681a91b03c1802210db91c17b56fb684ac7ed0a663530c9d` | 11,969 |
| `fit-1/parameters.f32` | `fcb5aa806ec2c1133d7c75658fb84ef718d6a2490e25cfdc872d2cab548a9b0e` | 21,719,304 |
| `count-3/dialogue-count.json` | `a48e64e9a015aadcc16a27e8505b38511c6014678a1bea351d2802307bf70f09` | 5,368 |
| `count-3/model.ng5` | `90f0a6574b9d27a196af0040be96e8006923055b2cd68d4a499804aed01b6859` | 402,186,967 |
| `probe-fit-1/dialogue-fit.json` | `7c583f81cd4d9a1cf3d3799051fde87f9bad3ed15d32ae200da573bc980fe849` | 11,142 |
| train `tokens.u16` / `response_mask.u8` | `4a554b0ef8be12344f21f1bd6bdc9faeeeddcef212a138a8772a8bf42604c4fa` / `09dc0fe5d0e2de7b5072e4586b5cb9ef3379856600ff90061c561a48826e75e7` | — |
| heldout `tokens.u16` / `response_mask.u8` | `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225` / `ed43a9eade79d84dc73be608fb788f1e7566021fbdbba35ce3e5eddbc4fce8b4` | — |

Full report documents are embedded in
[`docs/evidence/chat-r1b-2026-09-25.json`](../evidence/chat-r1b-2026-09-25.json).

## 11. Limitations

- **Development screen only.** 113 steps / 0.28 % of an epoch. Not an R1 pass, not a language,
  chat, geometry, integer or energy result.
- **Serving gap.** Width 576 is outside the retained serving loader's 128/256 contract; the
  artifact is a floating training checkpoint. R3 needs a serving-side width extension.
- **Throughput is contended.** Step times span 5.48–19.30 s; the host ran the D8 cycle
  concurrently. The projection reports both bounds.
- **Held-out population** is 512 spread windows (84,328 supervised targets) for the matched
  comparison; the full-heldout count number (3.94 M targets) is reported for reference.
- The count model is interpolated single-discount Kneser–Ney with count pruning, not
  modified Kneser–Ney; the discount was selected on train windows only.
- No greedy output was generated; the R1 "non-degenerate output" limb is NOT_RUN, correctly, since
  the likelihood criterion was not met.

## 12. Cost

| Item | Value |
|---|---|
| Release build (incremental, shared target) | 2 m 27 s first, 15–35 s after edits |
| Focused tests | 1 m 25 s (lib test build) |
| Count reference | 14.7 s wall (fit 8.3 s; re-run 17.0 s) |
| Smoke fit | 2008 s wall (1800 s update deadline + two 512-window evaluations + save) |
| Timing probe | 32 s |
| Peak RSS | 2.30 GiB |
| New SSD storage | count model 402 MB + parameters 21.7 MB + reports ≪ 1 MB |
| External / paid compute | none. One cargo process at a time; other host processes untouched. |

## 13. Next decision

The response-masked objective and its full gradient path are now demonstrated at 5.4 M
parameters, and the count reference for the chat corpus exists and is reproducible. The two
findings that should drive the next move are the **negative screen** (count wins by 3.75 nats at
0.28 % exposure) and the **throughput ceiling** (137–374 targets/s; 61–166 h/epoch). The lead
should either authorize a bounded full-R1 exposure to read the curve, or switch the R1 vehicle to
a configurable dense learner with a documented path back into the integer session, or change
scale. The sealed panel `fresh` array stays closed until design selection.
