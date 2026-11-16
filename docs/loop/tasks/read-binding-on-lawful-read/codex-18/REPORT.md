# REPORT: read-binding-on-lawful-read, codex-18

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The held design's rank key is false.** `R_b(i) = (rel_b[i] << 3) - d_pos(q_b, i)` assumes
  `d_pos ∈ {0,2,4,6,8}`. `positioned_root_cost` (`discrete.cu:37-40`) sums `|query[d] - root[d]|`
  over eight coordinates, so the distance is an unbounded L1. At `rel 1, distance 8` versus
  `rel 0, distance 0` the key ties, while the read's comparator (`discrete.cu:324`) puts the
  relation first. `m_bind = 2` on that gap is not a defined margin.
- **The CUDA launcher for `discrete_counterfactual` is not in `uor-r4-discrete`.** This unit is the
  host reference only; wiring the sidecar into the training kernel is the next unit, not this one.
- **No GPU run was made.** The local 4090 was idle, but the 1B run is another session's; the finite
  CUDA qualification waits for an explicit window.
- **The item is held and its scope excludes the discrete crate.** The owner's direction on
  2026-11-16 reopens it; the claim/scope ceremony was not used.
- No model or quality claim.

## What changed

- `crates/uor-r4-discrete/src/read_binding.rs` (new, live): `Candidate`, the exact `outranks`
  comparator, `BindSpan` (four-word sidecar), and `bind_hinge_cost` on the rank gap. The held
  scalar key is retained only in a test that records its boundary tie.
- `crates/uor-r4-discrete/src/lib.rs`: `pub mod read_binding;`.
- `docs/labs/read-binding-on-lawful-read/README.md`: section 2.4, the correction and the unchanged
  `m_bind = 2` / `W_miss = 4`.

## Measurements

- Four finite tests pass: the boundary tie; the rank-gap margin and miss; the absent span; the
  permuted-sidecar rank flip.
- Discrete library: **363 passed**, 1 pre-existing failure, 1 ignored.

## Checks run at the head

- `cargo test --locked -p uor-r4-discrete --lib read_binding` — 4 passed.
- `cargo test --locked -p uor-r4-discrete --lib` — 363 passed, 1 pre-existing failure.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --lib` — no warning in the new module.
- `rustfmt --check` on the new module — clean.

## Still open

- Wire the sidecar and the rank-gap hinge into the counterfactual training path.
- The finite CUDA qualification and the 4-arm × 3-seed panel.
- Whether the 1B run is on the Flock/Relation path at all; if it is native Q31/FBS, the analogous
  mechanism is the readout rank margin, not this hinge.

## Plain summary

The held read-binding mechanism had a broken rank key: it assumed the read's distance was a small
eight-valued code, but it is an unbounded eight-coordinate L1, so a one-step relation lead does not
dominate it. The hinge now uses the read's exact comparator and a rank gap, with the pre-registered
penalties unchanged. The host reference and its four finite tests are live; wiring it into training
is the next unit.
