# Track B: down-projection arithmetic on identical saved inputs

This diagnostic separates two quantities for the fixed block-11 down projection:
local arithmetic differences on identical input bits, and changes caused by
supplying different saved gated inputs. CPU-native and Metal-native gated values
come from the preceding block-11 common-reference-input replay, not from an
independent exact MLP intermediate. All eight rows and the [576,1536] weight
matrix are preserved.

Each input is projected by CPU, Metal and the teacher's pinned uor-matmul exact
GEMM (`b13c98449948174f590e337c4dc25dfc394a07d0`). The latter accumulates products
exactly and rounds each dot once to F32. It is not expected to be bit-identical
to every ordinary F32 GEMM. Replays on the matching input/backend must reproduce
the previous saved down outputs bit for bit before interpreting the differences.

For fixed input x, the comparison is C(x)-E(x) or M(x)-E(x). Comparing E(x_cpu)
and E(x_metal) isolates the effect of the different candidate inputs under one
common projection arithmetic. These quantities do not measure total error
against the full exact reference, because neither gated input is that reference.

Measured source: `93085dd5220f65dfcc8bfc718e5b7b85c3448756`.
Both matching-origin anchors reproduce all 4,608 saved elements bit for bit.
For **each** common input, the CPU and Metal output files are also bitwise
identical across all eight rows. This directly excludes a CPU-versus-Metal
contraction difference for these fixed inputs, shapes and build; it does not
establish equality for other shapes, data or kernels.

At zero-based position 5:

| Quantity | Maximum absolute difference | RMS difference |
|---|---:|---:|
| CPU-origin input: either backend versus exact | 0.000087738037109375 | 0.00000921491799054 |
| Metal-origin input: either backend versus exact | 0.0001220703125 | 0.00000988623858097 |
| Exact outputs from the two different inputs | 0.00067138671875 | 0.0000282053359825 |
| Original CPU/Metal down-output gap | 0.0006103515625 | 0.0000258875555410 |
| Difference between the two local rounding errors | 0.00006103515625 | 0.000004666555589996 |

The last three rows form a signed, coordinate-wise decomposition: the original
backend gap equals the exact-output input effect plus the difference in local
rounding errors. Their maximum norms must not be added as scalars. For this
position, differing gated inputs explain most of the cross-backend gap; exact
down arithmetic retains it. The small rounding contribution partially cancels
it. This is not total error against the original model-source reference.

Across all eight positions, either backend's largest local error against exact
is 0.0078125 for either input origin. These residual-space values do not measure
the final-logit acceptance criterion and cannot be compared to that gate as a
pass/fail substitute. Ordinary F32 GEMM and once-rounded exact dot products have
different rounding semantics; a difference alone is not a kernel defect.

The down-projection comparison is complete for this scope. Next examine the
upstream gate/up projections on a common saved post-normalized input and the
reference/native SiLU operation order on a common gate preactivation. Preserve
the exact down projection as the common downstream control. Do not repeat this
completed comparison unchanged or run another full reference forward before a
specific correction warrants it. The original full `1e-4` parity failure remains
unchanged; B2 fitting is unqualified.

Execution took 3.18 seconds with maximum RSS 821,411,840 bytes. Adding direct
access to the already-pinned exact library refreshed dependent crates: build
194.62 seconds, maximum compiler RSS 3,187,933,184 bytes, above the 2 GiB initial
estimate. The build completed without an artificial threshold restart, following
the owner's correction; these actual costs replace the estimate in accounting.
The new dev-dependency does not add a different exact library revision or change
production arithmetic. Files are claimed, sealed and verified at
`/Volumes/UOR-Workspace/uor-r4-models/track-b/down-replay-93085dd5-20260930`.
The [machine-readable receipt](track-b-down-replay-2026-09-30.json) preserves all
file hashes and the complete 32 local comparisons plus eight input-effect rows.
