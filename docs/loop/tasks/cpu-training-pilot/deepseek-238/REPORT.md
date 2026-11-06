# REPORT: cpu-training-pilot, deepseek-238 (D84 centroid cache evaluator)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `2b08d79c4`.

## What did not work, first

- The first centroid evaluator ran in `uor-r4-float-train`, outside the named D84
  CPU module. This unit moves evaluation into `uor-r4-cpu-train`.
- Its first matched control rotated across all eligible observations, so a rare or
  singleton target could receive another target's query distribution. The replacement
  groups eligible target IDs by exact observation count, cycles each non-singleton group
  once, keeps singleton targets on their own centroid in both arms, and faults if the
  resulting shuffle or table is vacuous.
- Its power, pooled identity and pooled comparison existed only in prose. The new
  evaluator enforces fitted-row/observation floors, computes split and pooled
  `(position,target)` identities, and emits the pooled paired bar.
- The first evaluator loaded a 27.4M-row TRAIN query file into memory. The new fit streams
  the file once, updating the SHA-256 and both candidate/control sums in the same pass.
- The current main CPU crate failed to compile its arbitration binary because two enum
  arms were absent from one match. That baseline repair is isolated in its own commit.

## What changed

- Added `query_cache_eval` to `uor-r4-cpu-train`.
- Added `cpu-query-cache-evaluate`.
- Added `cpu-query-cache-aggregate` for the two-seed control/power/bar precedence.
- Verifies the cache seal, manifest, table, split rows and query hashes, and requires
  Q12 F32 query values on the exact `1/4096` grid.
- Converts queries and tables to exact i16 Q12 and evaluates whole-table L1 in ordered
  64-query Rayon blocks.
- Streams TRAIN into target and exact-observation-count-stratified control sums, then
  evaluates original, centroid and control tables on held-out rows. The report names the
  control strategy and records its strata and singleton-target counts.
- Reports per-panel and pooled counts, pooled identities, power, candidate/control paired
  tests, top-8 preservation and deterministic decision hashes. It also checks the
  registered TRAIN token and mask hashes and reports split contexts.
- Added focused tests for Q12 conversion/ties, an exact-count non-vacuous shuffle with a
  self-mapped singleton stratum, power, registered identity, tampering and worker-count
  determinism.

No GPU scorer, trainer, serving path or model artifact changed.

## Measurements

Nothing model-level was measured. The worker-count test compares one and four Rayon
threads after removing wall time and worker count; every semantic field is equal.

The new binary is the D84 evaluator for the separate n3S query-cache experiment. Its
hardware and full run resource ledger belong to that experiment sheet; no real cache was
available at this head.

## Checks run at the head

- `cargo test -p uor-r4-cpu-train --locked --offline --no-fail-fast`:
  **430 library tests passed, 4 ignored; all binary tests passed**.
- Focused `query_cache_eval` tests: **5 passed**.
- `cargo check -p uor-r4-cpu-train --bin cpu-query-cache-evaluate --locked --offline`:
  passed.
- Scoped strict Clippy, with only the four pre-existing crate lint categories allowed
  (`type_complexity`, `needless_borrow`, `cloned_ref_to_slice_refs`,
  `unnecessary_min_or_max`): passed.
- D23/D25/D32 ratchets: passed.
- New-file Rustfmt and `git diff --check`: passed.
- Full harness checks are run before delivery.

## Skills used

No system-provided Codex skill applied. D84 and repository task rules were used directly.

## Still open

The n3S GPU capture must use the merged D84 binary pin. The one-model sheet must be
updated from the float-crate evaluator to `cpu-query-cache-evaluate`, then receive a fresh
non-author FIT and D67 review. No real centroid panel has been opened.

## Plain summary

TRAIN has 27.4 million rows, so loading all captured queries into memory was too large and
the old matcher treated some labels differently from their control. This CPU module now
streams TRAIN, moves every eligible label in a count-preserving control, enforces the
registered power and row-identity gates, and reports the final pooled comparison in one
Rust binary. The GPU capture still has to run before any model result exists.
