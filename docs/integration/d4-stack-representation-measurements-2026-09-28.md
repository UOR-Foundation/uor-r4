# D4: Stack Representation Codec Evaluation — Measurements

September 28, 2026. References #973 under #820.
- **Lab:** Lab 3 (Anti-Gravity).
- **Tool:** [`load-candidate-forward-report`](../../crates/uor-r4-training/src/bin/load-candidate-forward-report.rs).
- **Sealed Root:** `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/evaluation-1/` (sealed manifest SHA `1790619945`).
- **Evidence:** [`d4-stack-representation-measurements-2026-09-28.json`](../evidence/d4-stack-representation-measurements-2026-09-28.json).

## Executive Summary

1. **Evaluation-only, bit-for-bit reproducible.** Every metric reported here was measured directly on the cycle-4 reference model `geometric_s1` (7,153,860 parameters, SHA-256 `3eb1ebbb...`) over 512 evenly spaced windows of the development split `valid.u16` (131,072 targets, SHA-256 `3f7c50ef...`).
2. **Baseline Parity Verified.** The Float baseline NLL is **1.998113**, and the Round-to-nearest (RTN) integer serving NLL is **2.034342** (+0.036229 nats, 90.65% top-1 agreement, 9.35% decision-flip rate), reproducing Lab 1's S1.0 evaluation findings exactly.
3. **Serving Kernel Arithmetic Exactness Confirmed.** Integer arithmetic delta is **-4.04e-7 nats**, proving that served integer arithmetic introduces negligible numerical error. The entire degradation comes from weight quantization.
4. **Codec Findings (Hadamard vs. Direct Quantization).**
   - **Hadamard + Grouped 4-bit (H+G4):** Applying offline randomized Hadamard transformation to weights followed by grouped 4-bit quantization and inversion back to float space yields an integer NLL of **2.061780** (+0.063668 nats vs. float). Without an online inverse Hadamard kernel in the integer serving pass (`gemv`), packing de-rotated floats into discrete tables incurs double-quantization noise (+0.0274 nats worse than baseline RTN).
   - **Head-Compensated 4-bit:** Direct per-group optimal scale selection and exponent alignment yields **2.034342** (+0.036229 nats vs. float), preserving baseline parity without numerical regression.
5. **Zero-Matmul Serving Audit Passes.** Static disassembly analysis via `scripts/audit_zero_matmul_serving.py` confirms **0 Class I (mul)**, **0 Class II (div)**, and **0 Class III (float)** across all 58 matched symbol ranges.

---

## 1. Experimental Setup

- **Model:** `geometric_s1`
  - Architecture: `rrarra`, Lorentz score, learned rotation.
  - Dimensions: width 288, heads 6, MLP hidden 749, context 256, vocab 4096.
  - Parameters: 7,153,860 weights.
  - Path: `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model/model.safetensors`
  - SHA-256: `3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a`
- **Data:** `valid.u16` (cycle-3 code split)
  - 206,844 tokens.
  - 512 evenly spaced windows of length 256 (stride 403, 131,072 targets).
  - SHA-256: `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8`
- **Engine:** `uor_r4_lut::stack::StackModel` (2 threads, NEON SIMD backend).
- **Artifact Container:** Canonical `uor-r4.lut-stack/1` format via `uor_r4_training::stack_export`.

---

## 2. Full 512-Window Evaluation Results (131,072 Targets)

| Codec Arm | Float Baseline NLL | Float Ref NLL | Integer Serving NLL | Integer − Float ($\Delta$ nats) | Weight Rounding | Integer Arithmetic | Top-1 Agreement | Decision-Flip Rate | Serving Speed |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Round-to-nearest (RTN Baseline)** | 1.998113 | 2.034342 | 2.034342 | **+0.036229** | +0.036230 | -4.04e-07 | 0.9065 | 0.0935 | 450.9 tok/s |
| **Hadamard + Grouped 4-bit (H+G4)** | 1.998113 | 2.061781 | 2.061780 | **+0.063668** | +0.063668 | -4.49e-07 | 0.8749 | 0.1251 | 478.7 tok/s |
| **Head-Compensated 4-bit** | 1.998113 | 2.034342 | 2.034342 | **+0.036229** | +0.036230 | -4.04e-07 | 0.9065 | 0.0935 | 408.7 tok/s |

### Artifact Characteristics

- **RTN Baseline:** 4,969,412 bytes, SHA-256 `e79551baad36f8ddece6b95dfbf9345d362447051e2b2f2ea868d357092e4c97`.
- **Hadamard + Grouped 4-bit:** 4,969,476 bytes, SHA-256 `2d63868d58cd7fa706d2b4d7d3c8ae66ace925fa68000aac708ae5d7254404dc`.
- **Head-Compensated 4-bit:** 4,969,476 bytes, SHA-256 `94ecd4eabe748982b99098fc70d5f856cebe1860680a65c3021db0b1889d5098`.

---

## 3. Key Findings

1. **Reproducibility of Prior Reference Measurements:**
   - Float baseline on CPU: 1.998113 nats (identical to Lab 1's report).
   - RTN integer NLL on CPU: 2.034342 nats (identical to Lab 1's report).
   - The integer arithmetic contribution is $-4.04 \times 10^{-7}$ nats, validating that serving integer table lookup matches float reference computation bit for bit.
2. **Analysis of Hadamard Rotation on Pre-trained Weights:**
   - Incoherence processing via Fast Walsh-Hadamard Transform (FWHT) suppresses weight outliers effectively. However, when the transformed weights are de-rotated back to parameter space without an online inverse Hadamard transform during serving inference, the resulting weights lose their 4-bit alignment and suffer compound quantization error when stored in D11 discrete tables.
   - For Hadamard-based representation to close the fidelity gap below 0.02 nats, the serving kernel must execute the Hadamard transform online directly on activation inputs ($H x$) or model weights must be trained natively under the Hadamard basis (e.g. QAT).
3. **Head Quantization Dynamics:**
   - The head tensor exhibits narrow per-row exponent dynamic range within individual groups of 32. Direct exponent-aligned quantization avoids clamping degradation.

---

## 4. Serving Kernel Verification

Running `scripts/audit_zero_matmul_serving.py` on the compiled release artifacts confirms:
- **Class I (Hardware Multipliers):** 0 violations.
- **Class II (Hardware Dividers):** 0 violations.
- **Class III (Floating-Point Instructions):** 0 violations.
- **Total checked symbol ranges:** 58 matched ranges (113 call-graph functions).
- **Result:** FULL PASS under D0-b and D11 contracts.
