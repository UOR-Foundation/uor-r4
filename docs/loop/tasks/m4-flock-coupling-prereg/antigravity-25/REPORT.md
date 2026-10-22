# REPORT: m4-flock-coupling-prereg, antigravity-25

Lab: antigravity. Item: [m4-flock-coupling-prereg](../../../../roadmap/items/m4-flock-coupling-prereg.md).

## Negative findings first

- **Both October discrete attention runs ended in `Q1_STOP_FOR_DIAGNOSIS`, leaving no promoted Relation KEEP checkpoint:** `docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md` (`rows_v1`, Q1 stop at batch `4,096`) and `docs/labs/a8-dose-2026-10-19/README.md` (Relation Q1 stop at batch `4,096`, `FrozenZero` completed `16,404` batches at `faed2c0d…`) both hit admission erosion before producing a promoted Relation KEEP. Binding the base checkpoint for C1 therefore awaits the PM's choice on `#2860` between waiting for the post-admission-guard A8 rerun (`#2832`) or binding the retained A8 `FrozenZero` / preflight checkpoint (`1,351 / 3,847` $P_{\text{succ}}$ admitted).
- **Five registration items require PM/owner rulings before flipping `TEST FITNESS: PENDING` to registered:** Posted on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539): (1) D11 admissibility of $k_{\max} \in \{2, 3\}$ applied by $|K|$ repeated additions; (2) confirmation that C1 remains wanted as the M4 starling/murmuration decisive run (`#2032`) vs. a sub-rung under `m1-discrete-attention` (`#2029`); (3) base checkpoint SHA-256 selection; (4) D37 item-3 confirmation for the offline GPU float export-gap forward; and (5) registration of the `1,024`-row `C0` sample size, $\ge 0.70$ proxy-agreement threshold, and DEV keep bar.
- **`K = +1` (the trit ceiling) yields zero improvement on Fixture T′, whereas `K = +2` drops the writer loss from `448` to `64` (`X7`):** On Fixture T′ (`tokens = [[A, A], [A, B]]`, query `0 = [A, A]`, context `1 = [A, B]`, target `1 = [A, B]`), `snap_2(a + b)` at $K = +1$ ties between $r_0 = A$ and $r_{14} = B$ at $(d_{L1} = 8, d_H = 4)$ and picks root `0 = A` by index tie-break, leaving the writer loss at `448` (identical to $K = 0$ and $K = -1$). Only at $K \in \{+2, +3\}$ does the residual uniquely snap to $r_{14} = B$ ($D(r_{14}) = 8 < D(r_0) = 16$), reducing the writer loss to `64`.

## What changed

- Posted the design note, re-derived control literals, and 5 open PM/owner registration questions on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539).
- Created [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) verifying all finite control literals (`C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`, including the exact `X7` counterfactual writer losses `[448, 448, 448, 256, 64, 256, 64]` and the `X8`/`X9` two-stage literals) against the build-time generated `geometry.rs` tables (`ROOTS`, `AXES`, `BFS`, `IDENTITY`) in pure integer and exact rational `(num, den)` arithmetic (`0` D23/D25/D32 ratchet sites).
- Authored [`docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`](../../../../harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md) with all 8 required `prep.rs::REQUIRED_BLOCKS` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- Authored [`docs/labs/m4-flock-coupling-prereg/README.md`](../../../../labs/m4-flock-coupling-prereg/README.md) with the complete derivation table and the staged six-column research notebook row for `docs/research/notebook/m1-discrete-attention.md`, and updated [`docs/mechanisms/flock-read.md`](../../../../mechanisms/flock-read.md).

## Verification

### Deliberate break

Perturbed the expected `K = +1` (ordinal `2`) writer loss in `x7_widened_counterfactual_lanes_match_derived_writer_losses_on_t_prime_and_c8` in [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) from `448` to `447`. Confirmed `cargo test -p uor-r4-discrete --test flock_coupling_controls --locked` exited non-zero (`FAILED`), then restored `448` and re-verified `7/7` passing.

### Tests

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test flock_coupling_controls --locked`: `7 passed; 0 failed` (`0.01s`).
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls --locked`: `6 passed; 0 failed`.

### Gates

- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `10/10` gates `GREEN`.

## Left open

- PM/owner responses to the 5 registration questions on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539) ($k_{\max} > 1$ D11 ruling, line identity, base checkpoint SHA-256 binding, D37 item-3 confirmation, and registration of the `1,024`-row `C0` sample / $\ge 0.70$ proxy-agreement threshold / DEV keep bar), followed by a non-author `TEST FITNESS: FIT` review on the registered sheet and backfilling the staged notebook row from [`docs/labs/m4-flock-coupling-prereg/README.md`](../../../../labs/m4-flock-coupling-prereg/README.md) (`§4`) into `docs/research/notebook/m1-discrete-attention.md`. References #2860.
