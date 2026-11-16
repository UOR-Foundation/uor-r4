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

## The zeta phase code: what more zeros buy, and what they do not

Owner, 16 Nov: "the problem with the prime pool is we are only using 2k … we have 50 million zeta
zeros we can use in the repo." The repository holds **512** zeros
(`crates/uor-r4-core/src/zeta_zeros.rs`, 48.8 KB, Odlyzko); the `riemann-lean` JSONs are fetch
receipts (30 zeros each) naming the 100k and 2M tables. 50M would be a fetch (LMFDB/Odlyzko) and a
pinned table of about 400 MB at f64, 200 MB at u32, or 100 MB at 16 bits a zero.

The probe measures the two things a wider phase code changes, on `k` primes, the code being
`θ_j(p) = γ_j·ln p` quantized to a byte per channel:

| channels | distinct codes (k = 2,000) | phase(p+2) finds p (k = 2,000) | distinct (k = 8,000) | locality (k = 8,000) |
|---|---|---|---|---|
| 1 | 256 (0.128) | 0.098 | 256 (0.032) | 0.026 |
| 2 | 1,589 (0.795) | 0.511 | 3,445 (0.431) | 0.307 |
| 4 | 1,845 (0.923) | 0.649 | 5,149 (0.644) | 0.473 |
| 8 | 1,974 (0.987) | 0.721 | 6,789 (0.849) | 0.620 |
| 16 | **2,000 (1.000)** | 0.753 | 7,707 (0.963) | 0.720 |

**What the zeros buy: resolution, exactly as the owner says.** One byte resolves `1/256` of a turn,
a factor `e^{2π/256} ≈ 1.0247` in `p`, so one channel is a 256-bucket logarithm. Sixteen
incommensurate channels fingerprint `ln p` finely enough that **every one of 2,000 primes has a
distinct code**, and 8,000 primes reach 96 %; more channels extend that without bound. Fifty
million channels make capacity a non-issue.

**What they do not buy: proximity of meaning.** The code is a *smooth function of `ln p`*, so the
nearest code is the numerically nearest prime (`phase(p+2)` finds `p` on 72–75 % of rows). Near
codes mean near **magnitudes**, not related meanings: two words with the same meaning and unrelated
primes get unrelated codes, and no number of zeros changes which word lands where. The missing piece
is the **assignment rule** — word → prime/code — and the archive's quota/uniqueness passes make
codes *unique*, not *meaningful*.

**The gcd correction.** The pool was not too small: `2k` primes for `k` entities is exactly enough
**if each prime belongs to one entity**. The probe drew two primes per entity *with replacement*
from a pool of `2k`, so primes were reused — 1.0 other entity sharing a factor at pool `2k`, 3.8 at
`k/2` — and that is why the exact rate was 0.62 and 0.23. Assign, do not draw, and gcd is exact at
any pool size. Zeros are irrelevant to that axis.

**Ordered next:** the assignment experiment. Give each word a code by a *content* feature (or learn
the map from the token table into the phase space), then measure paraphrase retrieval with the two
falsifiers the literature demands: the **value swap** (permute values; unchanged accuracy means
nothing reads the pairing) and the **oracle-key headroom** (gold code versus derived code).

## The routed-zero read (owner's design), and its two falsifiers

Owner: "use the prime addressing and then do least energy routing on the zeros in Riemannian for
super cheap operations".

The zeros are fetched and pinned: `data/zeta_zeros_2m.receipt.json` records Odlyzko's `zeros6.gz`
(**2,001,052 zeros**, gzip sha `6acacb47…`, last zero 1,132,490.66) and the probe loads the first J
from `UOR_ZETA_ZEROS` (falling back to the 16 embedded). The zeros are the *channel identities*; the
cheap arithmetic is the homomorphism `θ(∏p) = Σθ(p)`, so a prime-product address is a sum of phases.

The arm: each **class** owns a sparse set of 8 channels among the first 4,096 zeros; the store holds
four surface forms per class (the class's channels plus a per-form phase signature); a query is a
**new** surface form (same channels, new phases). The read is the least energy over the union — a
shared channel costs its circular phase distance, a channel on one side only costs a fixed penalty —
one subtract and one compare per channel, no multiplication.

| Arm | Accuracy |
|---|---|
| routed (least energy) | **1.000** |
| permuted control (the class→channel assignment rotated; occupancy identical) | **0.000** |
| **value-swapped** (only the stored value rotated) | **0.000** |

The control and the value swap both collapse, so the read is following the channels and the stored
pairing, not the surface. (The first value-swap run scored 1.000 and was a **faulty falsifier**: it
rotated the values on both sides. Only the stored side is rotated now, which is what makes it a
test.)

**The honest limit:** the routing here is *planted* — the query shares the class's channels by
construction. The mechanism (route + least energy) works; the open piece is the **content routing**:
which channels a class gets from the content. That is the assignment rule, and it is the next unit.

## The 1M/2M decision

The receipt is pinned and the loader reads the fetched gzip; 4,096 channels were enough for the
arm above, and the table supports up to 2M. What more channels buy is resolution (measured above:
16 channels already separate 2,000 primes), so the count is not the constraint — the assignment is.
