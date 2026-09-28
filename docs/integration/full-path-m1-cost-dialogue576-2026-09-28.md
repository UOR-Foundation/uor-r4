# Full-Path M1 Cost Qualification: Certified Width-576 Dialogue Model

**Date**: 2026-09-28  
**Author**: Lab 3 (Anti-Gravity)  
**Track**: T3 (Mission Runtime & Measured Efficiency)  
**Status**: QUALIFIED (All Alpha Acceptance Latency & Memory Invariants Satisfied)  
**References**: #963, #820, #964, #962, PR #1450  

---

## 1. Executive Summary

This evaluation establishes the first comprehensive, end-to-end full-path cost profile on Apple Silicon (M1) for the certified native width-576 dialogue model (`dialogue-child-bundle-1`).
Serving executes under the strict **D11 numerical contract**: strictly zero transformers, zero hardware matrix multiplications, zero hardware integer multipliers (`mul`, `madd`, `smull`), zero hardware dividers (`sdiv`, `udiv`), and zero floating-point operations across 100% of compiled serving symbols.

Across the complete 58-turn multi-turn dialogue panel (38 requests, 1,433 greedy decisions, 2,526 total incremental steps):
- **Bit-for-Bit Parity**: **100% PASS** (0 departures across 1,433 decisions against `responses-integer.json`).
- **Cold Load Latency**: **67.137 ms** (disk read, JSON metadata, binary tables, bundle validation).
- **Tokenizer Encode Latency**: **3.445 µs/token** (pure CPU subword tokenization).
- **Prompt Ingestion Latency**: **3.268 ms/token** (78.440 ms for a 24-token prompt context).
- **Per-Token Autoregressive Step Latency**:
  - **Mean**: **3.163 ms/step** ($\le 4.0\text{ ms}$ invariant satisfied).
  - **Median (p50)**: **2.915 ms/step**.
  - **p90**: **3.878 ms/step** ($\le 4.0\text{ ms}$ invariant satisfied).
  - **p95**: **4.907 ms/step**.
  - **p99**: **7.182 ms/step**.
  - **Throughput**: **304.3 tokens/second** (single-thread pure CPU execution).
- **Session Serialization Latency (50-Turn Context)**:
  - **Save Latency**: **0.082 ms**.
  - **Restore Latency**: **0.266 ms**.
  - **Roundtrip Parity**: Bit-identical state restoration verified.
