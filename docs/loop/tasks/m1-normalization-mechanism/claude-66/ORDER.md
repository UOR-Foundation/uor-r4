# ORDER: m1-normalization-mechanism, claude-66 (a separate normalization run path on the proxy trunk)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `origin/main`. On 2026-10-23 Casey picked option (a) of the design call on #2842 (6376686610) in Claude's chat: "build option (a), a separate arm-parameterized normalization run path on the proxy trunk that records the mode and leaves the A8 path locked, as a Claude unit with ORDER, tests, the prime_zeta re-pin and a full discrete suite run."

## Goal

The normalization sheet (`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`) runs three arms on the A8 proxy trunk: `N_snap` (`scale_matched_snap_v1`), `C_unscaled` (`unit_snap_control_v1`) and `C_raw` (`unnormalized_control_v1`). All three are in the GPU read kernel (#3071, #3088), and the training layer records the mode in checkpoints. But the only trunk entry, `attention` (`attention_learning.rs`), is the A8 registered path. It is locked to its one configuration and refuses every non-default mode (`validate_config`, `validate_initial_storage`, `controller.rs` `validate_registered_data`). So no command can run the two control arms (claude-57 `NOT_FIT`, prerequisite 1).

This unit adds a separate entry for the normalization test:

- **Commands.** `normalization` and `normalization-resume <config> <admission> <out>`, and `normalization-build-identity <config>`.
- **What it admits.** Exactly the A8 proxy-trunk configuration (Relation8 sweep 1, K8, nearest-address objective, active-leaf initialization, interleaved read-every-8, the A8 pinned codebook table, seed 1 or 2) with any of the three modes.
- **Recording.** The mode is in the scope string, recorded in run state and checked on resume; a mismatch refuses.
- **The A8 lock stays.** The existing `attention` entry and its registrations still refuse non-default modes, and their scope strings, schemas and behaviour do not change.

The sheet is still `NOT_FIT` and its authors will edit it, so this unit pins no sheet hash in code; an `include_bytes!` hash test would turn main red on every edit. The `normalization` and `normalization-resume` commands refuse with "normalization sheet not yet registered" before any CUDA work. `normalization-build-identity` works for each arm, so §6 can name the argv and a FIT reviewer can check it. Registering the FIT sheet is a later one-line PR.

`training::driver_source_sha256()` covers the changed sources and will move. This unit re-pins `prime_zeta.rs` in the same PR and runs every discrete target, not `--lib` (lesson from #3099: main went red when a pin outside `--lib` drifted).

## Cancel probe (run first)

At the base, does any command run `C_unscaled` or `C_raw` through the trunk, or does every trunk entry refuse a non-default `normalization_mode`? If one runs, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**` (the entry, its validation and tests, `main.rs` wiring, the `prime_zeta.rs` re-pin)
- This task folder

## Done when

1. **The entry admits the three arms.** `normalization-build-identity` accepts the A8 proxy-trunk config with each of the three modes and prints a scope that names the mode. It refuses Route, Flock, other seeds, other codebooks and other trunk settings.
2. **Runs stay held.** `normalization` and `normalization-resume` refuse "not yet registered" before any CUDA work.
3. **The mode is checked on resume.** Resuming or restoring with a different mode refuses.
4. **The A8 path is unchanged.** `attention` and `attention-build-identity` still refuse every non-default mode, and the existing A8, parent and injective scope strings and tests are unchanged.
5. **Each claim is pinned.** Finite CPU tests pin 1–4, and at least one test fails with its fix set aside.
6. **The pin is re-pinned.** `prime_zeta.rs` pins the new `driver_source_sha256()`, recomputed and stated in the REPORT.
7. **Checks pass:**
   - `cargo test -p uor-r4-discrete`, every target;
   - `cargo clippy -p uor-r4-discrete --all-targets -- -D warnings`;
   - `cargo fmt --check`;
   - `loop report --check` and `loop pr-check --local`;
   - D23/D25/D32 ratchets at baseline.

## Stop and ask if

- Admitting the three arms needs a change to the A8 entry's behaviour, a scope string or a schema.
- Recording the mode needs a GPU-side change, or anything outside `crates/uor-r4-discrete`.
- The DEV scoring for the sheet's §6 step 5 needs a new evaluation path. If so, name it in the REPORT as the next unit rather than building it here.
