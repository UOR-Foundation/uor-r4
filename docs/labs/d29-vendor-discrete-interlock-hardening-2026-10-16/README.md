# D29 Vendor, Training, Discrete & Instrument Static Interlock Hardening — 2026-10-16

- **Lab / session:** `antigravity` (`ag1`)
- **Milestone / line:** M1 qualification & enforcement tooling (`#2028`, `#2029`)
- **Commission / audit references:** `#2028` comments `6221953405`, `6222117146`, `6222315060` (Audit Ranks 2, 3, and 5 host-accessible static checks)
- **Execution scope:** Source-only / host-only (`model_compute_executed: false`, `apply_calls: 0`, zero pods, `$0` GPU cost). Zero `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` Cargo targets modified; zero files hashed by `crates/uor-r4-discrete`'s `source_sha256()` or `docs/labs/k16-timing-2026-10-15/instrument`'s `standalone_source_hash()` modified.
- **Line / count / headline:** M1 qualification & enforcement tooling · count `0/3` unchanged · M1 headline `37/232` unchanged (suspended D28/R14); training, M2, and Stage B remain **HELD**; Q4 remains **NOT_EXECUTABLE**; H1 magnitude remains **NOT_APPROVED**.

## Summary of Changes

1. **Expanded `forbidden()` & Predicated/Labeled PTX Scanner (`forbidden_ptx`) in [`scripts/check_native_cuda_vendor.rs`](../../../scripts/check_native_cuda_vendor.rs):**
   - Expanded `forbidden()` with:
     - Identifier/symbol substring families: `uelu_`, `wgmma`, `tcgen05`, `ulog_`, `__nv_log`, `__nv_tanh`, `__nv_erf` (alongside existing `softmax`, `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary`, `cublas`, `cudnn`, `nvrtc`, `matmul`, `gemm`, `wmma`, `mma_sync`, `uexp_`, `__nv_exp`).
     - Exact identifier tokens: `erf`, `erff`, `erfc`, `erfcf`, `erfinv`, `erfinvf`, `tanhf`, `elu`, `sgemm`, `dgemm`, `hgemm`, `gemv` (`elu` is checked as an exact identifier and `uelu_` as a prefix family so `relu` / `urelu_*` in `third_party/candle-kernels-0.9.2/src/unary.cu` do not false-positive).
     - Instruction substrings: `wgmma.`, `tcgen05.`, `lg2.`, `sin.approx`, `cos.approx` (alongside `mma.`, `wmma.`, `ex2.`, `tanh.`).
   - Added `strip_ptx_predicates_and_labels` and `forbidden_ptx(text: &str)`, which strip leading PTX predicate guards (`@%p0`, `@!%p1`, etc.) and single-colon instruction labels (`$L__BB0_1:`) on each non-comment PTX line while preserving double-colon PTX instruction qualifiers (`tcgen05.mma.cta_group::1.kind::f16`), and reject `ex2.`, `lg2.`, `sin.`, `cos.`, `tanh.`, `rcp.approx`, `sqrt.approx`, `rsqrt.approx`, `mma.`, `wmma.`, `wgmma.`, and `tcgen05.`. Wired into `--ptx-dir` verification and `--self-test`.

2. **Host-Side Training Compiled `.cu` Census (`TRAINING_COMPILED_CU` + `forbidden_training_cuda`):**
   - Added host-side verification of `crates/uor-r4-training/src/{cuda_stack.cu, spiralcore_fbs.cu, starling_native.cu}` in `verify()` without requiring `--features cuda`:
     - Scans all three `.cu` sources with `forbidden_training_cuda()` (`forbidden()` plus whole-word `log`, `logf`, `log2`, `log2f`, `log10`, `log10f`, `log1p`, `log1pf`, `__logf`, `nvcuda`).
     - Verifies that none of the `>=30` excluded historical kernels (`docs/history/d29-excluded-kernels-2026-10-13/cuda_stack_historical.cu`) and none of the quarantined dense-contraction kernels (`native_context_quarantined.cu`, `selected_potential_quarantined.cu`) exist in or are defined by `TRAINING_COMPILED_CU`.

3. **Host-Side Discrete D32 CUDA Kernel Census (`DISCRETE_ENTRIES` + `forbidden_discrete_cuda`):**
   - Added `forbidden_discrete_cuda()` and host-side verification of `crates/uor-r4-discrete/src/discrete.cu` in `verify()`:
     - Rejects all `forbidden()` tokens plus D32 prohibited floating-point and tensor-core tokens (`float`, `double`, `__half`, `__nv_bfloat16`, `f16`, `f16x2`, `f32`, `f64`, `bf16`, `bf16x2`, `tf32`, `e4m3`, `e5m2`, `mma`, `wmma`, `wgmma`, `tcgen05`, `tensor`, `tensormap`, `log`, `logf`, `log2`, `log2f`, `log10`, `log10f`, `log1p`, `log1pf`, `rcp.approx`, `sqrt.approx`, `rsqrt.approx`).
     - Verifies that the `extern "C" __global__ void` kernel definitions in `discrete.cu` match the exact 11-entry `DISCRETE_ENTRIES` inventory.

4. **Host-Side Runtime Compiler & Manifest `cudarc` Coverage Across `uor-r4-discrete` and Standalone Instruments:**
   - Added a recursive `.rs` source scan across `crates/uor-r4-training/src`, `crates/uor-r4-discrete/src`, `docs/labs/k16-timing-2026-10-15/instrument/src`, and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/src` rejecting `nvrtc::`, `compile_ptx`, and `cuLinkCreate` in non-comment code.
   - Extended the direct-`cudarc` manifest loop in `verify()` to include `crates/uor-r4-discrete/Cargo.toml` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml` (`default-features = false`, no `BANNED_FEATURES` in `cudarc` features or `[features].cuda`), and added a standalone-instrument manifest check covering `docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml` (`[patch.crates-io]` patches `cudarc` to `third_party/cudarc-0.19.10` and `[features].cuda` enables no `BANNED_FEATURES`).

## Verification

- `rustc --edition=2021 scripts/check_native_cuda_vendor.rs -o /tmp/check-native-cuda-vendor && /tmp/check-native-cuda-vendor --self-test && /tmp/check-native-cuda-vendor --repo .` — PASS (all 6 static verification stages + planted self-test vectors pass).
- `uor-harness ratchet d23` — PASS (`2,253` sites across `154` files, baseline `2,253`).
- `uor-harness ratchet d25` — PASS (`93` sites across `26` files, baseline `93`; `0` gated tensor-core sites).
- `cargo run -p uor-r4-core --bin r4-policy-check` — PASS (`uor-r4.agent-execution-policy/6`, `native_geometric_ai`).
- `uor-harness check` — PASS.
- `python3 scripts/check_claim_wording.py` — PASS.
