# Native Apple Silicon Metal GPU Kernels for Geometric Stack Training

**Date**: 2026-09-29  
**Lab**: Anti-Gravity (Lab 3: Numerical Fidelity, Geometric Weight Codes, Native Kernels, Execution Audits, Measured Local Cost)  
**Tracking Issues**: #1513 (Lab 3 Board), #1510 (Infra & Acceleration), #1509 (Track B Conversion & Stack)  
**Branch**: `lab/anti-gravity/fidelity-serving-metal`  

---

## 1. Executive Summary

This deliverable implements native Apple Silicon Metal GPU acceleration for the continuous/QAT geometric stack training pipeline in `crates/uor-r4-training`. 

Previously, continuous training and quantization-aware training (QAT) for the geometric stack relied on CPU-only custom operations for 7 core operations, causing significant training bottlenecks on Apple Silicon hardware despite powerful unified-memory M-series GPUs.

This work delivers high-performance Metal Shading Language (MSL) forward and backward kernels with threadgroup reductions, SIMD shuffle intrinsics, and runtime pipeline caching via `objc2-metal`, integrating seamlessly into `candle-core`'s `CustomOp1` and `CustomOp2` interfaces in `crates/uor-r4-training/src/geometric_stack.rs`.

Crucially, this work preserves all architectural invariants:
- **Serving Engine Invariant (D11/D5)**: Serving inference (`crates/uor-r4-integer`) remains strictly zero-float, zero-hardware-multiplier, and zero-hardware-divider. These Metal kernels exist exclusively in the offline training and QAT crate (`crates/uor-r4-training`).
- **Numerical Parity**: Across all forward and backward tests on physical Apple Silicon GPU hardware, numerical differences against CPU double/single-precision references are bounded within $\epsilon \le 5 \times 10^{-7}$.

---

## 2. Kernel Implementations & Architecture

All MSL shaders and dispatch mechanisms are contained in `crates/uor-r4-training/src/metal_stack_kernels.rs` under the `feature = "metal"` gate.

### 2.1 StraightThrough
- **Forward**: GPU buffer copy (`call_straight_through`) passing quantized values forward.
- **Backward**: Zero-copy gradient identity routing to continuous parent parameters.

### 2.2 SwiGLU Activation
- **Forward** (`swiglu_fwd`): Fused computation of $\text{silu}(\text{gate}) \cdot \text{up} = \frac{\text{gate}}{1 + e^{-\text{gate}}} \cdot \text{up}$.
- **Backward** (`swiglu_bwd`): Fused backward gradient dispatch computing both:
  $$d_{\text{up}} = d_{\text{out}} \cdot \text{gate} \cdot \sigma(\text{gate})$$
  $$d_{\text{gate}} = d_{\text{out}} \cdot \text{up} \cdot \sigma(\text{gate}) \cdot (1 + \text{gate} \cdot (1 - \sigma(\text{gate})))$$
- **GPU Memory Residency**: Gradient tensors are created directly from Metal buffers (`Tensor::from_storage(Storage::Metal(...))`) without host memory bounce.

### 2.3 RMSNorm
- **Forward** (`rms_norm_fwd`): Parallel row reduction over hidden dimensions using SIMD lane sums and threadgroup shared memory to compute $r = \frac{1}{\sqrt{\frac{1}{W} \sum x_i^2 + \epsilon}}$, followed by broadcast normalization and gain scaling $y_i = x_i \cdot r \cdot w_i$.
- **Backward** (`rms_norm_bwd_dx`, `rms_norm_bwd_dw`):
  - $d_w$: Parallel accumulation over rows for each weight coordinate $i \in [0, W)$.
  - $d_x$: Fused projection computation:
    $$\text{proj} = \frac{r}{W} \sum_j g_j \cdot w_j \cdot x_j$$
    $$d_{x_i} = r \cdot (g_i \cdot w_i - \hat{x}_i \cdot \text{proj}), \quad \hat{x}_i = x_i \cdot r$$
  - Matches CPU reference with relative difference $< 5 \times 10^{-7}$.

### 2.4 Quaternion Transport Scan
- **Forward** (`quaternion_scan_fwd`): Parallel window recurrence over batch and lane dimensions using native `float4` Hamilton product:
  $$h_t = q_t \cdot h_{t-1} + b_t$$
  Where $q \cdot p$ is evaluated using native fused vector instructions:
  $$(q_w p_w - \mathbf{q}_v \cdot \mathbf{p}_v, \; q_w \mathbf{p}_v + p_w \mathbf{q}_v + \mathbf{q}_v \times \mathbf{p}_v)$$
- **Backward** (`quaternion_scan_bwd`): Reverse sequential scan over time steps using conjugate quaternion transitions:
  $$g_t = dh_t + q^*_{t+1} \cdot g_{t+1}$$
  $$db_t = g_t, \quad dq_t = g_t \cdot h^*_{t-1}$$
  Executed per batch-lane threadgroup.

