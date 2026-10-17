# Pre-registration — M1 read-code capacity diagnostic

## 4.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Authority:** owner D22/D24/D32; [M1 admission sheet](m1-discrete-rung0-admission-2026-10-15.md); [D34](../../integration/DECISIONS.md:1642); the math-falsifier review that produced this dispatch.
- **Line:** M1 qualification; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP, acceptance or count reset.
- **Delivery:** **docs only**. No instrument, kernel, launch, allocator or model change in this PR. Every new measurement is **NOT_RUN**. This is the design submitted for PM-seat review.
- **Scope:** **diagnostic, not qualification / not a capability claim / not a fresh-transfer result**. Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**. FIT releases none of these holds. These are not D-Release-class decisions; no owner hold changes.

## 4.2 Question and smallest falsifiable hypothesis

**Question.** At the tested implementation scope — seed-1 Q4, `C192 / B4 / V4096`, `ReadSpec::Route`, `K=8`, frozen 14,287-row DEV panel — is the wall the **read-code representation** (at most `16^B = 65,536` codes) or the **learned output map / training law** (16,384 codebook vectors + 20 Route trits + K=8 admission, trained by one-coordinate integer-hinge descent)?

**H_rep (representation wall).** A count table keyed by the exact 4-root read code, built on the same 1,049,856 TRAIN targets the model consumed, cannot beat the global masked-TRAIN unigram reference (`1,226/14,287`) on DEV. The 16-bit code is not informative enough to support more than the marginal target distribution.

**H_law (learning-law wall).** The same count table beats the unigram reference by at least the pre-registered noise floor. The read code carries information the trained output map fails to extract; the binding constraint is the output map/learning procedure, not the code alphabet.

**H_mixed.** The code table beats the trained writer but not the unigram, or only on a subset (e.g., high-count codes). The code is partially informative; the next diagnostic is code utilization/capacity, not a learning-law change.

**Disconfirming observation at this scope.**
- H_law is **disconfirmed** if `top1(code table) ≤ 1,226 + floor` on the model-matched table.
- H_rep is **disconfirmed** if `top1(code table) ≥ 1,226 + floor`.
- H_mixed is **disconfirmed** if the code table lies outside the band `(149 + floor_writer, 1,226 + floor_unigram)`.
- All three are **underpowered** if fewer than 90% of DEV rows have a TRAIN-seen code, or the per-code count distribution is too sparse to separate the predictors.

## 4.3 Observable signature and per-row outputs

The diagnostic produces one lossless per-row record per DEV row (14,287) and a compact code/target stream for TRAIN (2,425,925 rows; 2 bytes code + 2 bytes target ≈ 9.8 MB). No free-running generation is loaded; the model is scored teacher-forced.

Per DEV row:

| Field | Type | Source |
| --- | --- | --- |
| `row_ordinal` | u32 | frozen panel order; `ordered_row_binding_sha256 = e4ef44fb…` |
| `doc_id`, `target_position` | u32 | pinned `ft/dev` tokens/mask |
| `query_token`, `target_token` | u16 | pinned corpus |
| `code` | u16 (4 nibbles) | 4 root ids from `Forward::vectors()` (`device.rs:325-327`) |
| `root_first`, `root_last` | u8 | code nibbles 0 and 3 |
| `admitted_cells[8]` | u8 | `Forward::admitted_cells` (`device.rs:307-308`), `ABSENT=255` |
| `writer_prediction`, `writer_hit1` | u16/bool | sealed Q4 receipt, re-verified; `Forward::predictions` on a fresh forward |
| `table_prediction`, `table_hit1` | u16/bool | model-matched count table |
| `table_prediction_admitted`, `table_hit1_admitted` | u16/bool | table restricted to the same 8 admitted cells |
| `address_only_prediction`, `address_only_hit1` | u16/bool | 8-bit `(first,last)` table |
| `interior_only_prediction`, `interior_only_hit1` | u16/bool | 8-bit `(root1,root2)` table |
| `unigram_hit1` | bool | global TRAIN unigram top-1 |
| `bigram_prediction`, `bigram_hit1` | u16/bool | query-token → most frequent successor from the same 1,049,856 TRAIN rows |
| `code_train_count`, `code_majority_fraction` | u32/q16 | matched table |
| `seen_code`, `fallback_used` | bool | coverage flags |
| `source_sha256`, `checkpoint_sha256`, `panel_binding_sha256` | hex | identity |

