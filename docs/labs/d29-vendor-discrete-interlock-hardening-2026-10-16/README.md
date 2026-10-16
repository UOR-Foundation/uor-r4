# D29 Vendor, Training, Discrete & Instrument Static Interlock Hardening — 2026-10-16

- **Lab / session:** `antigravity` (`ag1`)
- **Milestone / line:** M1 qualification & enforcement tooling (`#2028`, `#2029`)
- **Commission / audit references:** `#2028` comments `6221953405`, `6222117146`, `6222315060` (Audit Ranks 2, 3, and 5 host-accessible static checks)
- **Execution scope:** Source-only / host-only (`model_compute_executed: false`, `apply_calls: 0`, zero pods, `$0` GPU cost). Zero `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` Cargo targets modified; zero files hashed by `crates/uor-r4-discrete`'s `source_sha256()` or `docs/labs/k16-timing-2026-10-15/instrument`'s `standalone_source_hash()` modified.
- **Line / count / headline:** M1 qualification & enforcement tooling · count `0/3` unchanged · M1 headline `37/232` unchanged (suspended D28/R14); training, M2, and Stage B remain **HELD**; Q4 remains **NOT_EXECUTABLE**; H1 magnitude remains **NOT_APPROVED**.

## Summary of Changes

1. **Comment Lexer Hardening (`code`), Expanded `forbidden()`, & Predicated/Labeled PTX Scanner (`forbidden_ptx`) in [`scripts/check_native_cuda_vendor.rs`](../../../scripts/check_native_cuda_vendor.rs):**
   - Hardened `code(text: &str)` to step over C/Rust character literals (`'c'`, `'\"'`, `'\\'`, etc., without treating Rust lifetimes `'a` as multi-character strings) and Rust raw string literals (`r"..."`, `r#"..."#`, `br"..."`, `br#"..."#`) so quotes inside character or raw-string literals cannot desynchronize block-comment stripping.
   - Expanded `forbidden()` with:
     - Identifier/symbol substring families: `uelu_`, `wgmma`, `tcgen05`, `ulog_`, `__nv_log`, `__nv_tanh`, `__nv_erf` (alongside existing `softmax`, `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary`, `cublas`, `cudnn`, `nvrtc`, `matmul`, `gemm`, `wmma`, `mma_sync`, `uexp_`, `__nv_exp`).
     - Exact identifier tokens: `erf`, `erff`, `erfc`, `erfcf`, `erfinv`, `erfinvf`, `tanhf`, `elu`, `sgemm`, `dgemm`, `hgemm`, `gemv` (`elu` is checked as an exact identifier and `uelu_` as a prefix family so `relu` / `urelu_*` in `third_party/candle-kernels-0.9.2/src/unary.cu` do not false-positive).
     - Instruction substrings: `wgmma.`, `tcgen05.`, `lg2.` (alongside `mma.`, `wmma.`, `ex2.`, `tanh.`).
   - Added `strip_ptx_predicates_and_labels` and `forbidden_ptx(text: &str)`, which split PTX text on `['\n', ';', '{', '}']` and strip leading PTX predicate guards (`@%p0`, `@!%p1`, `@ %p0`) and single-colon instruction labels (`$L__BB0_1:`) while preserving double-colon PTX instruction qualifiers (`tcgen05.mma.cta_group::1.kind::f16`), rejecting `ex2.`, `lg2.`, `tanh.`, `mma.`, `wmma.`, `wgmma.`, and `tcgen05.` in vendor PTX (preserving compatibility with allowlisted `usin_*`, `ucos_*`, `usqrt_*`, `urecip_*`, and `bdiv_*` exports in `unary.ptx` / `binary.ptx`). Wired into `--ptx-dir` verification and `--self-test`.

