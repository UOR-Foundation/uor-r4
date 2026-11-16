# REPORT: one-model-end-to-end, codex-22 (the hinge's trainer works, and it cannot reach C2′)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-21](../codex-21/REPORT.md).

## What did not work, first

- **The registered 64-leaf mask cannot address a served window.** The competing sets here sit at
  positions up to 192; a trainer that refused them would have refused every row. The host carries
  the competing positions as a list, and the device's four-word mask is the fixed-width form of
  the same labels.
- **A strict-descent rule on the rank hinge stalls.** The rank is discrete: it moves only when the
  order flips, and a step that narrows the distance without flipping it leaves `H_bind` unchanged,
  so nothing was ever accepted. The step now descends the **pairwise distance gap** (the pick
  step's own loss) and keeps the rank cost as the tie-break and the reported objective.
- **The first rank count included the query row as its own candidate**, shifting every rank by one
  and making every fixture look violated.
- **A two-place margin is unreachable with two candidates.** The unit test said "the hinge closes"
  and the fixture could not: with one bound and one competing row the best possible gap is one
  place. The test now asserts what the step promises — the bound becomes the first pick, the miss
  leaves, the one-place term survives — and the zero-cost case is its own test.
- **The reverse arm does not converge.** Its accepted steps keep accumulating (1,514 by step 2,048
  at `ext-s1`) while the dump-level hinge *rises*: the accept is per batch, as in the pick trainer,
  and a direction the geometry does not offer cannot be bought by more steps.

## What changed

- `crates/uor-r4-native-vector/src/bind_train.rs` (new): the hinge on the served read's own order.
  `BindLabels` (a bound position and a competing list a row), `BindRow`/`BindReport` (the cost, the
  margin, the miss, both ranks, and the descent gap), `Q31FirstPickRead::bind_hinge`, and
  `Q31FirstPickRead::train_bind_step` — the pick step's sign-residual credit between the bound and
  the best competing row, accepted while `(gap, rank cost)` strictly falls.
- `crates/uor-r4-native-vector/src/read_train.rs`: `addresses`, `add_credit` and `apply_credit` are
  crate-visible so the hinge shares the exact credit math instead of re-deriving it.
- `crates/uor-r4-discrete/examples/native_bind_train.rs` (new): reads a `Q31ReadDump` with its
  `.ask/.other` sidecars, trains, and writes one JSON line an eval step with the hinge, the two
  pick counts and an optional `--eval-dump`'s held reading.
- `docs/labs/read-binding-on-lawful-read/README.md`: the correction points at the implementation.

Eight finite tests: the one-place margin, the competing row ahead, no competing label, a bound
outside the window, unlabelled rows costing nothing, the label refusals, the step moving the bound
to the front, and the shape refusals. **47 passed** in native-vector (39 before).

## Measurements

Held split: windows 0–127 train, 128–255 measured; identity start, shift 8, 2,048 steps, and every
run stops improving at its accept limit. 512 held rows (256 a bucket).

| State | Arm (bound / competing) | Identity start | Trained, train half | **Held** | Held misses |
|---|---|---|---|---|---|
| ext-s1 | asked / other | 4/512 | 442/512 | **416/512 (0.813)** | 508 → 96 |
| ext-s1 | other / asked (C2′) | 1/512 | 6/512 | **3/512 (0.006)** | 509 |
| ext-s2 | asked / other | 35/512 | 431/512 | **391/512 (0.764)** | 484 → 121 |
| ext-s2 | other / asked (C2′) | 2/512 | 0/512 | **0/512 (0.000)** | 512 |

Same-window arm (`ext-s1`, all 256 windows fitted and scored): the state's own read carries a
hinge of 4,215 with 584 misses and 440 bound-first; the trainer takes it to **522** with 111
misses and **913/1,024 (0.892)** bound-first, from 22,991 and 9/1,024 at identity.

## What it means

- **The hinge is a working trainer.** It descends the registered cost with the registered
  constants, on real served streams, and the held rows move with the train rows (0.813 / 0.764
  against 0.888 / 0.872 for the plain pick target's held rows at `codex-21`). The gap to the pick
  target is the two-place margin and the miss this cost charges that the pick target does not.
- **The hinge does not reach the other fact either.** 0.006 and 0.000 held, below the plain pick
  target's 0.125: the stronger supervision does not widen the family. The C2′ ceiling is the gate
  family and the trunk's geometry, not the loss, which is what `codex-18` read from the pick target
  and what `codex-21` saw through the served artifact.
- The relation `/15` read remains the mechanism that carries C2′: exact on the identifiable rows,
  and it never depends on the read's stream.

## Still open

- **The device twin.** `native_q31_read_pick_loss` computes a distance rival; the rank hinge needs
  the bound's and the competitor's ranks, so the kernel takes two sidecar words a row. This unit is
  the host reference, as `train_pick_step` was before its kernel.
- The archived lab's 120-item memory panel is not in this repository; the held split above is the
  probe's own window family, which is what the served artifact carries.
- The 1B read is claude's line.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` — 47 passed.
- `cargo build --release --locked -p uor-r4-discrete --example native_bind_train` — clean.
- Four held-split runs (`ext-s1`/`ext-s2` × both arms) and the all-window arm; the held-split
  `ext-s1` forward run repeats byte-for-byte after the eval-dump refactor.
