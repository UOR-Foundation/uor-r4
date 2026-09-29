# Track B harmonic attention: proposed mathematical design

Date: 2026-09-29. Status: **DESIGN / NOT_RUN**. Owning board:
[#1515](https://github.com/UOR-Foundation/uor-r4/issues/1515); epic:
[#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509).

This memo proposes an implementation for the owner's B2 arms. It is not an
accepted replacement for the plan of record or Lab 1's direction. The kernel,
normalization, GQA sharing, LoRA configuration and acceptance details below
require a recorded pre-registration before model compute. No B2 model was
trained or evaluated for this memo. Formulas and operation counts are
mathematical derivations, not measured M1 performance or serving qualification.
The conversion pipeline's exact-reference parity gate remains a prerequisite.

## What the primary literature supports

- [Chem-GMNet, arXiv:2605.13262, sections 3–4](https://arxiv.org/html/2605.13262v1)
  is the intended GM-Net citation. It studies molecular encoders, with a
  bidirectional, unnormalized harmonic recurrence and a full quadratic softmax
  branch. It supplies mathematical motivation, not causal LLM-conversion
  evidence. The manuscript promises a code release but supplies no implementation
  link. Its displayed Gegenbauer coefficient convention needs an explicit
  normalization choice before implementation.
- [SLAY, arXiv:2602.04915, section 2](https://arxiv.org/html/2602.04915v1)
  uses the spherical kernel `t²/(2 + epsilon - 2t)` and polynomial/exponential
  feature approximations. Its pointwise-positivity safeguards do not transfer
  automatically to an arbitrary truncated Gegenbauer sum.
- [Hedgehog, section 4.1](https://arxiv.org/html/2402.04347v1#S4.SS1)
  finds that polynomial feature maps can recover spikiness and monotonicity in
  bounded regimes, with substantial feature/state cost. Thus “harmonics cannot
  form spiky attention” is too categorical. [Based, sections 4–5](https://arxiv.org/html/2402.18668v1)
  deliberately uses quadratic Taylor features and studies the recall/state-size
  tradeoff.
- [LoLCATs, section 3.1](https://arxiv.org/html/2410.10254v2#S3.SS1)
  supports attention-output MSE with the original projections frozen, followed
  by a distinct LoRA recovery stage. Its hybrid uses local softmax and linear
  attention. B2's timing of LoRA training must be stated rather than silently
  conflated with that two-stage protocol.
- [Scatterbrain, section 4.2 and appendix C](https://proceedings.neurips.cc/paper/2021/file/9185f3ec501c674c7c788464a36e7fb3-Paper.pdf)
  supplies the sparse residual correction: replace the low-rank contribution
  on selected entries and correct normalization too. A rank-table hybrid does
  not inherit that paper's unbiased softmax-estimation claim.

## Spherical normalization and exact feature dimension

Let `d = d'`, `x,y` lie on `S^(d-1)`, `t = x·y`, and
`lambda = (d-2)/2`. Use the convention

\[
P_\ell(t)=\frac{C_\ell^\lambda(t)}{C_\ell^\lambda(1)},\qquad
N_\ell=\binom{d+\ell-1}{\ell}-\binom{d+\ell-3}{\ell-2}.
\]

The second binomial is zero when its lower index is negative. For harmonics
orthonormal under uniform **probability** measure on the sphere, the addition
formula is

\[
\sum_{m=1}^{N_\ell}Y_{\ell m}(x)Y_{\ell m}(y)
=N_\ell P_\ell(t).
\]

See [the spherical-harmonics appendix, equations 28–31](https://proceedings.neurips.cc/paper/2021/file/9ac5a6d86e8924182271bd820acbce0e-Supplemental.pdf#page=13).
Surface-area measure instead introduces an additional sphere-area factor.
Do not mix these conventions or omit `C_l(1)` from a conventional Gegenbauer
polynomial. The first four normalized polynomials are

\[
P_0=1,\quad P_1=t,\quad
P_2=\frac{dt^2-1}{d-1},\quad
P_3=\frac{(d+2)t^3-3t}{d-1}.
\]

| Projected dimension | Bands `N0,N1,N2,N3` | Features through L=1 | L=2 | L=3 |
|---:|---|---:|---:|---:|
| 16 | 1, 16, 135, 800 | 17 | 152 | 952 |
| 32 | 1, 32, 527, 5,952 | 33 | 560 | 6,512 |

The sphere constraint removes the trace coordinate, explaining 152 rather than
the 153 coordinates of an unconstrained symmetric quadratic Taylor map at d=16.

## Pointwise-positive kernel proposal

Nonnegative Gegenbauer coefficients imply a positive-semidefinite kernel; they
do not imply nonnegative pair scores. For example, the sum of unweighted
orthonormal bands through degree two at `t=0` is
`1 + N2 P2(0) = -d/2`, or -8 at d=16. Dividing by a denominator with an epsilon
does not repair those negative attention weights.

Propose the prospectively fixed family

\[
K_L(t)=\delta+\left(\frac{1+t}{2}\right)^L,\qquad \delta=10^{-6},
\]

for the prescribed L in {1,2,3}. This is one concrete proposal, not an assertion
that it approximates this teacher well. It is strictly positive on [-1,1]. Its
exact expansion `K_L = sum_l b_l P_l` has these coefficients:

| L | `b0 - delta` | `b1` | `b2` | `b3` |
|---:|---:|---:|---:|---:|
| 1 | 1/2 | 1/2 | — | — |
| 2 | (d+1)/(4d) | 1/2 | (d-1)/(4d) | — |
| 3 | (d+3)/(8d) | 3(d+3)/(8(d+2)) | 3(d-1)/(8d) | (d-1)/(8(d+2)) |

Construct band maps `h_l` with `h_l(x)·h_l(y)=P_l(t)` and concatenate
`sqrt(b_l) h_l(x)`. This realizes the stated polynomial kernel in real
arithmetic. Floating-point kernel/recurrence agreement must be checked in the
implementation. If degree weights later become learned, a nonnegative mixture
of these shifted-power kernels retains score positivity. Arbitrary nonnegative
Gegenbauer weights do not provide that guarantee.

## Packed trace-free construction

Use `h0=1`, `h1=x`. The following fixed orthogonal transforms construct the
minimal higher-degree bands without a large dense change-of-basis matrix.

For any trace direction `c`, let `u=c/||c||`, `v=u-e0`, and
`H=I-2vv^T/(v^Tv)`. This Householder transform maps `u` to `e0`. Apply H to a
coordinate group and discard coordinate zero. Dot products of the remaining
coordinates equal dot products after orthogonal projection onto `ker(c^T)`.
The vectors c used below are not parallel to e0 for d=16 or 32.

**Degree two.** Pack the symmetric tensor `x tensor x` with diagonal coordinates
`x_i²` and off-diagonal coordinates `sqrt(2) x_i x_j`, i<j. Apply H to the
diagonal group with `c=(1,...,1)`, discard its first transformed coordinate, and
keep all off-diagonal coordinates. This removes the trace, corresponding to
`x tensor x - I/d`. The resulting inner product is `t²-1/d`. Scaling the whole
band by `sqrt(d/(d-1))` produces P2 and `d(d+1)/2-1` coordinates.

**Degree three.** Keep all-distinct coordinates `sqrt(6) x_i x_j x_k`, i<j<k.
For each singleton index i, construct a group of d coordinates

\[
w^{(i)}=[x_i^3,\;\sqrt3 x_i x_j^2\ (j\ne i)].
\]

Its trace functional is `c=(1,1/sqrt(3),...,1/sqrt(3))`; trace groups have
disjoint support in the packed symmetric tensor. Apply the fixed H within
each group and discard its first coordinate. This removes d trace directions.
Equivalently the full tensor becomes

\[
T_{ijk}(x)=x_ix_jx_k-
\frac{x_i\delta_{jk}+x_j\delta_{ik}+x_k\delta_{ij}}{d+2}.
\]

Its contraction with T(y) is `t³-3t/(d+2)`. Scale the retained coordinates by
`sqrt((d+2)/(d-1))` to obtain P3. The count is
`binom(d,3)+d(d-1)=binom(d+2,3)-d`, equal to N3.

All runtime steps are products, fixed gathers, reductions and linear
transforms with ordinary derivatives. Build fixed constants in Rust, bind their
ordering/version to the artifact, and exercise the actual differentiable path.
No random-feature approximation or dense D-by-D basis matrix is needed.

## Projection, teacher target and acceptance controls

- Keep the teacher's full-width Q/K/V and output projections. On the student
  path, apply the same RoPE as the teacher, normalize Q/K, apply learned
  64-to-d projections, and **renormalize after projection** before the harmonic
  map. LoRA deltas on the original projections, if enabled in this stage, are
  separate declared parameters with fixed rank, initialization and scope.
- Proposed normalization: compute s=`||v||²`; if s >= `epsilon_norm²`, return
  `v/sqrt(s)`, otherwise return the fixed unit direction e0 and record the
  fallback. Proposed `epsilon_norm=1e-6`; freeze it prospectively. A denominator
  `sqrt(s+epsilon)` alone does not land on the sphere. Count fallback rows and
  check finite/unit norms; do not quietly accept a different kernel.
- Record discarded pre-normalization Q/K norms and their distributions. Report
  held-out operator error stratified by fixed norm bins: normalization removes
  amplitude information used by teacher attention. This is a diagnostic of the
  chosen mechanism, not permission to change the arm after seeing its score.
- Fix the target to the teacher's **post-WO, pre-residual attention output**.
  Use the identical teacher hidden-state input for the isolated layer's teacher
  and student paths. Retain per-head pre-WO errors only as diagnostics.
- Before fitting, separate training and development documents; all overlapping
  windows from a document remain in the same partition. Evaluate held-out MSE
  at step zero and at a pre-registered cadence through at most 500 updates, for
  each of at least two matched seeds (proposed 0 and 1). The 50% reduction gate
  is relative to each seed's own step-zero **held-out** MSE. A training-loss
  reduction alone does not pass it.
- Report raw MSE, target-energy-normalized squared error
  `sum ||prediction-target||² / sum ||target||²`, and zero/mean predictors.
  The zero predictor has normalized error one when target energy is nonzero;
  estimate the mean predictor from training targets only. An undefined
  zero-energy denominator is reported explicitly. Preserve these baselines to
  detect an apparently large relative gain from an unusually poor step zero.
- Student-composed, all-layer NLL/KL on fixed held-out teacher-tokenizer windows
  is decisive for conversion. Passing isolated layer MSE does not establish the
  <=0.15-nat all-layer hybrid criterion. Keep teacher-forced layer diagnostics
  distinct from actual composed student evaluation and generated behavior.
- Freeze windows, tokenization, optimizer/dose, coefficient convention, score
  scales, seeds and checkpoints before comparison. Pair each harmonic arm with
  its hybrid and dense control. Any absent flock integration/arm is **NOT_RUN**,
  not a harmonic-only proxy for the hybrid result.

## Causal recurrence and sparse correction

For each head or shared KV group maintain M of shape `[D,64]` and z of shape
`[D]`. Insert the current key/value before querying, matching inclusive causal
self-attention:

\[
M_t=M_{t-1}+\phi(k_t)v_t^\top,\quad
z_t=z_{t-1}+\phi(k_t),\quad
y_t=\frac{\phi(q_t)^\top M_t}{\phi(q_t)^\top z_t}.
\]

For a deduplicated causal flock support S, let `a_tj` be its nonnegative
**unnormalized** score and `h_tj=K_L(qhat_t·khat_j)`. Replace selected scores:

\[
y_t=\frac{\phi(q_t)^\top M_t+
\sum_{j\in S}(a_{tj}-h_{tj})v_j}
{\phi(q_t)^\top z_t+\sum_{j\in S}(a_{tj}-h_{tj})}.
\]

The residual correction can be signed; the final pair scores are a on S and h
outside S. Correct both numerator and denominator. A normalized softmax-over-k
output cannot substitute for a without defining another mechanism. Any row
rescaling for stable exponentiation must scale the harmonic and sparse terms
consistently. Track nonpositive/nonfinite denominators and numerical
cancellation; an unreported clamp must not hide a failed computation.

Compute h for selected keys from the scalar degree-L polynomial and a
d-dimensional dot, avoiding a D-dimensional feature dot per selected key.
Do not import decay/gating silently: it changes the causal weights and the
correction must then use the same decayed contribution.

## State and arithmetic estimates

The table assumes F32 persistent state, value width 64, 30 layers, and 9 query
heads. D includes the constant band. Core MAC counts include the M update,
query read and denominator dot; z updates are additional additions. One MAC is
one multiplication plus accumulation; these are not D11 multiplier-free counts.

**Independent maps:** 9 different key maps and states per layer, including when
the original K/V tensors came from only 3 KV heads. State is `4*30*9*65*D` bytes;
core work is `30*9*129*D` MAC/token.

**GQA-shared maps:** keep one key map and state for each of the original 3 KV
groups, with the 9 query heads reading their group's state using their own
query maps. K/V LoRA and key projection semantics must also preserve this
sharing. State is `4*30*3*65*D` bytes; core work is
`30*D*(3*64+9*65)` MAC/token. It is not valid to claim these costs for 9
independent learned key maps.

| d,L | D | Independent state | Independent core MAC/token | GQA-shared state | GQA-shared core MAC/token |
|---|---:|---:|---:|---:|---:|
| 16,1 | 17 | 1.138 MiB | 0.592 M | 0.379 MiB | 0.396 M |
| 16,2 | 152 | 10.176 MiB | 5.294 M | 3.392 MiB | 3.543 M |
| 16,3 | 952 | 63.734 MiB | 33.158 M | 21.245 MiB | 22.191 M |
| 32,1 | 33 | 2.209 MiB | 1.149 M | 0.736 MiB | 0.769 M |
| 32,2 | 560 | 37.491 MiB | 19.505 M | 12.497 MiB | 13.054 M |
| 32,3 | 6,512 | 435.965 MiB | 226.813 M | 145.322 MiB | 151.795 M |

These state/core counts apply at 512, 2,048 and 8,192 context. They exclude
feature construction, normalization, LoRA, sparse selection/correction,
training activations and weight traffic. Added 64-to-d Q/K projections cost
0.553/1.106 M MAC/token at d=16/32 for independent maps, or 0.369/0.737 M when
GQA shared. Packed lifts and trace removal take O(D+d²) scalar work per map;
their executed operations and memory traffic must be included in the final
cost report rather than folded into the core estimate.

The unchanged SmolLM2 dense attention/MLP projections plus vocabulary head
already require approximately 134.480 M MAC/token. Original dense attention's
QK-plus-AV products and F32 GQA cache are:

| Context | QK + AV MAC/token, all layers | Original GQA KV cache |
|---:|---:|---:|
| 512 | 17.695 M | 22.5 MiB |
| 2,048 | 70.779 M | 90 MiB |
| 8,192 | 283.116 M | 360 MiB |

The hybrid remains O(T) in stored context and scanning work if flock retains
and scans the full K/V history. The harmonic state is then **additional** to
that cache; selecting k values does not by itself establish O(k) selection
cost. An index, compressed-key store or exact log has its own storage and
access cost. Account for it before any total-path saving claim. None of the
tables gives M1 tokens/s, energy, measured bytes/token or trained quality.

## Required decisions before B2 compute

Lab 1/principal review should resolve the proposed positive kernel and epsilons,
GQA map sharing, full-width projection/LoRA scope, sparse-score scale and
available flock interface. The pre-registration must fix the doc-disjoint
panels, MSE cadence, seeds, total resource allowance and wall-time stop. Compile
and exercise kernel identity, causal recurrence/dense equivalence, hybrid
overlap/normalizer consistency and gradient flow before the bounded model
smoke. Preserve the owner's six harmonic arms, their paired hybrids and dense
control; report unavailable integration or budget-limited arms as NOT_RUN.