2. **Host-Side Training Compiled `.cu` Census (`TRAINING_COMPILED_CU` + `forbidden_training_cuda`):**
   - Added host-side verification of `crates/uor-r4-training/src/{cuda_stack.cu, spiralcore_fbs.cu, starling_native.cu}` and `crates/uor-r4-training/build.rs` in `verify()` without requiring `--features cuda`:
     - Verifies the `.cu`/`.cuh` directory set in `crates/uor-r4-training/src` is exactly `TRAINING_COMPILED_CU`, checks `#include` directives against excluded/quarantined paths, and scans all three `.cu` sources with `forbidden_training_cuda()` (`forbidden()` plus whole-word `log`, `logf`, `log2`, `log2f`, `log10`, `log10f`, `log1p`, `log1pf`, `__logf`, `nvcuda`).
     - Verifies that `crates/uor-r4-training/build.rs` does not reference quarantined modules or dynamic directory globs, and that none of the `>=30` excluded historical kernels (`docs/history/d29-excluded-kernels-2026-10-13/cuda_stack_historical.cu`) and none of the non-empty quarantined dense-contraction kernel sets (`native_context_quarantined.cu`, `selected_potential_quarantined.cu`) exist in or are defined by `TRAINING_COMPILED_CU`.

3. **Host-Side Discrete D32 CUDA Kernel Census (`DISCRETE_ENTRIES` + `decomposition_source::generate` + `forbidden_discrete_cuda`):**
   - Added `forbidden_discrete_cuda()` and host-side verification of both `crates/uor-r4-discrete/src/discrete.cu` and the generated diagnostic module `decomposition_source::generate(&discrete_cu)` in `verify()`:
     - Verifies `crates/uor-r4-discrete/src` contains only `discrete.cu` among `.cu`/`.cuh` files with no excluded `#include` directives.
     - Rejects all `forbidden()` tokens plus D32 prohibited floating-point, transcendental, and tensor-core tokens (`float`, `double`, `__half`, `__nv_bfloat16`, `f16`, `f16x2`, `f32`, `f64`, `bf16`, `bf16x2`, `tf32`, `e4m3`, `e5m2`, `mma`, `wmma`, `wgmma`, `tcgen05`, `tensor`, `tensormap`, `log`, `logf`, `log2`, `log2f`, `log10`, `log10f`, `log1p`, `log1pf`, `sin.approx`, `cos.approx`, `rcp.approx`, `sqrt.approx`, `rsqrt.approx`, and statement-level predicated/labeled `ex2.`, `lg2.`, `sin.`, `cos.`, `tanh.`, `rcp.approx`, `sqrt.approx`, `rsqrt.approx`, `mma.`, `wmma.`, `wgmma.`, `tcgen05.`).
     - Verifies that `discrete.cu` defines the exact 11-entry `DISCRETE_ENTRIES` inventory and `decomposition_source::generate(&discrete_cu)` defines the single `discrete_counterfactual_decomposition` entry.

4. **Host-Side Runtime Compiler & Manifest `cudarc` Coverage Across `uor-r4-discrete` and Standalone Instruments:**
   - Added a recursive `.rs` source and `build.rs` scan across `crates/uor-r4-training/{build.rs,src}`, `crates/uor-r4-discrete/{build.rs,src}`, `docs/labs/k16-timing-2026-10-15/instrument/src`, `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/{build.rs,src}`, `docs/labs/stage-a-admission-instrument-2026-10-15/instrument/src`, `docs/labs/fresh-scoring-eval-2026-10-15/instrument/src`, and `docs/labs/admission-redirect-packet-2026-10-15/instrument/src` rejecting `nvrtc::`, `compile_ptx`, and `cuLinkCreate` in non-comment code.
   - Extended the direct-`cudarc` manifest loop in `verify()` to include `crates/uor-r4-discrete/Cargo.toml` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml` (`default-features = false`, no `BANNED_FEATURES` in `cudarc` features or `[features].cuda`), and added a standalone-instrument manifest check covering `docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml` (`[patch.crates-io]` patches `cudarc` to `third_party/cudarc-0.19.10` and `[features].cuda` enables no `BANNED_FEATURES`).

## Verification

- `rustc --edition=2021 scripts/check_native_cuda_vendor.rs -o /tmp/check-native-cuda-vendor && /tmp/check-native-cuda-vendor --self-test && /tmp/check-native-cuda-vendor --repo .` — PASS (all 6 static verification stages + planted self-test vectors pass).
- `uor-harness ratchet d23` — PASS (`2,253` sites across `154` files, baseline `2,253`).
- `uor-harness ratchet d25` — PASS (`93` sites across `26` files, baseline `93`; `0` gated tensor-core sites).
- `cargo run -p uor-r4-core --bin r4-policy-check` — PASS (`uor-r4.agent-execution-policy/6`, `native_geometric_ai`).
- `uor-harness check` — PASS.
- `python3 scripts/check_claim_wording.py` — PASS.
