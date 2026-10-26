# M1 address-as-read Stage 0: D11 copy non-vacuity and artifact parity

- **Item:** `m1-address-as-read` (#2833), M1/M3 exact memory. **Unit:** kimi-40, `References` only.
- **Decision:** `docs/loop/tasks/m1-address-as-read/kimi-40/DECISION.md`. This supersedes the old 54/80 emission plan and removes BPB from this sheet.
- **Run:** zero training. One RTX 5090 GPU job, <15 min and <$0.25 estimated; one run, no retry after FIT.
- **Artifact:** T16_fz Seed 1 D11 `trunk-model.d11`, 542,260 bytes, SHA-256 `cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd`. This is the position-stripped Stage T snapshot, not Level 3 trunk tables.
- **Tokenizer:** `tokenizer.json`, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **DEV:** `ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 14,287 rows.

## Question

Can the pinned D11 artifact and its GPU scorer produce a non-vacuous exact copy of an explicitly supplied value, while remaining bit-identical to the integer reference? If not, a hidden-fact address-oracle panel would be an invalid instrument.

## Stage 0 controls

1. **Artifact identity and GPU parity.** Load the pinned D11 with `IntegerDiscreteModel::from_bytes` and the existing D11 scorer kernel. Score all 14,287 DEV rows on the GPU pod. The GPU kernel and integer reference must agree on every prediction, top-8 and admitted cell. Record top-1/top-8/admission and any position-strip difference from retained 455/1,484 rather than assuming equality. Two independent runs must produce byte-identical row records.
2. **Copy non-vacuity.** Use 40 fixed synthetic prompt/value pairs whose values are disjoint from any later panel. Each value is explicitly present before the query. Render and tokenize exactly as later arms would, within 192 tokens. Report token counts and any non-single-character tokenization.
3. **Controls.** Measure the same values with no hint and with a random eight-letter hint, plus the exact hint. The no-hint and random-hint controls are never interpreted as a capability gain. A planted wrong value must change the emitted result.
4. **Scoring.** Report teacher-forced per-token top-1/top-8 and free-running exact-string recovery. Teacher-forced top-8 is descriptive oracle assistance, not generation capability. Free-running uses the same integer decoding path and a registered stop at the value boundary or context limit.

## Non-vacuity floor

For each of the two primary comparisons—exact hint versus no hint, and exact hint versus random hint—let `b` be exact-hint-only wins and `c` control-only wins, `d = b - c`. The copy probe is non-vacuous only if both satisfy:

`d > 24 && d^2 > 4(b+c)`.

This is the existing strict paired floor on 40 pairs. If either comparison fails, stop as an instrument/artifact-capability limitation. Do not build panel-v3 and do not call it a mechanism negative. Teacher-forced top-8 cannot substitute for the primary free-running exact-string comparison.

## Stop rules

- Any artifact/tokenizer/DEV hash or shape mismatch, GPU/reference disagreement, nondeterminism, or planted-control failure is an instrument stop (D31), not a treatment result.
- A position-strip difference is recorded before interpreting copy results; it is not silently repaired or hidden.
- A failed non-vacuity floor ends this configuration. No new panel, training or changed threshold after FIT.
- No BPB, no fresh-V2 opening, no model training, no new Python, no `Closes #2833`.

## Next branches

- **Both comparisons pass and parity is exact:** a later unit may draft a hidden-fact panel with matched hint controls and a new FIT/D67 review. That unit still will not claim that the store is the model's read without a separate consumer path.
- **Either comparison fails:** preserve the scoped copy/parity receipt. Do not build panel-v3. Reassess the artifact/instrument before any address-oracle work.

## Cost and storage

Host preparation and finite tests: $0. GPU: one pod, <15 min, <$0.25 estimated, inside 4 pods/$8/h. Reports <2 MiB, preserved with SHA-256 in the HF store before pod teardown. Full preparation, build, review, run and retry costs are recorded in the REPORT.

## TEST FITNESS line

TEST FITNESS: PENDING - M1 #2029 m1-address-as-read Stage 0
sheet: docs/harness/prep/m1-address-as-read-stage0-2026-10-26.md
reviewer: pending fresh non-author review
