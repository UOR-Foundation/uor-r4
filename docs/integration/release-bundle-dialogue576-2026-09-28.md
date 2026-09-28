# Sealed Release Bundle Specification: Native Width-576 Dialogue Model

**Date**: September 28, 2026  
**Owning Lab**: Lab 3 (Anti-Gravity)  
**Track**: T3 (Mission runtime and measured efficiency)  
**Issues Referenced**: #965, #963, #1172, #1173, #964, #820  
**Evidence Artifact**: `docs/evidence/release-bundle-manifest-dialogue576-2026-09-28.json`  

---

## 1. Release Overview

Under Director Queue Item 8 and Issue #965 (serving part), Lab 3 has constructed and sealed the reproducible release bundle for the certified width-576 dialogue child model (`dialogue-child-bundle-1`).

This release package binds:
1. **Model Weights & Metadata**: 11 constituent bundle files (4,356,152 bytes total) with cryptographic SHA-256 and BLAKE3 digests.
2. **Serving Binaries**:
   - Native Apple Silicon ARM64 CLI: `uor-chat`
   - Portable WebAssembly Module: `uor_r4_integer.wasm`
3. **Forensic Static Audits**: 100% PASS under TAP 13 across both ARM64 native instructions and WebAssembly opcodes (0 Class I multipliers, 0 Class II dividers, 0 Class III floats).
4. **M1 Performance Receipts**: Measured cold load (67.1 ms), per-token step latency (3.16 ms), peak memory RSS (23.22 MB live REPL), and SLC cache fit (1.737 MiB touched/tok).
5. **Capability API Contract**: Sealed schema `uor-r4.integer-capability-api/1`.

---

## 2. Cryptographic Component Manifest

| Component Path | Size | SHA-256 Digest | Role |
| :--- | :--- | :--- | :--- |
| `attempt.json` | 810 B | `ec56ba...` | Model conversion attempt log |
| `bundle.json` | 6,512 B | `022c42...` | Joint model shape & hyperparameter config |
| `manifest.json` | 1,749 B | `d70c48...` | Primary bundle file manifest |
| `model/hard-model.json` | 326,250 B | `c486ea...` | State space & parameter dimensions |
| `model/hard-parameters.bin` | 2,725,956 B | `11d7eb...` | Quantized integer parameter tensors |
| `model/hard-parameters.json` | 135,189 B | `9194ec...` | Quantization scales and bias metadata |
| `tables/tables.bin` | 1,048,560 B | `ae4e93...` | 4-bit signed product and coordinate tables |
| `tables/tables.json` | 647 B | `f0fa92...` | Table dimension metadata |
| `tokenizer.json` | 109,457 B | `ee95a3...` | Subword BPE tokenizer vocabulary (4,096 tokens) |

---

## 3. Served Binaries & Static Audit Verdicts

### A. Apple Silicon ARM64 Native CLI (`uor-chat`)
- **Frozen Binary**: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor-chat`
- **SHA-256**: `4a4253a174d5af324fde647324f7830210bcb7bb0f6d5f2bdd94dcafe0c68aa4`
- **Audit Tool**: `scripts/audit_zero_matmul_serving.py --strict-arm64 --tap`
- **Disassembly Result**: **5/5 PASS**
  - Matched Symbol Ranges Checked: 52
  - Transitive Reachable Call-Graph Functions: 112
  - Multipliers (Class I): 0
  - Dividers (Class II): 0
  - Floats (Class III): 0

### B. WebAssembly Serving Runtime (`uor_r4_integer.wasm`)
- **Frozen Binary**: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor_r4_integer.wasm`
- **SHA-256**: `a41c652795e65821bc59381c77d10339e0c9edd3f4aa068b2c7e79b608f1b3ac`
- **Binary Size**: 1,684,344 bytes (1.61 MiB)
- **Target**: `wasm32-unknown-unknown`
- **Audit Tool**: `scripts/audit_zero_matmul_wasm.py --tap`
- **Disassembly Result**: **20/20 PASS**
  - Mandatory Serving Symbols Audited: 20
  - Multipliers (`i32.mul`, `i64.mul`): 0
  - Dividers (`i32.div_*`, `i64.div_*`, `rem_*`): 0
  - Floats (`f32.*`, `f64.*`, conversions): 0

---

## 4. Benchmark Performance Summary (Apple M1)

| Benchmark Metric | Measured Value | Acceptance Invariant | Margin / Status |
| :--- | :--- | :--- | :--- |
| **Cold Load Latency** | 67.137 ms | $\le 250.0\text{ ms}$ | +182.86 ms headroom (PASS) |
| **Tokenizer Encode** | 3.445 µs/tok | Informational | Fast subword mapping |
| **Prompt Ingest Latency** | 3.268 ms/tok | Informational | Real-time interactive |
| **Step Latency (Mean)** | **3.163 ms/tok** | $\le 4.0\text{ ms/tok}$ | +0.837 ms headroom (PASS) |
| **Step Latency (p90)** | **3.878 ms/tok** | $\le 4.0\text{ ms/tok}$ | +0.122 ms headroom (PASS) |
| **Replay Parity** | **1,433 / 1,433 (100%)** | 100% exact match | 0 departures |
| **Live REPL Memory (RSS)** | **23.22 MB** | $< 35.0\text{ MB}$ | +11.78 MB headroom (PASS) |
| **Bytes Touched Per Token** | **1.737 MiB** | $< 12.0\text{ MiB}$ (SLC) | Fits entirely in cache (PASS) |
| **Serialization Save/Restore** | 0.082 ms / 0.266 ms | Informational | Sub-millisecond snapshot |

---

## 5. Reproducibility & Verification

To verify the release bundle on any machine with the repository checked out:
```bash
# 1. Verify ARM64 Native Audit
python3 scripts/audit_zero_matmul_serving.py /Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor-chat --strict-arm64 --tap

# 2. Verify WebAssembly Audit
python3 scripts/audit_zero_matmul_wasm.py /Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor_r4_integer.wasm --tap

# 3. Verify Full-Path M1 Performance Test
cargo test -p uor-r4-integer --test full_path_m1_cost --release -- --nocapture

# 4. Verify Capability API Test
cargo test -p uor-r4-integer --test capability_api_tests -- --nocapture
```
