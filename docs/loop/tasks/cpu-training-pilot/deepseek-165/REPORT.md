# REPORT: cpu-training-pilot, deepseek-165

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7018b09cd`.

## Status

Prepared and authorized. The fresh seed-2762 panel has not been generated.

## Question

Can sign-Hamming agreement between the current Q12 writer query and each
token's shared vector-table row choose a better next token among the exact-copy
candidates than depth-first longest-32 selection?

## What did not work

Two independent FIT reviews blocked the first implementation states: the
distance test reused the implementation as its oracle, and the stateful
geometric replay plus random-32 behavior lacked direct coverage. Both were
repaired with finite tests before FIT and GO.

## What changed

- Added `GeometricCopy32` over the existing exact-copy candidate set.
- Added deterministic ranking by distance, deeper depth and lower token id.
- Bound the selected distance into the decision digest.
- Added eight-arm evaluation output, oracle naming and a geometric-change
  control against longest-32.
- Added independent sign-Hamming, distance-ordering, tie, replay and random-32
  control tests.
- Registered the bounded CPU development screen and recorded independent FIT
  and GO.

## Measurements

- implementation pin: `2378b0414c674923a2c62de44865260f3b778abc`;
- FIT reviewer: fresh-context Codex subagent `fit_geometric_review_3`;
- GO reviewer: fresh-context Codex subagent `go_geometric_review`;
- GO head:
  `8c941102764f61538e4aa4a9beb4669d87a53901`;
- fresh-panel result: **not run at registration**.

## Checks run at the head

- CPU library: **295 passed**, 4 expected ignores.
- All CPU-training binary test targets passed.
- Focused copy-reply tests: **19 passed**.
- Independent sign-Hamming oracle: **1 passed**.
- Scoped Clippy with `-D warnings`: passed.
- D23/D25/D32 ratchets: no growth.
- rustfmt and `git diff --check`: passed.
- Fresh output root and seed 2762: absent before compute.

## What remains

Execute the registered fresh panel and both artifact seeds with 32 Rayon
workers, verify deterministic repeats and all fail-closed controls, publish
compact evidence, classify the result, and complete exact-head review and
protected delivery.

## Size

The initial registered implementation and tests are within the ORDER scope.
Final evidence and documentation size is recorded after the run.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

All measurement and delivery work is open. This report makes no language,
complete-reply or serving claim.

## Plain summary

The geometric copy selector is implemented, independently reviewed for test
fitness, and authorized to run a fresh CPU comparison. No fresh result exists
yet.
