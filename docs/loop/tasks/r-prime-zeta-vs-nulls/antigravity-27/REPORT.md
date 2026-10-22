# REPORT: r-prime-zeta-vs-nulls, antigravity-27

Lab: antigravity. Item: [r-prime-zeta-vs-nulls](../../../../roadmap/items/r-prime-zeta-vs-nulls.md).

## Negatives

- **Raw `coverage_census` alias ceiling (`dev_alias_ceiling_rows`) cannot separate injective codebooks:** In [`crates/uor-r4-discrete/src/coverage_census.rs`](../../../../../crates/uor-r4-discrete/src/coverage_census.rs) (lines 34–87), `alias_representatives(table, spec)` partitions vocabulary tokens into alias groups sharing an identical full $32$-integer vector and names each group by its lowest token ID `representative[t]`. For any injective codebook (`distinct_vectors == 4,096`), every alias group has size `1` (`representative[t] == t`), so `within_cell_tie_mass == 0` and `dev_alias_ceiling_rows == 14,287 / 14,287` across `PrimeZeta`, `TangentNull`, `NormalNull`, `QuotaOnlyInjective`, and `RandomInjective`. Separation at zero updates requires writer-cell crowding capacity (`cell_collision_pairs`, `max_cell_occupancy`) and GPU reader-admitted target reach (`B3_init(P_succ)`, `B3_full`, `Net G-Cap`).
- **Stale `driver_source_sha256` assertion in `prime_zeta.rs`:** Running `~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta` initially failed on `driver_source_identity_changes_with_rank_objective_metadata` (`crates/uor-r4-discrete/src/prime_zeta.rs:827`), which still carried the pre-renormalization hash `ed4655a2…` instead of the current `origin/main` `driver_source_sha256()` digest `b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24` (because `prime_zeta.rs` is compiled only in `bin` targets, not `--lib`). Updated the assertion to `b13d4920…` so all `prime_zeta` tests pass.
- **Cancel probe failure:** Omitting the writer-cell quota and full-vector uniqueness checks in `generate_integer_injective_null` causes `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in `crates/uor-r4-discrete/src/prime_zeta.rs` to fail (`FAILED`). Restoring them returns `ok`.
- **Zero-update GPU pod battery (`Step 5`) not yet executed:** Per [`docs/roadmap/items/r-prime-zeta-vs-nulls.md`](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (Steps 2, 4, 5) and CHARTER §4.5 / §5.2, the comparison bar is not yet registered (`PROPOSED, NOT REGISTERED`) and any GPU evaluation run requires a non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) and owner/PM approval before renting a pod (`~$0.20`).

## What was done

- Posted the design note and bar/pod-spend decision card on [`#2881` (`6353762758`)](https://github.com/UOR-Foundation/uor-r4/issues/2881#issuecomment-6353762758) and cross-linked on [`#2029` (`6353766545`)](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6353766545).
- Implemented `CodebookMode::RandomInjective { seed: u32 }` and `CodebookMode::QuotaOnlyInjective { seed: u32 }`, `generate_integer_injective_null()`, `writer_cell_occupancy()`, and finite unit test `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../../../crates/uor-r4-discrete/src/prime_zeta.rs) using 100% pure integer arithmetic over `geometry::ROOTS: [[i8; 8]; 16]` (`0` new `D23`/`D25`/`D32` sites).
- Extended [`crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`](../../../../../crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs) to export `random_injective:SEED` and `quota_only_injective:SEED` tables and report `min_cell_occupancy`, `max_cell_occupancy`, and `cell_collision_pairs`, with CLI parser unit tests (`cli_mode_parser_accepts_all_five_modes_and_rejects_unknown`).
- Authored [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`, bar marked `PROPOSED, NOT REGISTERED`) and [`docs/labs/r-prime-zeta-vs-nulls/README.md`](../../../../labs/r-prime-zeta-vs-nulls/README.md) with staged 7-arm configs (`p8.json`, `q0.json`..`q2.json`, `r0.json`..`r2.json`), [`pz-nulls-battery.sh`](../../../../labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh), and [`SHA256SUMS`](../../../../labs/r-prime-zeta-vs-nulls/evidence/SHA256SUMS).

## Verification

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta`: `6 passed; 0 failed` across all binaries (`address-cells-codebook`, `prime-zeta-codebook`, `prime-zeta-qualify`).
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --bin prime-zeta-codebook cli_mode_parser`: `1 passed; 0 failed`.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32`: all `OK` (`0` sites added).
- `uor-harness loop gates --full`: `10 / 10 GREEN`.

## Open items

- Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md), owner/PM confirmation of the comparison bar and `~$0.20` pod spend on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881) (`6353762758`), execution of [`docs/labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh`](../../../../labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh), and recording the completed row in `docs/research/notebook/m1-prime-zeta.md`. References #2881.
