# Track B: shape-preserving trace of the failed eight-token window

Measured source `34371f15de2176a237a913a6944a95be07baec08` adds a diagnostic
executable using existing reference and shared-model hooks. It does not change
production model arithmetic or the parity criterion. Tokens remain `[1,2,3,4,5,6,7,8]`,
reference sequence capacity 32, and shared execution batch 1/time 8.

The reference, shared CPU, and shared Metal each reproduced their corresponding
sealed-parent logits **bit for bit**, across 393,216 elements per path. These
anchors tie the traces to the actual failed window rather than a shorter
causally equivalent shape that might use a different F32 reduction path.
The diagnostic retained reference post-layer residuals and Q/K/V, shared Q/K/V,
and 720 per-layer/position/channel comparisons per backend.

At zero-based position 5 (token 6), where both shared backends had their worst
final-logit difference, Q/K/V error grows sharply between the captures at
zero-based layers 11 and 12:

| Backend | Layer | Q maximum error | K maximum error | V maximum error |
|---|---:|---:|---:|---:|
| CPU | 11 | 5.67e-5 | 6.68e-5 | 7.51e-5 |
| CPU | 12 | 2.24e-3 | 3.33e-3 | 5.42e-4 |
| Metal | 11 | 5.15e-5 | 6.29e-5 | 6.68e-5 |
| Metal | 12 | 2.18e-3 | 3.23e-3 | 5.25e-4 |

Values are rounded for this table; the [receipt](track-b-layer-trace-2026-09-30.json)
preserves exact values. This localizes an amplification boundary for this token;
it does not prove a particular faulty operation or explain every failing row.
The interval includes layer-11 attention/output projection, residual and MLP,
then layer-12 normalization/projection/rotation. Shared residuals and intermediate
MLP values were not captured. Do not relabel this as an isolated MLP finding.

Next: replay the layer-12 normalization/projection using identical saved
reference residuals and weights to separate local operator error from propagated
input error; if the local error is small, inspect layer 11's remaining operations
using the same inputs. Preserve the original shape and compare any reduced
shape explicitly. The existing reference outputs can be reused; no full parity
repeat is justified without a causal correction.

The warm release build took 4.03 seconds. The diagnostic completed in 86.03
seconds, maximum process RSS 1,278,197,760 bytes. Its report was exclusively
claimed, sealed and verified at
`/Volumes/UOR-Workspace/uor-r4-models/track-b/layer-trace-34371f15-20260930`.
These are shared-host execution costs, not serving benchmarks. No training or
language qualification occurred; the previous numerical parity failure remains.

Independent source review identified one status-handling gap: the measured
version reported diagnostic completion even if an anchor differed. All three
anchors in this actual run match. The successor changes mismatched-anchor status
to `ANCHOR_MISMATCH` with exit 2, while preserving its outputs. That reporting
correction does not alter the measured model operations or this result.
