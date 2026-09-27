# Cycle 4: a capacity-matched, parallel-trainable geometric stack

2026-09-27 · Claude lab track · References #820, #973 · Work card: [#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851700710)

**Status.** An evidence note from the lab track, not a decision record. Run records: [cycle-4 packet](../evidence/geometric-stack-cycle4-2026-09-27/README.md). Labels: **Measured** (Rust runs in the lab sandbox, seeds stated), **Derived**, **Literature**, **Hypothesis**. Results are development measurements on code; nothing here is a final holdout or a language qualification.

## 0. Findings

*(Filled in when the runs finish.)*

## 1. Why capacity, and why a parallel layout

**The retained native model is small and sequential.** *Derived* from its shape inventory (`uor_r4_integer::config::JointConfig::shapes`):
- one gated recurrent layer and one 64-wide read at width 256;
- 1,678,466 parameters, of which 625,794 are not the embedding or output bias;
- trained position by position, because each step's read depends on the state that the same step updates.

**The retained reference is four times larger.** #1017 is a Llama-shaped transformer: 6 layers at width 288, 6 heads, SwiGLU 768, tied embedding, 7,155,360 parameters (5,975,712 outside the embedding).
- On the 233,472-target comparison tail it scores **1.574024** nats per token.
- The native continuation finals score **1.975–1.996**, 0.40 nats behind, and produced 0/5 acceptable prose (*Measured*, [current state](current-state.md)).

**No experiment has separated capacity from mechanism.** The main line's campaigns held the native model's size fixed. The question this cycle asks is:
- at #1017's parameter count and on identical data, does a native geometric model close the gap to an ordinary transformer?
- or does its mechanism cost quality?

Answering it needs a native model that can be made deeper. That in turn needs training that processes a whole window at once rather than one position at a time.

## 2. The geometric stack

`crates/uor-r4-training/src/geometric_stack.rs`. A stack of pre-norm residual layers. Each layer is one temporal mixer followed by a SwiGLU MLP. `pattern` picks the mixer of each layer: `r` for the recurrence, `a` for the read.

The default `rrarra` has two recurrence layers for each read, the ratio of Griffin (*Literature*, arXiv 2402.19427). There the recurrent layer is a real gated linear recurrence and the read is local attention.

### 2.1 Quaternion transport recurrence (`r`)

Each lane of four channels carries a quaternion state:

```text
h_t = lambda_t (u_t * h_{t-1}) + sqrt(1 - lambda_t^2) c_t
```

- `*` is the Hamilton product.
- `u_t` is a unit quaternion computed from the input. The implementation normalizes `raw_t / sqrt(|raw_t|^2 + 1e-6)`, so `u_t` is unit up to that epsilon. `raw_t` is a learned linear map of the normalized input plus a bias that starts at the identity quaternion `(1, 0, 0, 0)`.
- `lambda_t = a^(8 r_t)` is a gated decay:
  - `r_t` is a sigmoid gate of the input;
  - `log a = -softplus(-decay)` is learned per lane;
  - at a fully open gate, the lanes start with timescales spread geometrically from 2 to 1,000 tokens.
- `c_t` is the input branch after a width-4 causal depthwise convolution, whose taps start at the identity.
- The drive factor is implemented as `sqrt(max(1 - lambda^2, 1e-6))`. It is Griffin's normalization: for uncorrelated drives of equal variance it keeps the state's variance at the drive's.

The layer outputs `W_out (h_t ⊙ gelu(g_t))`, where `g_t` is a second input branch.

- **Whole-window training, not a parallel scan over time.** *Derived for the first part, as implemented for the second.*
  - The gates, rotation and drive depend only on the layer's input, not on the recurrent state. So every projection of a layer runs once per window as one matrix product. In the retained learner, each position's read and update depend on the state the previous position produced.
  - Only the scan itself runs position by position: a few multiply-adds per lane and step, batched over windows and parallel across them.
  - The transition acts on `h` by left multiplication, a linear map, so the recurrence is an affine scan. Its composition law `(q2, b2) ∘ (q1, b1) = (q2 * q1, q2 * b1 + b2)` is associative. A parallel prefix scan over time is therefore possible, but this implementation does not use one.
