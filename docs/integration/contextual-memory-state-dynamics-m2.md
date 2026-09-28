# Contextual Memory & State Dynamics: Architectural Specification and Synthesis (Milestone M2)

**Author**: `teamwork_preview_worker_m2_1`  
**Milestone**: M2 — Contextual Memory & State Dynamics  
**Requirement Focus**: Requirement R2 (Contextual Memory & State Dynamics)  
**Date**: 2026-09-26  
**Status**: Authoritative Architectural Synthesis Deliverable  
**Target Repository**: `~/uor-r4-worktrees/geometric-lm-goal` (Branch: `codex/geometric-lm-goal`)  
**Primary Authorities**:
- `AGENTS.md` (Native Geometric Language Model & Progressive Progress Control)
- `docs/integration/DECISIONS.md` (Decisions D0-b, D8, D9)
- `crates/uor-r4-integer/` (`model.rs`, `generation.rs`, `config.rs`, `math.rs`)
- `crates/uor-r4-core/` (`prime_route_attention.rs`, `prime_route_geometric_attention.rs`, `native_geometric/hopf_metric.rs`, `native_geometric/anchors.rs`, `canonical_lexical_ingestion.rs`)
- `crates/uor-r4-training/` (`joint_model.rs`, `joint_admission.rs`, `joint_evaluation.rs`)

---

## Executive Overview

Requirement R2 directs the project to:
> *"Investigate multi-turn state dynamics and exact prime-addressed memory retrieval within the context horizon, diagnosing and mitigating sequence collapse, entity confusion, or memory decay."*

This document delivers the comprehensive, mathematically rigorous architectural synthesis for Milestone M2. It unifies five core geometric and serving subsystems (Features 6 through 10 of `PROJECT.md`), establishes their formal interface contracts, proves their invariant conservation laws under integer serving constraints, and details the exact failure taxonomy observed across empirical evaluations.

---

## 1. 256-Token Causal Horizon Dynamics & D9 Admission Invariant (Feature 6)

### 1.1 Historical Context & The D9 Admission Mandate
During the September 2026 research campaigns, bounded candidate admission policies were explored to restrict the attention scoring footprint:
- **`Orthant64`**: Admitted at most 32 recent tokens plus up to 32 tokens retrieved from 2 partial sign tables (4 coordinates each, 16 buckets, depth 8).
- **`Recent64`**: Admitted strictly the most recent 64 tokens, truncating all preceding context.

Empirical evaluation on the 16 frozen source-edit noun substitution pairs (`crates/uor-r4-training/src/joint_evaluation.rs`) demonstrated that both bounded policies suffered catastrophic **source-retention failure**: whenever an entity was introduced in the prompt beyond the 64-token horizon, the model was mathematically blind to its presence, leading to arbitrary noun hallucinations.

Owner-directed **Decision D9** (`docs/integration/DECISIONS.md:321-322`) formally terminated the sparse admission loop and established the permanent invariant:
> *"Preserve the accepted learned-code parents and full access to all causally available events within the 256-token window."*

In the standalone integer serving runtime (`crates/uor-r4-integer/src/model.rs:130-134`), this invariant is verified at bundle loading:
```rust
if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
    || manifest.get("admission").is_some_and(|value| value != "full")
    || manifest["numerical_contract"] != crate::config::quantized_numerical_contract()?
{
    return Err(invalid("integer import requires the retained full-access numerical contract"));
}
```
Under this contract, the serving runtime operates exclusively under `full256`. It **never evicts, truncates, or overwrites** historical slots during a session.

### 1.2 Data Structures and Causal History Retention
Session history is held in `IntegerSession` (`crates/uor-r4-integer/src/model.rs:40-46`):
```rust
pub struct IntegerSession {
    identity: String,
    state: Vec<i32>,         // Recurrent state: [i32; 256] in Q11
    keys: Vec<Vec<i32>>,     // Historical write keys: capacity 256, each [i32; 64] in Q8
    values: Vec<Vec<i32>>,   // Historical write values: capacity 256, each [i32; 256] in Q14
    tokens: Vec<u32>,        // Observed token IDs: capacity 256 in u32
}
```
All three history vectors (`keys`, `values`, `tokens`) grow monotonically with each emitted token up to capacity 256.

### 1.3 Dimensional Separation & Compression Bottleneck
The architecture enforces a strict 4:1 dimensional separation between the recurrent state and the attention routing interface:
- **Recurrent State Space ($d = 256$)**: Encodes 64 quaternion 4-vectors in $S^3$. The state vector $s_t$ is 256-dimensional, fixed-point scaled in Q11 ($\pm 32767 \approx \pm 16.0$).
- **Read Space ($r = 64$)**: Attention queries and keys operate in 64 dimensions, scaled in Q8 ($\pm 32767 \approx \pm 128.0$). This compresses query-key matching by $75\%$ while forcing coordinate-level semantic consolidation.

**Projection Matrices** (`crates/uor-r4-integer/src/config.rs:67-85`):
1. `read.query.weight`: $[64, 256]$ in signed 4-bit weights with power-of-two row scales. Projects normalized provisional state (Q10) to query $q_t \in [i32; 64]$ (Q8).
2. `read.key.weight`: $[64, 256]$ in signed 4-bit weights. Projects normalized state (Q10) to key $k_t \in [i32; 64]$ (Q8).
3. `read.value.weight`: $[256, 256]$ in signed 4-bit weights. Projects normalized state (Q10) to value $v_t \in [i32; 256]$, passed through `tanh` lookup in Q14.
4. `read.no_read.weight`: $[1, 256]$ in signed 4-bit weights. Computes scalar null-read affinity competing against historical keys.

### 1.4 Attention Ingestion, Relative Age Bias & Softmax Trace
At step $t$, with `previous = session.len()`:
1. **Query Generation**:
   $$\tilde{s}_t = \text{RMSNorm}(s_t^{\text{prov}})$$
   $$q_t = \text{affine}(\tilde{s}_t, \text{"read.query"}), \quad null_t = \text{affine}(\tilde{s}_t, \text{"read.no_read"})[0]$$
2. **Key Scoring & Fixed-Point Scaling**:
   For each historical occurrence $i \in [0, previous - 1]$:
   $$\text{dot}_i = \sum_{c=0}^{63} q_t[c] \cdot k_i[c]$$
   With $q_t \in \text{Q8}$ and $k_i \in \text{Q8}$, the product is Q16. Dividing by $\sqrt{r} = \sqrt{64} = 8 = 2^3$ shifts scale to Q19 (`WORK_BITS - 19`). It is scaled to Q40 and augmented by relative age bias:
   $$\text{score}_i = \text{quantize}\left(\text{dot}_i \ll (40 - 19) + \text{read.age}[previous - 1 - i], 40, 8\right)$$
   where `read.age` is a parameter vector of length 255 initialized to $-(previous - 1 - i) / 64$.
