# Track B: block-11 common-input replay and attention substitutions

Measured source: `356a9a24151ec54e368bbbd8f794ed37e64e1b6d`.
This diagnostic reuses the bitwise-anchored eight-token trace, supplying the
same saved post-layer-10 reference residual to block 11 on CPU and Metal.
All eight rows, native grouped-query heads, weights and causal positions are
preserved. Three arms use native QKV/attention, saved reference QKV with native
attention, or saved reference QKV with the existing source attention functions.
The output projection, residual additions, normalization and MLP remain native
and identical between arms. No production implementation or gate changed.

At the previously identified worst token (zero-based position 5), maximum
absolute final-residual errors against the saved post-layer-11 reference are:

| Backend | Native | Reference QKV | Reference QKV + source attention |
|---|---:|---:|---:|
| CPU | 0.00238037109375 | 0.00048828125 | 0.0010986328125 |
| Metal | 0.00177001953125 | 0.00048828125 | 0.0010986328125 |

Across all eight tokens, the corresponding maxima are CPU
0.03515625 / 0.025390625 / 0.025390625 and Metal
0.02734375 / 0.029296875 / 0.029296875. These are residual-space values,
not final-logit gate measurements. Improvement at position 5 is not uniform
improvement across positions or a parity pass.

The native common-input QKV errors at position 5 are only 3.10–4.77e-6.
Replacing QKV changes CPU attended values by at most 2.38e-6 and the post-attention
normalized values by 3.43e-7. The same substitution changes the gated MLP product
by 0.00026703, the down projection by 0.00183105 and the final residual by
0.00189209. On Metal the respective post-normalization / gated / down / final
changes are 3.73e-7 / 0.00015259 / 0.00128174 / 0.00128174.

This establishes a local sensitivity path: small upstream perturbations grow
through the gated product and down projection in this block. It does **not**
establish an incorrect MLP implementation. Substituting source attention does
not eliminate the remaining discrepancy and increases position-5 error relative
to the reference-QKV/native-attention arm. Rounding and cancellation can make
partial substitutions nonmonotonic; these arms do not rank arithmetic owners.
Only QKV and the final residual have independent saved reference anchors.
Other intermediate comparisons measure substitution effects, not stage-wise
reference error. Nor does this replay recover the original candidate's block-11
residual or isolate all errors inherited from earlier blocks.

The next discriminating check is the unchanged MLP/down projection on common
saved intermediate inputs. Compare CPU and Metal using identical post-normalized
or gated values, and obtain the pinned exact projection result before attributing
remaining error to a kernel. Reuse these sealed intermediate arrays; no new full
reference pass is needed. Preserve the original failed `1e-4` gate and do not
start B2 fitting without qualification.

Warm build: 3.99 seconds. Replay: 4.59 seconds, maximum RSS 1,287,274,496 bytes.
Fresh root, claimed before loading and sealed/verified after execution:
`/Volumes/UOR-Workspace/uor-r4-models/track-b/block11-replay-356a9a24-20260930`.
It contains 48 QKV comparisons and 480 stage comparisons (including 48 final
reference comparisons), complete across two backends, three arms and eight
positions. Raw stage arrays remain in that root. The
[machine-readable receipt](track-b-block11-replay-2026-09-30.json) records file
hashes, source identity and result summaries. This diagnostic is not a model
quality, serving compliance, energy or frontier-capability result.
