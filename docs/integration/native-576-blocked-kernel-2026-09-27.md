# Four-Row Blocked Product Table Reuse in Native Integer Serving

References #973 and #820. This delivery implements four-row blocked product table reuse for wide integer matrices (width 576 and 1152) in `uor-r4-integer`. It does not promote an unverified model or claim general language capability, geometric superiority, or full-path energy savings.

## 1. Problem and Architecture

In the zero-matmul integer serving pipeline, affine transformations and vocabulary projection execute without floating-point or hardware multiplier instructions, using signed-4 shift-and-add arithmetic over packed dyadic weights.

For each token step, an input vector $\mathbf{x} \in \mathbb{Z}^{W}$ is converted into precomputed signed-4 multiple tables:
$$\text{products}[k][n] = x_k \cdot n \quad \text{for } n \in \{-7, \dots, 7\}$$
In scalar row-by-row iteration, each row independently loads and indexes these product tables. For width-576 and width-1152 layers—particularly the 4096-row vocabulary projection—repeatedly streaming product tables across all rows incurs substantial cache pressure and memory bandwidth overhead.

### 4-Row Blocked Accumulation

The blocked kernel `low_bit_dot_4_contiguous_wide<const WIDTH: usize>` processes four adjacent matrix rows simultaneously:

```rust
#[inline(always)]
fn low_bit_dot_4_contiguous_wide<const WIDTH: usize>(
    products: &[[i64; 16]; WIDTH],
    weights: &[u8],
) -> [i64; 4] {
    let mut a = [0i64; 4];
    let mut b = [0i64; 4];
    for i in (0..(WIDTH >> 1)).step_by(2) {
        let p0 = &products[i << 1];
        let p1 = &products[(i << 1) + 1];
        let p2 = &products[(i << 1) + 2];
        let p3 = &products[(i << 1) + 3];
        let mut offset = i;
        for row in 0..4 {
            let w0 = weights[offset];
            let w1 = weights[offset + 1];
            a[row] += p0[usize::from(w0 & 15)] + p1[usize::from(w0 >> 4)];
            b[row] += p2[usize::from(w1 & 15)] + p3[usize::from(w1 >> 4)];
            offset += WIDTH >> 1;
        }
    }
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
```

### Mathematical Invariants & Overflow Bounds

1. **Intermediate Accumulator Headroom**:
   With signed-4 codes in $[-7, 7]$ and input coordinates $x_k \in [-2^{31}, 2^{31}-1]$, the maximum possible sum across 1152 coordinates is bounded by:
   $$1152 \times 7 \times 2^{31} \approx 1.73 \times 10^{13} < 2^{44}$$
   Because $2^{44} \ll 2^{63}-1$ (`i64::MAX` $\approx 9.22 \times 10^{18}$), 4-row parallel accumulation in `i64` cannot overflow and incurs zero rounding error or precision loss.
2. **Dyadic Scaling and Quantization**:
   Checked dyadic rescaling `scaled(i128::from(dot), shift)` is executed on the final `i64` sums, preserving exact mathematical equivalence with unblocked execution.
3. **Tail and Non-Block Alignment**:
   For row counts not divisible by 4 (and non-block-aligned vocabulary sizes such as 4095), the kernel processes full 4-row blocks first, then evaluates the residual scalar tail using `low_bit_dot`. Truncated input buffers and mismatched slice lengths fail closed with `Result::Err`.

## 2. Hardware Invariant Audit

Static disassembly audit via `scripts/audit_zero_matmul_serving.py` (strict ARM64 disassembly mode):

| Artifact Checked | Matched Symbol Ranges | Class I (Multipliers) | Class II (Dividers) | Class III (Floats) | Missing Mandatory | Audit Status |
|---|---:|---:|---:|---:|---:|:---:|
| `libuor_r4_integer.rlib` | 31 | 0 | 0 | 0 | 0 | **PASS (5/5)** |
| `target/release/uor-chat` | 30 | 0 | 0 | 0 | 0 | **PASS (5/5)** |

Emitted functions audited include:
- `IntegerModel::project_vocab_with_products_into` (2,096 instructions, 0 forbidden)
- `IntegerModel::matrix_work_wide_into::<1152>` (399 instructions, 0 forbidden)
- `IntegerModel::matrix_work_direct_into` (912 instructions, 0 forbidden)
- `IntegerModel::step_conversational_into` (12,756 instructions, 0 forbidden)
- `low_bit_dot` (72 instructions, 0 forbidden)

## 3. Empirical Measurements on Apple Silicon M1

Measurements taken in release profile on local Apple Silicon M1 (8 cores, 16 GiB RAM):

| Metric | Measured Value | Ceiling / Target | Status |
|---|---:|---:|:---:|
| `project_vocab` latency (100 iterations) | 1.103 ms / call | $\le 2.0$ ms | **PASS** |
| `step_conversational` latency (100 iterations) | 1.693 ms / step | $\le 2.5$ ms | **PASS** |
| Quaternion streaming generation (128 tokens) | 1.920 ms / token | $\le 4.0$ ms / token | **PASS** |
| Householder pair streaming generation (128 tokens) | 3.424 ms / token | $\le 4.0$ ms / token | **PASS** |
| Peak RSS during streaming generation | 18.20 MB | $< 35.0$ MB | **PASS** |
| Steady-state RSS drift across 128 tokens | 0.00 MB | 0.00 MB | **PASS** |

## 4. Verification Record

- **Test Suite**: All 157 unit and integration tests in `crates/uor-r4-integer` pass with 0 failures, including adversarial stress tests, long-horizon recall, hierarchical memory, and hardware invariant checks.
- **Dedicated Dispatch Tests**: `packed_signed4_matrix_dispatch_rows_scales_and_embedding` thoroughly tests widths 128, 256, 512, 576, 1152 across row counts 1, 2, 3, 4, 5, 6, 7, 9 (exercising sub-block, block-aligned, and multi-row tails), non-block-aligned vocabulary size 4095, truncated parameters, and error paths.
- **Code Style & Claims**: `cargo fmt --check` clean; `python3 scripts/check_claim_wording.py` clean.
