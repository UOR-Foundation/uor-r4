# REPORT: one-model-end-to-end, codex-29 (the planted ring probe: the ring routes, gcd needs unique primes)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parents: [codex-23](../codex-23/REPORT.md), [the zeta re-evaluation](../../../../labs/one-model-end-to-end/ZETA-RING-REEVALUATION-2026-11-16.md).

## What was built

`docs/labs/one-model-end-to-end/instrument/src/bin/phase-ring-probe.rs` — the fair re-test the zeta
record specifies. K slots on a 256×256 torus, each an address and a value byte; a query steps a
stored address by δ on one channel, so it is never seen verbatim; the read returns the nearest
stored address by **circular distance**.

The control is the one that cannot be explained by injectivity: **every address through a fixed
random bijection**, so occupancy and the value distribution are identical and only ring adjacency is
destroyed. The paired difference is the reading.

## Measurements

| K | δ | ring | bijection | ring − bij |
|---|---|---|---|---|
| 500 | 1 | **0.988** | 0.042 | **+0.946** |
| 500 | 2 | 0.934 | 0.052 | +0.882 |
| 2,000 | 1 | **0.923** | 0.013 | **+0.910** |
| 2,000 | 2 | 0.769 | 0.020 | +0.750 |
| 8,000 | 1 | **0.725** | 0.004 | **+0.720** |
| 8,000 | 2 | 0.346 | 0.006 | +0.341 |

The gcd arm on the same slots (entity = two primes, query = one of them × a fresh prime, rank by
gcd):

| K | prime pool | exact | other entities sharing a factor (mean) |
|---|---|---|---|
| 500 | 1,000 | 0.622 | 1.0 |
| 500 | 250 | 0.232 | 3.8 |
| 2,000 | 4,000 | 0.633 | 1.0 |
| 2,000 | 1,000 | 0.251 | 4.0 |

## What it means

- **The ring routes.** At one ring step it is 0.72–0.95 above an occupancy-matched control that has
  identical injectivity. The archive's "ring does not work" was never this test: the two negatives
  in the record compared arms retaining 1/456 against 1/16 of the space. The owner is right that the
  archive's negative does not settle the ring.
- **The ring's failure mode is density**, exactly as an address scheme should fail: at δ = 2 with
  8,000 slots in 65,536 cells, the window holds enough competitors to fall to 0.35. The design
  consequence is a capacity bound per channel, not a defect.
- **gcd is fast and exact only when a prime belongs to one entity.** Drawing two primes per entity
  from a pool of 2K already leaves a mean of 1.0 other entities sharing a factor, and the exact rate
  is 0.62; with a pool of K/2 it is 0.25 with 4.0 others. "gcd is very fast" holds; "gcd is exact"
  needs a unique assignment, which a pool draw does not give.
- **What this does not test:** whether zeta phases put *related tokens* near each other. The probe
  plants the relation as a ring step; the phases are not in it. That is the next test (the C2 ring's
  placement), and it needs a semantic relation planted across phases, not across ring steps.

## Checks

`cargo run --release --offline --bin phase-ring-probe` (the instrument crate, no GPU). Deterministic
by seeded LCG; the table above is one run and repeats.
