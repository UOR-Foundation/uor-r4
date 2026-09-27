# Cycle 4: a capacity-matched, parallel-trainable geometric stack

2026-09-27 · Claude lab track · References #820, #973 · Work card: [#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851700710)

**Status.** An evidence note from the lab track, not a decision record. Labels: **Measured** (Rust runs in the lab sandbox, seeds stated), **Derived**, **Literature**, **Hypothesis**. Results are development measurements on code; nothing here is a final holdout or a language qualification.

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
- `u_t` is a unit quaternion computed from the input: `raw_t / |raw_t|`, where `raw_t` is a learned linear map of the normalized input plus a bias that starts at the identity quaternion `(1, 0, 0, 0)`.
- `lambda_t = a^(8 r_t)` is a gated decay:
  - `r_t` is a sigmoid gate of the input;
  - `log a = -softplus(-decay)` is learned per lane;
  - at a fully open gate, the lanes start with timescales spread geometrically from 2 to 1,000 tokens.
- `c_t` is the input branch after a width-4 causal depthwise convolution, whose taps start at the identity.
- `sqrt(1 - lambda^2)` keeps the state's scale bounded (Griffin's normalization).

The layer outputs `W_out (h_t ⊙ gelu(g_t))`, where `g_t` is a second input branch.

- **Parallel over time.** *Derived.* The transition is a scaled unit quaternion acting by left multiplication, a linear map of `h`. The recurrence is therefore an affine scan whose composition law `(q2, b2) ∘ (q1, b1) = (q2 * q1, q2 * b1 + b2)` is associative. So it can be evaluated for a whole window without the per-position dependence of the retained learner.
- **Stable by construction.** *Derived.*
  - A unit quaternion preserves the norm, and the decay is below 1, so no product of transitions can grow.
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

- **Why fuse.** Candle's CPU elementwise ops are single-threaded, and its batched matrix product runs one small matrix at a time. Fusing took single-arm throughput on 4 threads as follows.

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

*(Filled in when the runs finish.)*

## 6. Main comparison

*(Filled in when the runs finish.)*

## 7. Ablations

*(Filled in when the runs finish.)*

## 8. What this changes

*(Filled in when the runs finish.)*

## 9. Costs

*(Filled in when the runs finish.)*
