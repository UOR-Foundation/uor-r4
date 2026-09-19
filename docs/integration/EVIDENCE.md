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

Both artifacts are gitignored local material, not guaranteed by cloning Git. The digests
bind the 2026-09-19 measurements to the exact bytes evaluated.

## Rows

| Date | Card | Measurement | Scorer | Data | Result | Receipt |
|---|---|---|---|---|---|---|
| 2026-09-19 | P7 | Per-mechanism ablation BPB sweep, 9 ablations, 2,048 positions | discrete (full-vocab) | open development slice | 6 mechanisms contribute; `vsa` inert (−0.0007); `lattice_coarse` harmful (−0.0066, 64 % of artifact bytes); `lanes` harmful (−0.0071); `jepa` largest contributor (+0.3743) from 62 bytes | [receipt](../evidence/native_geometric_p7_ablation_2026-09-19.txt), [result](cards/P7-falsification-sweep-RESULT.md) |
| 2026-09-19 | P7 | Query/key role inversion repaired; controlled induction test added | — | — | `r_query != r_key` gave identical tokens `d_H ≈ 2048 ≥ THRESHOLD` ⇒ zero weight for matches, weight for noise. Fixed by a shared `r_role`; 23/23 `native_geometric::vsa` tests pass | [result](cards/P7-falsification-sweep-RESULT.md) |
| 2026-09-19 | P7 | Metric/serving divergence quantified | discrete vs continuous | open development slice | continuous training-time 1.2372 BPB vs discrete artifact **1.8055** BPB — a 0.57 BPB gap between the reported number and the shipped model | [receipt](../evidence/native_geometric_p7_ablation_2026-09-19.txt) |

## Not yet measured

- **P1** M1 ground truth (bitnet.cpp BitNet b1.58 2B4T, llama.cpp SmolLM3-3B / Qwen3-4B
  Q4, the TLA bundle). Not run. No denominator exists for any efficiency claim.
- **P4** geometry controls (random phases vs zeta zeros, random token→root assignment vs
  the learned assignment). Not run. The two provable checks (Hamming ≡ angle-class table;
  reachable-root census) are specified but not yet written.
- **P7** `induction` ablation and the `E1.4` recall-attribution measurement. Not run.
  `attribute-recall` exists and its smoke tests pass; the full 5..=13 k sweep is
  projected at ~20 min debug per the tool's own report and was not executed.
- The historical 9,984-case regression replay (`scripts/verify_qualification.sh`) was not
  re-executed in the 2026-09-19 change.
