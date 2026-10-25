# Full-vector address metric on the A8 placement: zero-update GPU pre-flight

- **Date:** 2026-10-25. **Lab:** claude (claude-110, #3230). **Item:** `m1-address-key-full-vector` (#2834), M1 Stage 1.
- **Supersedes** antigravity-37's draft, `m1-address-key-full-vector-DRAFT-2026-10-23.md` (`b8aa3aefd`). It keeps that draft's arms and bars, and adds:
  - the kernel;
  - the pinned tables;
  - the configs;
  - the CUDA control;
  - the grader.

  Registration follows D67: the lab's own fresh pre-run review replaces the draft's retired owner/PM registration.
- **Rules:**
  - zero training updates;
  - integer arithmetic only, with no softmax or squashing (§3.2) and no matmul (§3.3);
  - GPU for every model quantity (§3.5);
  - host counting of stored outputs for grading only (D24);
  - a negative closes a configuration (D22).

## Question

The writer's two-block outer key has 5 distance levels. Suppose it is replaced by a full-vector cell metric, on a full-vector placement of the same 4,096 tokens into the same 256 cells. Does starting reach at checkpoint 0 then rise above A8's 1,351 of 3,847 P_succ rows, with capacity clearing 1,941 winnable positions?

## Arms (checkpoint 0, zero updates, seed 1, `Relation { kept: 8, sweeps: 1 }`, `ActiveLeafV1`, interleaved schedule)

| Arm | Config | Pinned table (file sha256) | Writer metric (`s[15]`) | K | Role |
|---|---|---|---|---:|---|
| A8-Outer | `docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json` | `address_structured_v1`, `6600bdb1…` | `outer_l1_v1` (0) | 8 | exactness control |
| FV8-Env | `docs/labs/m1-address-key-full-vector/configs/fv8-env.json` | `address_full_vector_v1`, `2c886736…` | `cell_envelope_l1_v1` (1) | 8 | treatment 1 |
| FV8-TwoScale | `…/configs/fv8-two-scale.json` | `2c886736…` | `full_vector_cell_l1_v1` (2) | 8 | treatment 2 |
| FVS8-TwoScale | `…/configs/fvs8-two-scale.json` | `address_full_vector_shuffled_v1_seed_20261019`, `5e5abf33…` | (2) | 8 | shuffled-packing control |
| FV16-TwoScale | `…/configs/fv16-two-scale.json` | `2c886736…` | (2) | 16 | bracket only, against A16's 6,023 / 14,287 and 2,178 / 3,847; decides nothing |

`tests/address_metric_configs.rs` pins that each treatment config equals A8's except for its table, its metric and (FV16) K, and that every K8 arm has its own scope.

## Bar

1. **G-Exact (instrument):** A8-Outer reproduces, exactly:
   - 4,244 of 14,287 full-DEV target-admitted rows;
   - 1,351 of 3,847 P_succ admitted;
   - `top1_correct` 133.
2. **G-Q (instrument):** `device::address_metric_device_tests::device_admission_matches_the_host_oracles_for_each_metric` passes on the pod. The device's admitted cells must equal the `address_cells.rs` oracles, for all three metrics.
3. **G-Occ/G-Cap:** on the FV table, `coverage-census --token-cells --bar 1941` reports:
   - 4,096 distinct vectors, 256 occupied cells;
   - zero tokens in alias groups, and zero DEV rows targeting a non-representative;
   - alias ceiling 14,287 > 1,941.

   And min(alias ceiling, the treatment's full-DEV admitted) > 1,941.
4. **Starting reach (primary):** a treatment T, FV8-Env or FV8-TwoScale, each judged on its own, passes when all three hold:
   - B3(P_succ, T) > 1,351;
   - on the 3,847 P_succ rows, paired discordance against A8-Outer gives b − c > F = max(24, 2·√(b + c)), where b counts rows T admits and A8 does not, and c the reverse;
   - the same rule holds against FVS8-TwoScale.

   Two treatments are judged, so the label names which passed. FV8-TwoScale is the sheet's prior primary.

| Outcome | Condition | Unlocks |
|---|---|---|
| INSTRUMENT | G-Exact or G-Q fails | no treatment number is read; fix and rerun |
| PASS | gates 1–3 hold, and a treatment clears the bar against A8 and FVS8 | Stage 1 is met: #2834 is done; `m1-wider-code-carrier` (#2847) and Stage T's decision review (#2836) take it as input |
| PARTIAL | it beats A8, not FVS8 | the gain is packing balance, not predecessor geometry; #2834 stays open, and a decision review picks the next metric |
| FAIL | no treatment beats A8 + F, or G-Occ/G-Cap fails | closes (256-cell outer partition, B = 4 full-vector metric, K = 8) under D22; routes to #2847 |

## Base artifact

None is trained. Each arm is `init-corpus` at checkpoint 0 from its pinned table. The tables were built on the host (D24 data preparation) from TRAIN `3dee858f…` by `address-cells-codebook`, which rebuilds A8's `6600bdb1…` byte for byte. They are sealed in the HF store at `results/claude/address-key-2026-10-25/tables/` with a SHA256SUMS.

## Data split

- **TRAIN** `mw-balp/train`: tokens `3dee858f84b750f5…`, mask `b5e7352d67d6883a…`. Used for table construction only.
- **DEV** `ft/dev`: tokens `0919e90f953e988e…`, mask `a838bb1a67e96572…`, tokenizer `d36d3e8700a123e6…`. That is 14,287 rows, of which 3,847 are P_succ. Each arm scores it once, on the GPU.
- No FRESH slice is read.

## Steps

1. **Done:**
   - the host metric (antigravity-37);
   - the kernel dispatch, the zero-update gate and the CUDA control (claude-110 `78fec30ea`, `507ad9429`);
   - the configs (`d07fa3966`).
2. This sheet's non-author FIT, then the D67 pre-run review.
3. **On one leased RTX 5090, at a pinned source commit:**
   - first G-Q: `cargo test --release --features cuda --lib address_metric_device_tests -- --ignored`;
   - then, per arm: `init-corpus`, `eval-corpus`, and `attention-dev-b3` (the K8 arms);
   - per table: `attention-export-token-cells` and `coverage-census --token-cells --bar 1941`.
4. Archive everything, with SHA256SUMS, to `results/claude/address-key-2026-10-25/run/` before `uor-pod down`.
5. Grade on the Mac from the stored `dev-b3` rows. The paired counts use the rows' `target_admitted` flags, with the host grader added to this unit before the run. Record one notebook row.

## Stop rule

- **G-Exact or G-Q fails:** stop as an instrument fault (D31), never a mechanism negative.
- **Projection:** stop if `eval-corpus` on any arm projects past 10 minutes.
- **Hard ceiling:** 45 minutes wall clock on the pod, and free disk of at least 30 GiB.

## Cost

One RTX 5090, about $0.60/h, for under 45 minutes: about 15 minutes to build and qualify, then a few minutes per arm. That is **about $0.45**, with a cap of $1.

## Minimum test power

Every arm is scored on all 14,287 DEV rows and all 3,847 P_succ rows (at least 200 per CHARTER §4.3). The paired floor, F = max(24, 2·√(b + c)), is a two-sigma one-sided discordance test, with a floor of 24 rows (0.6% of P_succ).

## Test fitness

TEST FITNESS: PENDING - M1 #2029 m1-address-key-full-vector