Aggregates are exact integer sums per predictor and per denominator (14,287; 1,920 admitted; 3,847 `P_succ`; 10,440 complement; 1,335 conflict).

## 4.4 Strongest simple controls and matching

1. **Ordinary control (strongest simple):** global masked-TRAIN unigram top-1 = **1,226/14,287** (8.58%), top-8 = 4,060. This is the existing `Device::count_reference` (`device.rs:1453-1505`, `:1806-1867`; `discrete.cu:539-560`). It ignores context entirely. If the read code cannot beat it, the code adds no usable information beyond the marginal.
2. **Same-representation comparator:** the trained writer at Q4 = **149/14,287** overall and **149/1,920 = 7.76%** conditional on its admitted rows. This is the exact artifact the diagnostic is testing; its per-row hits are sealed (`evaluation-16404.json`, sha256 `4fff1d3c…`).
3. **Data-matched conditional comparator:** a count table built on the **exact 1,049,856 TRAIN targets the model consumed** (16,404 batches × 64, in corpus order), not the full 2,425,925. The full-TRAIN table is a separate data-scaling arm, not the decisive arm.
4. **Stronger ordinary control:** query-token bigram table from the same 1,049,856 rows (4,096 × 4,096 sparse counts). If the 16-bit code table cannot beat the bigram, the code adds nothing beyond token identity.
5. **Admission-matched comparator:** the code table restricted to the writer's 8 admitted cells. This is the **decisive conditional comparison**, because top-1 ≤ admitted rows (1,920 at K=8).
6. **Cost-matched:** host counting (seconds to minutes); no new model parameters; no new training.

## 4.5 Design and dose

**Instrument.** The existing `Device::forward` already exposes the read vector (`Forward::vectors()`, `device.rs:325-327`), the writer's predictions (`predictions`), and the admitted cells (`admitted_cells`). No new kernel is needed for DEV. For TRAIN, the production `forward` also runs the writer, which is the expensive path; a read-only variant that calls the same `read_vector` (`discrete.cu:270-273`) and skips only the writer is the bounded evaluation-only instrument. It changes `source_sha256`, so it requires:
- a reviewed evaluation-only source/checkpoint bridge (same pattern as the Stage-A instrument, `stage-a-admission-instrument-2026-10-15/README.md:13-18`);
- the production loader unchanged;
- a `uor-harness ratchet d23|d25|d32` pass on the new source;
- no training path change.

If the PM refuses a source change, the fallback is the existing `forward` on the model-matched 1,049,856 rows only, with no full-TRAIN arm. That fallback is projected at 1–2 GPU-hours and biases against H_law (a smaller table is a weaker comparator); it must be labeled as such.

**Dose.**
1. Restore the four pinned checkpoints; authenticate each against `a5a22dce…`, `dde889aa…`, `102beed4…`, `4477eb5c…`.
2. Load Q4 (`4477eb5c…`, cursor 16,404, 407 accepted updates, epoch 0) with `ReadSpec::Route`, `K=8`.
3. Enumerate TRAIN in pinned corpus order: for each of the 2,425,925 masked targets, build the C192 window exactly as `PreparedDriver::batch` does (`training.rs:265-272`); capture the 32-int read vector and the target token. The first 1,049,856 rows form the model-matched table; all rows form the full table.
4. Enumerate DEV: 14,287 rows in 224 batches (223 × 64 + 15), same order; capture the read vector, admitted cells and writer prediction.
5. Host: decode each 8-int block to a root id by exact match (`root_id`, `discrete.cu:282-285`); any `ABSENT` is an instrument fault, not a finding. Build `count[code][token]` for both TRAIN regimes; score DEV with a fixed tie rule (highest count, then lowest token id) and a fixed fallback (global TRAIN unigram). Build the address-only, interior-only, bigram and unigram tables.
6. Join to the sealed Q4 per-row hits by ordinal; verify `ordered_row_binding_sha256 = e4ef44fb…` and the target identity on every row.
7. Emit the per-row record and exact aggregates; run the instrument-validity checks.

