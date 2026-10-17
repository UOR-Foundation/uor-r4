# Host-Only D32 Discrete-Float Ratchet, Hopper/Blackwell Tensor-Core Intrinsics & Proof-Model Audit Expansion — 16 October 2026

**Status:** Static host-only interlock hardening (`NOT_RUN` for any GPU/model compute; `$0` GPU spend). Training, M2, and Stage B remain `HELD`; Q4 remains `NOT_EXECUTABLE`; H1 magnitude remains `NOT_APPROVED`.

## Objective & Causal Scope

Follow-on to `#2547` (`6725e0ea9`), `#2551` (`ec65ce5c5`), `#2558` (`e2fb90379`), and the `#2028` [interlock coverage audit (`6221953405`)](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6221953405). Closes the remaining static host-side interlock gaps in `uor-r4-harness` (`ratchet` / `prlint`) and `uor-r4-proof-model` (`inference_audit`) without touching any `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` target or any file hashed by `crates/uor-r4-discrete`'s `source_sha256()` or `k16-timing`'s `standalone_source_hash()`:

1. **D32 Discrete-Float Ratchet (`uor-harness ratchet d32` / `Kind::DiscreteFloat`):**
   - Adds `Kind::DiscreteFloat` (`d32` / `discrete-float`) in `crates/uor-r4-harness/src/ratchet.rs` (`discrete_float_sites()`), wired into `crates/uor-r4-harness/src/main.rs` (`cmd_ratchet` with `--list` and `--update-baseline`) and `crates/uor-r4-harness/src/prlintrules.rs` (`interlock_holds`).
   - Scans comment/string-stripped code across `crates/uor-r4-discrete/` and `docs/labs/*/instrument/` for:
     - `D32_FLOAT_TYPES`: `f16`, `bf16`, `f32`, `f64`, `tf32`, `float`, `double`, `__half`, `__nv_bfloat16` (identifiers).
     - Floating-point literals: decimal (`1.0`, `2.`, `.5f`), exponent (`1e-5`, `2E+3`), hex-float (`0x1.8p-2`), and Rust typed float literals (`3f32`, `4_f64`), while excluding Rust tuple indexing (`pair.0`, `pair.0.1`, `(pair).0`, `arr[0].1`), ranges (`0..10`), and integer method calls (`i.max(1)`).
   - Locks `scripts/no-discrete-float-baseline.json` at `{"crates/uor-r4-discrete/src/prime_zeta.rs": 31}` (`19` `float-type` + `12` `float-literal` sites in the documented one-time offline integer-table constructor) and enforces **zero** floating-point sites across all other 41 `.rs` and `.cu` files in `crates/uor-r4-discrete/` and `docs/labs/*/instrument/`.

2. **Hopper (`wgmma`) & Blackwell (`tcgen05`) Tensor-Core Intrinsics in `uor-harness ratchet d25` (`Kind::Matmul`):**
   - Extends `tensor_core_count()` in `crates/uor-r4-harness/src/ratchet.rs` with `TC_WGMMA` (`wgmma`) and `TC_TCGEN05` (`tcgen05`) alongside `wmma` and `mma.sync`, matching the PTX scanner in `scripts/check_native_cuda_vendor.rs`.

3. **D11/D23/D25/D32 Proof-Model Disassembly & Dependency Denylists (`crates/uor-r4-proof-model/src/inference_audit.rs`):**
   - Expands `InferenceAuditVerifier::FORBIDDEN_MNEMONICS` with packed AVX float ops (`vaddps`, `vsubps`, `vmulps`, `vdivps`), fused multiply-add (`fma.`, `fmadd`, `fmsub`, `fnmadd`, `vfmadd`, `vfmsub`, `vfnmadd`), float conversions (`cvt.`, `cvtsi2ss`, `cvtsi2sd`, `cvtss2si`, `cvtsd2si`, `vcvt`, `scvtf`, `ucvtf`, `fcvtzs`, `fcvtzu`), PTX transcendentals and reciprocal/rsqrt approximations (`ex2.`, `lg2.`, `sin.`, `cos.`, `tanh.`, `rcp.`, `rsqrt.`, `sqrt.approx`, ` sqrt.rn`), and PTX/SASS tensor-core instructions (`mma.`, `wmma.`, `wgmma`, `tcgen05`, `hmma`, `imma`, `qmma`).
   - Expands `InferenceAuditVerifier::FORBIDDEN_DEPENDENCIES` with CUDA/BLAS/ML/tensor libraries (`cudnn`, `cutlass`, `cutensor`, `cusolver`, `cusparse`, `nvrtc`, `nccl`, `candle`, `burn`, `tch`, `ort`, `onnx`, `nalgebra`, `ndarray`, `faer`, `gemm`, `matrixmultiply`).

## Verification

- `cargo test -p uor-r4-harness --lib prlintrules`, `cargo test -p uor-r4-harness --lib ratchet`, and `cargo test -p uor-r4-harness --lib prgates::check_pr_tests` — all `8`, `9`, and `2` unit tests pass (including `discrete_float_ratchet_catches_types_and_literals_without_tuple_or_range_false_positives`, `tensor_core_count_is_non_overlapping_and_gated`, `d25_matmul_growth_produces_an_error`, `d32_discrete_float_growth_produces_an_error`, `the_real_interlock_is_callable_and_clean`, and `every_ported_gate_and_the_dispatcher_agree_with_python`).
- `cargo test -p uor-r4-proof-model --no-default-features` — all `5` unit tests pass (including `test_disassembly_audit_rejects_fma_conversions_ptx_and_tensor_core` and `test_dependency_audit_rejects_gpu_and_tensor_deps`), with `pub mod inference_audit;` ungated from `full-model` in `crates/uor-r4-proof-model/src/lib.rs`.
- `cargo run -p uor-r4-harness -- ratchet d23` — `OK: no softmax added. 2253 remaining across 154 files (baseline 2253).`
- `cargo run -p uor-r4-harness -- ratchet d25` — `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93).` (`gated: 0 tensor-core intrinsic sites across 0 files`).
- `cargo run -p uor-r4-harness -- ratchet d32 --list` — `OK: no discrete-path floating-point sites added. 31 remaining across 1 files (baseline 31).`
- `rustc --edition 2021 scripts/check_native_cuda_vendor.rs -o /tmp/check_native_cuda_vendor_ag1 && /tmp/check_native_cuda_vendor_ag1 --self-test && /tmp/check_native_cuda_vendor_ag1 --repo .` — `check_native_cuda_vendor self-test: ok` and `check_native_cuda_vendor: ok`.
- `python3 scripts/check_claim_wording.py` — `claim-wording check passed`.