- **Non-expanding transitions, not a bound on the states.** *Derived.*
  - A unit quaternion preserves the norm and `lambda_t < 1`. Every product of transitions is therefore non-expanding: `|q_t * ... * q_s| <= 1`.
  - This does not bound the driven state uniformly.
    - As implemented, `|u_t| = |raw_t| / sqrt(|raw_t|^2 + 1e-6)`, slightly below 1, and the drive multiplier is `kappa = sqrt(max(1 - lambda^2, 1e-6))`.
    - For constant coefficients and drives of norm at most `B`, `|h_t| <= rho |h_{t-1}| + kappa B` with `rho = lambda |u| < 1`. The asymptotic bound is therefore `kappa B / (1 - rho)`.
    - With a unit rotation and an unclamped drive this becomes `sqrt((1 + lambda) / (1 - lambda)) B`, which grows as `lambda` approaches 1.
    - It says nothing about the residual stream across layers.
  - Setting `u_t` to the identity (`rotation=false`) leaves a real gated linear recurrence with one decay per lane. That is the ablation of the transport.
- **Exact backward.** *Derived.* It is the reverse scan:
  - the total gradient of `h_t` is `G_t = dh_t + conj(q_{t+1}) * G_{t+1}`;
  - then `db_t = G_t` and `dq_t = G_t * conj(h_{t-1})`.

  Both identities follow from `L(q)^T = L(conj q)` and `R(p)^T = R(conj p)` for the left- and right-multiplication matrices.

### 2.2 Multi-head read (`a`)

Six heads of width 48. Row `t` reads positions `0..t` and a NoRead slot whose value is zero, with scores:

```text
Lorentz:  s_tj = -beta_h (arcosh(z_tj) - delta_h) + age_h(t - j),   z = sqrt(1+|q|^2) sqrt(1+|k|^2) - <q, k>
Dot:      s_tj = <q_t, k_j> / sqrt(48) + age_h(t - j)
```

- The Lorentz score is the lifted hyperboloid distance of cycles 2 and 3.
- `beta_h` and `delta_h` are learned per head, with a flat start (`beta = 1`).
- `age_h` is a learned bias per head and distance. It starts at ALiBi slopes `2^(-8(h+1)/6)`, from 0.40 to 0.0039 per token (*Literature*, arXiv 2108.12409).
- The NoRead logit is a learned linear function of the query position.
- There is no rotary position embedding. Position reaches the read through the recurrence layers and the age term.

### 2.3 The control

#1017's exact shape, trained from scratch on the same windows:
- RoPE softmax attention with 6 heads of width 48 (theta 10,000, rotate-half pairing);
- SwiGLU 768;
- RMSNorm with epsilon 1e-5;
- tied embedding.

Its attention runs on the same fused kernel as the read, with the Dot score and no NoRead or age term.

### 2.4 Parameter parity

The geometric stack's MLP width is chosen so its total matches the control's.

