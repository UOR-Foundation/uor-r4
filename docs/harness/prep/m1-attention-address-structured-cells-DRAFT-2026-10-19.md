# Address-structured injective cells for the learned attention read: pre-flight only — DRAFT

- **Status:** DRAFT. **Not registered. TEST FITNESS: PENDING.**
  - **This sheet runs pre-flight only:** zero training updates on any arm. A training sheet follows only if an arm clears the gates below.
  - Training stays HELD. Compute needs FIT, then PM dispatch, then the owner's release for this finite zero-update run.
- **Line:** M1 learned attention (#2029). **Lab:** claude.
- **Owner decision:** Claude chat, 2026-10-19 — "draft an address-structured injective cell table sheet (with the K-scaled table as a bracketing control), pre-flight only first."
- **Parent and context:**
  - The injective sheet `m1-attention-injective-init-DRAFT-2026-10-18.md` (FIT).
  - Its executed pre-flight (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/`).
  - The successor options card (#2029 `6257846074`).
- **Rules:**
  - D24: GPU only for every model quantity. Admission is scored on the device; the host only builds tables from TRAIN counts.
  - D23/D25/D30: no softmax, squashing or matmul anywhere, including the construction.
  - D32: integer tables.
  - D22: a negative closes a configuration only.
  - D40: no owner phrase is involved in a zero-update pre-flight.

## Why

The injective PrimeZeta table removed aliasing: 4,096 distinct vectors, a ceiling of 14,287 and zero tie mass. But it spread tokens uniformly over all 256 cells. With K = 8 admitted cells, a target's cell was admitted in only 427 of 14,287 DEV rows (3.0% ≈ 8/256), and in 127 of 3,847 P_succ rows, below the 500 floor.

The read can only rank what it admits. This sheet tests whether **placing each token's cell where the read looks** raises admission without giving up injectivity.

## Mechanism facts this design rests on (from source, `origin/main`)

- **Cell of a token.** `token_cell = root_id(block 0) · 16 + root_id(block B−1)` (`discrete.cu:286-289`). Every block must be one of the 16 orbit roots (`discrete_validate`, status 6).
  - Any token can therefore be placed in any cell, and each cell has 256 distinct vectors available.
  - Blocks 1 and 2 are free, but they enter within-cell ranking (`token_distance`) and the read's payload.
- **Admission at checkpoint 0.** The writer admits the `kept_cells` = K occupied cells nearest the read output `q`, by L1 distance on (block 0, block B−1) roots, with ties to the lower cell (`writer`, `discrete.cu:307-318`).
  - At checkpoint 0 (ActiveLeafV1: leaf gates +1, every other gate 0), `q_b = snap_2(u_b + m_b)`. Here `u` is the **last context token**, and `m` is the snapped consensus of up to 8 earlier context leaves, ranked by frame-root proximity (`flock_vector`, `discrete.cu:125-268`).
  - So **admission is centred on the last token's address, pulled by the context consensus.** It is not the last token's cell alone.
- **Admission is a model quantity.** Predicting it means running the read and the writer. Under D24 it is measured only on the GPU, through the existing zero-update pre-flight tools. The host constructs candidate tables and never scores them.
- **K limits.** Writer K has a maximum of 16 (`Spec::validate`; 16-slot admitted buffers), and the attention validator pins K = 8. The **bracketing control therefore uses K = 16**, the largest supported, through the generic `init-corpus` / `eval-corpus` path.

## Construction (host data preparation from TRAIN only; deterministic; integer)

**`address_structured_v1`** builds an injective V4096 / B4 table in which a token's cell is chosen to sit near the cells of the tokens that **precede** it in TRAIN. It does this in four steps.
1. **Count.** Count TRAIN response-masked bigrams `(s → t)`: predecessor `s` and target `t`, at the same eligible positions the driver scores. The counts are integers.
2. **Cell geometry.** Each cell is a pair of root ids `(r0, r3)`. Distance is the same L1 on root coordinates that the writer uses.
3. **Greedy placement.** Tokens are taken in descending TRAIN frequency, and each goes to the cell minimising `Σ_s count(s→t) · L1(cell(s), cell)` over its already-placed predecessors. The placement respects a **per-cell quota of 16**, so occupancy stays 256 cells × 16, exactly as for PrimeZeta. **The only change is where tokens sit, not how many cells are used.**
   - Ties go to the lowest cell id.
   - The first token with no placed predecessor goes to the cell PrimeZeta assigns it.
4. **Distinctness.** Within a cell, blocks 1 and 2 are assigned distinct root pairs in token-id order, so all 4,096 vectors are distinct. `distinct_vectors` is recounted on load.

**Source work before any run** (separate PRs, each reviewed):
- **(a)** A `CodebookMode::AddressStructuredV1` and constructor, exported in the existing `uor-r4.codebook-table/1` format with its `table_sha256`.
- **(b)** The injective scope and attention plumbing accept the new mode with no K or arm change.
- **(c)** The generic `init-corpus` / `eval-corpus` path accepts the bracketing K = 16 Relation config, or a minimal opt-in allows it.

The construction reads **TRAIN only** (pin `3dee858f…`). DEV is never used to choose cells.

## Arms (all checkpoint 0, zero updates, seed 1, Relation{8,1}, objective A, ActiveLeafV1, interleaved, `rows_v1`, Dose 1 pins)

| Arm | Table | Writer K | Role |
|---|---|---|---|
| P8 | PrimeZeta (`83c1380a…`) | 8 | **Control**, re-measured at this head (previously 427 full, 127 P_succ) |
| A8 | `address_structured_v1` | 8 | **Treatment** |
| P16 | PrimeZeta | 16 | **Bracket:** admission from width alone |
| A16 | `address_structured_v1` | 16 | Bracket: the combined effect, reported |

The P16 / A16 configs differ from P8 / A8 only in `kept_cells`.

## Bar (fixed before any measurement)

The gate below is the parent injective sheet's §3 battery, applied unchanged to the **A8** arm. It is what would promote A8 to a training sheet:
- **G-Init:** the table equals the pin word for word, both negative controls pass, and routes, gates and epoch equal the unpinned initializer's.
- **G-Occ:** `occupied_cells` > 16, with zero within-cell tie mass and no split alias group.
- **B3_init(P_succ) ≥ 500.**
- **Net G-Cap > max(500, top1_control(cp0) + 24)**, joined from the raw alias ceiling, the zero-update `B3` / `P_succ` admission and the control's checkpoint-0 top-1.

Readings:
- **Effect of placement.** A8's P_succ admission against P8's, by paired McNemar-style discordance on the same 3,847 rows. It is reported with `F = max(24, 2·sqrt(b+c))` as the decision floor for "placement moved admission".
- **Bracket.** P16 tests whether admission scales with K (expected about 2× P8 if uniform). A16 reports the combined effect. Neither bracket arm can promote anything by itself, because writer K = 16 is outside the attention validator.

## Base artifact

None. Every arm is a fresh checkpoint 0 (`init-corpus`, `new_model(0)`) from its pinned table and config. Each is bound by table file SHA, config SHA and lineage in the receipt.

## Data split

- **TRAIN** (`3dee858f…` / mask `b5e7352d…`) is used only by the host construction.
- **DEV** (`0919e90f…` / `a838bb1a…`, 14,287 rows; P_succ 3,847 by the panel's canonical selection) is used only for zero-update device scoring.
- **Fresh** slices are not touched.

## Steps

1. Merge source pieces (a), (b) and (c) above after review. Pin the A8 table's file and `table_sha256`, rebuilt on the pod and required to match.
2. FIT this sheet with a fresh-context adversarial reviewer subagent (owner direction 6255848646), posting every finding. Then PM dispatch.
3. On one GPU via `uor-pod`: build, then for each arm run `init-corpus` and `eval-corpus` (and `attention-dev-b3` for the K = 8 arms), then `attention-export-token-cells` + `coverage-census --token-cells`, then G-Init with both negatives. For the K = 16 arms, P_succ admission is joined from `eval-corpus` rows to the K = 8 panel's `is_psucc`, which is a host join of stored outputs.
4. Post the receipt. If A8 clears every gate, draft the A8 training sheet (the parent's Relation versus FrozenZero contrast with only the table changed). Otherwise post the successor decision.

## Stop rule

- Any instrument fault stops for diagnosis (D31), never as a negative.
- An arm failing a gate closes that configuration only (D22).
- Owner caps and the 30 GiB disk floor are hard stops.

## Cost

Under 30 min on one RTX 5090. The build is about 10 min; each arm's checkpoint-0, evaluation and export are under 1 min. That is about $0.40, the same instrument as the 2026-10-19 pre-flight.

## Minimum test power

- B3_init and G-Cap are exact counts on the full fixed DEV panel (14,287 rows; 3,847 P_succ). The 500 floor exceeds the panel's 2-SE row floor.
- The A8-against-P8 effect uses paired discordance on identical rows with `F = max(24, 2·sqrt(b+c))`. It is reported regardless of the gate.

TEST FITNESS: PENDING
