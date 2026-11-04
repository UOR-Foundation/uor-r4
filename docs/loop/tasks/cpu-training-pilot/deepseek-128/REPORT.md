# REPORT: consolidate CPU count verification, deepseek-128

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[verification consolidation](../../../harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md).
Base: `e5b7c500c`.

## What did not work, first

- **The first pre-registration review returned NOT FIT.** Its finite test
  compared a table set with itself, did not exercise a real D11 import, and the
  recipe did not hash generated artifact files directly. The replacement test
  round-trips source, exact, changed-semantics and wrong-table D11 artifacts;
  every run now hashes both generated files against the accepted oracles.
- **The second pre-registration review returned NOT FIT.** The source gate
  covered only `crates/uor-r4-cpu-train` even though the implementation pin also
  included `crates/uor-r4-discrete`. The gate now binds all `crates/**` and both
  root Cargo manifests.
- **The optimization's peak-memory risk was real but bounded.** Exact
  verification briefly holds source expected tables and the decoded exact table
  set together. The final eight-run peak was 1,544,884 KiB, 552,268 KiB below the
  registered stop. Both complete table sets are dropped before the shuffled
  arm imports its independent source tables.
- **No model-math or capability question was tested.** The exact artifacts,
  tables, decisions, checksums and normalized reports all had to reproduce
  deepseek-125/126; only verification implementation cost changed.

## What changed

The previous verification path parsed the full 19-table trunk ten times, twice
for each semantic comparison, independent expected-table reconstruction,
decoded-table comparison and table summary. The new path performs four imports:

1. source semantics plus tables;
2. exact exported artifact;
3. a fresh source import for the independent shuffled expectation; and
4. shuffled exported artifact.

The first source table set is moved directly into exact expected-count
reconstruction rather than cloned. Each exported arm is decoded once for
semantics, full decoded-table equality and summaries. The shuffled expected
tables still come from a separate full source import. The 21-stage profile,
source-order parallel row work, update/export behavior and model arithmetic are
unchanged.

The finite regression test now constructs actual exported D11 source, exact,
changed-semantics and wrong-table artifacts. It checks successful import,
semantics, full decoded tables and summaries through the production helpers,
then requires both semantic and table mismatches to be refused.

## Measurements

The fixed split held 200 training documents / 7,080 targets and 193 evaluation
documents / 512 targets. Eight fresh roots ran the registered mirrored order.
The first registered panel also passed under the prior binary, but a package
format check then normalized three test-only expressions and changed the linked
binary hash. A fresh FIT and this aligned attempt-2 panel replaced it; the
first attempt remains diagnostic evidence and was not reused for the numbers
below.

The final panel pinned source
`fcb7c8c5327800cb372529926ada62e6ba5cfb55`, baseline binary
`e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
and optimized binary
`8517f0c1aeb69a9a0da3137e561233e7d5d83f1f3180b5233b42ec60bb2087df`.

| Pass | Binary | Workers | Wall | Peak RSS | Verification |
|---|---|---:|---:|---:|---:|
| forward | baseline | 1 | 12.314 s | 1,542,344 KiB | 5,227.987 ms |
| forward | optimized | 1 | 9.739 s | 1,542,528 KiB | 3,005.727 ms |
| forward | baseline | 16 | 11.299 s | 1,543,856 KiB | 5,607.159 ms |
| forward | optimized | 16 | 9.050 s | 1,544,884 KiB | 2,562.550 ms |
| reverse | optimized | 16 | 8.770 s | 1,544,692 KiB | 2,799.789 ms |
| reverse | baseline | 16 | 11.803 s | 1,543,868 KiB | 5,389.042 ms |
| reverse | optimized | 1 | 9.400 s | 1,542,468 KiB | 3,133.832 ms |
| reverse | baseline | 1 | 10.717 s | 1,541,952 KiB | 4,600.360 ms |

### Registered bars

| Workers | Baseline wall | Optimized wall | Ratio | Reduction | Verification ratio | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 11,515.375 ms | 9,569.549 ms | **0.8310** | **16.90%** | **0.6247** | **37.53%** |
| 16 | 11,550.905 ms | 8,910.020 ms | **0.7714** | **22.86%** | **0.4877** | **51.23%** |

Both wall means clear the 90% ceiling and both verification means clear the
80% ceiling. Wall speedups were **1.203x / 1.296x**; exact plus shuffled
verification speedups were **1.601x / 2.051x**. At one worker,
`exact_verify` fell from 2,926.114 to 1,694.338 ms and `shuffled_verify` from
1,988.059 to 1,375.442 ms. At sixteen workers they fell from
3,185.693/2,312.407 ms to 1,269.445/1,411.724 ms.

All eight runs produced exact artifact
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and shuffled artifact
`19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.
Every normalized report was
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
No-update / exact / shuffled remained **332/246/270**, **331/245/269** and
**332/246/270** admitted / top-1 / top-8. Exact changed four rows and shuffled
changed zero. Every arm retained zero query-coordinate, admitted-cell, top-8
and token mismatches over 512 rows / 16,384 Q12 coordinates.

Every run recorded 21 finite stages with accounting equal to measured wall
within 0.001 ms, completed below 600 seconds, and stayed strictly below the
2-GiB RSS stop. The eight roots occupy **1,390,924,382 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds both binaries, reports and every
artifact copy.

## Checks run at the head

- Four fresh-context non-author TEST FITNESS reviews: the first two returned
  NOT FIT; the repaired source returned FIT; and the formatting-only source
  revision returned FIT at docs head
  `befb6888b9042fac7fdfe1804ed6dcbb1e082127`.
- Full `uor-r4-cpu-train` release suite: **237 tests passed, 0 failed, 4
  release benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Package formatting for `uor-r4-cpu-train` and `uor-r4-discrete`: passed.
- Focused consolidated D11 success/mismatch, ordered-fold, stage fail-closed
  and RSS tests: passed.
- All eight attempt-2 runs passed artifact, table, decision, normalized-report,
  21-stage accounting, timeout, worker, source, binary and strict-RSS gates.
- Prep check, shell syntax, JSON parsing, evidence checksum reconstruction and
  `git diff --check`: passed.
- Full loop gates: **11/11 green**. Report check and local policy check passed;
  the policy warnings are advisory D76 scope/commit-order and pre-existing
  FILEMAP rows.
- Roadmap check: **139 items**; documentation catalog: **1,175 records**, no
  missing paths; state formats: **8/8 ok**; D23/D25/D32 ratchets unchanged.
- `uor-harness check`: **0 errors, 136 warnings**, chiefly pre-existing stale
  claims, presence and merged branches outside this unit.

## Skills used

No system-provided skill applied in the implementation session. Each fresh
reviewer read and applied the repository's `uor-research-architect` skill.

## Still open

- Verification is no longer the dominant measured group. Further CPU
  performance work should start from the optimized profile rather than assume
  the earlier deepseek-127 hotspot still dominates.
- Complete replies, fresh phrasing, useful local chat and GPU qualification
  remain open.

## Plain summary

The count driver now verifies the same two exported artifacts with 60% fewer
full-trunk imports. It is 20.3%/19.0% faster end to end and 50.1%/49.6% faster
in verification at one/sixteen workers, while every artifact, table, decision,
checksum and normalized report stayed byte-identical. This is a verified CPU
implementation speedup, not a language-capability result.
