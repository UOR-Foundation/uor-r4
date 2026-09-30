# Track B: anchored reference tail and isolated MLP/normalization effects

Measured sources: `3c6007ec3174188480a7aa4e203159d345da0318` and
successor `0db94dbf024c60bc240b4cf73068e54015522d48`.
Both reconstruct **all 4,608 original reference post-layer-11 residual values
bit for bit**, using saved reference attention output and post-layer-10 residual,
the pinned exact WO/gate/up/down maps, and literal reference scalar RMS/SiLU
operation order. Both parent seals and their manifest link are verified. No new
full reference forward ran. The successor adds only normalization/input arms.

This creates anchored reference intermediate values for the fixed eight-token
window. Native block-11 stages from the earlier common-input replay can now be
compared with those values. It does not reconstruct the original candidate's
full-forward state or prove correctness for other inputs and layers.

At position 5, native post-normalized input errors are 4.17e-7 CPU / 3.87e-7
Metal. Native up-projection errors are 3.81e-5 / 3.43e-5, gated-product errors
0.00033569 / 0.00022125, and down-output errors 0.00231934 / 0.00170898.
These stage comparisons include accumulated differences from earlier stages.
They are not isolated local operator errors.

Each intervention below changes one operator or supplied intermediate and keeps
reference arithmetic downstream, including the pinned exact down projection.
Values are maximum absolute **down-output** differences, not final logits.

| Intervention | Position 5 | Maximum across eight positions |
|---|---:|---:|
| cpu-gate-projection-only | 0.0001220703125 | 0.005859375 |
| cpu-up-projection-only | 9.1552734375e-05 | 0.00390625 |
| cpu-silu-only | 0.0001220703125 | 0.0001220703125 |
| cpu-rms-only | 0.0003662109375 | 0.017578125 |
| cpu-incoming-attention-only | 0.00152587890625 | 0.00390625 |
| metal-gate-projection-only | 0.0001220703125 | 0.005859375 |
| metal-up-projection-only | 9.1552734375e-05 | 0.00390625 |
| metal-silu-only | 0.0001220703125 | 0.0001220703125 |
| metal-rms-only | 0.0003662109375 | 0.021484375 |
| metal-incoming-attention-only | 0.0010986328125 | 0.005859375 |
| host-division-silu-only | 0.0001220703125 | 0.0001220703125 |

The native RMS arm receives the exact reference after-attention state. The
incoming-attention arm instead supplies the saved native after-attention state
to reference RMS and exact downstream maps. Its effect excludes the direct
residual addition, and does not isolate QKV, attention or WO individually.

For position 5, the incoming-state effect is larger than each isolated local
MLP or normalization effect. Across the full window, normalization arithmetic
has the largest isolated maximum in this panel. The maxima occur at different
coordinates and these interventions interact; **do not sum them into an additive
error budget**. Their sizes neither prove a broken kernel nor establish that a
single replacement fixes the complete model.

The SiLU comparison confirms the existing operation-order difference: source
uses `x * (1 / (1 + exp(-x)))`; native uses its SiLU implementation. Host division
is a separate common-exponential control. It yields the same position-5 down
maximum as the native SiLU substitutions, but this is not whole-array equality
or a universal explanation of device exponential differences.

This local diagnostic sequence is complete. A numerical correction must address
the actual normalization/activation/projection arithmetic contract while
retaining differentiation; do not replace the computation with a reference
provider or rename failed evidence. Next prepare a coherent source-order
normalization/activation candidate, evaluate it against the already saved full
parity outputs, and require the unchanged gate before B2 fitting. Do not run
another unchanged reference pass or continue enumerating local fixtures without
a correction or a decision they can change. Ordinary F32 contractions and
once-rounded exact dot products retain different rounding semantics; the prior
common-input down result ruled out a CPU/Metal difference only for its fixed case.

Initial build/replay: 5.17s / 7.59s, replay maximum RSS 855,326,720 bytes.
Successor build/replay: 5.04s / 14.22s, replay maximum RSS 864,010,240 bytes.
Both roots are claimed, sealed and verified:

- `/Volumes/UOR-Workspace/uor-r4-models/track-b/mlp-replay-3c6007ec-20260930`
- `/Volumes/UOR-Workspace/uor-r4-models/track-b/mlp-replay-0db94dbf-20260930`

The successor contains 160 original-stage comparisons, 48 local comparisons and
88 intervention rows. The seven original arms are retained alongside the four
new normalization/input arms. [Machine-readable receipt](track-b-mlp-replay-2026-09-30.json)
binds both roots and exact identities; complete results and reviews are on the
operational branch. Production arithmetic remains unchanged. This is not a
full-parity pass, language qualification or a geometric-model capability result.
