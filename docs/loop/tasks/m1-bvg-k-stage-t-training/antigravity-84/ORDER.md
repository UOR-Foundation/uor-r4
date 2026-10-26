# ORDER: m1-bvg-k-stage-t-training, antigravity-84

Lab: antigravity. Item: [m1-bvg-k-stage-t-training](../../../../roadmap/items/m1-bvg-k-stage-t-training.md). Base: `d5453ca30`.

## Goal

The item's done-when reads:

> Stage 2 bar: 'beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.'

Complete the D68 decision review weighing `#2834` and dropping the unmet `W4` (`#2846`) fallback routing ([`DECISION.md`](DECISION.md)), update `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`, build the four owed Stage T instrument items (`crates/uor-r4-discrete/src/stage_t.rs`, `crates/uor-r4-discrete/src/bin/stage-t-run.rs`, and seed-1/seed-2 configs), obtain fresh-context `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review, run the five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) and two-seed `BVG-K` (`K = 16`) Stage T runs and checkpoint-restricted W2 evaluations on RTX 5090 pod(s), archive to `caseyallard/uor-r4-store`, and record the result in `docs/labs/m1-bvg-k-stage-t-training/README.md` and `docs/research/notebook/m1-discrete-attention.md`.

## Cancel probe (run first)

1. **Dependencies & `#2834` weighed ([`DECISION.md`](DECISION.md)):** All five `depends_on` items (`#2845`, `#2835`, `#2831`, `#3179`, `#2834`) are `done` on `origin/main`. In `#2834`, `FV8-Env` passed zero-update reach (`1,980 / 3,847` `P_succ`, `5,709 / 14,287` full-DEV admitted), while checkpoint-0 `top-1` fell (`133 -> 118`) and non-outer training/D11 serving are refused (`zero_update_only`). Meanwhile `A8-Outer` already admits `4,244–4,292 / 14,287` DEV rows (`~3x` the Stage 2 bars `U = 1,226`, `T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`), so Stage T (`K = 16` on `A8-Outer`) directly tests whether `16x` broader per-batch coordinate coverage clears the Stage 2 bars on two seeds.
2. **Host `bvgk_block` and `attention_audited` check:**
   - Command: `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib bvgk_block` -> `8 passed; 0 failed`.
   - `attention_audited::validate_config` rejects `coordinate_block != 1` and `seed != 1`, and `expected_family` assumes `K = 1` read indexing (`(completed / 8) % 45 < 20`), whereas at `K = 16` `o_read % 45 == 16` straddles `Combine` (`0..20`) and `Relation` (`20..45`).
   - Classification (`CHARTER.md §4.2`): **`instrument` (`GO`, not cancelled)** — build the four owed instrument items in `stage_t.rs` and `stage-t-run.rs` before running any GPU arm.

## Plan

1. Commit `ORDER.md` and `DECISION.md` first, then write `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`.
2. Implement `crates/uor-r4-discrete/src/stage_t.rs`, `crates/uor-r4-discrete/src/bin/stage-t-run.rs`, and `docs/labs/m1-bvg-k-stage-t-training/configs/*.json` (straddle-aware block accounting, Step 0 GPU controls `AKd`/`V1d`/`KId`/`V8d`/`V9d`, checkpoint-restricted W2 tables `T_restr(A_W)`/`G_restr(A_W)`, and Stage 2 grader).
3. Run fresh-context D68 decision review, `TEST FITNESS: FIT`, and D67 pre-run review, and post on `#2836`.
4. Run Step 0 controls, two-seed `T16_fz` and `T16_rel` training, and Step 3 W2 evaluations on RTX 5090, archive to `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/` with `SHA256SUMS`, release/down the pod, write `REPORT.md`, run fresh-context D66 review, and queue `#3239`.

## Files in scope

- `crates/uor-r4-discrete/Cargo.toml`
- `crates/uor-r4-discrete/src/lib.rs`
- `crates/uor-r4-discrete/src/stage_t.rs`
- `crates/uor-r4-discrete/src/bin/stage-t-run.rs`
- `docs/harness/prep/m1-bvg-k-stage-t-training*`
- `docs/labs/m1-bvg-k-stage-t-training/**`
- `docs/loop/tasks/m1-bvg-k-stage-t-training/**`

## Done when

Stage 2 bar (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:32`): the D68 decision review, updated sheet with fresh-context `TEST FITNESS: FIT`, D67 pre-run review, four Stage T instrument pieces, and two-seed GPU execution on RTX 5090 are completed, sealed to `caseyallard/uor-r4-store` with `SHA256SUMS`, and graded against the Stage 2 bar on two seeds, with `uor-harness loop gates --full` and `uor-harness loop pr-check --local` passing.

## Stop and ask if

- Any Step 0 GPU instrument control (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) fails on the pod (repair instrument defect first per `CHARTER.md §4.2`).
- Any file outside `#2836`'s `scope:` globs would be required.
- Any mechanism or grader would require softmax/squashing (`§3.2`), dense matmul (`§3.3`), or floating-point tokens/literals in `crates/uor-r4-discrete` (`D32`).
