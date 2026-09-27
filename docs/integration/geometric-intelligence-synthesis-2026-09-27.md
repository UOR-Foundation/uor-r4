# Geometric Intelligence Synthesis: Zero-MatMul Serving & Dialogue Continuity

**Date:** 2026-09-27  
**Lab:** Lab 3 (Anti-Gravity)  
**Branch:** `lab/anti-gravity/geometric-intelligence-synthesis`  
**Tracking Issues:** References #820, #962, #963, #964, #973  

## 1. Executive Summary

This report documents the synthesis of Lab 3's core deliverables for the native geometric language model:
1. **Dialogue Streaming Continuity**: Real-time token streaming with incremental UTF-8 boundary decoding and exact-token history retention across multi-turn sessions in `uor-chat` (PR #1434).
2. **Four-Row Blocked Product Table Reuse**: Dedicated zero-allocation wide affine projections and register-resident accumulator blocked kernels for widths 576 and 1152 in `uor-r4-integer` (PR #1436).
3. **Rigorous Hardware Invariant Verification**: AArch64 disassembly auditing certifying strictly 0 Class I (multipliers), 0 Class II (dividers), and 0 Class III (floating-point) instructions in compiled numerical serving symbols.
4. **Dialogue Retention Analysis**: Comprehensive evaluation of continuous dialogue relations against quantized integer representations, connecting the findings to the learned response-aware code fitting from `fourth-lab-sequential-code-choice-fit-1`.

---

## 2. Hardware Invariant & Performance Record (Apple Silicon M1)

All measurements executed on Apple Silicon (M1-class CPU, 8 cores, 16 GB unified RAM, macOS) under the release profile (`cargo build --release -p uor-r4-integer`).

### 2.1 Static Disassembly Opcode Audit (AArch64)

Audited via `scripts/audit_zero_matmul_serving.py --tap`:

| Target Artifact | Matched Symbols | Class I (Multipliers) | Class II (Dividers) | Class III (Floats) | Verdict |
|---|---:|---:|---:|---:|:---:|
| `target/release/uor-chat` | 30 | 0 | 0 | 0 | **PASS (5/5)** |
| `target/release/libuor_r4_integer.rlib` | 30 | 0 | 0 | 0 | **PASS (5/5)** |

Emitted instruction inspection verifies zero instances of `mul`, `smull`, `umull`, `madd`, `sdiv`, `udiv`, `fmul`, `fadd`, `fmov`, or vector FP operations across all numerical kernel symbols.

### 2.2 Empirical Latency & Memory Telemetry

Measured across standard 128-token and 1,000-token evaluation passes:

| Metric | Measured Value | Hardware Ceiling | Status |
|---|---:|---:|:---:|
| **Single-Token Latency (Avg)** | **1.0008 ms/token** | $\le 4.0$ ms/token | **PASS** (4.0× margin) |
| **Latency p50** | **1.0020 ms/token** | $\le 4.0$ ms/token | **PASS** |
| **Latency p90** | **1.0240 ms/token** | $\le 4.0$ ms/token | **PASS** |
| **Latency p99** | **1.0280 ms/token** | $\le 4.0$ ms/token | **PASS** |
| **Latency Max** | **1.0320 ms/token** | $\le 4.0$ ms/token | **PASS** |
| **Peak Generation RSS** | **7.06 – 8.27 MB** | $< 35.0$ MB | **PASS** (4.2× margin) |
| **Steady-State Memory Growth** | **0.00 MB** (across 750 tokens) | $< 1.0$ MB | **PASS** (zero leak) |
| **Vocabulary Projection (4096 rows)** | **0.349 ms / call** | $< 1.0$ ms / call | **PASS** |
| **Live 4K Streaming (Quaternion)** | **1.076 ms / token** | $\le 4.0$ ms/token | **PASS** |
| **Live 4K Streaming (Householder)** | **0.983 ms / token** | $\le 4.0$ ms/token | **PASS** |
| **GPU Dependencies** | **0 (Pure CPU execution)** | 0 | **PASS** |

Numerical equivalence testing across 57 test vectors against golden scalar references confirms `max_abs_diff = 0` (bitwise exact integer arithmetic across 233,472 logits).

---

## 3. Dialogue Retention: Continuous Child vs Quantized Representations

The native dialogue evaluation panel assesses 38 multi-turn requests (58 turns total, 3,914 prediction positions) across four representation forms:
- **FF (Floating-point continuous)**: Full-precision continuous parameters and interfaces.
- **QF (Quantized parameters, continuous interfaces)**: Parameters rounded to discrete dyadic signed-4 grids `[-7, 7]`, floating interfaces.
- **QQ (Quantized parameters & interfaces)**: Parameters and state updates quantized to dyadic integer ranges.
- **Integer (Native runtime)**: Pure integer serving engine with zero-matmul arithmetic.

### 3.1 Error Localization Across the Conversion Ladder

| Step | Mean Half-L1 Diff | Greedy Token Differences / 3,914 | Assistant Target Diffs / 1,508 | Assistant NLL |
|---|---:|---:|---:|---:|
| **FF (Baseline)** | — | — | — | 1.0359 |
| **FF → QF** | **0.162424** | **909** | **237** | 1.3035 |
| **QF → QQ** | **0.001022** | **3** | **0** | 1.3035 |
| **QQ → Integer** | **0.000887** | **4** | **1** | 1.3035 |

### 3.2 Key Findings

1. **The Parameter Precision Boundary**: Over 99% of total divergence occurs at the continuous-to-quantized parameter boundary (**FF → QF**), where independent nearest-neighbor rounding perturbs delicate recurrent and read-gate balances.
2. **Integer Serving Fidelity**: The step from quantized parameters to pure integer execution (**QF → QQ → Integer**) introduces virtually zero degradation (mean half-L1 $< 0.0011$, 0 assistant greedy differences between QF and QQ, and 52/58 identical turn choices between QQ and Integer).
3. **Semantic Relation Loss under Nearest-Rounding**:
   - *Cat Name*: Continuous FF recalls "Momo"; nearest-quantized variants truncate at "The cat is named".
   - *Favorite Color*: Continuous FF recalls "green"; nearest-quantized variants lose the relation ("greenhouse" distortion).
   - *Job Recall*: Continuous FF retains teacher occupation; nearest-quantized confuses occupation with school location.
   - *Preserved Relations*: Paris capital recall, Tokyo sister location, and Blue car color survive quantization intact.

---

## 4. Synthesis with Learned Response-Aware Dialogue Codes

To bridge the precision gap without violating D0-b (strictly zero floating point or multipliers in runtime), the project developed response-aware legal integer code learning:
- **Sequential Fit Status**: `fourth-lab-sequential-code-choice-fit-1` completed 231/512 scheduled updates across 260,430 supervised target visits over 5.34M choosable coordinates before encountering a storage soft-stop.
- **Lineage Integrity**: The 231-step checkpoint preserves all 21 Adam parameter states, normalization schedule, and alpha choice variables under `/Volumes/UOR-Workspace/uor-r4-lab/fourth-lab-sequential-code-choice-fit-1/checkpoint-final`.
- **Runtime Serving Invariant**: Because code-choice learning optimizes discrete selections on the exact frozen grid `s * (lower + choice * d)` during training, the resulting codes export directly into the signed-4 packed tables. The serving runtime executes the exact same integer low-bit dot product kernels without requiring any runtime multipliers or float operations.

---

## 5. Architectural Alignment & Next Steps

- **Lab 1 Alignment**: Sparse geometric memory addressing (PR #1437) can replace wide dense affine projections with discrete $H_4 \times H_4$ Galois lattice lookups, further reducing per-token parameter bandwidth.
- **Lab 2 Alignment**: The exact signed $H_4$ classifier (PR #1435) provides common-scale integer root identification over all 120 roots, enabling discrete geometric read routing in `uor-chat`.
- **Next Decision Point**: Complete the 58-turn dialogue observation on the 231-step checkpointed codes to verify whether entity recall (Momo, green) recovers before scheduling the remaining 281 continuation updates.
