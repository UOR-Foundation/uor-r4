# Stack Mission Port Design (Track T3): Zero-MatMul R2 Kernel Mapping

**Author**: Lab 3 (Anti-Gravity)  
**Date**: 2026-09-27  
**Scope**: Track T3 Mission Runtime (`ROADMAP.md` §4.3, References #973, #820, #963, #964).  
**Invariant Contract**: D0-b / Rules R1–R5 (Strictly 0 Class I multipliers, 0 Class II dividers, 0 Class III floats in served numerical kernels). Read-only design pending T1(a) exposure gate report (~07:00 UTC 09-28).

---

## 1. Executive Summary & Design Scope

The geometric stack (`crates/uor-r4-training/src/geometric_stack.rs`, Cycle 4) demonstrates competitive scaling using alternating quaternion transport recurrence (`r`) and multi-head hyperbolic/dot reads (`a`). However, the existing serving comparator (`crates/uor-r4-lut/src/stack.rs` under D10) violates Mission Rule R1/R2 by executing runtime hardware multiplications on activations, gate products, Hamilton products, and attention value mixing.

Under Track T3, the stack serving engine will be ported **directly as an extension of `uor-r4-integer`**, strictly preserving the zero-multiplier serving contract (D0-b) and reusing the 4-row blocked product tables, register-resident accumulators, and CORDIC/Taylor integer kernels established in Milestone M1/M2 and PR #1436.

---

## 2. Op-by-Op Mapping: `geometric_stack.rs` $\to$ `uor-r4-integer` R2 Kernels

| Operation | `geometric_stack.rs` / `uor-r4-lut` Op | `uor-r4-integer` R2 Kernel Mapping | Tables & Memory Cost |
|---|---|---|---|
| **Linear Projections** | `gemv` 4-bit nibbles with hardware multiply | `IntegerModel::affine_direct_into` / `affine_wide_into` using 4-row blocked product table reuse (PR #1436) | 32 B L1 register product table; 0.5 B/weight parameter read |
| **RMSNorm & Rsqrt** | Float `rms_norm` / `isqrt` with wide division | Exact integer squared norm via `math::sum_squares`, `math::isqrt` digit-by-digit, and `math::scale_pow2` | 0 B table; 288 B read/write |
| **Quaternion Recurrence Scan** | Hamilton product `a (x) b` (16 multiplies) | **Discrete 2I Lane**: 120-state finite automaton lookup (1 byte state, 2 table reads/token).<br>**General Integer**: Cayley-Dickson shift-add / `math::checked_mul` | 240 B group Cayley table; 2.3 KiB persistent recurrence state |
| **Recurrence Decay & Keep** | `lambda = exp(-rate * r)`, `keep = sqrt(1 - lambda^2)` | Dual-channel 256-entry Q31 lookup table `decay_keep_lut[rate_idx]` returning `(lambda_q31, keep_q31)` directly | 2,048 B table (256 $\times$ 8 B); 0 hardware multiplies |
| **Causal Convolution** | `c_t = sum(taps * history)` via `grid_apply` | Fixed dyadic taps applied via arithmetic shifts and adds (`math::scale_pow2`) into ring buffer | 0 B table; 3.4 KiB history ring buffer |
| **SwiGLU MLP Gating** | Elementwise product `silu(g) * u` (hardware multiply) | Grouped 4-bit/8-bit product table lookup `low_bit_dot` or 16-bit shift-add decomposition | 1 KiB SiLU Q16 table + 256 B product table; 768 channels |
| **Read Scores (Dot)** | `<q, k> / sqrt(d)` | Signed 4-bit LUT dot product (`packed_rows::low_bit_dot`) or exact shift-add `low_bit_dot_4_contiguous_wide` | Shared with projection table; $2 \times 48$ B per head |
| **Read Scores (Lorentz)** | $-\beta (d_L(q, k) - \text{offset})$ with `lift` | `LorentzRead::score` with `lorentz::excess_q32`, tabulated `arcosh1p_q24`, and `math::scale_pow2` | 4.1 KiB `arcosh1p` table (1,025 $\times$ 4 B); 0 multipliers |
| **Softmax & Normalization** | Softmax exp and wide division by sum | `lorentz::exp_q32` (Taylor series via `checked_mul`) or 256-entry Q32 negative exp LUT (`exp_neg`); ticket draw / `math::div_round` | 1 KiB exp table; 256 entries |
| **Value Mixing** | $y = \sum w_j v_j$ with 64-bit/128-bit multiplies | Grouped LUT accumulation (T-MAC style) over quantized value matrices; zero per-product multiplications | Zero-table in-place accumulation; $2 \times d$ bytes/pos |
| **Vocab Head & Sampling** | Dense unquantized head & float sampler | `IntegerModel::project_vocab_with_products_into` (0.844 ms / call across 4,096 rows) + `sampling::Sampler` (`exact_min_p_threshold`) | Product table reused from affine; 0 Class I/II/III ops |

---

## 3. Census of Hardware Multiplications in `uor-r4-lut/src/stack.rs` and Replacements

Audit of `crates/uor-r4-lut/src/stack.rs` identified 28 distinct multiplication sites on runtime activations. Every instance is mapped to a zero-multiplier kernel in `uor-r4-integer`:

1. **Lines 72–75 (`hamilton`)**:
   - *Expression*: 16 products (`a0 * b0`, `a0 * b1`, etc.) in quaternion Hamilton product.
   - *Replacement*: Snapped to 2I icosian discrete group: 120-element table lookup (0 multiplies, 2 reads/token). For continuous integer fallback: Cayley-Dickson 8-multiplication shift-add via `math::checked_mul`.
2. **Line 84 (`lift`)**:
   - *Expression*: `i128::from(*x) * i128::from(*x)` in hyperboloid coordinate lifting.
   - *Replacement*: `uor_r4_integer::math::sum_squares` (shift-and-add exact vector squared norm).
3. **Line 93 (`lorentz_distance`)**:
   - *Expression*: `i128::from(query_lift) * i128::from(key_lift)`.
   - *Replacement*: `uor_r4_integer::math::checked_mul` or shift-add decomposition over leading bits.
4. **Line 600 (`advance`, SwiGLU MLP)**:
   - *Expression*: `i64::from(a) * i64::from(*u)`.
   - *Replacement*: Quantized 8-bit $\times$ 8-bit dyadic product table or 4-row blocked `low_bit_dot`.
5. **Line 680 (`recurrence`)**:
   - *Expression*: `lambda * lambda` in complement floor calculation.
   - *Replacement*: Precomputed combined decay/keep table `decay_keep_lut` directly storing $\sqrt{1 - \lambda^2}$.
6. **Line 692 (`recurrence`)**:
   - *Expression*: `i128::from(*v) * i128::from(*v)` in quaternion norm calculation.
   - *Replacement*: `math::sum_squares` over 4 quaternion coordinates.
7. **Line 698 (`recurrence`)**:
   - *Expression*: `i128::from(lambda) * unit` in quaternion rotation scaling.
   - *Replacement*: Dyadic scaling via `math::scale_pow2` or CORDIC rotation.
8. **Line 710 (`recurrence`)**:
   - *Expression*: `i128::from(keep) * i128::from(b.wide[4 * lane + k])` in drive injection.
   - *Replacement*: Shift-add scaling via `math::scale_pow2` and `math::checked_mul`.
9. **Lines 793, 800, 811 (`read`, Attention Scores)**:
   - *Expression*: `i64::from(*a) * i64::from(*b)` (dot product $\langle q, k \rangle$) and scaling by $1/\sqrt{d}$.
   - *Replacement*: Signed-4 blocked product table `low_bit_dot_4_contiguous_wide`; dyadic power-of-two scaling.
10. **Lines 844, 856, 863 (`read`, Value Mixing & Normalization)**:
    - *Expression*: `w * i64::from(*v)`, `i128::from(w) * i128::from(*v)`, and `m * reciprocal`.
    - *Replacement*: Grouped LUT accumulation (T-MAC style, accumulating partial weights directly into shift-accumulators); normalization via `math::div_round`.

---

## 4. Hardware Footprint & Bytes-Per-Token Budget

### 4.1 Lookup Table Footprint (M1 L1 Cache Resident)
All tables required for full R2 stack serving fit entirely within the 64 KiB L1 data cache of Apple Silicon M1:
- **Signed-4 Blocked Product Table**: 32 bytes (16 $\times$ i16) per thread.
- **Arcosh1p Q24 Table**: 4,100 bytes (1,025 $\times$ u32).
- **SiLU Q16 Table**: 1,024 bytes (512 $\times$ i16).
- **Exp_Q32 / Sigmoid Table**: 1,024 bytes (256 $\times$ u32).
- **Decay & Keep Factor Table**: 2,048 bytes (256 $\times$ u64).
- **2I Icosian Cayley Lookup Table**: 1,920 bytes (120 $\times$ 16 entries).
- **Total Static Table Footprint**: **~10.2 KiB** ($\le 16.0$ KiB ceiling, $\mathbf{6.2\times}$ margin under 64 KiB L1D).

### 4.2 State and Per-Token Memory Movement
For width $d = 288$, 6 layers (`rrarra` pattern: 3 recurrence, 3 read), context $C = 256$:
- **Persistent Recurrence State ($O(1)$ memory)**:
  - 3 recurrence layers $\times$ 72 lanes $\times$ 4 coordinates $\times$ 8 bytes (i64) = **6,912 bytes**.
  - Convolution tap history: 3 layers $\times$ 3 history steps $\times$ 288 channels $\times$ 4 bytes = **10,368 bytes**.
  - Total recurrent state: **17.28 KiB** (completely persistent across turns; zero heap reallocations).
- **Read KV Cache ($O(N)$ memory)**:
  - 3 read layers $\times$ 288 channels $\times$ 2 bytes (INT16) = **1,728 bytes / token**.
  - At full context ($N = 256$): **442.3 KiB** total KV cache footprint.
- **Bytes-Per-Token Parameter Traffic**:
  - Weight access per token (4-bit weights packed 2 per byte):
    - Recurrence layer: $(2 \times d \times d + 2 \times d \times d + 3 \times d \times \text{mlp}) / 2 \approx 497$ KB.
    - Read layer: $(3 \times d \times d + d \times d + 3 \times d \times \text{mlp}) / 2 \approx 580$ KB.
    - Total parameter access: $\approx 3.23$ MB per token across all 6 layers.
  - At M1 memory bandwidth (68 GB/s theoretical, ~50 GB/s sustained), 3.23 MB/token sets the theoretical memory bandwidth floor to **~0.065 ms / token** (~15,000 tok/s ceiling).

---

## 5. Implementation Roadmap & Guardrails

1. **Architecture Containment**: All new code extends `crates/uor-r4-integer`. No new crates, binaries, or engines are spawned (`uor-r4-lut` remains frozen as an unmaintained D10 comparator).
2. **Phase Gate Alignment**: This design remains read-only until Lab 1 (Claude) reports T1(a) exposure results (~07:00 UTC on 09-28). If the stack passes the $\le 0.03$ nats gate vs the transformer control, this design becomes the primary serving implementation.
3. **Audit Enforcement**: Any introduced assembly symbols in `uor-r4-integer` will be immediately added to `scripts/audit_zero_matmul_serving.py` and validated for 0 Class I/II/III instructions prior to merging.
