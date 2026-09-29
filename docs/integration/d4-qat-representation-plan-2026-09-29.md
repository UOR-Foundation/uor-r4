# D4 QAT Representation Plan: Gap Reduction via Quantization-Aware Training

- **Date:** 2026-09-29
- **Author:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Native Execution, and Measured Efficiency)
- **Coordinated With:** Claude main (Lab 1: Director & Architecture Owner)
- **Status:** PROPOSED RESEARCH PLAN & ADAPTER DESIGN (subject to Lab 1 review and owner approval)
- **Referenced Decisions:** D0-b, D4, D5, D9, D11, D12
- **Owning Issue:** #973 (under #820)

---

## 1. Executive Summary & Objective

The objective of Track D4 is to deliver a weight representation and codec that preserves the learned behavior of the selected geometric language model at:
$$\Delta\text{NLL} \le +0.0200\text{ nats degradation against float at equal inputs}$$
$$\text{Storage Rate} \le 4.25\text{ bits per weight (including all group scales, framing, and padding against the true unpadded weight count)}$$

Our empirical float-reference study on `geometric_s1` (PR #1479, report root `float-reference-1`) established the exact baseline:
- Float Reference: 1.998113 nats (100.00% top-1 agreement).
- S1.0 GPTQ Reference: +0.0257 nats (flip rate 8.17%).
- RTN Baseline: +0.036230 nats (90.65% agreement).
- Online Hadamard + Grouped 4-bit (H+G4): +0.027007 nats (91.28% agreement, 4.2725 bits/weight).
- Online Hadamard + E8 Lattice (H+E8): +2.423478 nats (31.18% agreement, 2.0143 bits/weight).

Under owner decision D12, neither post-training arm met the D4 gate; both remain unpromoted at this scope. Under D0-b/D11, the integer serving arithmetic is bit-identical to D10 NEON (`max_abs_logit_difference == 0` across 2,048 targets, $\le 10^{-6}$ nats). **The entire degradation is representation error.**

This plan specifies the proposed numerical mechanisms, exact storage accounting against unpadded denominators, and QAT integration hooks to close the remaining gap (+0.0270 nats $\rightarrow \le +0.0200$ nats) via Lab 1's existing Quantization-Aware Training interface.

---

## 2. Forensic Distortion Breakdown

In the S1.0 layerwise attribution (`docs/integration/s1-stack-serving-measurements-2026-09-28.md`), the representation loss from single-group perturbations is localized as follows:

| Parameter Group (S1.0c) | GPTQ $\Delta\text{NLL}$ | Top-1 Agr. | RTN $\Delta\text{NLL}$ | Top-1 Agr. | Causal Observation |
|:---|---:|---:|---:|---:|:---|
| **Output Head (`head`)** | **+0.0119 nats** | 0.9467 | **+0.0149 nats** | 0.9395 | Largest single share (46.3% of GPTQ gap). Embedding folded with final RMSNorm gain ($W_{\text{head}} = W_{\text{embed}} \cdot g_{\text{norm}}$). Logit margins between top tokens are small ($\Delta z \sim 0.1-0.5$); 4-bit grouping inverts top-1 decisions. |
| **All MLPs (`l0`–`l5.mlp`)** | **+0.0061 nats** | — | **+0.0113 nats** | — | Non-power-of-two width (749 padded to 768) introduces scale distortion. `l0.mlp` is +0.0027, `l5.mlp` is +0.0021 under GPTQ. |
| **Layer 5 (`l5.mixer_maps`, `l5.mlp`, `l5.scalars`)** | **+0.0047 nats** | — | **+0.0077 nats** | — | Deepest layer. Under GPTQ: `l5.mixer_maps` (+0.0018), `l5.mlp` (+0.0021), `l5.mixer_scalars` (+0.0008). |
| **Input Embedding (`embed`)** | **+0.0026 nats** | 0.9742 | **+0.0026 nats** | 0.9742 | Unfolded discrete lookup table. |
| **Mixer Maps Layers 0–4** | **+0.0016 nats** | — | **+0.0027 nats** | — | Recurrence and read projections in layers 0–4 (`l0`: +0.0004, `l1`: +0.0004, `l2`: +0.0001, `l3`: +0.0004, `l4`: +0.0003 under GPTQ). |
| **State Mixer Scalars (All)** | **+0.0013 nats** | 0.9919 | **+0.0013 nats** | 0.9919 | Conv taps and decay rates via 8-bit grid codes. |
| **Simultaneous Perturbation (`all`)** | **+0.0257 nats** | 0.9184 | **+0.0362 nats** | 0.9065 | Full export model evaluated against float reference. |

> [!IMPORTANT]
> **Non-Additive Perturbation Caveat:**
> Single-group perturbation measurements (evaluating one quantized group while keeping all other parameters at float) are **non-additive hypotheses**, not provably independent causal partitions.
> While the linear sum of single-group perturbations (+0.0254 nats under GPTQ) happens to align closely with the simultaneous perturbation (+0.0257 nats) on `geometric_s1`, in general non-linear neural networks contain interaction effects across layers.
> Therefore, attributing the representation gap to individual groups serves as an empirical hypothesis to prioritize QAT intervention, not a decoupled mathematical theorem.

**Finding:** The output head alone accounts for almost half of the total degradation. Any post-hoc quantizer (RTN, GPTQ, or FWHT) treats the weights as fixed and attempts minimum MSE reconstruction $\min_W \|W - \hat{W}\|_F^2$. However, minimum Frobenius weight error does NOT minimize cross-entropy loss $\mathcal{L}_{\text{CE}}$, because logit ranking is highly non-linear around the top-1 boundary.

---

## 3. Causal Interventions via Lab 1 QAT Hook

Lab 1 has implemented the QAT interface in `crates/uor-r4-training/src/geometric_stack.rs`:
```rust
pub trait MapCodec: Send + Sync {
    fn name(&self) -> &str;
    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>>;
}
```
In served training mode (`StackModel::set_served_representation`), the forward pass evaluates through `codec.round_trip()`, while the backward pass uses a Straight-Through Estimator (STE):
$$\frac{\partial \mathcal{L}}{\partial W} \approx \frac{\partial \mathcal{L}}{\partial W_{\text{served}}}$$

### Intervention A: Straight-Through Estimator (STE) Head Adaptation
- **Hypothesis:** By evaluating the output head through `Grouped4BitCodec` during the final 256–512 steps of training, the float master weights will adjust to separate the pre-quantization logit margins.
- **Expected Causal Effect:** Eliminates top-1 logit inversions caused by grid quantization, reducing output head degradation from +0.0119 nats to $< +0.0040$ nats.
- **Matched Control:** Float checkpoint fine-tuned for identical steps without `set_served_representation`.

### Intervention B: Structural Group-32 Dimension Alignment
- **Problem:** In `geometric_s1`, $d_{\text{mlp}} = 749$. When padded to whole groups of 32 ($768$), $19$ zero units per row are stored:
  $$\text{Storage} = 24 \text{ groups} \times (16\text{ B nibbles} + 1\text{ B scale}) = 408\text{ B} = 3,264\text{ bits}$$
  $$\text{Bits / Original Weight} = \frac{3,264}{749} = 4.3578\text{ bits/weight}$$
  Across the full model, this structural padding pushed total storage to **4.2725 bits/weight**, violating the $\le 4.25$ D4 gate.
- **Intervention:**
  1. For future stack architectures (S4+), enforce $d_{\text{mlp}} \equiv 0 \pmod{32}$ (e.g. $d_{\text{mlp}} = 768$ or $736$). At exact multiples of 32:
     $$\text{Storage Rate} = \frac{16 + 1}{32} \times 8 = 4.2500\text{ bits/weight}$$
  2. For legacy width-749 checkpoints, implement partial-group packing: the trailing 13 weights pack into 7 nibble bytes + 1 scale byte = 8 bytes (instead of 17 bytes), achieving $399\text{ B} = 3,192\text{ bits} \rightarrow 4.2616$ bits/weight.

### Intervention C: Conway-Sloane E8 Lattice Codebook STE
- **Problem:** H+E8 exhibited catastrophic degradation (+2.4235 nats) because post-training VQ mapped 8-dimensional vectors to the 240 minimal roots on a single sphere ($r = \sqrt{2}$), collapsing radial dynamics.
- **Intervention:** Under QAT, an 8D E8 quantizer with learned per-channel gain allows the network to learn representations that naturally cluster into E8 lattice Voronoi cells.
- **Rate Target:** 8 dimensions encoded in 8-bit index + 8-bit scale = 2.00 bits/weight, achieving extreme laptop memory reduction if calibrated via STE.

---

## 4. Bounded Experiment Protocol & Machine Safeguards

1. **Machine Slot Discipline:**
   - `/Volumes/UOR-Workspace/locks/model-slot.json` is currently held by OpenCode (Lab 2) until ~04:02 UTC.
   - Strictly **zero training runs or heavy model fits** will be launched by Lab 3 while the lock is active.
2. **Implementation Scope for Lab 3:**
   - Implement production `MapCodec` adapters directly in `crates/uor-r4-training/src/geometric_stack.rs`:
     - `D4Grouped4BitAdapter` (wrapping `Grouped4BitCodec` in RTN and MinMSE modes)
     - `HeadCompensatedMapCodec` (error-diffused group quantization for output-head logit margin preservation)
   - Maintain comprehensive unit and regression coverage in `crates/uor-r4-training/tests/d4_map_codec_adapter.rs`.
   - Ensure complete export compatibility: weights trained or saved with these adapters round-trip into `StackArtifact` without requiring runtime multiplier or divider instructions.
   - Deliver through PR #1479 to unblock Lab 1's QAT training campaign under D4/D11.
3. **Downstream Consumer:**
   - Lab 1 (Claude main) consumes `D4Grouped4BitAdapter` and `HeadCompensatedMapCodec` via `StackModel::set_served_representation` during the S1 QAT fine-tuning run.
