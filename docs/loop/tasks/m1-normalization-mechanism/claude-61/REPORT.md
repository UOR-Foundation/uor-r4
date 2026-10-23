# REPORT: m1-normalization-mechanism, claude-61 (carry the normalization mode through snapshots, driver configs and export)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `61706a39e`, on PR #3071 after claude-60. `References #2842`; the item stays `ready`.

## What did not work, first

1. **The GPU round trip has not run.** The ignored device test now also checks that a `UnitSnapControlV1` snapshot restores to a unit-mode device that still runs forward. It needs one pod, and Casey's approval comes first. The same pod run covers claude-60's device test and the default-path A8 checkpoint-0 identity check.
2. **Five other admission gates do not check the mode yet.** `attention_learning.rs:197/811`, `controller.rs:162`, `relation_qualification.rs:995` and `attention_audit_qualification.rs:1569` compare `read_initialization` but not `normalization_mode`, so a unit-snap model could pass them. The ORDER asked for no behaviour change there. The normalization sheet's arms run through the training driver, which does check the mode, so its runs are covered. Those five paths are a follow-up if a control arm is ever run through them.
3. **`validate_and_extract_cells`** (public) still accepts a non-default mode. Only `export_d11` refuses one.
4. **Merging rotates `source_sha256`,** together with claude-60. This is one rotation for the whole PR, sequenced by the PM before any A8 GPU stage (#2842, 6368080811).
5. **Pre-existing clippy debt is unchanged and not in touched lines.** It moved by line only: `training.rs:1941` (was `:1875`), `d11_export.rs:420` (was `:413`), `normalization_mechanism.rs:687`, and dead code without `cuda`.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/snapshot.rs` | `normalization_mode` with `#[serde(default, skip_serializing_if = "NormalizationMode::is_scale_matched_snap")]`. The golden test also checks the explicit-default hash domains and that the text has no `normalization_mode`. New: `unit_snap_control_mode_is_stored_and_reloaded`. |
| `crates/uor-r4-discrete/src/device.rs` | `snapshot()` records the mode instead of refusing. `from_snapshot`, the only rebuild-from-snapshot path, builds through `new_with_normalization`. The ignored GPU test's assertion (d) is now a round trip. |
| `crates/uor-r4-discrete/src/training.rs` | A `DriverConfig.normalization_mode` field with the same serde. `validate()` refuses `UnnormalizedControlV1`, and unit snap with a `Route` read. A mode-mismatch refusal sits at each of `read_initialization`'s three checks: `validate_checkpoint`, `admit_model` and evaluation admission. `new_model` builds devices with the mode. New: `normalization_mode_round_trips_and_checkpoint_mismatch_refuses`. |
| `crates/uor-r4-discrete/src/d11_export.rs` | `export_d11` refuses a non-default mode, since the integer engine has no unit snap. New: `export_refuses_a_non_default_normalization_mode`. |
| `attention_learning.rs`, `attention_anchor_device.rs`, `driver_qualification.rs` | The default field in config literals, so they compile. |
| `docs/labs/m1-normalization-mechanism/README.md` | The wiring section is updated to the current state. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Crate tests.** `cargo test -p uor-r4-discrete --lib`: 227 passed, 0 failed, 1 ignored (pre-existing). The five new or changed tests pass on their own.
- **Default identity.** The golden-byte snapshot test passes unchanged. `model_snapshot_sha256` is equal with the field written explicitly and with it omitted. The text of a default snapshot and of a default `DriverConfig` contains no `normalization_mode`. The UORLUT01 round-trip test passes, so default export bytes are unchanged.
- **Negative gate.** With the `validate_checkpoint` mismatch check set aside, `normalization_mode_round_trips_and_checkpoint_mismatch_refuses` fails (a panic at `training.rs:1565`). After restoring it, the test passes.
- **Stub-nvcc check.** `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline -p uor-r4-discrete --features cuda --all-targets` exits 0.
- **Ratchets.** `d23` 4,451, `d25` 93 and `d32` 31, each equal to its baseline. The PR diff removes no `cfg(any())` gate.

## Checks run at the head

`cargo test -p uor-r4-discrete --lib`, the stub-nvcc `cargo check --features cuda --all-targets`, `cargo clippy -p uor-r4-discrete --all-targets --no-deps` (touched lines clean), `uor-harness ratchet d23|d25|d32`, `loop report --check`, `loop pr-check --local`.

## Skills used

- **`scope-guard`:** the change stays in `crates/uor-r4-discrete/**` and the lab folder. The five ungated admission paths and the public extractor are recorded, not changed.
- **`solution-integrity`:** default identity is pinned by the existing golden bytes and the hash-domain equality, not just asserted.

## Still open

- **The pod run, with Casey's approval:** the unit-snap device test (`cargo test -p uor-r4-discrete --features cuda --lib unit_snap_device -- --ignored`), plus the default-path A8 checkpoint-0 identity check at this PR's head.
- **The five admission gates** listed above, if a control arm is ever run through those paths.
- **The PM merges before any A8 GPU stage** (#2842, 6368080811).

## Plain summary

A `C_unscaled` model can now be configured, saved, reloaded and refused when mismatched. Existing checkpoints and configs are byte-for-byte unchanged. What remains is one GPU run that checks the new mode and confirms existing results are unchanged.
