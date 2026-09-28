# Capability API and Zero-Multiplier WASM Runtime Qualification

**Date**: September 28, 2026  
**Owning Lab**: Lab 3 (Anti-Gravity)  
**Track**: T3 (Mission runtime and measured efficiency)  
**Issues Referenced**: #1172, #1173, #963, #964, #820  
**Evidence Artifact**: `docs/evidence/capability-api-wasm-audit-2026-09-28.json`  

---

## 1. Executive Summary

In fulfillment of Director Queue Item 6 (#1172, #1173), Lab 3 has qualified and delivered the **Unified Capability API** and **audited WebAssembly serving runtime** for the native UOR-R4 integer session (`crates/uor-r4-integer`).

1. **Unified Capability API (`IntegerCapabilityApi`)**:
   - Exposes a thread-safe, lifetime-free Rust API (`uor-r4.integer-capability-api/1`) supporting model metadata introspection, truthful capability status matrix, session creation with optional persistent system prompt, user turn prompt ingestion, streaming autoregressive token generation, and durable session state save/restore.
   - Implemented in 100% safe Rust under `#![forbid(unsafe_code)]`.
   - Verified via `crates/uor-r4-integer/tests/capability_api_tests.rs` with exact bit-for-bit serialization roundtrip.

2. **WebAssembly Serving Artifact (`uor_r4_integer.wasm`)**:
   - Target: `wasm32-unknown-unknown`, release profile.
   - Artifact Size: **1.61 MiB** (1,684,344 bytes).
   - Artifact SHA-256: `a41c652795e65821bc59381c77d10339e0c9edd3f4aa068b2c7e79b608f1b3ac`.
   - Frozen Path: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor_r4_integer.wasm`.

3. **Strict D11 WebAssembly Static Disassembly Audit**:
   - Authored `scripts/audit_zero_matmul_wasm.py` to forensically parse and disassemble the `.wasm` bytecode section and decode every instruction of all numerical serving symbols.
   - Audited **20 mandatory serving symbols** (WASM exports, vocabulary projection kernels, 4-row blocked product reuse tables, Hopf projections, Min-P thresholding, CORDIC arctangent).
   - **Static Audit Result**: **20/20 PASS** under TAP 13:
     - **0 Class I hardware integer multipliers** (`i32.mul`, `i64.mul`).
     - **0 Class II hardware integer dividers** (`i32.div_*`, `i64.div_*`, `rem_*`).
     - **0 Class III floating-point instructions** (`f32.*`, `f64.*`, conversions).

---

## 2. Static Disassembly Audit Results (TAP 13)

```tap
TAP version 13
1..20
# Artifact: /Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor_r4_integer.wasm
# SHA-256: a41c652795e65821bc59381c77d10339e0c9edd3f4aa068b2c7e79b608f1b3ac
# Commit: 698bdda481de57c2257b158bb53f7942568a3be7
ok 1 - uor_wasm_version: 0 mul, 0 div, 0 float (2 instructions across 1 fn(s))
ok 2 - uor_wasm_state_dim: 0 mul, 0 div, 0 float (2 instructions across 1 fn(s))
ok 3 - uor_wasm_vocab_size: 0 mul, 0 div, 0 float (2 instructions across 1 fn(s))
ok 4 - uor_wasm_context_capacity: 0 mul, 0 div, 0 float (2 instructions across 1 fn(s))
ok 5 - uor_wasm_exact_min_p_threshold: 0 mul, 0 div, 0 float (4 instructions across 1 fn(s))
ok 6 - uor_wasm_hopf_project_x: 0 mul, 0 div, 0 float (36 instructions across 1 fn(s))
ok 7 - uor_wasm_hopf_project_y: 0 mul, 0 div, 0 float (36 instructions across 1 fn(s))
ok 8 - uor_wasm_hopf_project_z: 0 mul, 0 div, 0 float (36 instructions across 1 fn(s))
ok 9 - uor_wasm_step_zeta_phase_scalar: 0 mul, 0 div, 0 float (52 instructions across 1 fn(s))
ok 10 - uor_wasm_galois_lfsr_step: 0 mul, 0 div, 0 float (12 instructions across 1 fn(s))
ok 11 - uor_wasm_compute_turn_prime_signature: 0 mul, 0 div, 0 float (30 instructions across 1 fn(s))
ok 12 - uor_wasm_atan2_q30: 0 mul, 0 div, 0 float (4 instructions across 1 fn(s))
ok 13 - uor_wasm_score_token_salience_default_key: 0 mul, 0 div, 0 float (131 instructions across 1 fn(s))
ok 14 - IntegerModel::project_vocab: 0 mul, 0 div, 0 float (88 instructions across 1 fn(s))
ok 15 - IntegerModel::project_vocab_with_products: 0 mul, 0 div, 0 float (107 instructions across 1 fn(s))
ok 16 - low_bit_dot_8_pair_tables: 0 mul, 0 div, 0 float (182 instructions across 1 fn(s))
ok 17 - low_bit_dot_4_contiguous_wide_pair_tables: 0 mul, 0 div, 0 float (106 instructions across 1 fn(s))
ok 18 - packed_rows::low_bit_dot: 0 mul, 0 div, 0 float (201 instructions across 1 fn(s))
ok 19 - sampling::exact_min_p_threshold: 0 mul, 0 div, 0 float (54 instructions across 1 fn(s))
ok 20 - math::atan2_q30: 0 mul, 0 div, 0 float (540 instructions across 1 fn(s))
```

---

## 3. Compiler Defense Against Idiom Recognition

During initial wasm32 compilation, LLVM's `LoopIdiomRecognizePass` identified the software Radix-4 shift-add loop in `mul_i32_radix4` and attempted to synthesize hardware `i64.mul` instructions.

To strictly enforce D11 non-negotiable constraints, Lab 3 hardened the operand chunk evaluation using `core::hint::black_box` in `crates/uor-r4-integer/src/math.rs`:
```rust
let chunk = (core::hint::black_box(v) & 3) as u32;
```
This preserves 100% safe Rust under `#![forbid(unsafe_code)]` while preventing the LLVM compiler from emitting hardware multiplier instructions. Disassembly verification confirmed that `uor_wasm_hopf_project_x`, `_y`, and `_z` execute with strictly **0 hardware multipliers**.

---

## 4. Pages Studio Integration Readiness (#1173)

The generated `uor_r4_integer.wasm` (1.61 MiB) can be linked directly by `Casey-allard/uor-r4-wasm-chat` (Pages Studio). The capability API provides an exact contract mapping for JavaScript/TypeScript front-ends:
- Ingestion, generation, and state serialization require zero host WebGL or WebGPU acceleration.
- Entire numerical pipeline executes via CPU integer WebAssembly instructions, conforming to D11.
