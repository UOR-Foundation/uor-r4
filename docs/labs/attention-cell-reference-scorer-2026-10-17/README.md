# Lab Record — Standalone Rust Same-Cell Reference Scorer (`attention-cell-reference-scorer`) (2026-10-17)

- **Lab:** Antigravity (`ag1`, host-only / source-only)
- **Milestone / Issue:** M1 ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029))
- **Scope / Commission:** [#2029 comment `6239938443`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6239938443) (`Codex/cf1` adapter request), [#2029 comment `6240046545`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240046545) (`ag1` claim), [#2029 comment `6240085282`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240085282) (Kimi PM acknowledgement), [#2029 comment `6240113871`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240113871) (`Codex/cf1` schema/field alignment), [#2029 comment `6240513498`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240513498) (Kimi PM ruling on GPU `token_cells.u16` export + host corpus count tables), and [#2029 comment `6240654230`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240654230) (`Codex/cf1` `attention-export-token-cells` alignment)
- **Compute / Spend:** `$0.00` (`0` pods, `0` GPU seconds, `0` model forwards, `0` updates applied)

## 1. Objective & Isolation Boundary

Provide a standalone, in-tree Rust receipt validator and same-cell reference scorer (`attention-cell-reference-scorer`) for M1 Unit 2b pre-flight (`preflight-dev`) and Dose 1 paired DEV (`paired-dev`) that:

1. Lives in `crates/uor-r4-harness` (`crates/uor-r4-harness/src/attention_reference.rs` and `crates/uor-r4-harness/src/bin/attention_cell_reference_scorer.rs`) with **zero modifications** to `crates/uor-r4-discrete/**`, `Cargo.toml` (workspace root), or `Cargo.lock`.
2. Preserves the gate8-compiled CUDA artifact (`fdc1b80b612fdca165545900a1e1cfe0929230027fc69cad48bbdb636760dc4e`), `driver_source_sha256` (`f14075f28b229535a740a76e78bda5418fd02274fc089b442c63a60c1705376d`), and `discrete_source_sha256` (`cb8fb6d055b78dd6cc28ac0c806c9001e34181dfca616c1b299907d85e16a8d3`).
3. Consumes actual GPU-observed 16-slot cell captures (`uor-r4.attention-preflight-dev-capture/1` from `attention-capture-dev`) and raw (`uor-r4.discrete-corpus-driver/1`) or enveloped (`uor-r4.attention-learning-evaluation/1`, `transfer == null`) DEV evaluation receipts, never substituting scalar `B3` counts for slot inventories or executing model compute on the host.

## 2. Schema & Field Bindings (`#2029` Comments `6240113871`, `6240513498`, `6240654230`)

- **Panel (`--panel`):** `uor-r4.attention-preflight-panel/1` (`status == "PINNED_PANEL_AND_PROBE_METADATA_ONLY"`, `scoring_performed == false`, `cuda_context_created == false`, `model_constructed == false`, `updates_applied == 0`, `config_file.sha256 == config_sha256`, `dev_binding.source.tokens.sha256 == 0919e90f...`, `dev_binding.source.mask.sha256 == a838bb1a...`, `14,287` eligible rows, `3,847` `P_succ` rows).
- **Capture (`--capture` / `--relation-capture` / `--control-capture`):** `uor-r4.attention-preflight-dev-capture/1` (`status == "ACTUAL_GPU_FORWARD_CELL_IDS_CAPTURED_ZERO_APPLY"`, `physical_cell_words_per_row == 16`, `unused_slot_marker == 4294967295`, `updates_applied == 0`, `proposals == 0`, `snapshot_unchanged == true`, `training_capability_issued == false`, `config_file.sha256 == config_sha256`, `checkpoint_file.sha256 == driver_checkpoint_sha256`, and `b3` summary counts matching the row-by-row tally). Every row's `admitted_cells[16]` is checked for `<= 8` unique active cell IDs in `0..256` followed by `u32::MAX` padding, with active count equal to `EvaluationRow.admitted_cells` and `eval_target_admitted == (EvaluationRow.admitted_cells - EvaluationRow.address_rivals == 1)`.
- **Evaluation (`--eval` / `--relation-eval` / `--control-eval` / `--ablation-eval`):** Accepts raw `uor-r4.discrete-corpus-driver/1` (`training::DRIVER_SCHEMA`, without `transfer_corpus`) or `uor-r4.attention-learning-evaluation/1` (`transfer == null`) wrapping `uor-r4.discrete-corpus-driver/1`. Verifies `evaluation_lineage_sha256 == capture.lineage_sha256`, `model_epoch == capture.model_epoch`, `accepted_updates_in_model == capture.model_epoch`, `evaluation_mutates_model == false`, `updates_applied_by_evaluation == 0`, `model_snapshot_sha256`, and `capture.evaluation_file.sha256`.
- **Sealed D34 Reference (`--d34`):** `uor-r4.successor-table-control/1` (`2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec`, `ordered_row_binding_sha256 == e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446`).

## 3. Same-Cell Candidate Re-Ranking Modes (Kimi PM Ruling `6240513498`)

1. **Bundled Candidate Cell Table (`--candidate-cells` / `--relation-candidate-cells` + `--control-candidate-cells`):**
   - Scores same-cell `T1`, `T1b`, `T2`, `U_succ`, and `U_train` across all eight registered DEV slices (`full_panel`, `copy_successor_subpanel`, `complement_subpanel`, `conflict_rows`, `d_1_8`, `d_9_32`, `d_33_96`, `d_97_192`) from a `uor-r4.attention-candidate-cell-table/1` JSON receipt after verifying `driver_checkpoint_sha256`, `model_snapshot_sha256`, `ordered_row_binding_sha256`, and all `14,287` rows' target-cell admissions against `capture.rows[r].eval_target_admitted`.
2. **GPU `token_cells.u16` Export + Registered Host Corpus Count Tables (`--token-cells` / `--relation-token-cells` + `--control-token-cells` with `--train-tokens`, `--train-mask`, `--dev-tokens`, `--dev-mask`, plus `bundle-candidate-cells`):**
   - Consumes a GPU-exported `token_cells.u16` file (`8,192` bytes = `4,096` little-endian `u16` cell IDs in `0..256` from `attention-export-token-cells`, or `8,256`-byte `UORT` v1, or `report.json` / `uor-r4.attention-token-cells/1` receipt) with **zero host `root_id` or `token_cell` codebook arithmetic**, automatically verifying any sibling `report.json` (`driver_checkpoint_sha256`, `model_snapshot_sha256`, `token_cells_sha256`, `updates_applied == 0`, `snapshot_unchanged == true`) and cross-checking on all `14,287` DEV rows that `capture.rows[r].admitted_cells.contains(&token_cells[target_token]) == capture.rows[r].eval_target_admitted`.
   - Verifies the four registered `UORT` / mask SHA256s (`3dee858f...`, `b5e7352d...`, `0919e90f...`, `a838bb1a...`), builds the non-model `T1`, `T1b`, `T2` (first `1,049,856` masked `TRAIN` targets), `U_succ` (`3,847` `P_succ` targets), and `U_train` count tables in Rust, enforces the Ruling 1 conservation checks (`train_bigram_total_pairs == 1,049,856`, `u_train_total_counts == 1,049,856`, `u_succ_total_counts == 3,847`, `t1_all_match_target == true`), records canonical output table SHA256s (`t2_ranked_table_sha256`, `u_train_ranked_table_sha256`, `u_succ_ranked_table_sha256`, `dev_window_successors_sha256`), cross-checks all `14,287` rows' unconditional predictions and hit flags against the sealed `d34-successor-table-record.json`, and either scores same-cell references directly or writes a sealed `uor-r4.attention-candidate-cell-table/1` via `bundle-candidate-cells`.
3. **Fail-Closed Without Token-Cells/Corpus or Candidate Cell Table:**
   - Emits `"status": "UNAVAILABLE_MISSING_GPU_TOKEN_CELL_TABLE_AND_RANKED_CANDIDATE_RECEIPT"` with explicit `missing_inputs` while reporting the unconditional sealed-D34-on-`B3` lower bound separately.

## 4. Verification & Non-Claims

- `cargo fmt --check -p uor-r4-harness`: PASS
- `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`: PASS
- `cargo test -p uor-r4-harness`: PASS (including unit tests for `uor-r4.discrete-corpus-driver/1` + `FilePins`, 16-slot validation, raw `8,192`-byte `token_cells.u16` parsing + out-of-range rejection, `token_cells` cross-check against `eval_target_admitted`, full slice scoring via `CandidateCellTable`, and exact integer ceiling `paired_integer_floor`)
- `uor-harness ratchet d23 --repo .` and `uor-harness ratchet d25 --repo .`: PASS
- `python3 scripts/check_claim_wording.py`: PASS
- **Non-claims:** Evidence utility only; outputs report `"scientific_verdict": "DEFERRED_TO_PM_DISPOSITION"`. No GPU or model execution occurred. Training, M2, and Stage B remain **HELD**; Q4 remains **NOT_EXECUTABLE**.