**Zero-apply statement.** No `apply`, proposal, optimizer, epoch advance or checkpoint rewrite. `snapshot_unchanged: true` and `updates_applied: 0` before/after every batch. The checkpoints are read-only inputs. The capture is a forward pass only.

**No softmax / no matmul statement.** The design introduces no softmax, softplus, sigmoid, tanh, GELU or normalized exponential, and no matrix multiplication/GEMM. Counts are integer additions; scoring is exact integer comparison; the read is the existing integer `read_vector`; the writer is the existing integer `writer`. Any prohibited site in the new source is a blocker under D23/D25/D32, not a design choice.

## 4.6 Bars and uncertainty

- **Full panel, n = 14,287.** Paired discordance for each comparator: `b` = comparator hit & code-table miss, `c` = comparator miss & code-table hit; difference `c − b`; null SE `sqrt(b + c)`. Bar for "code table beats comparator" = `c − b ≥ max(floor, 2·sqrt(b+c))`.
  - vs writer (149): floor **20 rows** (D22 2-SE top-1 floor).
  - vs unigram (1,226): floor **68 rows** (2 SE of 1,226/14,287 ≈ 33.9; 2 SE ≈ 68). Use the larger of 68 and the paired 2 SE.
  - vs bigram: same rule, with its own paired counts.
