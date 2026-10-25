# ORDER: m1-bvg-k-stage-t-training, antigravity-84

Lab: antigravity. Item: [m1-bvg-k-stage-t-training](../../../../roadmap/items/m1-bvg-k-stage-t-training.md). Base: `d5453ca30`.

## Goal

The item's done-when reads:

> Stage 2 bar: 'beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.'

Complete the D68 decision review weighing `#2834` (`m1-address-key-full-vector`) and dropping the unmet `W4` (`#2846`) fallback routing ([`DECISION.md`](DECISION.md)), update the Stage T pre-registration sheet (`docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`, matching `#2836`'s `scope:` glob), build the four owed Stage T instrument items (`crates/uor-r4-discrete/src/stage_t.rs`, `crates/uor-r4-discrete/src/bin/stage-t-run.rs`, and seed-1/seed-2 configs), obtain fresh-context `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review, run the five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) and the two-seed `BVG-K` (`K = 16`) Stage T runs and checkpoint-restricted W2 table evaluations on rented RTX 5090 pod(s), seal the artifacts to `caseyallard/uor-r4-store`, and record the result in `docs/labs/m1-bvg-k-stage-t-training/README.md` and `docs/research/notebook/m1-discrete-attention.md`.

## Cancel probe (run first)

1. **Dependencies & `#2834` result weighed ([`DECISION.md`](DECISION.md)):**
   - All five `depends_on` items (`m1-w2-capacity-capture` `#2845`, `m1-bvg-k-block-apply-c2` `#2835`, `m1-a8-full-dose-rerun` `#2831`, `m1-w2-code-utilization` `#3179`, and `m1-address-key-full-vector` `#2834`) are `status: done` on `origin/main` (`d5453ca30`).
   - In `#2834` (`claude-110`, PR `#3235`), `FV8-Env` passed the Stage 1 zero-update reach bar (`1,980 / 3,847` `P_succ` rows, `5,709 / 14,287` full-DEV admitted, `top-8 = 1,126`), while checkpoint-0 `top-1` dropped from `133` (`A8-Outer`) to `118 / 14,287`, and training/serving under non-outer `AddressMetric` are refused (`zero_update_only`). Meanwhile, on `A8-Outer` (`address-structured-v1.json`, `6600bdb1…`), `4,244–4,292 / 14,287` DEV rows already admit the target cell (`~3x` the Stage 2 bars `U = 1,226`, `T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`), and `#2831` stopped at `871 / 14,287` (`Q1`) on `K = 1` `consecutive_rejections`. Thus Stage T (`BVG-K` `K = 16` on `A8-Outer`) directly tests whether `16x` broader per-batch coordinate coverage and verified joint moves clear the Stage 2 bars on two seeds.
2. **Host `bvgk_block` and `attention_audited` check:**
   - Command: `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib bvgk_block`
   - Result: `8 passed; 0 failed` (`bvgk_block` unit tests pass). `attention_audited::validate_config` currently rejects `coordinate_block != 1` and `seed != 1`, and `AuditMachine::expected_family` assumes `K = 1` read indexing (`(completed / 8) % 45 < 20`), whereas at `K = 16` the read block at `o_read % 45 == 16` (`indices 16..32`) straddles `Combine` (`0..20`) and `Relation` (`20..45`).
   - Classification (`CHARTER.md §4.2`): **`instrument` (`GO`, not cancelled)** — build the four owed instrument items (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:24-28`) in `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs` before running any GPU arm.

## Plan

1. Commit `ORDER.md` and `DECISION.md` in `docs/loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/` first.
2. Write the updated Stage T pre-registration sheet `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md` (matching `#2836`'s `scope:` glob `docs/harness/prep/m1-bvg-k-stage-t-training*`, weighing `#2834`, and dropping the unmet `W4` `#2846` fallback routing per `claude-107/DECISION.md:34`).
3. Implement `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs` (plus `pub mod stage_t;` in `crates/uor-r4-discrete/src/lib.rs`, without touching `training.rs` or moving `driver_source_sha256`), covering:
   - Straddle-aware block evidence accounting (`K = 1` and `K = 16`, `seed = 1` and `seed = 2`) under `AuditPolicy::registered()`;
   - The five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`);
   - Checkpoint-restricted W2 table construction and evaluation (`T_restr(A_W)` and `G_restr(A_W)` on the checkpoint's admitted cells `A_W` alongside `U = 1,226`, `T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`, and `T2 = 6,300`), and exact integer Stage 2 grading;
   - Committed Seed-1 and Seed-2 configs in `docs/labs/m1-bvg-k-stage-t-training/configs/`.
4. Run fresh-context D68 decision review, `TEST FITNESS: FIT`, and D67 pre-run review, and post on `#2836`.
5. Lease RTX 5090 pod(s) inside fleet caps, run Step 0 controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`), run `T16_fz` and `T16_rel` on Seed 1 and Seed 2 (plus `T1` Seed 2 if needed) and Step 3 W2-restricted-table evaluation, archive all artifacts to `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/` with `SHA256SUMS`, release/down the pod(s), record the result in `docs/labs/m1-bvg-k-stage-t-training/` and `docs/research/notebook/m1-discrete-attention.md`, write `REPORT.md`, run fresh-context D66 review, and queue `#3239`.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-bvg-k-stage-t-training*`
- `docs/labs/m1-bvg-k-stage-t-training/**`
- `docs/loop/tasks/m1-bvg-k-stage-t-training/**`

## Done when

Stage 2 bar (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:32`): the D68 decision review (`DECISION.md`), updated sheet (`docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`) with fresh-context `TEST FITNESS: FIT`, D67 pre-run review, four Stage T instrument pieces (`stage_t.rs`, `stage-t-run`, seed-2 configs, five Step 0 GPU controls `AKd`/`V1d`/`KId`/`V8d`/`V9d`, and checkpoint-restricted W2 tables `T_restr(A_W)`/`G_restr(A_W)`), and two-seed GPU execution on RTX 5090 pod(s) are completed, sealed to `caseyallard/uor-r4-store` with `SHA256SUMS`, and graded against the Stage 2 bar ('beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds'), with `uor-harness loop gates --full` and `uor-harness loop pr-check --local` passing.

## Stop and ask if

- Any Step 0 GPU instrument control (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) fails on the pod (stop before reading any training metric, repair the instrument defect first per `CHARTER.md §4.2`).
- Any file outside `#2836`'s `scope:` globs would be required.
- Any mechanism or grader would require softmax/squashing (`§3.2`), dense matmul (`§3.3`), or floating-point tokens/literals in `crates/uor-r4-discrete` (`D32`).
