# B3: E8 Lattice Weight Coding on SmolLM2-360M MLP Layers (Track B Study)

**Date**: 2026-09-29  
**Lab**: Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Native Kernels, Measured Cost)  
**Governing Directive**: Lab 1 Board [#1513](https://github.com/UOR-Foundation/uor-r4/issues/1513), Epics [#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509) (Track B) and [#1510](https://github.com/UOR-Foundation/uor-r4/issues/1510) (Infra)  
**Evidence Artifact**: `docs/evidence/b3-e8-smollm2-360m-2026-09-29.json`  
**Sealed Report Root**: `/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-full-32layers`  
**Execution Binary**: `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/b3-e8-smollm2`  

---

## 1. Executive Summary & Scientific Verdict

Under Lab 1 directive B3, Anti-Gravity evaluated geometric lattice weight coding using the canonical $E_8$ lattice codebook on all 32 layers of SmolLM2-360M-Instruct's MLP weights (`gate_proj`, `up_proj`, `down_proj`, totaling 235,929,600 weights) at 2, 3, and 4 bits per weight against standard 4-bit round-to-nearest (RTN).

### Pre-Registered Kill Criterion
- **Rule**: `Kill: 3-bit E8 is more than 0.05 nats worse than round-to-nearest 4-bit. Record it and stop.`
- **Measured**:
  - $\Delta \text{NLL}_{\text{RTN-4bit}} = +0.080876 \text{ nats}$ (at 4.2764 bpw)
  - $\Delta \text{NLL}_{\text{E8-3bit}} = +1.083241 \text{ nats}$ (at 3.0000 bpw)
  - Excess degradation over RTN 4-bit: $\mathbf{+1.002365 \text{ nats}}$
- **Verdict**: **`KILL CRITERION TRIGGERED (+1.002365 > 0.0500 nats)`**.

Per [D12](DECISIONS.md#d12--negative-evidence-retention-and-promotion-gates) and [D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract), this negative result is preserved at its exact technical scope. Offline lattice vector quantization of dense MLP weights without retraining fails to preserve transformer language behavior at 3.0 bits per weight. Anti-Gravity halts the Track B second lever without parameter sweeps or ungrounded retries, and advances immediately to **Item 3: Metal Kernel Port** for 30M model training acceleration.

---

## 2. Experimental Setup & Pre-Registered Arms

- **Target Model**: SmolLM2-360M-Instruct
  - Path: `/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct`
  - Weight SHA-256: `e6bffe7435d7ddc10fd3b9a9efd429dafbacb1cb17015fb5562664e7532bf86e`
  - Architecture: `LlamaForCausalLM` ($L=32, d_{\text{model}}=960, d_{\text{ffn}}=2560, V=49152$)
- **Quantization Scope**:
  - Layers: $0..32$ (all 32 layers)
  - Matrices per layer: `gate_proj`, `up_proj`, `down_proj` ($3 \times 32 = 96$ matrices)
  - Weights per layer: $3 \times (960 \times 2560) = 7,372,800$ weights
  - Total quantized weights: $\mathbf{235,929,600 \text{ weights}}$
  - Note on alignment: hidden width 960 and intermediate width 2560 are exactly divisible by 8 (E8 block dimension) and 32 (RTN group size), ensuring 0 zero-padding overhead.
- **Evaluation Dataset**:
  - Tokens: `/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16`
  - Evaluation protocol: 8 windows of context length $T=256$, batch size 1 ($2,048$ evaluation tokens total)
- **Codecs Evaluated**:
  1. **Arm 0 (Float Baseline)**: Full precision unquantized $f32$ reference.
  2. **Arm 1 (RTN 4-bit)**: Group size 32, 16 code bytes + 1 scale byte per 32 weights ($17/32 = 4.2500$ bpw payload).
  3. **Arm 2 (E8 2-bit)**: 1-stage $E_8$ vector quantization using the 241 canonical roots of $E_8$ (`uor_r4_integer::codec::E8Codebook`), 1 index byte + 1 scale byte per 8 weights ($2/8 = 2.0000$ bpw).
  4. **Arm 3 (E8 3-bit)**: 2-stage residual $E_8$ lattice vector quantization with coupled block scale, 2 index bytes + 1 scale byte per 8 weights ($3/8 = 3.0000$ bpw).
  5. **Arm 4 (E8 4-bit)**: 2-stage residual $E_8$ lattice vector quantization with two independent scales, 2 index bytes + 2 scale bytes per 8 weights ($4/8 = 4.0000$ bpw).

---

## 3. Measured Empirical Results

All metrics were computed by `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/b3-e8-smollm2` and sealed into `docs/evidence/b3-e8-smollm2-360m-2026-09-29.json`:

| Arm | Codec Description | Target bpw | Serialized bpw | Total Serialized Bytes | Mean NLL (nats) | $\Delta$ NLL vs Float | KL vs Teacher | Top-1 Agr (%) | Codec Roundtrip Gate |
|:---:|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Arm 0** | Float Reference Baseline | 32.00 | 32.0000 | 943,718,400 | 9.454779 | 0.000000 | 0.000000 | 100.00% | N/A |
| **Arm 1** | RTN 4-bit (Group size 32) | 4.25 | 4.2764 | 126,116,608 | 9.535655 | +0.080876 | 0.156995 | 61.52% | **PASSED** |
| **Arm 2** | E8 2-bit (1-stage E8) | 2.00 | 2.0000 | 58,983,552 | 15.410751 | +5.955971 | 8.062934 | 3.27% | **PASSED** |
| **Arm 3** | E8 3-bit (2-stage residual) | 3.00 | 3.0000 | 88,474,752 | 10.538020 | +1.083241 | 2.175266 | 13.62% | **PASSED** |
| **Arm 4** | E8 4-bit (2-stage residual, 2 scales) | 4.00 | 4.0001 | 117,966,336 | 10.020737 | +0.565957 | 1.548932 | 20.56% | **PASSED** |

### Per-Window Negative Log-Likelihoods (nats)
- **Arm 0 (Float)**: `[9.1990, 9.7272, 9.3151, 9.7833, 9.8666, 9.1119, 9.5055, 9.1296]`
- **Arm 1 (RTN 4-bit)**: `[9.2707, 9.6937, 9.3866, 9.9108, 9.9631, 9.1497, 9.6475, 9.2631]`
- **Arm 2 (E8 2-bit)**: `[15.3792, 16.0318, 14.6360, 15.2763, 16.3360, 15.5366, 14.5176, 15.5726]`
- **Arm 3 (E8 3-bit)**: `[10.3751, 10.4352, 10.1280, 10.5850, 10.7146, 10.2878, 10.7712, 11.0071]`
- **Arm 4 (E8 4-bit)**: `[9.7986, 10.2740, 9.9081, 10.2495, 10.3144, 9.7774, 10.1761, 9.6679]`

---

## 4. Analysis of Findings

1. **Exact Codec Round-Trip Verification**:
   Every one of the 96 MLP weight matrices across all four quantization arms passed exact bit-for-bit serialization and deserialization checks before dequantization, ensuring that serialization bugs are completely ruled out as a source of degradation.
2. **Bits Per Weight Accounting**:
   Bitrates were computed strictly from stored file bytes:
   $$\text{bpw} = \frac{8 \times \text{serialized\_bytes}}{\text{quantized\_weights}}$$
   The E8 codecs achieve exact theoretical storage: 2.0000 bpw for 2-bit, 3.0000 bpw for 3-bit, and 4.0001 bpw for 4-bit.
3. **Lattice Vector Quantization Collapse**:
   - At 2 bits per weight, 1-stage $E_8$ vector quantization collapses completely ($\Delta \text{NLL} = +5.955971\text{ nats}$, top-1 agreement $3.27\%$).
   - At 3 bits per weight, 2-stage residual $E_8$ achieves $10.538020\text{ nats}$ ($\Delta \text{NLL} = +1.083241\text{ nats}$), which is $\mathbf{+1.002365\text{ nats}}$ worse than RTN 4-bit, severely triggering the kill criterion ($> 0.05\text{ nats}$).
   - Even at 4 bits per weight, 2-stage residual $E_8$ suffers $\Delta \text{NLL} = +0.565957\text{ nats}$ and only $20.56\%$ top-1 agreement, compared to RTN 4-bit which achieves $\Delta \text{NLL} = +0.080876\text{ nats}$ and $61.52\%$ agreement.
4. **Physical & Geometric Cause**:
   Standard transformer MLP matrices have heavy-tailed coordinate distributions aligned with individual Cartesian axes rather than isotropic 8D sphere/lattice packings. Without pre-conditioning rotations (such as Walsh-Hadamard transforms) or quantization-aware retraining (QAT), vector quantization over the discrete $E_8$ root lattice introduces severe directional quantization errors that compound through the SwiGLU activation across all 32 layers.

---

## 5. Next Steps

With Item 2 completed and preserved as a definitive negative:
1. Deliver Item 2 via PR referencing #1513, #1509, #820.
2. Update `docs/integration/current-state.md` to reflect the B3 kill criterion verdict.
3. Immediately advance to **Item 3: Metal Kernel Port** to unlock training throughput for 30M models (`cpu-accelerate` baseline first, then 7 custom stack ops in `geometric_stack.rs`).