| Model | Pattern | MLP | Parameters |
|---|---|---:|---:|
| Transformer control (#1017 shape) | `aaaaaa` | 768 | 7,155,360 |
| Geometric stack, Lorentz, rotation | `rrarra` | 749 | 7,153,860 |

The recurrence layer has four times as many 288-wide input channels in its gates and branches as a read layer has outside its projections. The stack pays for them with a slightly narrower MLP.

## 3. Implementation and throughput

Everything is Rust on Candle 0.9.2, with the crate's `forbid(unsafe_code)` kept.

**Fused ops.** Five ops are fused, each with an exact backward. Each is parallel over rows, windows or (window, head) blocks.

| Op | What it computes |
|---|---|
| Recurrence core | Convolution, decay gate, rotation normalization, scan and output gate |
| Read | Inner products, score transform, mask, NoRead, softmax, value mix and, for the control, RoPE |
| RMSNorm | Row normalization |
| SwiGLU | `silu(gate) * up` |
| Cross-entropy | Mean next-token loss |

- **Why fuse.** The recurrence core scans time sequentially inside each window and runs windows in parallel (§2.1). Candle's CPU elementwise ops are single-threaded, and its batched matrix product runs one small matrix at a time. Fusing took single-arm throughput on 4 threads as follows.

  | Arm | Before (tokens/s) | After (tokens/s) |
  |---|---:|---:|
  | Transformer control | 770 | 1,500 |
  | Geometric stack | 727 | 1,720 |

  Two arms run concurrently at 2 threads each reach about 1,100 tokens/s each, which is how the runs below were scheduled. *Measured*, idle sandbox, batch 16 × 256.
- **Checks.** Eleven focused tests:
  - the scan against the sequential recurrence;
  - each fused op against its composition of Candle operations, forward and backward: the recurrence core with rotation on and off, the Lorentz and Dot reads, the RoPE read, RMSNorm, SwiGLU and cross-entropy;
  - finite-difference gradients through the scan, the reads and the whole model;
  - causality of every model kind;
  - parameter parity;
  - save and load.

## 4. Data

- **Repository code.**
  - Cycle 3's corpus: 7,000,992 training and 206,844 development tokens.
  - The lab BPE with 4,096 tokens, 3.64 bytes per token.
  - The development split is unchanged, so the retained native model's cycle-3 results on it stay comparable.
- **Registry code.**
  - The Rust sources of the sandbox's Cargo registry, the latest version of each of 222 crates.
  - Files up to 200 kB, 5,975 files, 72,414,505 bytes.
  - Encoded with the same BPE: 29,039,409 tokens at 2.49 bytes per token. The BPE was fit on this repository.
- **Main runs.** They draw windows from the two training streams with equal probability. Over 29,999,104 target visits that is about 2.1 passes over the repository split and 0.5 over the registry.

| File | SHA-256 |
|---|---|
| `c3/data/code/train.u16` | `b12707b012e5447f0c613236ea7f8819f2cc123861dafe49f7f8226ccc457137` |
| `c3/data/code/valid.u16` | `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8` |
| `c3/data/code/lens.u16` | `edc0a8d6bcabf8c2a789113a71ebdd4ce6a3dd53de08e77aff13c56bc7c2e61f` |
| `c3/data/code/merges.txt` | `7453bfa086a18d36948d23fd060d2a74bc3ba8d87882d457cb397396dde84405` |
| `c4/data/registry.txt` | `cc96831b8431ccb7d428d8e5d1e7cacc18ce1e951b5ee957fd58e611f5e953ef` |
| `c4/data/registry.u16` | `af93ca73e51a798e4a19e1a470e97037939c95f7499e65ffb8e3526748026f65` |

## 5. Learning-rate pilot

*Measured.* Settings:
- 1,000 updates of 16 × 256 tokens (4,096,000 target visits per run), seed 1, on the repository code split only;
- warmup 100, then cosine decay to 10% of the peak rate;
- AdamW with beta2 0.95, weight decay 0.1 on matrices, clipping at 1.0.

Both arms drew identical windows. The final evaluation covers the 512 evenly spaced development windows of `joint-read-geometry`: 131,072 targets, the same windows as cycle 3's context-256 finals.

Final NLL in nats per token, with bits per byte in parentheses:

| Learning rate | Transformer control | Geometric stack | Stack − control |
|---|---:|---:|---:|
| 1e-3 | 2.8608 (1.1503) | 2.7851 (1.1199) | −0.076 |
| 2e-3 | **2.7680** (1.1130) | 2.6790 (1.0772) | −0.089 |
| 4e-3 | 2.8339 (1.1395) | **2.6097** (1.0494) | −0.224 |

- **Selection.** The rule was fixed before any pilot run finished: each arm's rate is the one with the lowest final NLL. That gives 2e-3 for the control and 4e-3 for the stack. At the selected rates the stack leads by **0.158 nats** (0.064 bits per byte).
- **Fairness.** The control's best rate lies inside the grid. The stack's best rate is the grid's largest, so its optimum may be higher. The tuning therefore does not disadvantage the control.
- **At every rate the stack leads.** At 4e-3 the control scores worse than at 2e-3 (2.834 against 2.768), while the stack improves further. That is underperformance of the control at that rate, not a measured numerical instability.
- **Context, not a matched comparison.** The retained native learner of cycle 3 scored 3.119–3.124 with the flat Lorentz read and 3.169–3.185 with the Dot read, on the same windows and data after the same 4.1M target visits. It had width 128 and about 0.69M parameters. Both 7.2M-parameter arms here are 0.35–0.58 nats better. Size, depth, layer structure and training configuration all differ between the two, so this cycle does not attribute the gap to any one of them.
- **Samples.** Greedy continuations of three development prompts fall into repetition loops in both arms at this budget, for example a repeated `use std::path::PathBuf;` line. The report roots keep all six arms' greedy and sampled continuations.
- **Cost.** Two runs shared the 4-core sandbox, with two threads each.
  - Training alone ran at 942–1,222 tokens/s per run, or 3,353–4,350 s per run.
  - Complete elapsed time per round, including evaluation, checkpointing and sampling, was 67.5, 59.9 and 73.8 minutes: 3 h 21 min for all six runs.
  - Rounds 1 and 2 overlapped builds and tests of this branch and of #1410.
  - These are training costs only. Neither number measures native serving.
- **Identities and records.** The [run packet](../evidence/geometric-stack-cycle4-2026-09-27/README.md) holds each run's complete report: settings, curve, final metrics, samples as token ids and text, and input, executable and model SHA-256. It also holds the launchers and the chosen-rate log, with hashes of every file.
  - Executable `fcf710ad…` (commit `dca1b790`, host-tuned release build).
  - Training split `b12707b0…`, development split `3f7c50ef…`.
- **Scope.** One seed per arm and a short budget.
  - The pilot carries no uncertainty estimate for these architectures. Cycle 3's seed spread belongs to the retained learner, not to these models.
  - The differences are between complete architecture-and-training packages, each at its own selected rate.
  - They measure early training. The main comparison (§6) measures the endpoint.

## 6. Main comparison

*(Filled in when the runs finish.)*

## 7. Ablations

*Measured*, seed 1. Each arm changes one thing in the geometric stack and keeps everything else of the pilot's selected stack run:
- learning rate 4e-3;
- 1,000 updates, 4,096,000 target visits, on the repository code split;
- identical training windows, because the seed is shared;
- the 512-window final evaluation (131,072 targets).

Each arm's MLP width is re-matched to the control's parameter count. The full stack is the pilot run itself.

| Arm | What changes | MLP | Final NLL (bits/byte) | − full stack |
|---|---|---:|---:|---:|
| Full stack | `rrarra`, Lorentz reads, quaternion transport | 749 | 2.6097 (1.0494) | — |
| `dot` | Dot reads instead of Lorentz | 749 | 2.5889 (1.0410) | −0.021 |
| `norot` | Identity transport (a real gated linear recurrence) | 814 | 2.6688 (1.0731) | +0.059 |
| `readsonly` | Six Lorentz read layers, no recurrence, no RoPE | 764 | **2.5531** (1.0266) | −0.057 |
| Transformer control (§5) | RoPE softmax attention, 2e-3 | 768 | 2.7680 (1.1130) | +0.158 |

Development NLL on 64 windows at 250, 500, 750 and 1,000 updates:

| Arm | 250 | 500 | 750 | 1,000 |
|---|---:|---:|---:|---:|
| Full stack | 3.7626 | 3.2337 | 2.8752 | 2.6864 |
| `dot` | 3.7893 | 3.2204 | 2.8493 | 2.6640 |
| `norot` | 3.7739 | 3.2313 | 2.9008 | 2.7374 |
| `readsonly` | 3.7351 | 3.2036 | 2.8145 | 2.6302 |

**What seed 1 shows, at this budget.**
- **The quaternion transport carries weight inside the recurrence.** Fixing it to the identity costs 0.059 nats, even though that arm spends the freed rotation parameters on a wider MLP.
- **The Lorentz score does not help inside the stack.** The Dot read is 0.021 nats better. That is below the 0.03 threshold stated on the card, so it calls for a second seed before any conclusion. In the retained one-layer learner, cycle 3 measured the opposite sign, 0.05–0.06 nats in favour of Lorentz at context 256.
- **The recurrence does not help at this budget.** Six read layers beat the `rrarra` stack by 0.057 nats and the RoPE control by 0.215 nats.
  - So the stack's lead over the control comes from the read layers, not from the recurrence.
  - Three things distinguish those read layers from the control's attention: the learned per-distance age bias with its ALiBi start, the NoRead slot, and the score. Seed 1 does not separate them.

**Runs added in response.** Each is justified by the rule above or by the attribution question.
- A second seed of the Lorentz and Dot stacks.
- A reads-only Dot arm (seed 1), which separates the Lorentz score from the age bias and NoRead within the best configuration.
- A second seed of the reads-only Lorentz arm.

The main comparison runs after them, unchanged.

**Interruptions.** A container reboot at about 08:22 UTC killed the first seed-2 pair before its first checkpoint. It restarted from scratch in new report roots (`*_r1`). The interrupted roots are kept. The pipeline now resumes any run from its latest checkpoint after a restart, into a new root.

**Cost.** Training took 2,739–3,386 s per arm, with two arms sharing the sandbox; `readsonly` ran alone at the end. The ablation stage ran from 05:58 to 07:42 UTC.

## 8. What this changes

*(Filled in when the runs finish.)*

## 9. Costs

*(Filled in when the runs finish.)*
