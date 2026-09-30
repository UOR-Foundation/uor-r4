# Track B: layer-12 replay separates local arithmetic from incoming state

Measured diagnostic source: `4ee0b88edb306ed17f983706187ea6a763fcbb1d`.
The sealed eight-token trace supplies all eight reference residual rows after
zero-based layer 11. The replay applies layer 12's unchanged shared normalization,
Q/K/V projections and RoPE on CPU and Metal with identical reference input and
weight bits. Batch/time/head shapes and positions 0–7 are retained.

A second arm supplies scalar-normalized inputs computed with the reference's
literal F32 order: sequential square sum, division by width, epsilon addition,
reciprocal square root, then `weight * (reciprocal * input)`. Both arms use the
same device projection/rotation operations. No reference forward or training ran.

At zero-based position 5, the native-normalization replay yields:

| Backend | Channel | Original candidate error | Same-input replay error |
|---|---|---:|---:|
| CPU | Q | 0.0022444725 | 0.0000016689301 |
| CPU | K | 0.0033278465 | 0.0000028610229 |
| CPU | V | 0.0005416274 | 0.0000007152557 |
| Metal | Q | 0.0021758080 | 0.0000019073486 |
| Metal | K | 0.0032315254 | 0.0000038146973 |
| Metal | V | 0.0005251765 | 0.0000006556511 |

These are maximum absolute channel errors against the saved reference Q/K/V.
Replacing native normalization with the reference scalar formula barely changes
these values. Across all eight positions and three channels, replay maxima are
`1.52587890625e-5` on CPU for either arm, `1.9073486328125e-5` for native Metal,
and `1.71661376953125e-5` for scalar-normalized Metal. The unchanged original
candidate maxima are `0.0033278465270996094` / `0.003231525421142578`.

This supports a scoped conclusion: **the original candidate's incoming state
is the dominant contributor to the observed layer-12 discrepancy in this
window**, rather than local layer-12 arithmetic evaluated on the same reference
input. It does not prove global numerical stability or identify the operation
that produced the upstream difference. Normalization can still amplify an
incoming perturbation; this replay does not measure that sensitivity separately.

Next inspect block 11 using the common saved post-layer-10 state, separating
attention/output projection, residual, post-attention normalization, MLP and
final residual. Preserve the original shape. Do not rerun the full reference
comparison unchanged, relax its `1e-4` criterion, or treat this diagnostic as a
parity pass. B2 fitting remains unqualified.

The warm build took 3.85 seconds and the replay 3.46 seconds, maximum process
RSS 1,285,062,656 bytes. The fresh root was claimed, sealed and verified:
`/Volumes/UOR-Workspace/uor-r4-models/track-b/layer12-replay-4ee0b88e-20260930`.
There are 96 comparison rows: two backends × two arms × eight positions × three
channels. The [machine-readable receipt](track-b-layer12-replay-2026-09-30.json)
preserves exact values and all file identities. Logs, metadata and independent
review are retained on the operational branch. No model-quality or serving-cost
claim is established by this shared-host diagnostic.
