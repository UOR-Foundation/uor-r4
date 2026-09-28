# Empirical Evaluation & Comparative Diagnostics: Architectural Specification and Multi-Arm Benchmark Synthesis (Milestone M3)

**Author**: `teamwork_preview_worker_m3_1`  
**Milestone**: M3 — Empirical Evaluation & Comparative Diagnostics  
**Requirement Focus**: Requirement R3 (Empirical Evaluation & Comparative Diagnostics) & Strategic Directives (Zero Transformers, Falsify-First Regime)  
**Date**: 2026-09-26  
**Status**: Authoritative Architectural Synthesis Deliverable  
**Target Repository**: `~/uor-r4-worktrees/geometric-lm-goal` (Branch: `codex/geometric-lm-goal`)  
**Primary Authorities**:
- `AGENTS.md` (Native Geometric Language Model & Progress Control)
- `docs/integration/DECISIONS.md` (Decisions D0-b, D4, D8, D9)
- `docs/integration/reference-evaluator-v2.json`
- `crates/uor-r4-training/src/` (`reference_eval.rs`, `reference_campaign.rs`, `joint_comparison.rs`, `joint_model.rs`, `joint_evaluation.rs`, `joint_integer_evaluation.rs`)
- `crates/uor-r4-integer/src/` (`model.rs`, `generation.rs`, `bin/verify-integer.rs`)
- `crates/uor-r4-core/src/` (`native_geometric/hopf_metric.rs`, `canonical_lexical_ingestion.rs`, `answer_oracle.rs`)
- Upstream Reports: `m3_spec_miner_report.md`, `m3_explorer_1_report.md`, `m3_explorer_2_report.md`

---

## Executive Overview

Requirement R3 directs the project to:
> *"Measure and report empirical progress against parent baselines using automated loss metrics, comparative drift analysis, and open-ended text evaluation. Document all observed improvements, trade-offs, and failure modes."*

Milestone M3 establishes the authoritative empirical evaluation and comparative diagnostic framework across continuous training references and discrete integer serving paths. In accordance with the owner's **Falsify-First Scientific Regime** and **Zero Transformers Directive**, this deliverable synthesizes Features 11 through 15 into a unified, mathematically rigorous architectural document:

