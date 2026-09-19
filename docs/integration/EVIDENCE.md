# EVIDENCE

One row per executed card or measurement. This is the index; the linked card `RESULT.md`
and the receipt carry the full record. Append-only. A row is never rewritten to change an
outcome; corrections are new rows.

Legend — **Scorer**: `discrete` = `ExportedGeometricModel` additive scorer;
`continuous` = `JepaTrainer` probability model; `shortlist` = the 64-candidate served path.
**Data**: whether the evaluation set was sealed before the measurement.

## Artifact under test

| Artifact | Size | sha256 |
|---|---:|---|
| `native_geometric_prose_model.rgm` | 2,691,950 B | `7023c4507039acc01653b2dd6a41cf4329635767c66811a17d76591a4f356d96` |
| `native_geometric_prose_model.json` | 26,875,413 B | `c115c064522a58df8b0816c7eda8cc5aa0f065331479b986f76376634aae2f2a` |
| `.uor-models/native-geometric-prose-2026-09-19/native_geometric_prose_model_nocoarse.rgm` (coarse tier removed) | 963,950 B | `a25a87c4c5cdd5f7e6af1479273e29cc996d3cc78a1984315af8cf9b70a10a1b` |

Both artifacts are gitignored local material, not guaranteed by cloning Git. The digests
bind the 2026-09-19 measurements to the exact bytes evaluated.

## Rows

| Date | Card | Measurement | Scorer | Data | Result | Receipt |
|---|---|---|---|---|---|---|
| 2026-09-19 | P7 | Per-mechanism ablation BPB sweep, 9 ablations, 2,048 positions | discrete (full-vocab) | open development slice | 6 mechanisms contribute (`jepa` +0.3743, `s2_readout` +0.3377, `engram` +0.2228, `bias` +0.1482, `lattice_fine` +0.0955, `lattice` +0.0927); `vsa` −0.0007, `lattice_coarse` −0.0066, `lanes` −0.0071 are NEGLIGIBLE | [receipt](../evidence/native_geometric_p7_ablation_2026-09-19.txt), [result](cards/P7-falsification-sweep-RESULT.md) |
| 2026-09-19 | P7 | **Superseding reading under D2** (ablation is a measurement, not a verdict) | discrete (full-vocab) | open development slice | `vsa` is a **wiring defect** (fixed random codebook disconnected from the learned assignment), class `enabler` ⇒ repair and re-measure, **not** a retirement; `lattice_coarse` is BPB-NEGLIGIBLE but flips **9.5 %** of decisions and is 64.2 % of artifact bytes ⇒ removal candidate on resource cost; `engram` MAJOR+ with 37.3 % flip rate | [receipt](../evidence/native_geometric_p7_ablation_2026-09-19.txt) |
| 2026-09-19 | P7 | Instrument corrected: equivalence margin `epsilon`, declared mechanism class, decision-flip rate and top-1 change added to `ablate-prose`; `LATTICE_COARSE`/`vsa` no longer labelled HARMFUL | — | — | Statistical significance separated from practical significance; `enabler`/`selector` mechanisms are never retired on a near-zero delta | [D2](DECISIONS.md) |
| 2026-09-19 | P7 action | **Coarse lattice tier removed** by re-serializing (no retraining) | discrete (full-vocab) | open development slice | Artifact 2,691,950 → **963,950 B (−64.2 %)**; peeled sha256 `a25a87c4…`; 864 scoring self-check comparisons, 0 mismatches; baseline BPB on the stripped artifact **1.7989** = the `lattice_coarse`-ablated value on the original; `lattice_coarse` on the stripped artifact +0.0000 / 0.0 % flip (self-consistency); retained mechanisms unchanged; wall-clock 24.5 s → 4.4 s per 2,048-position scoring pass (debug microbenchmark, **not** a serving claim) | [receipt](../evidence/native_geometric_p7_coarse_strip_2026-09-19.txt) |
| 2026-09-19 | P7 | Query/key role inversion repaired; controlled induction test added | — | — | `r_query != r_key` gave identical tokens `d_H ≈ 2048 ≥ THRESHOLD` ⇒ zero weight for matches, weight for noise. Fixed by a shared `r_role`; 23/23 `native_geometric::vsa` tests pass | [result](cards/P7-falsification-sweep-RESULT.md) |
| 2026-09-19 | P7 | Metric/serving divergence quantified | discrete vs continuous | open development slice | continuous training-time 1.2372 BPB vs discrete artifact **1.8055** BPB — a 0.57 BPB gap between the reported number and the shipped model | [receipt](../evidence/native_geometric_p7_ablation_2026-09-19.txt) |

## Not yet measured

- **P1** M1 ground truth (bitnet.cpp BitNet b1.58 2B4T, llama.cpp SmolLM3-3B / Qwen3-4B
  Q4, the TLA bundle). Not run. No denominator exists for any efficiency claim.
- **P4** geometry controls (random phases vs zeta zeros, random token→root assignment vs
  the learned assignment). Not run. The two provable checks (Hamming ≡ angle-class table;
  reachable-root census) are specified but not yet written.
- **P7** `induction` ablation, the pairwise `jepa` × `s2_readout` factorial, Shapley
  attribution, and the `E1.4` recall-attribution measurement. Not run.
  `attribute-recall` exists and its smoke tests pass; the full 5..=13 k sweep is
  projected at ~20 min debug per the tool's own report and was not executed.
- The 8,192-position confirmation sweep (slice A at offset 0) was **interrupted by the
  owner after two ablations**; the partial result (`vsa` −0.0010, `engram` +0.2210, both
  replicating the 2,048-position signs and magnitudes) is recorded as partial, not as a
  result. Slice B was not started.
- The historical 9,984-case regression replay (`scripts/verify_qualification.sh`) was not
  re-executed in the 2026-09-19 change.
