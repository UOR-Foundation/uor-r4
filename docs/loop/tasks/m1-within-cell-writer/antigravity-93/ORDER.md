# ORDER: m1-within-cell-writer, antigravity-93

Lab: antigravity. Item: [m1-within-cell-writer](../../../../roadmap/items/m1-within-cell-writer.md). Base: `441a2eda6`.

## Goal

The item's done-when reads:

> - Saved Stage T outputs decompose the 455 versus 2,140 gap into admission, top-8 candidate coverage, within-cell L1 ranking and DEV read-code coverage, including a TRAIN-only within-cell reranker control.
> - B0 compares current single-pass, nearest-endpoint single-pass, two-pass, frozen/permuted frames, a 16→16 frame remap, lookup controls and the within-cell reranker on the same re-derived epoch-0 FullVectorV1 table under OuterL1V1 and frozen DEV rows.
> - The primary comparison uses the strict paired rule `d > 24 && d^2 > 4(b+c)` on eight-bit interior-key accuracy over the pre-registered intersection of admitted rows. Total top-1, intersection-admitted top-1, admission, code coverage, non-inertness, comparison counts, resident bytes and throughput are reported separately.
> - New-law controls pin bounded arithmetic, negative gates, multiple active levels, `s[12]=1/2`, direct copy succession, frame selection on TRAIN/validation, and old-lineage parity. Every negative gate is shown failing before its implementation passes.
> - #3128 and #3129 close only when their acceptance controls and the relevant matched receipts are sealed. A negative closes only the tested configuration. Training requires a positive B0 plus a new sheet, component ablations, throughput projection and distinct data-order or initialization replication. D11 is a separate later unit.

## Cancel probe (run first)

1. Verify that `m1-bvg-k-stage-t-training` (`#2836`) is `done` and its sealed Stage T archive (`caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz`, `sha256 = c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`) is present and contains `t16-frozen-zero-seed1/{final-checkpoint.json,final-dev.json,w2-summary.json}` and `t16-relation-seed1/{final-checkpoint.json,final-dev.json,w2-summary.json}`.
2. Run a host decomposition probe on the sealed `final-dev.json` and `w2-summary.json` outputs of `T16_fz` (`epoch = 420`) and `T16_rel` (`epoch = 678`):
   - On `T16_fz` (`14,287` DEV rows): `5,009` rows admit the target cell (`9,278` unadmitted). Inside those `5,009` admitted rows, `OuterL1V1` ranks the target token in `top-8` on only `1,484` rows (`3,525` admitted rows place the target token outside `top-8`) and `top-1` on `455` rows (`1,029` rows in `top-2..8`; `0` strict-win rows without L1 ties, `455` tie-broken top-1 rows), whereas the checkpoint-restricted TRAIN read-code table `T_restr(A_W)` scores `2,140` `top-1` (`4,006` `top-8`) and the restricted bigram `G_restr(A_W)` scores `3,084` `top-1` (`4,674` `top-8`) on the exact same `5,009` admitted cells.
   - On `T16_rel` (`14,287` DEV rows): `3,827` rows admit the target cell (`10,460` unadmitted), with `1,124` in `top-8` (`2,703` outside `top-8`) and `371` in `top-1` (`753` in `top-2..8`), against `1,602` `top-1` for `T_restr(A_W)` and `2,364` `top-1` for `G_restr(A_W)`.
3. Classification: `mechanism` — the `2,140 - 455 = 1,685` gap on `T16_fz`'s `5,009` admitted rows is real and sits entirely in within-shortlist / within-cell token selection, confirming the premise of `#3267` (`#3128` / `#3129`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-within-cell-writer*`
- `docs/labs/m1-within-cell-writer/**`
- `docs/loop/tasks/m1-within-cell-writer/**`
- `docs/mechanisms/discrete-learning.md`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-within-cell-writer.md`
- `docs/roadmap/README.md`

## Done when

- Saved Stage T outputs (`T16_fz` and `T16_rel`) decompose the `455` versus `2,140` gap into admission, `top-8` candidate coverage, within-cell vs cross-cell L1 ranking, and DEV read-code coverage, including a TRAIN-only within-cell reranker control (`R_cell_code`, `R_cell_bigram`, `R_cell_unigram`, and permuted-code null `R_cell_code_perm`).
- Ahead-of-time `nvcc`-compiled CUDA module (`within_cell_writer.ptx` / `within_cell_writer.cubin` from `crates/uor-r4-discrete/src/within_cell_writer_source.rs`) and host/GPU evaluator (`within_cell_writer.rs`, `within-cell-writer-run`) implement versioned `ReadLawV2` (`#3128`) and frame-source modes (`CurrentSinglePassV1`, `NearestEndpointSinglePassV1`, `TwoPassEndpointOutputV1` `#3129`, `FrozenConstantFrameV1`, `PermutedFrameV1`, `TrainRemap16V1`, `GlobalWriterFrameV1`) while keeping `discrete.cu` and `driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) untouched.
- Every negative/refusal gate (`s[1] != 4`, inert position mode `s[14] >= 3`, `s[13] == 1` identity bypass, relation bound overflow, and zero non-inertness) is shown failing when set aside before passing with the gate active, and all finite controls (`256/256` weighted snap, negative gates, multi-level active carry, `s[12] = 1/2`, bounded recent-8 relation accumulation + per-sweep reset + single-copy succession, frame-aligned copy succession, TRAIN/validation frame remap, and bit-exact old-lineage GPU parity) pass on CUDA.
- One-page pre-registration sheet `docs/harness/prep/m1-within-cell-writer-2026-10-26.md` receives a fresh-context non-author `TEST FITNESS: FIT` and `D67 PRE-RUN REVIEW: PASS` posted on `#3267` before GPU evaluation.
- B0 runs on GPU on the re-derived epoch-0 `FullVectorV1` table (`2c886736...`) under `OuterL1V1` and frozen DEV rows (`14,287` rows, with TRAIN controls fit on the `1,049,856` TRAIN rows), comparing current single-pass, nearest-endpoint single-pass, two-pass, frozen/permuted frames, the `16->16` TRAIN frame remap, distinct lookup controls (`T_frozen`, `T_rebuilt`, `G`), and the within-cell reranker (`R_cell_code` and `R_cell_code_perm`) using the strict paired rule `d > 24 && d^2 > 4(b+c)` on 8-bit interior-key accuracy over the intersection of admitted rows, with separate reporting of total `top-1`, intersection-admitted `top-1`, admission, code coverage, non-inertness, comparison counts, resident bytes, and throughput (`tokens/s` and `rows/s`).
- All GPU outputs are archived to `caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/` with `SHA256SUMS`, the pod is torn down (`0 pods running`), and the results, synthesis, notebook row, and `REPORT.md` are committed and approved by a fresh-context D66 exact-head and claims review.

## Stop and ask if

- Never stop to wait for Casey; any classification, bar, or scope question is resolved by our own fresh-context D68 decision review and logged on `#3267` with `[antigravity] owner log:`.
- If any finite GPU control (`G-Q` / `C1..C6`) or old-lineage parity check fails, stop immediately, classify as `INSTRUMENT` under D31, and repair before reading any B0 evaluation number.
- If two-pass does not beat nearest-endpoint single-pass on the primary 8-bit interior-key comparison (`d > 24 && d^2 > 4(b+c)`), or if frozen/permuted frames tie it, stop the two-pass configuration per `m1-within-cell-writer.md` step 5 (closing only the tested configuration under D22, not `#3129` unless its acceptance conditions and matched receipts are sealed) and record the exact-memory lookup / within-cell reranker path as the positive successor.