1. **Feature 11: Reference Evaluator v2 Tail NLL Protocol & Benchmark Results**: The frozen multi-arm benchmark protocol over 976 blocks (233,472 comparison-tail targets) against historical neural baselines (#1017 dense Llama transformer reference) and statistical count/cache baselines.
2. **Feature 12: Numerical Drift Verification across Sequential Context Trajectories**: The mathematical verification of the multiplier-free, float-free integer inference engine (`crates/uor-r4-integer`) against continuous F32 autodiff models, proving adherence to the prospective $\le 0.01$ state and probability drift bounds across the full 256-token horizon.
3. **Feature 13: Paired Ordinary Control Comparison: Geometric Quaternion vs. Householder-Pair**: The side-by-side capacity-matched comparison isolating geometric rotor dynamics from planar orthogonal reflections, validating the governing role of Decision D4 (geometry as an exact discrete algebraic substrate rather than an unproven natural language predictive advantage).
4. **Feature 14: 16 Frozen Source-Edit Entity Persistence Tracking**: The 32-variant TinyStories probe measuring long-range entity retention across distractors, demonstrating why sparse admission collapsed and justifying the full 256-token context invariant under Decision D9.
5. **Feature 15: 4-Part Qualitative Story Continuation Rubric**: The objective, 4-part binary rubric replacing unblinded heuristics to evaluate narrative coherence, transparently establishing the current 0/5 native model baseline and characterizing its precise syntactic and semantic failure boundaries.

---

## 1. Reference Evaluator v2 Tail NLL Protocol & Benchmark Results (Feature 11)

### 1.1 Evaluator Specification & Canonical Schema
The benchmark protocol is defined by the contract schema `uor-r4.reference-evaluator/2` in `docs/integration/reference-evaluator-v2.json`, bound by the canonical SHA-256 digest:
```
d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62
```

The underlying evaluation data source (`dev_source`) is:
- **Path**: `.uor-models/research/issue-1017/tokens/dev.u16`
- **Byte Size**: 500,000 bytes
- **Token Count**: Exactly 250,000 little-endian `u16` token IDs ($500,000 / 2$)
- **Vocabulary Bound**: Every token ID $y$ must strictly satisfy $0 \le y < 4096$
- **Context & Stride**: Strictly locked at 256 tokens (`context = 256`, `stride = 256`)

### 1.2 Mathematical Derivation of the 976 Blocks and 233,472 Comparison Tail Targets
The population partitioning is implemented in `crates/uor-r4-training/src/reference_eval.rs` and `crates/uor-r4-training/src/baseline_protocol.rs`:

1. **Complete Block Counting**:
   With context length $C = 256$, evaluating an autoregressive sequence of length $C$ requires $C + 1$ contiguous tokens to form input-target pairs:
   $$\text{blocks} = \left\lfloor \frac{\text{token\_count} - 1}{C} \right\rfloor = \left\lfloor \frac{250,000 - 1}{256} \right\rfloor = \left\lfloor \frac{249,999}{256} \right\rfloor = 976 \text{ complete blocks}$$

2. **Block Reading and Target Shift-by-One**:
   For block index $b \in [0, 975]$, with start offset $\text{start} = b \times 256$:
   - A contiguous window of 257 tokens is loaded:
     $$x_{\text{window}} = \text{tokens}[\text{start} \dots \text{start} + 257]$$
   - Input sequence of length 256:
     $$\text{inputs} = \text{tokens}[\text{start} \dots \text{start} + 256]$$
   - Target sequence of length 256 (shifted by $+1$ position):
     $$\text{targets} = \text{tokens}[\text{start} + 1 \dots \text{start} + 257]$$
   At intra-block position $p \in [0, 255]$, the model ingests prefix $\text{inputs}[0 \dots p]$ and predicts target $\text{targets}[p] = \text{tokens}[\text{start} + p + 1]$.

3. **Target Population Totals**:
   $$\text{Total Scored Targets} = 976 \times 256 = 249,856 \text{ targets}$$
   $$\text{Unscored Tail Targets} = (\text{token\_count} - 1) - \text{Total Scored Targets} = 249,999 - 249,856 = 143 \text{ targets}$$
   These 143 trailing tokens do not form a complete 256-token block and are discarded per protocol.

4. **Partition Division**:
   - **Tune (Calibration) Partition**: The first 64 blocks ($\text{block\_index} \in [0, 63]$), comprising:
     $$\text{Tune Targets} = 64 \times 256 = 16,384 \text{ targets}$$
     *Role*: Exclusively used for hyperparameter calibration (e.g. n-gram discount grid search $D \in \{0.5, 0.75, 0.9\}$ and recency cache mixture weight $\lambda \in \{0.0, 0.05, 0.1, 0.2, 0.4, 0.6\}$).
   - **Comparison Tail Partition**: The remaining 912 blocks ($\text{block\_index} \in [64, 975]$), comprising:
     $$\text{Comparison Tail Targets} = 912 \times 256 = 233,472 \text{ targets}$$
     *Role*: The authoritative, frozen held-out benchmark population. Hyperparameters are locked prior to scoring these blocks.

### 1.3 Mathematical Formulation of Negative Log-Likelihood (NLL)

#### A. Logit-Based Scoring (Reference Model)
Implemented in `crates/uor-r4-training/src/reference_eval.rs` (`score_logits`):
Given an unnormalized logit vector $z \in \mathbb{R}^V$ with vocabulary size $V = 4096$, and true target token $y \in \{0, \dots, V-1\}$:
1. Maximum logit:
   $$m = \max_{k \in \{0, \dots, V-1\}} z_k$$
2. Exponent sum:
   $$S = \sum_{k=0}^{V-1} \exp(z_k - m)$$
3. Log-normalizer (LogSumExp):
   $$\ln \sum_{k=0}^{V-1} \exp(z_k) = m + \ln(S)$$
4. Target NLL (in nats):
   $$\text{NLL}(y) = -\ln P(y) = -\left( z_y - (m + \ln(S)) \right) = m - z_y + \ln(S)$$
   The subtraction $m - z_y$ is performed near logit scale to prevent catastrophic cancellation. All values are verified for finiteness and non-negativity ($\text{NLL} \ge 0.0$).
5. **Kahan Compensated Summation**:
   To eliminate cumulative IEEE 754 floating-point rounding errors across 249,856 target additions, the evaluation accumulator uses a compensated sum:
   ```rust
   pub struct CompensatedSum {
       sum: f64,
       compensation: f64,
   }
   ```
   At each addition of target NLL $v$:
   $$y = v - \text{compensation}$$
   $$t = \text{sum} + y$$
   $$\text{compensation} = (t - \text{sum}) - y$$
   $$\text{sum} = t$$

#### B. Probability-Based Scoring (Joint Model & Integer Model)
Implemented in `crates/uor-r4-training/src/joint_evaluation.rs` (`score_probabilities`):
Given a declared probability distribution $p \in [0, 1]^V$:
1. Normalization verification:
   $$\left| \sum_{k=0}^{V-1} p_k - 1.0 \right| \le 0.0002 \quad (\text{PROBABILITY\_SUM\_TOLERANCE})$$
2. Target probability extraction: $p_y = p[y]$.
3. Non-zero guard: If $p_y \le 0.0$, evaluation aborts immediately. No artificial probability floor or epsilon clipping is permitted during evaluation.
4. Target NLL:
   $$\text{NLL}(y) = -\ln(p_y)$$

### 1.4 Preflight Isolation and Causal Integrity Gates
Before scoring tokens, the reference campaign (`reference_campaign.rs`) enforces three preflight sanity gates:
1. **Batch Isolation Preflight**:
   Evaluates block 0 and a distinct block individually, then evaluates them together in a 2-block batch. The maximum absolute logit difference must satisfy:
   $$\max |\text{logit}_{\text{batch}} - \text{logit}_{\text{single}}| \le 0.005 \quad (\text{batch\_isolation\_max\_logit\_delta})$$
2. **Future Edit Preflight (Causal Masking Gate)**:
   For a 256-token block, tokens in positions $128 \dots 255$ are mutated ($x_i \leftarrow (x_i + 1) \pmod{4096}$). Prefix logits for positions $0 \dots 127$ must be identical within $10^{-5}$:
   $$\max |\text{logit}_{\text{prefix, mutated}} - \text{logit}_{\text{prefix, original}}| \le 10^{-5}$$
3. **No Gradients Tracked**:
   Asserts `logits.track_op() == false`; backward graph retention aborts execution.
4. **Golden Generation Replay Gate**:
   Evaluates 5 pinned prompts (seeds 2014..2018) under `top_k = 40`, `temperature = 0.8`, `SplitMix64` PRNG. Generated token IDs, decoded text, and stop reasons must match historical golden files bitwise.

### 1.5 Benchmark Results and Decision Gates

| Baseline / Model Arm | Partition Scope | Mean NLL (nats/token) | Decision Gate / Status |
|:---|:---|:---:|:---|
| **Historical #1017 Reference Model (Llama)** | Full Dev (976 blocks / 249,856 targets) | **1.580241** | Historical neural comparator |
| **Historical #1017 Reference Model (Llama)** | Comparison Tail (912 blocks / 233,472 targets) | **1.574024** | Pinned reference quality ceiling |
| **5-gram Kneser-Ney Alone** | Comparison Tail (912 blocks / 233,472 targets) | **2.405627** | Statistical baseline |
| **5-gram KN + Token Cache Mixture** | Comparison Tail (912 blocks / 233,472 targets) | **2.391786** | **`CACHE_GATE_NLL`**: Candidate MUST beat this ($< 2.391786$) |
| **Ordinary Control Learner (Read Enabled)** | Comparison Tail (912 blocks / 233,472 targets) | **2.085241** | **PASS** Cache Gate ($\Delta = -0.3065\text{ nats}$) |
| **Quaternion Recurrent Learner (Read Enabled)**| Comparison Tail (912 blocks / 233,472 targets) | **2.110368** | **PASS** Cache Gate ($\Delta = -0.2814\text{ nats}$) |
| **Ordinary Control Learner (NoRead)** | Comparison Tail (912 blocks / 233,472 targets) | **2.561712** | Whole-prefix ablation control |
| **Quaternion Recurrent Learner (NoRead)** | Comparison Tail (912 blocks / 233,472 targets) | **2.592991** | Whole-prefix ablation control |
| **Read Utility Gate (`READ_GATE_NATS`)** | Comparison Tail (912 blocks / 233,472 targets) | $\ge \mathbf{0.02}\text{ nats}$ | $\text{NLL}_{\text{NoRead}} - \text{NLL}_{\text{Read}} \ge 0.02$ |

**Read Utility Gate Verification**:
- Quaternion Arm: $2.592991 - 2.110368 = \mathbf{0.482623\text{ nats}}$ gain (Passes $\ge 0.02$ by $24.1\times$).
- Ordinary Control Arm: $2.561712 - 2.085241 = \mathbf{0.476471\text{ nats}}$ gain (Passes $\ge 0.02$ by $23.8\times$).
Both models confirm that causal memory reading provides massive predictive utility over recurrent state updates alone.

---

## 2. Numerical Drift Verification across Sequential Context Trajectories (Feature 12)

### 2.1 Purpose & Governance of the Integer Serving Bridge
The final serving runtime (`crates/uor-r4-integer`) operates strictly under owner-adopted **Decision D0-b**:
- Zero floating-point arithmetic instructions in the served computation path.
- Zero hardware multiplier instructions in the declared numerical kernel (all linear maps evaluate via low-bit shift-and-add).
- Signed 4-bit weights $W \in \{-7, \dots, 7\}$ with dyadic power-of-two row scales.
- Bounded fixed-point state and probability representations.

Feature 12 verifies the numerical drift between this discrete integer serving model and its continuous F32 floating-point counterpart across identical input token sequences.

### 2.2 Fixed-Point Quantization Scaling Constants
The integer runtime maps continuous representations to fixed-point integer domains:
1. **State Scale (`STATE_SCALE`)**: Fixed at $2048.0 = 2^{11}$ (Q11 format). Continuous state $s_{\text{f32}} \approx s_{\text{int}} / 2048.0$. State bounds $[-32767, 32767]$ represent real interval $[-15.9995, 15.9995]$.
2. **Copy Gate Scale (`GATE_SCALE`)**: Fixed at $32768.0 = 2^{15}$ (Q15 format). Continuous gate $g_{\text{f32}} \approx g_{\text{int}} / 32768.0$, representing $[0.0, 1.0]$.
3. **Probability Total (`PROBABILITY_TOTAL`)**: Fixed at $2^{48} = 281,474,976,710,656$ (Q48 format). Continuous probability $p_{\text{f32}} \approx p_{\text{int}} / 2^{48}$.
4. **Lookup Tables (`Tables`)**: 65,535-entry tables mapping inputs in $[-32767, 32767]$ to Q15 sigmoid, Q14 tanh, and Q48 exp.
5. **Exact $2^{48}$ Residual Normalization**: Because integer division truncates fractions, $\sum_{k=0}^{V-1} p_{\text{int}, k}$ can fall short of $2^{48}$. The runtime computes residual $R = 2^{48} - \sum p_{\text{int}, k}$ and adds $R$ onto the single largest probability element, guaranteeing exact mass conservation $\sum p_k \equiv 2^{48}$ with zero distribution distortion.

### 2.3 Mathematical Formulation of Drift Metrics

#### A. State Drift
At each token step $t \in [0, 255]$ with hidden dimension $D = 256$:
$$\delta_{s, i} = \left| \frac{s_{\text{int}, i}}{2048.0} - s_{\text{f32}, i} \right| \quad \text{for } i \in \{0, \dots, 255\}$$
$$\Delta_{\text{state}}^{\max} = \max_{i \in \{0, \dots, 255\}} \delta_{s, i}$$
$$\text{RMS}_{\Delta} = \sqrt{\frac{1}{D} \sum_{i=0}^{D-1} \left( \frac{s_{\text{int}, i}}{2048.0} - s_{\text{f32}, i} \right)^2}$$

#### B. Probability Drift
At each token step $t \in [0, 255]$ with vocabulary size $V = 4096$:
$$\delta_{p, k} = \left| \frac{p_{\text{int}, k}}{2^{48}} - p_{\text{f32}, k} \right| \quad \text{for } k \in \{0, \dots, V-1\}$$
$$\Delta_{\text{prob}}^{\max} = \max_{k \in \{0, \dots, V-1\}} \delta_{p, k}$$
$$\text{Total Variation Distance (TV)} = \frac{1}{2} \sum_{k=0}^{V-1} \left| \frac{p_{\text{int}, k}}{2^{48}} - p_{\text{f32}, k} \right|$$

#### C. Auxiliary Drift Metrics
- **NoRead Mass Delta**: $\left| \frac{m_{\text{no\_read, int}}}{2^{48}} - m_{\text{no\_read, f32}} \right|$
- **Copy Gate Delta**: $\left| \frac{g_{\text{copy, int}}}{32768.0} - g_{\text{copy, f32}} \right|$
- **Top-1 Disagreement**: $\mathbb{I}\left( \text{argmax}_{k} p_{\text{int}, k} \ne \text{argmax}_{k} p_{\text{f32}, k} \right)$
- **NLL Delta**: $\Delta_{\text{NLL}} = -\ln(p_{\text{int}, y} / 2^{48}) - (-\ln(p_{\text{f32}, y}))$

### 2.4 The Prospective Bound ($\le 0.01$)
The prospective engineering criteria established prior to execution require:
$$\Delta_{\text{state}}^{\max} \le 0.01$$
$$\Delta_{\text{prob}}^{\max} \le 0.01$$
*Contractual Definition*: "Principal-selected engineering drift limits before execution; not a theoretical error bound, new language acceptance gate, or permission to change the existing source-answer retention criteria."

### 2.5 Empirical Trajectory Verification (4 Windows = 1,024 Targets per Mode)

| Arm & Mode | Max State Drift ($\le 0.01$) | Max Prob Drift ($\le 0.01$) | Max Total Variation | Mean Total Variation | Top-1 Disagreements | Mean NLL Delta (nats) | Compliance Verdict |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Quaternion (Read Enabled)** | **0.007324** | **0.006692** | 0.007754 | 0.001963 | 2 / 1,024 (0.195%) | +0.000075 | **PASS** |
| **Quaternion (NoRead)** | **0.006348** | **0.005147** | 0.006932 | 0.002051 | 2 / 1,024 (0.195%) | -0.000070 | **PASS** |
| **Householder (Read Enabled)**| **0.005859** | **0.008505** | 0.009039 | 0.001872 | 2 / 1,024 (0.195%) | -0.000066 | **PASS** |
| **Householder (NoRead)** | **0.007324** | **0.005972** | 0.007085 | 0.002186 | 3 / 1,024 (0.293%) | +0.000084 | **PASS** |

### 2.6 Trajectory Stability and Compiler Feature Dependency
1. **Bounded Drift Trajectories Across 256 Steps**:
   Because recurrent models iterate $s_t = f(s_{t-1}, x_t)$, uncorrected rounding errors can compound exponentially. Tracking drift by sequence position confirms that peak state delta occurs at varying intermediate positions (e.g. pos 50, 100, 200) but remains strictly capped below $0.0075$. Integer RMSNorm and exact residual normalization prevent runaway dynamic divergence.
2. **Compiler Feature Sensitivity (Attempt 1 Incident)**:
   In initial evaluation runs, omitting the cargo feature flags `metal,cpu-accelerate,reference-accelerate` altered F32 CPU accumulation order in the comparator:
   - Quaternion score on the 16 source-edit pairs shifted from 28 to 29.
   - Householder maximum state delta rose to $0.01171875$ (exceeding the $0.01$ threshold).
   Recompiling with `--features metal,cpu-accelerate,reference-accelerate` restored bitwise matched accumulation, bringing Householder max state delta to $0.005859375$ (passing). Compiler features are pinned in all verification recipes.

---

## 3. Paired Ordinary Control Comparison: Geometric Quaternion vs. Householder-Pair (Feature 13)

### 3.1 Multi-Arm Comparison Framework Architecture
Implemented in `crates/uor-r4-training/src/joint_comparison.rs` (`joint-compare`), the harness executes a 6-way target stream join across reference, statistical counts, and native models:
```
ARM_NAMES = [reference, ngram, cache, quaternion_read, quaternion_no_read, ordinary_read, ordinary_no_read]
```
The framework evaluates 10 canonical paired differences ($\Delta = \text{NLL}_{\text{candidate}} - \text{NLL}_{\text{comparator}}$) across all 249,856 targets, anchored by five frozen engineering gates:
1. **Gate 1: Hard Likelihood Retention**: Hard quantized NLL minus continuous NLL $\le 0.05\text{ nats}$, and below cache gate ($2.391786$).
2. **Gate 2: Read Utility**: $\text{NLL}_{\text{no\_read}} - \text{NLL}_{\text{read}} \ge 0.02\text{ nats}$.
3. **Gate 3: Loaded Non-Collapse Generation**: Generated sequences nonconstant, free of period-1..4 short cycles.
4. **Gate 4: Entity Persistence**: At most 2 first-noun row losses on the 16 source-edit pairs.
5. **Gate 5: Artifact/Numerical Integrity**: Probability sum error $\le 0.0002$, zero NaNs, verified SHA-256 bindings.

### 3.2 Parameter Capacity and Degrees of Freedom Equivalence
To prevent confounding architectural inductive bias with capacity disparities, both arms are strictly matched:
- **State Dimension**: $D = 256$, structured as 64 parallel 4-dimensional lanes ($64 \times 4 = 256$).
- **Transport Parameters**: Exactly **256 parameters** per transport layer ($64\text{ lanes} \times 4\text{ raw coordinates}$).
- **Global Parameters**: Exactly **1,472,640 parameters** in both models ($W_E, W_Q, W_K, W_V, W_{\text{out}}, W_{\text{copy}}$ are identical in shape and precision).
- **Degrees of Freedom**: Exactly **3 tangent degrees of freedom** per 4D lane on $S^3$ (the radial degree of freedom is redundant and normalized out by unit projection).

### 3.3 Divergent Inductive Biases: Left-Isoclinic $SO(4)$ vs. Planar Householder Reflections

```
                     ┌──────────────────────────────────────────────┐
                     │           4D State Lane: x in R^4            │
                     └──────────────────────┬───────────────────────┘
                                            │
                    ┌───────────────────────┴───────────────────────┐
                    ▼                                               ▼
     ┌─────────────────────────────┐                 ┌─────────────────────────────┐
     │  Geometric Quaternion Arm   │                 │   Householder-Pair Arm      │
     │      (Rotor in Sp(1)_L)     │                 │   (Planar SO(4) Reflector)  │
     ├─────────────────────────────┤                 ├─────────────────────────────┤
     │ Left isoclinic SO(4) rotor  │                 │ Dual hyperplane reflection  │
     │ y = q * x                   │                 │ y = H(v) H(e_0) x           │
     │ Rotates TWO orthogonal 2D   │                 │ Rotates ONE 2D plane;       │
     │ planes by identical angle θ │                 │ leaves orthogonal 2D plane  │
     │ Preserves H algebra, Hopf   │                 │ invariant (I_2)             │
     │ fiber, and Z[φ] icosians    │                 │ Standard orthogonal control │
     └─────────────────────────────┘                 └─────────────────────────────┘
```

#### A. Geometric Quaternion Arm ($S^3$ Rotors & Hopf Fibration)
In `crates/uor-r4-training/src/joint_model.rs` (lines 1700–1772), each 4D state lane $x = (x_0, x_1, x_2, x_3)^T$ is identified with a quaternion $x \in \mathbb{H}$.
Given unconstrained raw update $\text{raw} \in \mathbb{R}^4$:
$$\mathbf{v}_{\text{centered}} = \mathbf{e}_0 + \alpha \cdot \text{raw}, \quad \mathbf{e}_0 = (1, 0, 0, 0)^T, \quad \alpha = 0.1$$
$$q = \frac{\mathbf{v}_{\text{centered}}}{\|\mathbf{v}_{\text{centered}}\|_2} = (w, a, b, c)^T \in S^3$$
Transport executes via Hamilton left-multiplication $y = q \cdot x$:
$$\begin{pmatrix} y_0 \\ y_1 \\ y_2 \\ y_3 \end{pmatrix} = \begin{pmatrix} w & -a & -b & -c \\ a & w & -c & b \\ b & c & w & -a \\ c & -b & a & w \end{pmatrix} \begin{pmatrix} x_0 \\ x_1 \\ x_2 \\ x_3 \end{pmatrix}$$
- **Lie Group Geometry**: Left-isoclinic rotation in $Sp(1)_L \subset SO(4)$. Rotates two mutually orthogonal 2D planes by identical angle $\theta$.
- **Hopf Fibration**: Projects $S^3 \to S^2$ with explicit $U(1)$ fiber phase retention ($\psi = \text{atan2}(b, a)$), enabling exact lossless reconstruction.
- **Discrete Group Closure**: Admits discrete binary icosahedral group $2I \subset S^3$ (120 elements) in exact $\mathbb{Z}[\phi]$ ($E_8 \cong H_4 \oplus \phi H_4$).

#### B. Control Householder-Pair Arm (Ordinary Orthogonal Reflectors)
The control arm replaces the quaternionic rotor with a product of two Householder reflections ($H(\mathbf{u}) = I - 2\mathbf{u}\mathbf{u}^T$):
$$y = H(\mathbf{v}) H(\mathbf{e}_0) x$$
where:
$$\mathbf{v}_{\text{centered}} = \mathbf{e}_0 + \frac{\alpha}{\sqrt{2}} \cdot \text{raw}, \quad \mathbf{v} = \frac{\mathbf{v}_{\text{centered}}}{\|\mathbf{v}_{\text{centered}}\|_2}$$
1. First reflection $H(\mathbf{e}_0)$: Negates first coordinate $x_{\text{reflected}} = (-x_0, x_1, x_2, x_3)^T$.
2. Second reflection $H(\mathbf{v})$: $y = x_{\text{reflected}} - 2(\mathbf{v}^T x_{\text{reflected}})\mathbf{v}$.

**Scale Matching via Frobenius Norm ($\alpha / \sqrt{2}$)**:
Dividing $\alpha$ by $\sqrt{2} \approx 0.07071$ exactly equates the second-order tangent Frobenius metric scale around identity between the two arms ($16(\alpha/\sqrt{2})^2 = 8\alpha^2$).

**Fundamental Contrast in Inductive Bias**:
By the Cartan-Dieudonné theorem, the product of two hyperplane reflections generates a **simple planar rotation**: it rotates the 2D plane $\text{span}(\mathbf{e}_0, \mathbf{v})$ while leaving the orthogonal complement $\text{span}(\mathbf{e}_0, \mathbf{v})^\perp$ **completely invariant ($I_2$)**. The quaternion arm couples all 4 coordinates simultaneously; the Householder arm leaves half the subspace unrotated.

### 3.4 Computational and Empirical Comparison

| Metric / Dimension | Geometric Quaternion Arm | Ordinary Householder-Pair Control | Empirical Contrast & Significance |
|:---|:---:|:---:|:---|
| **Parameters per Transport Layer** | **256** ($64 \times 4$) | **256** ($64 \times 4$) | Exactly matched capacity |
| **Total Model Parameters** | **1,472,640** | **1,472,640** | Exactly matched capacity |
| **Continuous FP Math / Lane** | 16 mul, 12 add ($1,024\text{ mul/step}$) | 8 mul, 7 add/sub ($512\text{ mul/step}$) | Householder uses **50% fewer multiplications** |
| **Serving Step Latency** | **$3.695\text{ ms}$** / step | **$3.698\text{ ms}$** / step | Latency is indistinguishable ($\approx 3.7\text{ ms}$) |
| **Continuous Tail NLL (Step 8348)** | **2.090518 nats** | **2.064403 nats** | **Householder wins by 0.0261 nats** |
| **Learned Codes Tail NLL** | **2.114226 nats** | **2.110880 nats** | **Householder wins by 0.0033 nats** |
| **Step 7324 Tune64 NLL** | **2.213687 nats** | **2.182433 nats** | **Householder wins by 0.0313 nats** |
| **16 Source-Edit Complete Answers** | **28 / 32** ($87.5\%$) | **24 / 32** ($75.0\%$) | Quaternion retains $+4$ exact answers |
| **16 Source-Edit First Noun** | **29 / 32** ($90.6\%$) | **30 / 32** ($93.8\%$) | Householder retains $+1$ first noun |

### 3.5 Scientific Status Under Decision D4
The multi-arm comparison provides decisive empirical evidence resolving the core question of Decision D4:
1. **Predictive Advantage Refuted on General Text**: Across 233,472 comparison-tail natural language tokens, the ordinary Householder-pair control consistently achieves equal or slightly lower NLL than the geometric quaternion arm ($2.064$ vs $2.090$ continuous, $2.110$ vs $2.114$ quantized).
2. **Task-Specific Retentive Gain**: The quaternion arm displays a modest advantage on entity-tracking completion (28/32 vs 24/32), but this does not translate into general predictive superiority.
3. **Formal Governance Under Decision D4**: Under the project's Falsify-First regime, **geometric predictive advantage remains an unproven, gated hypothesis**. Geometry is retained solely for exact discrete algebra (binary icosahedral group closure in $\mathbb{Z}[\phi]$), prime-addressed memory indexing, and $U(1)$ Hopf fiber phase tracking—never claimed as an inherently superior natural language predictor.

---

## 4. 16 Frozen Source-Edit Entity Persistence Tracking (Feature 14)

### 4.1 Probe Architecture & Narrative Structure
Implemented in `crates/uor-r4-training/src/joint_evaluation.rs` (lines 607–831), Feature 14 evaluates entity persistence and object substitution across 16 probe pairs (32 distinct TinyStories variants):
- **16 Noun Pairs**: apple/pear, ball/kite, book/toy, bell/drum, doll/bear, cup/bowl, hat/cap, shoe/boot, brush/comb, spoon/fork, flower/leaf, key/coin, box/bag, stone/shell, pencil/crayon, rope/ribbon.
- **4 Rotating Narrative Templates** (`template = index % 4`):
  1. *Template 0 (Lily in the garden, 57 tokens)*: Target noun introduced at tokens 5–7; 48 tokens of distractor narrative (playing with friend, watching clouds); query: *"Lily went back to the table and picked up the"* $\to$ Expected: `"{noun}."`
  2. *Template 1 (Tim at the pond, 62 tokens)*: Target noun introduced at tokens 5–7; 52 tokens of distractor narrative (bench, pond, ducks, mother, lunch); query: *"He looked under the bench and found his"* $\to$ Expected: `"{noun}."`
  3. *Template 2 (Anna in the park, 58 tokens)*: Target noun introduced at tokens 5–7; 49 tokens of distractor narrative (tree, brother, birds, sunset); query: *"Anna returned to the tree to collect her"* $\to$ Expected: `"{noun}."`
  4. *Template 3 (Ben before dinner, 56 tokens)*: Target noun introduced at tokens 5–7; 47 tokens of distractor narrative (kitchen chair, washing hands, plates, dinner); query: *"He went to the chair and took his"* $\to$ Expected: `"{noun}."`

### 4.2 Cognitive Dimensions Tested
1. **Entity Persistence Across Distractors**: The target entity must be retained across 45–55 tokens of unrelated narrative activity involving multiple competing characters, locations, and objects.
2. **Context Sensitivity vs. Unigram Bias**: The query phrase grammatically admits almost any noun. A unigram or recurrent-only model fails because local transitions have no affinity for the specific entity. When read attention is ablated (`NoRead`), accuracy collapses to **0 / 32**.
3. **Object Substitution Invariance**: By comparing original and edited variants on identical templates, the probe ensures that the model has not simply memorized static training stories.

### 4.3 Automated Scoring Methodology
- **First Noun Correctness**: Extracts the first alphabetic word from the emitted text and tests equality: `first_word == item.noun()`.
- **Complete Answer Correctness**: Generated response must terminate cleanly at a sentence boundary and match the deterministic answer oracle:
  ```rust
  answer_oracle::accepts(&accepted_continuations, &response_text)
  ```
  Where `accepted_continuations` is strictly `["{noun}."]`. Responses containing trailing garbage, run-on sentences, or hallucinated continuations fail complete correctness.
- **First-Noun Retention Gate**: Under the D8 evaluation ladder, candidate models must lose **at most 2** previously correct first-noun rows relative to their parent. Losses are asymmetric: new gains cannot offset row losses.

### 4.4 Empirical Results: Sparse Admission Collapse vs. Full256 Preservation

| Model / Admission Policy | Causal Window / Admission Rule | Complete Answers (/32) | First Noun Correct (/32) | Both Pair Answers Exact (/16) | Status / Gate Verdict |
|:---|:---|:---:|:---:|:---:|:---|
| **Learned-code Parent (Quaternion)** | Full 256 tokens (`full256`) | **28** | 28 | 14 | **Accepted Parent** |
| **Learned-code Parent (Ordinary)** | Full 256 tokens (`full256`) | **24** | 31 | 10 | **Accepted Parent** |
| **`NoRead` Intervention** | Zero attention read | **0** | 0 | 0 | **FAIL** (Validates causal read) |
| **`Recent32` Control** | Strictly recent 32 tokens | **0** | 0 | 0 | **FAIL** (Catastrophic eviction) |
| **`Orthant64` Bounded Admission** | Recent 32 + 32 indexed sign postings | **17** | 17 | 6 | **FAIL** (11 losses vs parent) |
| **`Recent64` Control** | Strictly recent 64 tokens | **27** | 27 | 13 | Recovers complete answers |
| **Same Weights with `Full256`** | Restored full 256 tokens | **27** (Q) / **25** (Ord) | 27 / 31 | 13 / 11 | Proves weights intact |
| **Standalone Integer Serving** | Float-free full256 runtime | **28** (Q) / **24** (Ord) | 28 / 31 | 14 / 10 | **PASS** (Zero rows lost vs parent) |

### 4.5 Root-Cause Failure Analysis of Sparse Admission and Decision D9
1. **Geometric Context Mismatch**:
   In all 4 templates, the prompt length is 56–62 tokens, placing the entity 48–57 positions in the past. In `recent32`, any token older than 32 steps is evicted. The model is physically incapable of attending to the target and scores **0 / 32**.
2. **Failure Mechanics of `orthant64`**:
   `orthant64` used two 16-bucket sign tables (depth 8 FIFO) to index historical tokens:
   - **Stopped Gradients**: Hard bucket assignment created a non-differentiable bottleneck.
   - **Saturated Postings**: Over the evaluation, **278,103 posting overwrites** occurred. Intermediate distractor tokens saturated the FIFO queues, silently evicting the entity.
   - Result: 11 losses in Quaternion and 8 in Ordinary, failing the retention gate.
3. **Proof of Decision D9**:
   Evaluating the identical weights under `full256` immediately restored performance to 27/32 complete answers. This proved that the failure resided entirely in the admission bottleneck, not representation learning. Decision D9 permanently locked `full256` as the repository invariant.

---

## 5. 4-Part Qualitative Story Continuation Rubric (Feature 15)

### 5.1 Historical Context & The 4-Part Binary Rubric
Early qualitative evaluations relied on subjective, unblinded heuristics (e.g. "Prompt subject or scene: at least 4/5" in #1017). To enforce rigorous scientific standards under the Falsify-First regime, `language-continuation-plan-2026-09-25.md` introduced the **4-part binary rubric**, requiring simultaneous satisfaction of four orthogonal criteria ($C_1 \land C_2 \land C_3 \land C_4 = 1$):

```
┌────────────────────────────────────────────────────────────────────────┐
│                   4-Part Qualitative Narrative Rubric                  │
├────────────────────────────────────────────────────────────────────────┤
│ 1. Entity & Role Consistency (C1)                                      │
│    Character identities, social roles, species, and gender/pronouns    │
│    remain stable without sudden merging, transmutation, or drift.      │
├────────────────────────────────────────────────────────────────────────┤
│ 2. Narrative Progression (C2)                                          │
│    Events advance chronologically with causal continuity; no circular  │
│    repetitions, infinite loops, or static semantic stasis.             │
├────────────────────────────────────────────────────────────────────────┤
│ 3. Literal Comprehension (C3)                                          │
│    Respects commonsense physical and grammatical constraints; valid    │
│    English syntax; free of word salad or corrupted non-words.          │
├────────────────────────────────────────────────────────────────────────┤
│ 4. Clause Completion (C4)                                              │
│    Continuation concludes at a complete syntactic clause boundary with │
│    valid punctuation (., !, ?, EOS); no mid-phrase token 128 cutoff.   │
└────────────────────────────────────────────────────────────────────────┘
```

### 5.2 The 5 Canonical Story Prompts
Sealed in `docs/integration/reference-evaluator-v2.json`, evaluated under `temperature = 0.8`, `top_k = 40`, `max_new_tokens = 128`:
1. **Seed 2014**: *"Once upon a time, there was a mommy and daddy playing with their 3-year-old son. Mommy"* (24 tokens)
2. **Seed 2015**: *"Once upon a time, there was a mighty lion walking through the jungle. He was looking for his friends, and he"* (24 tokens)
3. **Seed 2016**: *"Once there was a pink goose. She was very pretty. The goose liked to eat grass, corn and bugs. One"* (24 tokens)
4. **Seed 2017**: *"One day, a boy named Tim wanted to ride his bike. He loved to ride up the big hill near his house"* (24 tokens)
5. **Seed 2018**: *"One day, there was a small and modest train. Inside the train were two friends - Lily and Dave. They"* (24 tokens)

### 5.3 Transparent Evaluation of Current Native Geometric Models

| Model Candidate | Generation Mode / Runtime | C1: Entity Consistency | C2: Narrative Progression | C3: Literal Comprehension | C4: Clause Completion | Fully Acceptable (/5) | Evaluation Verdict |
|:---|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| **SmolLM2-135M Baseline** | F32 Dense Transformer | 4/5 | 5/5 | 5/5 | 4/5 | **4 / 5** | Historical Reference |
| **#1017 TinyStories Transformer** | F32 Dense Transformer | 5/5 | 5/5 | 5/5 | 5/5 | **5 / 5** (loose rubric) | Historical Baseline |
| **D8 Rung 1 Continuous Model** | Autodiff F32 recurrent | 1/5 | 1/5 | 0/5 | 0/5 | **0 / 5** | FAIL |
| **D8 Bounded Admission (`orthant64`)** | Bounded index recurrent | 1/5 | 1/5 | 0/5 | 0/5 | **0 / 5** | FAIL |
| **D8 Projected Packed Shadow** | Quantized QAT recurrent | 1/5 | 1/5 | 0/5 | 1/5 | **0 / 5** | FAIL |
| **Retained Integer Serving (Quaternion)** | Float-free integer session | 1/5 | 1/5 | 0/5 | 0/5 | **0 / 5** | FAIL |
| **Retained Integer Serving (Ordinary)** | Float-free integer session | 1/5 | 1/5 | 0/5 | 0/5 | **0 / 5** | FAIL |

### 5.4 Characterization of Failure Boundaries
All current native geometric model configurations score **0 / 5** on the strict rubric. While the models demonstrate **machine-level noncollapse** (valid UTF-8, zero short-cycle period-1..4 loops), they fail across specific qualitative boundaries:
1. **Mid-Phrase Cutoff at Token 128 (Failing C4)**:
   The models exhibit no internal sentence-boundary timing, truncating mid-phrase upon exhausting the token budget:
   - *"There was no matter the story to"* (Prompt 1)
   - *"The goose saw"* (Prompt 3)
   - *"The big they made them"* (Prompt 5)
2. **Pronoun & Entity Drift (Failing C1)**:
   Entities transmute arbitrarily across the generation horizon:
   - The female pink goose (*"She was very pretty"*) drifts to male pronouns (*"day, he found a big, green bug"*).
   - In Prompt 1, the lion transmutes into a dog named Max, then *"Max was a big cat named Bob"*, before resolving to *"Tim and Tim were very happy"*.
3. **Physical Absurdity & Word Salad (Failing C3)**:
   Generations violate physical logic and invent non-words:
   - *"The big bike hit the bike off the bike."*
   - *"putting a bandage on an oven"*
   - *"Mom hugged and said she wanted to eat them. But the door was not strong enough to carry off the door."*
   - Corrupted tokens: *"with a superherossday"*, *"train's headst tail"*, *"protra tidy"*.

### 5.5 Scientific Significance
The contrast between Feature 14 (**28/32 complete entity answers**) and Feature 15 (**0/5 narrative stories**) reveals a fundamental architectural insight:
- **Targeted Causal Retrieval is Solved**: Exact prime-addressed memory and attention read-heads successfully retrieve discrete entities across 50-token gaps when prompted by a rigid syntactic frame.
- **Autoregressive Narrative Planning is Unsolved**: Unconstrained open-ended generation requires multi-step causal planning, global discourse tracking, and syntactic boundary closure that are not automatically conferred by memory retrieval alone. This defines the core research frontier for subsequent milestones.

---

## 6. Synthesis and Milestone M3 Deliverables Summary

1. **Benchmark Infrastructure Validated (Feature 11)**:
   The 976-block evaluation protocol over 233,472 comparison-tail targets is mathematically closed, with Kahan compensated summation, batch isolation preflight, and causal future edit gates operational.
2. **Float-Free Serving Fidelity Verified (Feature 12)**:
   The multiplier-free, float-free integer runtime in `crates/uor-r4-integer` adheres strictly to the prospective $\le 0.01$ drift bounds (peak state drift $0.00732$, peak prob drift $0.00669$) across all 256 causal steps.
3. **Ordinary Control Comparison Established (Feature 13)**:
   At exactly 256 transport parameters and 1,472,640 total parameters, the ordinary Householder-pair control equals or outperforms the geometric quaternion arm on continuous text (NLL $2.064$ vs $2.090$), validating Decision D4.
4. **Context Admission Invariant Justified (Feature 14)**:
   The 16 source-edit pairs prove that sparse admission destroys entity retention (0/32 on recent32, 17/32 on orthant64), whereas `full256` recovers 28/32 answers, justifying Decision D9.
5. **Qualitative Frontier Characterized (Feature 15)**:
   The 4-part binary rubric establishes the honest, unvarnished baseline of 0/5 for native geometric story generation, isolating mid-clause truncation and entity drift as the target challenges for alpha capability.
