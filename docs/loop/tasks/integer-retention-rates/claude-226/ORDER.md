# ORDER: integer-retention-rates, claude-226 (the retained recurrence, its flag and its export)

Lab: claude. Item: [integer-retention-rates](../../../../roadmap/items/integer-retention-rates.md) (#3456). Base: `a5eb6863a`.

**Written by hand,** if `loop claim` refuses on Kimi's #3436 (as it did for point-key-integer's units).

## Goal

The item's done-when 1, 2 and 4, on the laptop, with finite tests and no run. Parity (done-when 3) is the next unit's, under a sheet, on the GPU.

## Cancel probe (run first)

- The base's geometric recurrence (`stack/session.rs`, `if s.rec_geometric()` at the recurrence) serves only λ = `1 << (30 − idx)` and `GEOMETRIC_KEEP_Q31`.
- `StackShape::validate` refuses any `rec_gates` but `"geometric"`, so an engine without this unit refuses a new value.
- No open PR touches the scope's files.

If any fails, stop and report.

Run on 31 Oct at the base:
- the recurrence is `(1u64 << (30 - idx), GEOMETRIC_KEEP_Q31[idx])`;
- `validate` refuses `rec_gates` other than `"geometric"` ("the recurrence gates are not geometric");
- no open PR touches the scope;
- `loop claim integer-retention-rates` refused on the same false overlap with #3436's `crates/*/README.md`.

Result: **go**. The probe reads the code, so it is an instrument check.

## The design

1. **The flag:** a new `rec_gates` value, `"geometric-retention"`.
   - `rec_geometric()` is true for both values, so every geometric path holds. A new `rec_retention()` is true for the new one only.
   - `validate` allows it, and an engine without this unit refuses it by its own `validate`.
2. **The retained recurrence:** at the geometric recurrence, with `rec_retention()`, λ_i = `(1 << 31) − (1 << (30 − i))` and keep_i = `RETAIN_KEEP_Q31[i]`, at Q31.
   - The table is `isqrt(2^62 − λ_i²)`, built by a `const fn` (adds, shifts and `isqrt` at compile time). The step does only a table read, so the serving audit sees no new instruction kind.
   - Entry 0 is 1/2 in both readings, so the forms first differ at entry 1.
3. **The export:**
   - `FbsTrunkSnapshot` gains `retain_rates: bool` (`serde(default)`, so old snapshots read the same), and the FBS export writes `rec_gates` from it. A snapshot without it exports byte for byte as at the base.
   - `fbs-snapshot` reads the run's `config.json`, two levels above `MODEL_DIR`, for `retain_rates`. It refuses `fold_scores` or `reflect_floor` on, whose integer forms do not exist.

## Files in scope

- `crates/uor-r4-integer/src/stack/{format.rs, session.rs, kernels.rs, tests.rs, synthetic.rs}`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs` and `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `docs/loop/tasks/integer-retention-rates/claude-226/**`

## Done when

- **Tests,** each with a negative gate:
  - the λ and keep tables against an f64 reference of the float rule at every index;
  - a synthetic retained stack's recurrence state matches a test-side reference over several steps, and differs from the decay reading's;
  - an artifact without the flag exports and decodes byte for byte as at the base;
  - the snapshot's config reading refuses `fold_scores` or `reflect_floor` on.
- **The suites:** `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib` and the float crate's tests pass. So do the stub-nvcc CUDA check (`fbs-snapshot` is CUDA), the serving audit and the WASM build.
- **The REPORT** gives the negatives first. A fresh review runs in slices of at most 600 lines.

## Stop and ask if

A file outside scope is needed, or the retained form needs an instruction the audit refuses.