3. **Softmax Partitioning**:
   The attention score vector has length $previous + 1$:
   $$scores = [null_t, score_0, score_1, \dots, score_{previous - 1}]$$
   Using the precomputed 65,535-entry `exp` table:
   $$w_i = \text{table.exp}[\max(scores) - scores[i]]$$
   $$masses[i] = \left\lfloor \frac{w_i \cdot 2^{48}}{\sum_j w_j} \right\rfloor$$
   After residual normalization, $\sum_{i=0}^{previous} masses[i] \equiv 2^{48}$, where $no\_read = masses[0]$ and $read\_masses = masses[1 \dots previous]$.
4. **Context Read Vector Aggregation**:
   $$read_t[c] = \text{quantize}\left(\sum_{i=0}^{previous - 1} read\_masses[i] \cdot values_i[c], 48 + 14, 11\right) \quad \forall c \in [0, 255]$$

### 1.5 Causal Write-After-Prediction Sequence
An essential causal invariant is that token $t$ **cannot read or copy itself**. The write commit occurs strictly after prediction and emission are finalized:
```
Step(token_t):
  1. Recurrent Update: provisional state computed from previous state and token_t embedding
  2. Attention Read: queries historical keys[0 .. previous-1] -> produces read_t
  3. State Finalization: blends provisional state with update(provisional, read) -> produces state_t
  4. Copy Gate Evaluation: computes gate g from [state_t, read_t]
  5. Vocabulary Logits: projects state_t -> vocabulary distribution V
  6. Probability Blending: mixes V with copy masses -> emits final probabilities P
  --- PREDICTION AND EMISSION FINALIZED ---
  7. Commit Write:
       key_t = affine(RMSNorm(state_t), "read.key")
       val_t = tanh(affine(RMSNorm(state_t), "read.value"))
       session.keys.push(key_t)
       session.values.push(val_t)
       session.tokens.push(token_t)
```
Because `session.keys.push` and `session.tokens.push` occur at step 7, token $t$ is strictly absent from `session.keys` during steps 2 through 6.

### 1.6 Boundary Dynamics at Tokens 254, 255, 256 & Prospective Budgeting
- **Token 254 (`session.len() == 254`)**: Attends to 254 historical tokens ($0 \dots 253$). The oldest token ($i=0$) reads `read.age[254 - 1 - 0] = read.age[253]`. Succeeds; pushes token to length 255.
- **Token 255 (`session.len() == 255`)**: Attends to 255 historical tokens ($0 \dots 254$). The oldest token ($i=0$) reads `read.age[255 - 1 - 0] = read.age[254]`, which is the exact final element of `read.age` ($len = 255$). Succeeds; pushes token to length 256.
- **Token 256 (Horizon Saturation)**:
  `session.len() == 256`. Any subsequent `step()` call triggers the safety guard:
  ```rust
  if session.len() >= self.config.context { // 256 >= 256
      return Err(invalid("integer session artifact/context/token mismatch"));
  }
  ```
  The step aborts immediately with an error, leaving state unmodified.
- **Prospective Budget Validation** (`crates/uor-r4-integer/src/generation.rs:299-305`):
  ```rust
  fn validate_generation_budget(existing: usize, generate: usize, capacity: usize) -> Result<()> {
      if generate == 0 || existing.checked_add(generate).is_none_or(|n| n > capacity) {
          return Err(invalid(
              "request exceeds full256 session budget; shorten the requested continuation explicitly",
          ));
      }
      Ok(())
  }
  ```
  Since `TextSession` initializes by ingesting BOS (token 0), `existing` starts at 1. The maximum allowable continuation is $256 - 1 = 255$ tokens. Requests exceeding the budget are rejected prospectively before any compute is expended.

---

## 2. Prime-Addressed Route Memory & Riemann Zeta-Zero Phase Ordinates (Feature 7)

### 2.1 Factor-Preserving Prime Identity Addressing
In `crates/uor-r4-core/src/prime_route_attention.rs`, discrete route identity is encapsulated by `PrimeAtom`:
```rust
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimeAtom(u32);
```
- **Validation Contract**: `PrimeAtom::new(value: u32)` guarantees primality using a deterministic $O(\sqrt{n})$ primality test `is_prime_u32(value)`. Values $< 2$ or composite integers are strictly rejected.
- **Register Size**: `u32` ensures that pairwise semiprime products fit in a native 64-bit word (`u64`) without multi-precision overhead.

#### Canonical Prime Registry Compilation
In `PrimeRegistry::compile(atoms: &[SemanticAtom])`:
1. Semantic inputs are sorted lexicographically by `(&atom.semantic_atom_id, &atom.payload_cid)`.
2. Prime numbers are assigned sequentially starting from **5**:
   $$p_0 = 5, \quad p_1 = 7, \quad p_2 = 11, \quad p_3 = 13, \quad p_4 = 17, \quad p_5 = 19, \quad \dots$$
3. Primes **2** and **3** are globally reserved:
   - Prime $2$ is the **canonical phase origin** (`phase_origin = PrimeAtom::new(2)`).
   - Prime $3$ is reserved for boundary constants ($BOS$).
4. The compiled registry is sealed with an immutable cryptographic BLAKE3 digest (`registry_kappa`), guaranteeing tamper-proof identity.

#### Non-Metric Identity Property
Continuous embedding spaces ($\mathbb{R}^d$) suffer from metric distortion, triangle inequality collapse, and the hubness problem. High-dimensional vector dot products cannot definitively establish whether a token appeared once, twice, or was absent.

In contrast, prime-addressed memory operates in the discrete multiplicative monoid $(\mathbb{N}, \times)$:
- Distinct primes are strictly coprime: $\gcd(p, q) = 1$ for all $p \ne q$.
- Token presence in a context multiset $N$ is binary and exact:
  $$p \mid N \iff \text{token } p \text{ occurred in context } N$$
- Identity is **topological** (membership in an algebraic lattice), completely eliminating continuous metric crowding and false geometric proximity.