- **Admitted conditional, n = 1,920.** Writer 149/1,920 (7.76%), SE ≈ 11.7; floor **24 rows** (2 SE). This is the decisive writer comparison.
- **Sub-panel, n = 3,847.** Floor **39 rows** (the M4 sheet's `P_succ` floor, `m4-finite-relational-head-2026-10-15.md:30-31`).
- **Wilson 95% intervals** for every rate; report the interval, not just the point.
- **D22 power.** The full 14,287-row frozen panel exceeds the ≥200-row requirement; no training and no seed is involved, so seed noise is not applicable. This is a diagnostic, not a decisive learning KEEP; a capability claim would require a fresh ≥2,000-row draw under a separate pre-reg.

## 4.7 Decision rule

| Observation (model-matched table) | Interpretation at this scope | Next decision |
| --- | --- | --- |
| `top1 ≤ 1,226` within the 68-row floor | H_rep: the 16-bit code adds nothing over the marginal | Representation/code-utilization diagnostic: realized code count, code–target mutual information, route-learning coverage; no learning-law change yet |
| `top1 ≥ 1,226 + 68` | H_law: the code carries information the trained map does not extract | Learning-law decomposition (objective vs optimizer vs schedule vs codebook capacity), with the code table as the fixed readout; no substrate change |
| `top1 ≈ 149` (within 24 rows) | Both the code and the trained map are far below the unigram; the representation is the wall | Representation change; also report code sparsity/coverage |
| `149 + 24 < top1 < 1,226 − 68` | H_mixed: the code is partially informative | Code-utilization/capacity diagnostic; do not change the learning law yet |
| `top1 < 149 − 24` | The geometric map extracts more than per-code counts; sparsity/coverage likely | Report the coverage curve; do not infer a representation wall |
| <90% of DEV rows have a TRAIN-seen code, or per-code counts are too sparse | INCONCLUSIVE / UNDERPOWERED | Report coverage and sparsity; next diagnostic is code utilization, not a capacity verdict |
| Root mismatch, capture ≠ production forward, hash/epoch change, count-sum mismatch | INSTRUMENT FAULT | Diagnose/repair (D31); never a mechanism negative |

**What changes the decision.** The single number that flips the programme is whether the model-matched code table crosses `1,226 + 68`. If it does, the next work is a learning-law decomposition; if it does not, the next work is a representation/code-utilization diagnostic. The admitted-conditional comparison against `149/1,920` localizes whether the output map or the admission channel is the immediate bottleneck.

## 4.8 Instrument validity

1. **Root round-trip.** Every captured block must match exactly one root; reconstruct the 32-int vector from the four root ids and compare bit-exactly to `Forward::vectors()` on a 1,024-row sample. Any mismatch is a fault.
2. **Production-path agreement.** On a 1,024-row sample, the read-only capture's q and admitted cells must equal the production `forward`'s q and admitted cells exactly.
3. **Sealed-receipt agreement.** The Q4 per-row `top1_correct` must match the sealed `evaluation-16404.json` (sha256 `4fff1d3c…`) on all 14,287 rows; the aggregate must be 149/747/1920.
4. **Count-table conservation.** The model-matched table's total counts = 1,049,856; the full table's total = 2,425,925; no token ≥ 4,096; no negative counts.
5. **Zero-apply.** Snapshot/epoch unchanged before/after; `updates_applied: 0`; checkpoint file hashes unchanged.
6. **Determinism.** The same code stream on a repeat of the first 1,000 rows must be byte-identical.
7. **D23/D25/D32 ratchets** on the new source; any prohibited site is a blocker.

**Which negatives only trigger diagnosis.** A root mismatch, a capture/production disagreement, a count-sum mismatch, a hash/epoch change, or a CUDA status fault is an **instrument fault** and triggers repair, not a representation or learning-law conclusion. An underpowered coverage result triggers a code-utilization diagnostic, not retirement of the E8 read mechanism.

## 4.9 Loaded artifacts, lineage and resource projection

| Item | Value |
| --- | --- |
| Base | `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe` |
| Q4 checkpoint | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` |
| Q1 / Q2 / Q3 | `a5a22dce…`, `dde889aa…`, `102beed4…` |
| Q4 evaluation | `4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8` |
| DEV tokens / mask | `0919e90f…` / `a838bb1a…` |
| TRAIN tokens / mask | `3dee858f…` / `b5e7352d…` |
| Tokenizer | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| Ordered row binding | `e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Source anchors | `discrete.cu:41-77` (Route read), `:71` (root output), `:264` (Flock/Relation root output), `:270-273` (dispatch), `:282-290` (root_id/token_cell), `:307-345` (writer), `:471-481` (admitted-only ranking); `device.rs:302-360`, `1175-1218`, `1453-1505`; `training.rs:265-291`, `859-897`; `lib.rs:47-65` |

**Resource projection.** One RTX 5090 through `uor-pod` only; 10–30 GPU-minutes for the read-only capture of 2.44M TRAIN + 14,287 DEV rows; host counting 1–5 minutes; build/controls 10–20 minutes; **1-hour lease**, <$0.50 at $0.908/h; <32 MiB derived records; $0 new GPU cost for this docs delivery. `df ≥ 30 GiB` before lease; caps ≤4 pods and ≤$8/h across labs.

**Stop conditions.** First root mismatch, capture/production disagreement, pin/hash mismatch, snapshot/epoch change, count-sum mismatch, CUDA/status fault, or disk-floor fault stops the affected execution and seals partial evidence for PM disposition. No on-lease patch, extra pass, or unreviewed retry.

**Record schema.** `uor-r4.read-code-capacity/1`, with `applied_updates: 0`, `model_compute_executed: true`, `zero_apply: true`, `source_sha256`, `checkpoint_sha256`, `panel_binding_sha256`, per-row records and exact aggregates. Seal with the existing `report_output::claim`/`seal` discipline.

## 4.10 D22, claim discipline, FIT and assumptions

**D22.** A negative closes the `(C192 / B4 / V4096, Route, K=8, seed-1 Q4, this code/readout)` configuration. It does not close the E8 read mechanism, the learning law, or any wider substrate. Stopping a mechanism family needs the owner.

**Claim discipline.** No capability claim. This is a diagnostic measurement on a frozen panel; it is not fresh transfer, not a language result, and not a qualification. A KEEP-style follow-up would need a fresh ≥2,000-row draw under a separate pre-reg.

**FIT / execution release.** `TEST FITNESS` applies to this design only. It authorizes no compute. Actual execution requires a separate PM release, exact source/device pins, the evaluation-only bridge, and (for any D-Release-class decision) owner release. These three diagnostics are not D-Release-class.

**Assumptions the lead must resolve.**
1. Authorize the read-only capture instrument, or accept the existing `forward` on the model-matched 1,049,856 rows with the bias disclosure.
2. Confirm the model-matched 1,049,856-target regime as primary, with the full 2,425,925 as secondary.
3. Confirm the 90% DEV-code coverage threshold and the sparse-code fallback.
4. Confirm the tie rule (highest count, then lowest token id) and the unigram fallback.
5. Confirm that a count-table readout is an acceptable **diagnostic** comparator under the native geometric programme, even though it is not a proposed serving mechanism.
6. Confirm the 68-row unigram floor and the 24-row admitted floor.
