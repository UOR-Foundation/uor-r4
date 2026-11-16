# ORDER: one-model-end-to-end, codex-22 (the read-binding hinge's trainer)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-21](../codex-21/REPORT.md). Owner direction: build the mechanisms, not the harness.

## Why

Line 2 of the owner's order is the discrete read-binding hinge. Its host reference is
`crates/uor-r4-discrete/src/read_binding.rs` (corrected 2026-11-16: the held
`(rel << 3) - distance` scalar key is not the read's order, so the hinge is the exact
comparator's **rank gap**). What does not exist is its **trainer**: the archived lab ran the
hinge through an external `Flock` driver, and that driver is not in the repository. Without a
trainer the hinge is a cost function with nothing that descends it, and line 2 is not built.

## Design

- **The hinge on the served read.** A row's candidates are its earlier rows in the window; the
  read orders them by its own L1 distance (a tie to the later row). With `rank_bound` and
  `rank_comp`, `margin = max(0, M_BIND - (rank_comp - rank_bound))`, `miss = W_MISS` unless the
  bound is the read's first pick, `H_bind = margin + miss`. `M_BIND = 2`, `W_MISS = 4`, the
  registered constants. A row with no bound label costs zero.
- **Labels from the served targets.** The dump's `.ask.u32` bounds and `.other.u32` competes
  (the memory arm), or the reverse (`--reverse`, the codex-18 control). The host carries the
  competing positions as a list, so a 192-row window loses nothing to the device's 64-leaf mask.
- **The step.** The pick step's sign-residual credit, taken between the bound and the best
  competing row (or the read's own first pick when the row carries no competing label). The
  rank hinge is discrete, so the descent criterion is the pairwise distance gap and the rank
  cost decides the ties.
- **No device twin in this unit.** The CUDA pick kernel computes a distance rival; the rank
  hinge needs both ranks. The host step is the reference, as `train_pick_step` was before its
  kernel.

## Files

- `crates/uor-r4-native-vector/src/bind_train.rs` (new): `BindLabels`, `BindRow`, `BindReport`,
  `bind_hinge`, `train_bind_step`, finite tests.
- `crates/uor-r4-native-vector/src/read_train.rs`: `addresses`, `add_credit`, `apply_credit`
  crate-visible.
- `crates/uor-r4-discrete/examples/native_bind_train.rs` (new): the trainer.
- `docs/labs/read-binding-on-lawful-read/README.md`: point the correction at the implementation.
- `docs/loop/tasks/one-model-end-to-end/codex-22/**`.

## Done when

1. The hinge and the step have finite tests: the rank gap, the miss, an absent label, a refusal,
   and a step that moves the bound to the front.
2. The trainer runs on the FBS dumps of both states, on a held window split, with the hinge
   before and after.
3. The memory arm's held bound-pick rate is stated against the plain pick trainer's, and the
   `--reverse` arm's is stated against codex-18's 0.125.
4. The mechanism's ceiling is named: whether the hinge reaches the other fact or not.

## Cancel

Stop if the hinge fails to fall on the rows it trains, if the step cannot move the bound to the
front on a fixture, or if the labels cannot be expressed without a device-side change. Report
the number; no retry with a loosened `M_BIND` or `W_MISS`.