### 2.2 Semiprime Experts & Euclidean GCD Handoff
Adjacent token transitions are modeled algebraically as **Semiprime Experts** (`SemiprimeExpert`):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemiprimeExpert {
    low: PrimeAtom,
    high: PrimeAtom,
}
```
- **Factor Normalization**: Factor order is normalized (`low <= high`), decoupling transition identity from temporal direction.
- **Square-Free Semiprimes vs Prime Squares**:
  - Distinct transition ($p_1 \ne p_2$): $e = p_1 \cdot p_2$ is a square-free semiprime ($\omega(e) = \Omega(e) = 2$, $\text{rad}(e) = e$).
  - Self-loop transition ($p_1 = p_2$): $e = p_1^2$ represents a valid local self-loop.
- **Euclidean GCD Handoff**: For consecutive transitions $e_t = (p_{t-1}, p_t)$ and $e_{t+1} = (p_t, p_{t+1})$, the connecting handoff token is extracted instantaneously:
  $$\gcd(e_t.\text{product}(), e_{t+1}.\text{product}()) = p_t$$
  This allows exact verification of causal path continuity in $O(\log(\min(e_t, e_{t+1})))$ integer operations without scanning historical memory tables.
- **Clifford Bivector Isomorphism**: For a sextet of carrier primes $(p_0, \dots, p_5)$, the 15 unordered factor pairs $p_i \cdot p_j$ form the Johnson graph $J(6,2)$, isomorphic to the 15 bivectors $B_{ij} = L_i L_j$ of the Clifford algebra $\text{Cl}(0,6)$ (ADR-0003).

### 2.3 Ordered N-let Multiset Encoding & Suppression of Saturated Products
A context sequence of $k$ tokens $(p_1, \dots, p_k)$ defines an n-let:
```rust
pub struct OrderedPrimeRoute {
    ordered: Vec<PrimeAtom>,
    factors: Vec<PrimeAtom>,
}
```
- `ordered`: Retains temporal sequence, causal order, and grammatical progression.
- `factors`: Stores prime atoms in strictly sorted order ($p_{(1)} \le p_{(2)} \le \dots \le p_{(k)}$).

#### The Saturated Product Overflow
In traditional Godel numbering, sequences are encoded as single scalar products $N = \prod p_i$. In an autoregressive language model, this product rapidly overflows hardware registers:
$$\prod_{i=1}^5 p_i > (4 \times 10^9)^5 \approx 10^{48} \gg 2^{128}$$
The UOR-R4 architecture strictly prohibits accumulating saturated numeric products. As verified in `tests/prime_route_attention_958.rs:288-308`:
```rust
let largest = PrimeAtom::new(4_294_967_291).expect("largest u32 prime");
let overflowing = OrderedPrimeRoute::new(vec![largest; 5]).unwrap();
assert_eq!(overflowing.checked_product_u128(), None);
assert_eq!(overflowing.factors(), &[largest; 5]);
```
#### Exact $O(n+m)$ Factor Overlap
Intersection between context routes is computed directly on the sorted factor multisets using a two-pointer merge algorithm (`OrderedPrimeRoute::factor_overlap`), guaranteeing exact multiset overlap in linear time with zero overflow risk.

### 2.4 Riemann Zeta-Zero Phase Ordinates Substrate
The connection between discrete prime routes and continuous angular dynamics originates from the explicit formula in analytic number theory connecting the prime-counting Chebyshev function $\psi(x)$ to the non-trivial zeros $\rho$ of $\zeta(s)$:
$$\psi(x) = x - \sum_{\rho} \frac{x^\rho}{\rho} - \log(2\pi) - \frac{1}{2}\log(1 - x^{-2})$$
Under the Riemann Hypothesis (RH), all non-trivial zeros lie on the critical line:
$$\rho_j = \frac{1}{2} + i\gamma_j, \quad \gamma_j \in \mathbb{R}$$
Substituting $\rho_j$ into $x^{\rho_j}$:
$$x^{\rho_j} = x^{1/2 + i\gamma_j} = \sqrt{x} \cdot x^{i\gamma_j} = \sqrt{x} \cdot \exp(i \gamma_j \log x)$$
This establishes two orthogonal coordinates:
1. **Radial Scale**: Governed by $\sqrt{x}$, yielding the Chebyshev residual $r(x) = \frac{\psi(x) - x}{\sqrt{x}}$.
2. **Angular Phase**: Governed by the log-polar frequencies $\theta_j(x) = \gamma_j \log x \pmod{2\pi}$.

In UOR-R4, the Riemann Hypothesis is adopted as an immutable coordinate-design axiom.

#### Fixed Zeta Grid & Fibonacci Spread
In `crates/uor-r4-core/src/zeta_zeros.rs`:
- An immutable table `ZETA_ZEROS: [f64; 512]` stores the imaginary parts of the first 512 non-trivial zeros from Odlyzko's benchmark dataset.
- In `prime_route_geometric_attention.rs:27`, an 8-channel sparse frequency grid is chosen via a **Fibonacci spread**:
  $$\text{ATTENTION\_ZETA\_CHANNELS} = [0, 1, 2, 3, 5, 8, 13, 21]$$
  corresponding to zeros $\{\gamma_1, \gamma_2, \gamma_3, \gamma_4, \gamma_6, \gamma_9, \gamma_{14}, \gamma_{22}\}$.
- The entire 512-zero basis is sealed by cryptographic hash `ZETA_GRID_KAPPA_REFERENCE`:
  `blake3:512243ed9e2c1deef0691515caf02ca25e3d5c7990184cd804f6d65c1cc8d94c`.

### 2.5 Log-Ratio Phase Deltas & Autoregressive Phase Transport
The angular coordinate of a prime atom $p$ in channel $j$ is defined as:
$$\theta_j(p) = \text{wrap}\left(\gamma_j \log p\right) \in [-\pi, \pi)$$
For any transition from prime $p_1$ to prime $p_2$, the phase delta is:
$$\Delta \theta_j(p_1 \to p_2) = \text{wrap}\left(\gamma_j \log\left(\frac{p_2}{p_1}\right)\right)$$

**Properties**:
1. **Scale Invariance**: Phase deltas depend exclusively on the ratio $p_2 / p_1$.
2. **Circle Group Homomorphism**: The mapping $p \mapsto \theta_j(p)$ is a homomorphism from $(\mathbb{Q}^+, \times)$ to the circle group $\mathbb{T} = \mathbb{R} / 2\pi\mathbb{Z}$:
   $$\Delta \theta_j(p_1 \to p_3) \equiv \Delta \theta_j(p_1 \to p_2) + \Delta \theta_j(p_2 \to p_3) \pmod{2\pi}$$

#### Dual Quantization Architecture
| Subsystem | Compiler / Attention Substrate | Integer Serving Runtime |
|---|---|---|
| **Type** | `PhaseQ29(i32)` | `u16` |
| **Representation** | Signed fixed-point in radians | Unsigned circle turns $\in [0, 1)$ |
| **Scaling** | $\text{round}(\text{radians} \times 2^{29})$ | $\text{round}\left(\frac{\text{radians}}{2\pi} \times 65536\right) \pmod{65536}$ |
| **Interval** | $[-2^{28}, 2^{28}) \cong [-\pi, \pi)$ | $[0, 65535] \cong \mathbb{Z} / 2^{16}\mathbb{Z}$ |
| **Arithmetic** | Wrapping with modulo normalization | Native `u16::wrapping_add` / `wrapping_sub` |
| **Float Use** | Allowed at compile time (`libm::log`) | **Zero float instructions**, zero multipliers |

In the integer serving path, updating 8 phase channels costs exactly 8 `u16::wrapping_add` instructions:
```rust
for (phase, delta) in self.phases.iter_mut().zip(geometry.phases) {
    *phase = phase.wrapping_add(delta);
}
```

### 2.6 Multi-Tier Candidate Memory & Lexicographic Energy Ranking
Candidate route memory is organized into a strict, bounded 7-row query structure (`ATTENTION_ROWS_PER_QUERY = 7`):
```
+-------------------------------------------------------------------------+
| Primary Causal Tier (Examined First)                                    |
| 1. Slot 0: I1[last_route]                  -> Unigram Transition Row    |
| 2. Slot 1: I2[previous_route, last_route]  -> Bigram Transition Row     |
| 3. Slot 2: IS[ordered_sentence_kappa]      -> Full Sentence History Row |
| 4. Slot 3: Divisor[last_route.atom]        -> Prime Factor Fallback Row |
+-------------------------------------------------------------------------+
| Secondary Fallback Tier (Activated ONLY if Primary Tier is Empty)       |
| 5. Slot 4: AdjacentSpin[Sector(center)]    -> Hopf/Torsion Center Row   |
| 6. Slot 5: AdjacentSpin[Sector(prev_bin)]  -> Preceding Torsion Bin Row |
| 7. Slot 6: AdjacentSpin[Sector(next_bin)]  -> Succeeding Torsion Bin Row|
+-------------------------------------------------------------------------+
```
- **Ceiling Invariant**: Every row admits at most `MANIFEST_MAX_CANDIDATES_PER_ROW = 128` candidates. The total candidates examined per query is strictly capped at $7 \times 128 = 896$.
- **Incremental Sentence Hash**: $I_S$ keys are maintained in $O(1)$ time per step using `OrderedSentenceRouteState`, which hashes the previous chain kappa, route count, and current route via BLAKE3 without re-hashing historical prefixes.

#### Lexicographic Energy Ranking
Candidates are ranked via a strict energy minimization hierarchy (`prime_route_geometric_attention.rs:2677-2699`):
1. **Phase Energy**: Circular distance on $\mathbb{T}^8$ between observed entry angular velocity and proposed exit angular velocity:
   $$\text{PhaseEnergy}(p_{\text{next}}) = \sum_{j=1}^8 \min(|\Delta \theta_j^{\text{in}} - \Delta \theta_j^{\text{out}}|, 2\pi - |\Delta \theta_j^{\text{in}} - \Delta \theta_j^{\text{out}}|)$$
   Matching velocity represents a smooth geodesic on the torus, yielding energy 0.
2. **Torsion Energy**: Circular distance on the retained $U(1)$ fiber phase.
3. **Spin Energy**: Geodesic acceleration on the $S^2$ Hopf base ($|next - 2\cdot last + prev|$).
4. **Factor Energy**: Distance $|p_{\text{last}} - p_{\text{next}}| + \text{divisor\_penalty}$.
5. **Source Breadth**: Count of hitting row sources ($I_1, I_2, I_S$, Divisor, Spin).
6. **Support Counts**: Frequency of co-occurrence ($I_S > I_2 > I_1 > \text{Divisor} > \text{Spin}$).
7. **Canonical Address**: Deterministic tie-breaker on `GeometricAddress`.

---

## 3. Canonical Hopf Fibration, U(1) Fiber Phase Tracking & Torsion Quantization (Feature 8)

### 3.1 Canonical Hopf Fibration Map $S^3 \to S^2$
The 3-sphere $S^3 \subset \mathbb{R}^4$ is identified with the group of unit quaternions:
$$q = a + bi + cj + dk \in \mathbb{H}, \quad a^2 + b^2 + c^2 + d^2 = 1$$
Equivalently, in two complex coordinates $(z_1, z_2) \in \mathbb{C}^2$:
$$z_1 = a + bi, \quad z_2 = c + di, \quad |z_1|^2 + |z_2|^2 = 1$$

The canonical Hopf fibration $\pi: S^3 \to S^2$ projects $q$ to a 3D unit vector $(x, y, z) \in S^2 \subset \mathbb{R}^3$:
$$\begin{aligned}
x &= 2(ac + bd) = 2\,\text{Re}(z_1 \bar{z}_2) \\
y &= 2(bc - ad) = 2\,\text{Im}(z_1 \bar{z}_2) \\
z &= a^2 + b^2 - c^2 - d^2 = |z_1|^2 - |z_2|^2
\end{aligned}$$
**Proof of Spherical Closure**:
$$x^2 + y^2 = 4(ac + bd)^2 + 4(bc - ad)^2 = 4(a^2 + b^2)(c^2 + d^2) = 4|z_1|^2 |z_2|^2$$
$$x^2 + y^2 + z^2 = 4|z_1|^2 |z_2|^2 + (|z_1|^2 - |z_2|^2)^2 = (|z_1|^2 + |z_2|^2)^2 = 1^2 = 1$$
The projection is guaranteed to map unit quaternions onto the unit 2-sphere $S^2$ (the Bloch sphere).

### 3.2 The Fiber Loss Problem & Topological Information Loss
Consider the continuous group action of $U(1) \cong S^1$ on $S^3$:
$$(z_1, z_2) \mapsto (e^{i\psi} z_1, e^{i\psi} z_2), \quad \psi \in [-\pi, \pi)$$
Evaluating the Hopf projection on the rotated state:
$$z_1' \bar{z}_2' = (e^{i\psi} z_1)(\overline{e^{i\psi} z_2}) = e^{i\psi} z_1 e^{-i\psi} \bar{z}_2 = z_1 \bar{z}_2$$
$$|z_1'|^2 - |z_2'|^2 = |e^{i\psi} z_1|^2 - |e^{i\psi} z_2|^2 = |z_1|^2 - |z_2|^2$$
**The projection $\pi(q)$ is completely invariant under the entire $U(1)$ fiber rotation.**

Consequently:
1. Every point $(x, y, z) \in S^2$ corresponds to an entire great circle $S^1$ in $S^3$.
2. The antipodal quaternions $q$ and $-q$ (corresponding to $\psi = \pi$) project to the identical base point:
   $$\pi(-q) = \pi(q)$$
   Yet in $SU(2)$, $q$ and $-q$ represent distinct spinor states (a $2\pi$ spatial rotation vs. identity).
3. Discarding the fiber phase erases polarity and causes irreversible information loss.

### 3.3 U(1) Fiber Phase Tracking & Reversible Reconstruction ($S^2 \times S^1 \to S^3$)
To recover the original state $q \in S^3$ from its observation on $S^2$, the system tracks the fiber phase $\psi \in [-\pi, \pi)$:
$$\psi = \begin{cases}
\text{atan2}(b, a) & \text{if } a^2 + b^2 > \epsilon \\
\text{atan2}(d, c) & \text{if } a^2 + b^2 \le \epsilon \text{ (near South Pole } z = -1 \text{)}
\end{cases}$$

#### Exact Bijective Reconstruction Formula
Given the base vector $(x, y, z) \in S^2$ and the fiber phase $\psi$:
1. Upper hemisphere radius:
   $$r_1 = \sqrt{\frac{1 + z}{2}} = \sqrt{|z_1|^2}$$
2. If $r_1 > \epsilon$:
   $$\begin{aligned}
   a &= r_1 \cos\psi, \quad b = r_1 \sin\psi \\
   c &= \frac{x \cos\psi + y \sin\psi}{2 r_1} \\
   d &= \frac{x \sin\psi - y \cos\psi}{2 r_1}
   \end{aligned}$$
3. If $r_1 \le \epsilon$ (at or near the South Pole $z = -1$, where $r_1 = 0$ and $r_2 = 1$):
   $$a = 0, \quad b = 0, \quad c = \cos\psi, \quad d = \sin\psi$$

This smooth local section guarantees exact, bijective reconstruction:
$$\text{Reconstruct}(\pi(q), \text{FiberPhase}(q)) \equiv q$$
Verified in `crates/uor-r4-core/src/native_geometric/hopf_metric.rs` across continuous and discrete representations (`test_hopf_fiber_reconstruction` and `test_fixed_point_q30_fiber_reconstruction_roundtrip`).

### 3.4 Continuous ($f64$) and Zero-Float Fixed-Point ($Q1.30$) Implementations
In `crates/uor-r4-core/src/native_geometric/hopf_metric.rs`:
1. **Continuous Floating-Point Layer** (`UnitS3`, `UnitS2`, `HopfFiberPoint`):
   - Used for continuous autodiff training and reference metrics.
   - Evaluates the Quantum Fubini-Study metric on $S^2$:
     $$d_{FS}(u, v) = \frac{1}{2}\arccos(u \cdot v)$$
   - Combined metric on $S^3$:
     $$d^2(p_1, p_2) = d_{FS}(\text{base}_1, \text{base}_2)^2 + \frac{1}{4}(\Delta \psi)^2$$
2. **Fixed-Point Q1.30 Serving Layer** (`UnitS3Q30`, `UnitS2Q30`, `HopfFiberPointQ30`):
   - Fixed-point format with scale $2^{30}$ (`Q30_SCALE = 1 << 30`).
   - Uses `mul_shift_add` (multiplier-free signed accumulation), integer Newton-Raphson square root (`isqrt_u64`), and a 16-step integer CORDIC arc-tangent table (`CORDIC_ATAN_TABLE_Q30`, `atan2_q30`).
   - **Zero floating-point instructions, zero dynamic heap allocations**.

```rust
// crates/uor-r4-core/src/native_geometric/hopf_metric.rs:675-703
pub fn from_hopf_fiber_q30(base: &UnitS2Q30, fiber_u1: &[i32; 2]) -> Self {
    let z = base.0[2] as i64;
    let u = fiber_u1[0] as i64;
    let v = fiber_u1[1] as i64;

    let r1_sq = ((Q30_SCALE + z).max(0) >> 1) as u64;
    let r1 = isqrt_u64(r1_sq << 30) as i64;
    let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };

    if r1 > 1024 {
        let a = (r1 * u) >> 30;
        let b = (r1 * v) >> 30;
        let x = base.0[0] as i128;
        let y = base.0[1] as i128;
        let u128 = u as i128;
        let v128 = v as i128;
        let denom = 2 * (r1 as i128);
        let c = ((x * u128 + y * v128) / denom) as i64;
        let d = ((x * v128 - y * u128) / denom) as i64;
        Self([clamp_q30(a), clamp_q30(b), clamp_q30(c), clamp_q30(d)])
    } else {
        Self([0, 0, clamp_q30(u), clamp_q30(v)])
    }
}
```

### 3.5 Torsion Quantization & State Dynamics
In `crates/uor-r4-core/src/prime_route_attention.rs:630-638`, `SpinTorsionState` is defined:
```rust
pub struct SpinTorsionState {
    pub s3: UnitS3Q30,
    pub hopf: UnitS2Q30,
    pub fiber: PhaseQ29,
    pub torsion: PhaseQ29,
}
```
Here, `fiber` tracks continuous $U(1)$ position along the circle, and `torsion` represents quantized transport winding/holonomy accumulated along the trajectory.

#### Hazards of Unobserved Fiber Loss
1. **Topological Memory Collapse**: Distinct hidden states differing only in fiber phase ($q_1 = e^{i\psi} q_2$) map to identical keys and addresses on $S^2$. The model cannot differentiate how it arrived at a base state.
2. **Holonomy Amnesia (Berry Phase Loss)**: In cyclic state updates, a closed loop on base $S^2$ encloses a solid angle $\Omega$, shifting the fiber phase by $\Delta \psi = -\frac{1}{2}\Omega \pmod{2\pi}$. Forgetting this holonomy causes the model to enter infinite periodic attractors (e.g. repeated phrase loops).
3. **Spinor Sign Erasure**: Linguistic polarities (subject vs. object, assertion vs. negation) map to antipodal spinor states $q$ and $-q$. Because $\pi(q) = \pi(-q)$, discarding the fiber collapses negation into affirmation.

---

## 4. Paired-H4 Icosian Algebra in Exact $\mathbb{Z}[\phi]$ & Fibonacci Matrix Representation (Feature 9)

### 4.1 The 600-Cell Binary Icosahedral Group $2I$ and the Icosian Ring
In 4D Euclidean space, the regular 600-cell polytope has 120 vertices that form a multiplicative subgroup of unit quaternions known as the **binary icosahedral group** $2I \subset Sp(1) \cong SU(2)$ of order $|2I| = 120$.

The coordinates of these 120 unit quaternions belong to the quadratic integer ring:
$$\mathbb{Z}[\phi] = \{ a + b\phi \mid a, b \in \mathbb{Z} \}, \quad \phi = \frac{1 + \sqrt{5}}{2}$$
where $\phi^2 = \phi + 1$ and $\phi^{-1} = \phi - 1$.

In the canonical root table (`crates/uor-r4-core/src/canonical_lexical_ingestion.rs:2163-2241`), coordinates are scaled by 2 to yield exact elements of $\mathbb{Z}[\phi]$:
1. **8 roots**: 8 permutations of $(\pm 2, 0, 0, 0)$
2. **16 roots**: 16 sign variations of $(\pm 1, \pm 1, \pm 1, \pm 1)$
3. **96 roots**: 12 even permutations $\times$ 8 sign variations of $(0, \pm 1, \pm \phi, \pm (\phi - 1))$
$$\text{Total} = 8 + 16 + 96 = 120 \text{ roots}$$

### 4.2 The Icosian Construction $E_8 \cong H_4 \oplus \phi H_4$ and Galois Coupling
By the classical construction of Conway & Sloane (SPLAG §8.2):
- The $\mathbb{Z}[\phi]$-span of the 120 icosians in $\mathbb{H}$ forms the icosian ring $I$.
- As an abelian group under addition, $I$ is a free $\mathbb{Z}$-module of rank 8.
- Equipped with the quadratic form $Q(q) = \text{Tr}_{\mathbb{Q}(\sqrt{5})/\mathbb{Q}}(N(q)) = N(q) + \sigma(N(q))$, the lattice $I$ is **isometric to the exceptional root lattice $E_8$**.

#### Project Contract & Shorthand
$$\text{Project shorthand: } E_8 = H_4 \times H_4 \quad (\text{formally } H_4 \oplus \phi H_4)$$
Each state in $E_8$ is represented by 8 integers $[a_0, a_1, a_2, a_3, b_0, b_1, b_2, b_3] \in \mathbb{Z}^8$, representing four $\mathbb{Z}[\phi]$ coordinates:
$$q_i = \frac{a_i + b_i \phi}{2}, \quad i \in \{0, 1, 2, 3\}$$

#### The Galois Coupling Constraint
The non-trivial Galois automorphism $\sigma \in \text{Gal}(\mathbb{Q}(\sqrt{5})/\mathbb{Q})$ maps $\sqrt{5} \mapsto -\sqrt{5}$, giving $\sigma(\phi) = 1 - \phi$.
Under Galois conjugation:
$$\sigma(a + b\phi) = a + b(1 - \phi) = (a + b) - b\phi$$
The coupled companion vector is defined as:
$$\text{companion}_i = \phi \cdot \sigma(q_i)$$
Evaluating the algebra:
$$\phi \cdot ((a + b) - b\phi) = (a + b)\phi - b\phi^2 = (a + b)\phi - b(\phi + 1) = -b + a\phi$$
In coordinates $(a, b) \in \mathbb{Z}^2$:
$$(a, b) \mapsto (-b, a)$$
This is an exact quarter-turn rotation $\begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$ in coefficient space!

> **CRITICAL ARCHITECTURAL INVARIANT** (`AGENTS.md`):  
> **The golden companion is NOT an independent learned state.** It is an exact algebraic function of the primary $H_4$ root: $\text{companion} = \phi \cdot \sigma(q)$. Storing or learning 8 free parameters breaks the Galois coupling constraint.

### 4.3 Fibonacci Recurrence Matrix Representation in Exact $\mathbb{Z}[\phi]$
Multiplication by $\phi$ in the basis $(1, \phi)$ is:
$$(a + b\phi)\phi = b + (a + b)\phi$$
In matrix form:
$$\begin{pmatrix} a' \\ b' \end{pmatrix} = \mathbf{F} \begin{pmatrix} a \\ b \end{pmatrix} = \begin{pmatrix} 0 & 1 \\ 1 & 1 \end{pmatrix} \begin{pmatrix} a \\ b \end{pmatrix}$$
$\mathbf{F}$ is the **Fibonacci companion matrix**. Repeated radial scaling by $\phi^n$ produces the Fibonacci sequence:
$$\mathbf{F}^n = \begin{pmatrix} F_{n-1} & F_n \\ F_n & F_{n+1} \end{pmatrix}$$

#### The Complete Operator Table in Integer Arithmetic
From `crates/uor-r4-core/src/canonical_lexical_ingestion.rs:2533-2576`:

| Operator Name | Matrix $\mathbf{M}$ | Action $(a, b) \mapsto$ | Inverse Operator | Inverse Matrix $\mathbf{M}^{-1}$ |
|---|---|---|---|---|
| `identity` | $\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}$ | $(a, b)$ | `identity` | $\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}$ |
| `golden-conjugation` | $\begin{pmatrix} 1 & 1 \\ 0 & -1 \end{pmatrix}$ | $(a + b, -b)$ | `golden-conjugation` | $\begin{pmatrix} 1 & 1 \\ 0 & -1 \end{pmatrix}$ |
| `multiply-phi` | $\begin{pmatrix} 0 & 1 \\ 1 & 1 \end{pmatrix}$ | $(b, a + b)$ | `multiply-phi-inverse` | $\begin{pmatrix} -1 & 1 \\ 1 & 0 \end{pmatrix}$ |
| `multiply-phi-inverse` | $\begin{pmatrix} -1 & 1 \\ 1 & 0 \end{pmatrix}$ | $(b - a, a)$ | `multiply-phi` | $\begin{pmatrix} 0 & 1 \\ 1 & 1 \end{pmatrix}$ |
| `phi-galois-companion` | $\begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$ | $(-b, a)$ | `inverse-phi-galois-companion` | $\begin{pmatrix} 0 & 1 \\ -1 & 0 \end{pmatrix}$ |
| `inverse-phi-galois-companion` | $\begin{pmatrix} 0 & 1 \\ -1 & 0 \end{pmatrix}$ | $(b, -a)$ | `phi-galois-companion` | $\begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$ |

Every operator is evaluated exclusively with 64-bit integer addition, subtraction, and negation—completely eliminating floating-point instructions.

### 4.4 Exact Integer Sign Determination Without Floats
In `crates/uor-r4-core/src/native_geometric/anchors.rs:144-177`, `exact_sign(value: ZPhi) -> Result<i8, _>` determines the sign of $a + b\phi$ without evaluating $\phi \approx 1.6180...$:
$$2(a + b\phi) = (2a + b) + b\sqrt{5}$$
1. If $2a + b = 0$, the sign is $\text{sgn}(b)$.
2. If $b = 0$ or $\text{sgn}(2a + b) == \text{sgn}(b)$, the sign is $\text{sgn}(2a + b)$.
3. If they have opposite signs, compare their squared magnitudes:
   $$\text{Rational}^2 = (2a + b)^2, \quad \text{Irrational}^2 = 5b^2$$
   - If $(2a + b)^2 > 5b^2 \implies \text{sgn}(2a + b)$
   - If $(2a + b)^2 < 5b^2 \implies \text{sgn}(b)$
   - Because $5$ is square-free in $\mathbb{Z}$, $(2a + b)^2 = 5b^2$ has no non-zero integer solutions.

### 4.5 Coordinate Sum Witnesses and Inverse Witnesses
In `canonical_lexical_ingestion.rs:2777-2848`:
- **Coordinate Sum Witness**: Every root $q$ in the 120-root table satisfies the Turyn quaternion norm:
  $$N(q) = \sum_{i=0}^3 q_i^2 = (4, 0) \in \mathbb{Z}[\phi]$$
- **Operator Inverse Witness**: Applying any operator followed by its inverse reproduces the exact integer tuple. Any deviation triggers an immediate integrity error.

---

## 5. Copy-Gate Mechanics & Exact 2^48 Vocabulary Blending (Feature 10)

### 5.1 Copy-Gate Architecture
The copy gate determines whether the next token is drawn from the general vocabulary or copied directly from the session prompt/history.
1. **Input Vector**: Concatenation of final recurrent state $state_t$ ($[i32; 256]$ in Q11) and attention read vector $read_t$ ($[i32; 256]$ in Q11):
   $$x_{\text{copy}} = [state_t, read_t] \in \mathbb{R}^{512}$$
2. **Linear Projection**:
   $$z_{\text{copy}} = \text{affine}(x_{\text{copy}}, 11, \text{"copy.gate"})[0] \quad (\text{Q8})$$
   Parameter `copy.gate.weight` has shape $[1, 512]$, with initial bias `copy_initial_bias = -1.0` (biasing the gate closed to favor vocabulary generation).
3. **Sigmoid Lookup**:
   $$gate = \text{table.sigmoid}[(z_{\text{copy}} + 32767) \text{ as usize}] \in [0, 32768] \quad (\text{Q15})$$

### 5.2 Copy Probability Mass Accumulation
Attention assigns probability mass to historical occurrence slots. The copy distribution $copy \in \mathbb{N}^{4096}$ accumulates over the session history:
```rust
let mut copy = vec![0u64; self.config.vocab_size];
for (&id, &mass) in session.tokens.iter().zip(&read_masses) {
    copy[id as usize] += mass;
}
```
**Properties**:
- If token ID $v$ was never observed, $copy[v] = 0$.
- If token ID $v$ appeared at multiple slots, its mass is the linear sum:
  $$copy[v] = \sum_{i \in \text{occurrences}(v)} read\_masses[i]$$
- Total copy mass across the vocabulary:
  $$\sum_{v=0}^{4095} copy[v] = \sum_{i=0}^{previous - 1} read\_masses[i] = TOTAL - no\_read \quad (TOTAL = 2^{48})$$

### 5.3 Exact Mass Conservation Blending Formulation
Let:
- $V \in \mathbb{R}^{4096}$: Softmax output over vocabulary logits, $\sum_v V_v = TOTAL = 2^{48}$.
- $C \in \mathbb{R}^{4096}$: Accumulated copy mass vector, $\sum_v C_v = TOTAL - a_0$, where $a_0 = no\_read$.
- $g \in [0, 1]$: Copy gate fraction, $g = gate / 32768$.

**Derivation**:
1. Read probability: $P(\text{read}) = \frac{TOTAL - a_0}{TOTAL} = 1 - \frac{a_0}{TOTAL}$.
2. Total copy mass available to emit is $g \cdot P(\text{read})$.
3. To preserve total mass $TOTAL = 2^{48}$, the vocabulary distribution must be scaled by:
   $$F_{\text{vocab}} = 1 - g \cdot P(\text{read}) = 1 - g \cdot \left(1 - \frac{a_0}{TOTAL}\right)$$
   In integer Q48 arithmetic (`crates/uor-r4-integer/src/model.rs:379-381`):
   ```rust
   let fraction = i128::from(TOTAL)
       - scaled(product(i128::from(gate), i128::from(TOTAL - no_read))?, -15)?;
   ```
4. Raw unregularized token probability:
   $$P_{\text{raw}}(v) = V_v \cdot F_{\text{vocab}} + C_v \cdot g$$
   In integer Q48 arithmetic:
   ```rust
   let raw = scaled(product(i128::from(v), fraction)?, -48)?
       + scaled(product(i128::from(c), i128::from(gate))?, -15)?;
   ```

**Proof of Mass Conservation**:
$$\begin{aligned}
\sum_v P_{\text{raw}}(v) &= F_{\text{vocab}} \sum_v V_v + g \sum_v C_v \\
&= \left[1 - g \left(1 - \frac{a_0}{TOTAL}\right)\right] \cdot TOTAL + g \cdot (TOTAL - a_0) \\
&= TOTAL - g(TOTAL - a_0) + g(TOTAL - a_0) \\
&= TOTAL = 2^{48}
\end{aligned}$$
The total probability mass is conserved identically.

### 5.4 Regularization & Residual Normalization
1. **Uniform Floor**: To prevent zero probabilities, an exact $10^{-8}$ uniform distribution floor is mixed in:
   $$P_{\text{uniform}}(v) = \left\lfloor \frac{P_{\text{raw}}(v) \cdot 99{,}999{,}999 + (TOTAL \gg 12)}{100{,}000{,}000} \right\rfloor$$
2. **Residual Normalization**: Integer rounding can introduce residual drift ($\pm \text{a few units}$). `normalize_residual` sums the distribution, computes $\Delta = TOTAL - \sum_v P(v)$, and absorbs $\Delta$ directly into the token with the largest probability:
   ```rust
   if total <= TOTAL {
       values[largest] += TOTAL - total;
   } else {
       values[largest] -= total - TOTAL;
   }
   ```
   This guarantees bitwise exact normalization $\sum_v P(v) \equiv 2^{48}$ on every step.

---

## 6. Failure Modes & Entity Confusion Taxonomy

Empirical evaluation across the 16 frozen source-edit pairs (`joint_evaluation.rs`), adversarial tests (`adversarial_challenge.rs`), and the native geometric records (`historical_version_intent.rs`) identifies five structural failure modes:

```
+----------------------------------------------------------------------------------------------------+
|                                    ENTITY CONFUSION TAXONOMY                                       |
+------------------------------+------------------------------------+--------------------------------+
| Failure Mode                 | Root Cause                         | Architectural Impact           |
+------------------------------+------------------------------------+--------------------------------+
| 1. Token-Level Summation     | copy[id] += mass sums occurrences  | Positional/role binding lost   |
| 2. Linear Age Bias           | -(age)/64 penalty on past tokens   | Distractors overpower targets  |
| 3. 33-Root Hashing Bottleneck| prime % 120 maps 256 onto 33 roots | 5-bit lossy geometric hash     |
| 4. Unseen Prefix Collapse    | Suffix match + IS miss             | Identical candidate energy     |
| 5. Subword Cycle Traps       | Frequent words aggregate mass      | Infinite period 1..4 loops     |
+------------------------------+------------------------------------+--------------------------------+
```

### 6.1 Token-Level Summation vs. Positional/Role Collapse
- **Mechanism**: The copy array accumulates by token ID:
  $$copy[id] = \sum_{i: token_i = id} read\_masses[i]$$
- **Failure**: If an entity appears in multiple conflicting roles across history (e.g. *"Alice called Bob. Bob called Charlie."*), attending to "Bob" produces a single copy probability mass for token `Bob`. The model cannot bind *which* occurrence of Bob was selected. Positional and grammatical binding is lost in the copy blend.

### 6.2 Linear Age Bias Overpowering Semantic Distractors
- **Mechanism**: Attention scores include $+ read.age[previous - 1 - i]$, initialized to $-(previous - 1 - i) / 64$.
- **Failure**: A token 10 steps ago receives an age penalty of $-10 / 64 = -0.156$. A target entity mentioned 80 steps ago receives an age penalty of $-80 / 64 = -1.250$ (a penalty of $> 1.0$ logit units).
  - In the frozen source-edit evaluation:
    *"One day, Lily carried her apple into the garden. She put it on the little table beside the door. ... Lily went back to the table and picked up the [apple.]"*
  - The noun `"door"` or `"table"` is much closer to the generation point than `"apple"`.
  - Under Post-Training Quantization (PTQ) or weak query-key margins, the age advantage of `"door"` overpowers the query-key affinity for `"apple"`, causing the model to emit `"door."` instead of the true object.

### 6.3 Same-Value Reassertions vs. True Revisions
- **Mechanism**: In conversation memory, a fact may be re-asserted without changing its value (Turn 1: *"Mara is in Harbor"*; Turn 3: *"Mara is in Harbor"*).
- **Failure**: Each assertion creates a distinct key-value entry with an updated timestamp. The older root assertion receives an increased age penalty. When queried for the *initial* location versus the *previous* location, standard attention strongly biases towards the most recent assertion due to the linear age decay, effectively treating a same-value reassertion as an eviction of older history.
- **Remedy**: In `crates/uor-r4-core/src/native_geometric/historical_version_intent.rs`, this necessitated adding explicit structural candidate classes (`reassertion_links`, `reassertion_heads`, `CLASS_HEAD_TRUNCATED`) to prevent same-value reassertions from corrupting chain traversal.

### 6.4 The 33-Root Hashing Bottleneck
- **Mechanism**: The historical token assignment `leaf = (prime % 120) as u16` maps vocabulary tokens onto roots of the 600-cell.
- **Failure**: Because primes $> 5$ are coprime to 120, their residues modulo 120 can only assume values in the multiplicative group $(\mathbb{Z}/120\mathbb{Z})^\times$, which has order $\phi(120) = \phi(8) \phi(3) \phi(5) = 4 \times 2 \times 4 = 32$. Including the small primes 2, 3, 5:
  $$|\{p \bmod 120 \mid p \in \mathbb{P}\}| = 32 + 1 = 33 \text{ distinct residues}$$
- **Impact**: The 256 byte tokens collapse onto **only 33 of the 120 roots**. Up to 11 different tokens collide onto the same root leaf, turning unlearned geometric placement into a 5-bit lossy hash.
- **Remedy**: Learned state transitions must rely on the **learned continuous/Q1.30 transport path** in `crates/uor-r4-integer/src/model.rs` and the full 256-dimensional causal state space, using the 120-root $H_4$ table as invariant anchors and error-detecting witnesses rather than a static unlearned token dictionary.

### 6.5 Unseen Global Prefix Collapse
- **Mechanism**: Demonstrated empirically in `tests/prime_route_geometric_attention_958.rs:514-550`. Consider two distinct prompts:
  $$\text{Prompt}_1 = [p, b, c_0], \quad \text{Prompt}_2 = [q, b, c_0]$$
  Both prompts share the recent bigram suffix $(b, c_0)$, but differ in their initial entity token ($p \ne q$). If the complete sentences are unseen in training, the sentence index misses ($I_S \to \emptyset$).
  Consequently:
  - Row $I_1[c_0]$ is identical for both.
  - Row $I_2[b, c_0]$ is identical for both.
  - Row $I_S$ is absent for both.
  - Divisor and Adjacent Spin fallbacks depend only on $c_0$.
  - Phase energy depends only on $\Delta \theta(b \to c_0)$ and candidate $c_0 \to \text{next}$.
- **Result**: The candidate support, candidate ranking, and selected token for $\text{Prompt}_1$ and $\text{Prompt}_2$ become **completely identical**.
- **Remedy**: Long-range prefix state must be bridged via the H4 cumulative path-lease mechanism (`select_path_or_abstain`) or the local context placement overlay rather than relying solely on $I_S$ exact match.

### 6.6 Subword Cycle Traps & Short-Period Loops
- **Mechanism**: Common words (e.g., `"the"`, `"it"`, `"she"`, `","`) appear repeatedly across the 256-token context. Because copy masses sum over all occurrences, diffuse background attention aggregates into a dominant copy probability for high-frequency tokens.
- **Impact**: This induces repetitive degeneration and cycle traps. In `crates/uor-r4-integer/src/generation.rs:307-319`, this failure mode forced the implementation of `short_cycle(&generated)`:
  ```rust
  fn short_cycle(tokens: &[u32]) -> Option<usize> {
      for period in 1..=4 {
          let twice = period << 1;
          let span = twice + period;
          if tokens.len() >= span {
              let tail = &tokens[tokens.len() - span..];
              if tail[..period] == tail[period..twice] && tail[..period] == tail[twice..] {
                  return Some(period);
              }
          }
      }
      None
  }
  ```
  If three consecutive identical sequences of length 1, 2, 3, or 4 occur, generation is forcefully halted.

---

## 7. Synthesis & Architectural Recommendations for Milestone M2

1. **Strictly Retain Full 256 Causal Horizon**: Decision D9 is validated. Bounded admission (`Orthant64`, `Recent64`) destroys long-range entity retention. Full causal history access is computationally efficient on host M1 and must remain the foundation of serving.
2. **Decouple Age Decay from Long-Range Entity Retrieval**: The linear age slope ($-1/64$) penalizes tokens at distance 128 by $-2.0$ logits. M2/M3 tuning should evaluate learned multi-scale age initialization (e.g. log-spaced decay) to protect distant prompt entities from recency bias.
3. **Preserve Exact Holonomy Tracking**: The zero-float Q1.30 Hopf fibration and 8-channel Riemann zeta phase transport provide essential holonomy tracking that prevents cyclic attractor collapse and distinguishes spinor polarities.
4. **Enforce Galois Coupling**: The golden companion $\phi H_4$ must remain strictly bound to the primary root via $\begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$. Decoupling the companion into free parameters violates the $E_8$ icosian lattice structure.
5. **Bridge Recurrent LM with Exact Relational Directory**: Raw token-level copy attention cannot replace structured relation records (`RelationState` with `owner`, `value`, `previous` pointers). Future milestones must bridge the integer recurrent LM with exact-addressed relation directories to achieve robust multi-turn factual memory.

---
*Synthesized and delivered by `teamwork_preview_worker_m2_1`.*
