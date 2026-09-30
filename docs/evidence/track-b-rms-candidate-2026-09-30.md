# Track B: RMS scalar-order candidate rejected after numerical regression

Candidate source: `d87485a884e19d49ae18e4c63a92a2ba891d5c03`.
The shared model changed RMS to explicit sum/division by width, reciprocal
square root and input-times-reciprocal before gain, while retaining native
backend reduction, F32, epsilon and differentiation. The four existing focused
causality/interface/gradient tests passed. This did not establish numerical fidelity.

The candidate then evaluated every one of 49,152 logits at each of eight original
positions on CPU and Metal. Reference and unchanged-parent outputs came from
the verified, bitwise-anchored trace; no reference forward was rerun. Tolerance
remained absolute `1e-4`.

**FAIL_NUMERICAL_GATE_WINDOW.** CPU regressed at every position by maximum
absolute error. Positions 1 and 2 changed from passing to failing. Total failing
CPU logits increased from 174,745 to 189,401; the maximum rose from
0.0070133209228515625 to 0.008243560791015625. Metal output is byte-identical to
the unchanged parent across all 393,216 values: 190,826 failing logits and
maximum 0.0067539215087890625 remain unchanged.

| Backend | Position | Parent max error | Candidate max error | Parent failing logits | Candidate failing logits |
|---|---:|---:|---:|---:|---:|
| cpu | 0 | 8.153915405273438e-05 | 9.608268737792969e-05 | 0 | 0 |
| cpu | 1 | 6.866455078125e-05 | 0.00014591217041015625 | 0 | 114 |
| cpu | 2 | 8.20159912109375e-05 | 0.00014495849609375 | 0 | 33 |
| cpu | 3 | 0.0006861686706542969 | 0.0010793209075927734 | 48863 | 49049 |
| cpu | 4 | 0.00011157989501953125 | 0.00018596649169921875 | 33 | 10710 |
| cpu | 5 | 0.0070133209228515625 | 0.008243560791015625 | 49152 | 49152 |
| cpu | 6 | 0.0009307861328125 | 0.0010662078857421875 | 35473 | 38357 |
| cpu | 7 | 0.0014667510986328125 | 0.001651763916015625 | 41224 | 41986 |
| metal | 0 | 6.341934204101562e-05 | 6.341934204101562e-05 | 0 | 0 |
| metal | 1 | 0.000141143798828125 | 0.000141143798828125 | 97 | 97 |
| metal | 2 | 7.641315460205078e-05 | 7.641315460205078e-05 | 0 | 0 |
| metal | 3 | 0.0007724761962890625 | 0.0007724761962890625 | 48923 | 48923 |
| metal | 4 | 0.0002079010009765625 | 0.0002079010009765625 | 15977 | 15977 |
| metal | 5 | 0.0067539215087890625 | 0.0067539215087890625 | 49152 | 49152 |
| metal | 6 | 0.000904083251953125 | 0.000904083251953125 | 36081 | 36081 |
| metal | 7 | 0.0013456344604492188 | 0.0013456344604492188 | 40596 | 40596 |

The result rejects this scalar-order-only change as the proposed correction.
It does not imply that reference normalization is defective, or identify native
reduction order as the sole remaining cause. A local sensitivity intervention
was insufficient to predict the complete composed model's response.

Production RMS was restored byte for byte from pre-candidate `9e3b4eae` in
commit `f1cb000f`. The candidate source commit, output arrays, failure receipts
and reusable saved-window comparison are retained. No broader comparison,
training or B2 fit followed. The restored source bytes equal the previously
validated parent; the restored operator was not separately rebuilt or rerun in
this cycle. The internal executable still represents the rejected candidate
until rebuilt and must not be treated as the current restored source.

This ends the current local arithmetic-correction trial. Keep the original
full-parity failure and ask the research council to choose the next coherent
numerical-contract/implementation path before spending on another arithmetic
variant. Ordinary F32 contractions and once-rounded exact projections use
different rounding semantics. No threshold or historical acceptance result is
changed by this escalation. Existing source integration and peer reviews can
continue while the research decision is pending.

Costs: focused tests including compilation 58.69s (four passed); diagnostic
build 50.29s; actual model replay 5.51s, peak RSS 1,297,645,568 bytes. The fresh
report was claimed, sealed and verified and exited 2:
`/Volumes/UOR-Workspace/uor-r4-models/track-b/rms-window-d87485a8-20260930`.
The [receipt](track-b-rms-candidate-2026-09-30.json) preserves identities and all
16 row comparisons. Independent source review establishes the experiment's
integrity, not candidate acceptance or full PR approval.
