# ORDER: end-to-end-integration, claude-396

Lab: claude. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-2](../codex-2/ORDER.md) (PR #4013, exact spin-1 frame invariance).

## Why this unit exists

Casey asked on 15 Nov 2026 for the recommended step: "For sh-spin, run a planted-symmetry retrieval
probe against popcount and L1 controls before building the l = 2 and 3 tables."

codex-2 proved that the spin-1 pair score is exactly invariant under a change of local frame. That
is a property of the arithmetic. It does not say the score can find anything. This unit asks the
next small question: when a key has been moved to another frame, does the score still pick it out,
and do the raw-coordinate controls (L1 and popcount) fail as they should?

## What this unit does

An exact-integer test probe. No GPU, no training, no float.

- Reuse the exact `Z[φ]` arithmetic and the 120 icosian poses from the codex-2 probe.
- Build a key set of K features. Plant one target key whose frame was changed by a group element.
- Score every key against the query with four scorers: the normalised spin-1 score, the bare pair
  score (`spin-raw`), L1 on raw coordinates, and popcount on raw coordinates.
- Run a null control (no frame change) and a planted case, at K = 8 and K = 32, 200 trials each.
- Print the table of correct, tied and wrong picks, and the spin margin.

## Files in scope

- `crates/uor-r4-integer/tests/sh_spin_retrieval_probe.rs` (new file, already on the branch)
- `docs/loop/tasks/end-to-end-integration/claude-396/**`
- one row in `docs/research/notebook/m4-serving.md`

Out of scope: any source change, the l = 2 and 3 tables, the token-to-coefficient map, serving
wiring, a dependency, a GPU run, a mechanism status.

## Done when

1. The probe's gates pass: `cargo test --locked -p uor-r4-integer --test sh_spin_retrieval_probe`.
2. The REPORT records the printed table and the classification.

## Cancel

Stop if the null control fails for the normalised score, or if the unit needs a source change.