- **Process Resident Memory (RSS)**:
  - **Cold Initial Process RSS**: **2.31 MB**.
  - **Post-Bundle-Load RSS**: **24.19 MB**.
  - **Post-Prompt-Ingest RSS**: **25.78 MB**.
  - **Replay Peak RSS**: **29.50 MB** ($< 35.0\text{ MB}$ invariant satisfied).
  - **Live Interactive \`uor-chat\` REPL RSS**: **23.22 MB** ($< 35.0\text{ MB}$ invariant satisfied with 11.78 MB headroom).
- **Analytical Bytes Touched Per Token**:
  - **Total**: **1,821,872 bytes** ($\approx 1.737\text{ MiB/token}$), well within Apple Silicon L2 cache (12 MiB shared on M1).
- **SoC Physical Energy**:
  - Marked **\`UNAVAILABLE\`** pending owner execution of pre-registered \`sudo\` harness (\`scripts/integer-m1-energy.sh\`).

---

## 2. Experimental Setup & Hardware Configuration

- **Hardware**: Apple Silicon (M1-class arm64), 8 cores (4P + 4E), 16 GiB unified memory.
- **Concurrency**: Pure single-thread execution (\`RAYON_NUM_THREADS=1\`).
- **Compiled Binary**: \`/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor-chat\`
- **Binary SHA-256**: \`4a4253a174d5af324fde647324f7830210bcb7bb0f6d5f2bdd94dcafe0c68aa4\`
- **Model Bundle**: \`/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1\`
- **Model Architecture**:
  - State dimension: $d = 576$ (INT16).
  - Read dimension: $d_k = 64$ (INT16).
  - Vocabulary size: $V = 4,096$ (4-bit packed signed tables).
  - Context capacity: $K = 256$ slots (32 persistent persona slots + 224 active dialogue slots).
  - Memory addressing: Exact prime-indexed circular ring buffer with $T^8$ Riemann zeta-zero phase coordinates and $S^3$ Hopf fibration holonomy.

---

## 3. Detailed Latency Breakdown

| Phase | Metric | Value | Invariant Ceiling | Verdict |
|---|---|---|---|---|
| **Cold Bundle Load** | Wall time | 67.137 ms | $\le 250.0\text{ ms}$ | **PASS** |
| **Tokenizer Encode** | Latency per token | 3.445 µs/tok | $\le 100.0\text{ µs}$ | **PASS** |
| **Prompt Ingestion** | Step time during ingest | 3.268 ms/tok | $\le 4.0\text{ ms}$ | **PASS** |
| **Per-Token Autoregressive Step** | Mean latency | **3.163 ms/tok** | $\le 4.0\text{ ms}$ | **PASS** |
| | Median (p50) | **2.915 ms/tok** | $\le 4.0\text{ ms}$ | **PASS** |
| | 90th percentile (p90) | **3.878 ms/tok** | $\le 4.0\text{ ms}$ | **PASS** |
| | 99th percentile (p99) | 7.182 ms/tok | $\le 12.0\text{ ms}$ | **PASS** |
| | Throughput | 304.3 tok/sec | $\ge 250\text{ tok/s}$ | **PASS** |
| **Session Save** | 50-turn context serialization | 0.082 ms | $\le 5.0\text{ ms}$ | **PASS** |
| **Session Restore** | 50-turn state deserialization | 0.266 ms | $\le 5.0\text{ ms}$ | **PASS** |

---

## 4. Memory Footprint & Analytical Traffic

### 4.1 Process Memory (RSS)
- **Baseline**: 2.31 MB
- **Post-Model Load**: 24.19 MB
- **Active Dialogue Replay Peak**: 29.50 MB
- **Live REPL Serving (\`uor-chat\`)**: 23.22 MB
- **Ceiling**: $< 35.0\text{ MB}$ (Headroom: **11.78 MB** in live REPL).

### 4.2 Analytical Bytes Touched Per Generated Token
| Component | Dimensions | Precision | Bytes Touched | Cache Location |
|---|---|---|---|---|
| Vocabulary Un-embedding Table | $4096 \times 576$ | 4-bit packed | 1,179,648 B (1.125 MiB) | L2 Cache (12 MiB) |
| Score Projection Matrix | $64 \times 576$ | 4-bit packed | 18,432 B (18.0 KiB) | L1D Cache (128 KiB) |
| Prime Memory Keys | $256 \times 64$ | INT16 (2 B) | 32,768 B (32.0 KiB) | L1D Cache (128 KiB) |
| Prime Memory Values | $256 \times 576$ | INT32 (4 B) | 589,824 B (576.0 KiB) | L2 Cache (12 MiB) |
| Active State Vector | 576 | INT16 (2 B) | 1,152 B (1.12 KiB) | L1D Cache (128 KiB) |
| $T^8$ Zeta Phase Coordinates | 8 | INT32 (4 B) | 32 B | Registers / L1D |
| $S^3$ Hopf Fiber Coordinates | 4 | INT32 (4 B) | 16 B | Registers / L1D |
| **Total Analytical Traffic** | | | **1,821,872 B (1.737 MiB)** | **100% On-Chip Cache** |

Because total per-token parameter and state traffic is 1.737 MiB, the entire active serving workload fits easily within the Apple M1's 12 MiB system-level cache, avoiding high-power DRAM bus roundtrips during token generation.

---

## 5. Artifact & Evidence Bindings

- **Evidence JSON**: \`docs/evidence/full-path-m1-cost-dialogue576-2026-09-28.json\`
- **Test Implementation**: \`crates/uor-r4-integer/tests/full_path_m1_cost.rs\`
- **Source Commit**: \`698bdda481de57c2257b158bb53f7942568a3be7\`
- **Binary SHA-256**: \`4a4253a174d5af324fde647324f7830210bcb7bb0f6d5f2bdd94dcafe0c68aa4\`
