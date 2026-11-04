# REPORT: public 19-table CPU count stage profile, deepseek-127

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[stage profile](../../../harness/prep/cpu-training-pilot-trunk-stages-2026-11-04.md).
Base: `3441ba625`.

## What did not work, first

- **This did not produce a faster training configuration.** It measured the
  existing configuration after deepseek-126's 1.5x parallel-speed bar failed.
  The new profile includes timing overhead and is not a speed comparison.
- **A single-stage optimization would address less than half the wall.**
  `exact_verify` was the stable largest stage, but exact and shuffled
  verification together consumed 47.6% / 49.7% of wall time at one / sixteen
  workers. All three held-out evaluations consumed another 31.5% / 31.5%.
- **No model or capability question was tested.** Artifact, table, decision and
  checksum identity with deepseek-125/126 was the required control, not a
  training result.

## What changed

`cpu-trunk-count-driver` now records 21 monotonic stages covering:

- input read/hash and validation;
- corpus split and reader load;
- training-row and control-row construction;
- exact and shuffled update/export/hash/write;
- exact and shuffled independent verification;
- all three held-out evaluations;
- parity, changed-decision and resource reductions.

The report includes each stage duration, attributed time and unattributed time
before final report serialization. Duplicate stage names and invalid accounting
fail closed. Model math, update order and artifact serialization are unchanged.

## Measurements

The fixed split again held 200 training documents / 7,080 targets and 193
evaluation documents / 512 targets. Four fresh runs covered `1,16` and the
mirrored `16,1` worker order.

| Workers | Pass | In-process wall | Peak RSS |
|---:|---|---:|---:|
| 1 | forward | 12.621 s | 1,540,880 KiB |
| 16 | forward | 11.793 s | 1,543,696 KiB |
| 16 | reverse | 11.483 s | 1,543,952 KiB |
| 1 | reverse | 12.626 s | 1,542,120 KiB |

Order-averaged walls were 12.624 s at one worker and 11.638 s at sixteen.

### Stable hotspot

| Stage | 1-worker mean | 1-worker share | 16-worker mean | 16-worker share |
|---|---:|---:|---:|---:|
| `exact_verify` | **3,629.871 ms** | **28.75%** | **3,460.038 ms** | **29.73%** |
| `shuffled_verify` | 2,382.383 ms | 18.87% | 2,327.910 ms | 20.00% |
| `shuffled_evaluation` | 1,513.467 ms | 11.99% | 1,407.129 ms | 12.09% |
| `exact_evaluation` | 1,294.897 ms | 10.26% | 1,162.702 ms | 9.99% |
| `no_update_evaluation` | 1,172.089 ms | 9.28% | 1,092.850 ms | 9.39% |

`exact_verify` is the largest stage at both worker counts, so the predeclared
hotspot rule selects it directly rather than a broader leading group.

The verification path currently calls full trunk import/decode work several
times per arm: semantic comparison, independent expected-count reconstruction,
decoded-table comparison and table reading each reconstruct the same data. The
next performance unit should consolidate those traversals while preserving the
independent full-table check. This is the measured next target; no optimization
was applied after seeing the profile.

All four runs produced exact artifact
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and shuffled artifact
`19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.
Every normalized report after deleting `stages`, wall time, RSS and replacing
threads was
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
Every arm retained zero query, admitted-cell, top-8 and token mismatches over
512 rows / 16,384 Q12 coordinates.

Peak RSS ranged from 1,540,880-1,543,952 KiB, below the strict 2-GiB stop.
Every run completed below the 600-second timeout. The four roots occupy
**695,462,223 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, reports and both artifact
sets.

## Checks run at the head

- Fresh-context non-author TEST FITNESS: FIT at docs head
  `db71dfdb79d5cda1ba41463b965f858bf8662bde`, after repairing one expected
  stage-key ordering error.
- Full `uor-r4-cpu-train` release suite: **236 tests passed, 0 failed, 4
  benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Focused stage-accounting, ordered-fold, row/artifact/parity and fail-closed
  RSS tests: passed.
- All four runs passed the pre-registered 21-stage, artifact, normalized-report,
  timeout and strict RSS gates.
- Prep shell syntax, evidence checksum reconstruction and `git diff --check`
  passed.

## Skills used

No system-provided skill applied.

## Still open

- Consolidate repeated full-trunk decoding in exact and shuffled verification,
  then measure whether it reduces wall time without weakening the independent
  table check.
- Complete replies, fresh phrasing, useful local chat and GPU qualification.

## Plain summary

The count driver's slower half is now measured rather than inferred. Checking
the two exported artifacts accounts for roughly half the wall time, with
`exact_verify` consistently the largest single stage. The next useful speed
work is to remove duplicate full-trunk parsing from verification, while keeping
the independent count reconstruction and exact CPU/serving controls.