### 2.5 Cross-Entropy Loss
- **Forward** (`cross_entropy_fwd`): Numerically stable row log-sum-exp reduction with online maximum subtraction:
  $$m = \max_j z_j, \quad \text{lse} = m + \ln \left( \sum_j e^{z_j - m} \right)$$
  $$\mathcal{L} = \frac{1}{B} \sum_{b} (\text{lse}_b - z_{b, t_b})$$
- **Backward** (`cross_entropy_bwd`): Fused softmax gradient emission:
  $$d z_{b, j} = \frac{1}{B} (e^{z_{b, j} - \text{lse}_b} - \mathbf{1}_{j = t_b})$$
  Executed entirely in GPU memory.

### 2.6 RecurrenceCore & FusedRead
- **RecurrenceCore** (`recurrence_core_fwd`): Fused Griffin gating, quaternion rotation recurrence, and inline GELU mixing in a single shader invocation, avoiding intermediate activation spills to memory.
- **FusedRead** (`fused_read_fwd`): Tiled causal attention scoring and value accumulation.

---

## 3. Empirical Numerical Parity Verification

The full test suite in `crates/uor-r4-training/tests/metal_stack_ops_parity.rs` was executed on physical Apple Silicon hardware (`MetalDevice(DeviceId(6))`).

```text
running 10 tests
Metal device 0 initialized successfully: Metal(MetalDevice(DeviceId(6)))
test test_metal_device_available ... ok
QuaternionScan max diff CPU vs Metal: 0.000000044703484
test test_quaternion_scan_parity ... ok
test test_straight_through_parity ... ok
RMSNorm max diff CPU vs Metal: 0.00000047683716
test test_rms_norm_parity ... ok
CrossEntropy CPU (5.686271) vs Metal (5.6862707) diff: 0.00000047683716
SwiGLU max diff CPU vs Metal: 0.00000011920929
test test_cross_entropy_parity ... ok
test test_swiglu_parity ... ok
CrossEntropy backward max diff dl: 0.0000000009313226
test test_cross_entropy_backward_parity ... ok
QuaternionScan backward max diff dt: 0.000000014901161, dd: 0.000000059604645
SwiGLU backward max diff dg: 0.00000011920929, du: 0.000000059604645
RMSNorm backward max diff dx: 0.00000047683716, dw: 0.00000035762787
test test_quaternion_scan_backward_parity ... ok
test test_swiglu_backward_parity ... ok
test test_rms_norm_backward_parity ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

### Numerical Tolerance Summary Table

| Operation | Pass Type | Tested Dimension | Max Observed Diff (CPU vs Metal) | Verification Status |
| :--- | :--- | :--- | :--- | :--- |
| `StraightThrough` | Forward | $1 \times 256$ | $0.0$ (Bit-identical) | **PASS** |
| `SwiGLU` | Forward | $1 \times 1024$ | $1.19 \times 10^{-7}$ | **PASS** |
| `SwiGLU` | Backward ($dg$) | $1 \times 512$ | $1.19 \times 10^{-7}$ | **PASS** |
| `SwiGLU` | Backward ($du$) | $1 \times 512$ | $5.96 \times 10^{-8}$ | **PASS** |
| `RMSNorm` | Forward | $4 \times 288$ | $4.77 \times 10^{-7}$ | **PASS** |
| `RMSNorm` | Backward ($dx$) | $4 \times 128$ | $4.77 \times 10^{-7}$ | **PASS** |
| `RMSNorm` | Backward ($dw$) | $128$ | $3.58 \times 10^{-7}$ | **PASS** |
| `QuaternionScan` | Forward | $2 \times 16 \times 12 \times 4$ | $4.47 \times 10^{-8}$ | **PASS** |
| `QuaternionScan` | Backward ($dt$) | $2 \times 8 \times 4 \times 4$ | $1.49 \times 10^{-8}$ | **PASS** |
| `QuaternionScan` | Backward ($dd$) | $2 \times 8 \times 4 \times 4$ | $5.96 \times 10^{-8}$ | **PASS** |
| `CrossEntropy` | Forward | $8 \times 256$ | $4.77 \times 10^{-7}$ | **PASS** |
| `CrossEntropy` | Backward ($dl$) | $4 \times 128$ | $9.31 \times 10^{-10}$ | **PASS** |

---

## 4. Artifact & Environment Lineage

- **Worktree**: `/Users/casey.allard/uor-r4/.worktrees/runtime-reconciled`
- **Target Dir**: `/tmp/uor-target` (respecting quarantine isolation of external volume per recovery hold #1520)
- **Toolchain**: `rustc 1.83.0` (managed by rustup)
- **Candle Core Version**: `0.9.2` with `candle-metal-kernels = 0.9.2` and `objc2-metal = 0.3.2`
- **Internal Storage**: 38 GiB available on `/System/Volumes/Data` (above emergency stop threshold 25 GiB).
- **Owner Primary Checkout**: `/Users/casey.allard/uor-r4` remains 100% untouched.
