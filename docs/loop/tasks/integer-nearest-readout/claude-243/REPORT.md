# REPORT: integer-nearest-readout, claude-243 (end-to-end parity, the raw export against the float reference with the tie rule on): MISS

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md), approved after two fresh rounds and logged on #3462 (6558047540). Sheet: [integer-nearest-readout-tie-parity-2026-11-01](../../../../harness/prep/integer-nearest-readout-tie-parity-2026-11-01.md), FIT (6558156900), pre-run GO in two rounds (6558226950). References #3462.

## Tokens per second (done-when 4)

R s1 (`a7r-s1.uorlut`, the raw export with the nearest readout), `fbs-toks ARTIFACT`'s plain protocol: 1 warmup and 20 timed runs of a 128-token prefill and 64 decode steps. Measured on this M1 on AC power, at load averages of 2.6 to 3.0 from other work.

| | Median | p95 |
|---|---|---|
| Decode | **1,204 tok/s** | 1,142 tok/s |
| Prefill (context) | 1,264 tok/s | 1,097 tok/s |

- **Peak resident memory:** 190,676,992 bytes (`/usr/bin/time -l`).
- **The nearest scan is counted.** A `--stages` run (its plain decode 1,257 tok/s) puts the writer at a 511,937 ns median of a 786,916 ns step (65%). That equals a whole-table nearest search's 511,292 ns over the 4,096 rows. The stages sum to 998‰ of the step, and the replies match.
- **Not compared under equal load** with claude-231's 1,365 tok/s on the snapped artifact.

## What did not work, first

- **The bar misses on p-w at every seed.** R's first-token disagreement with F′ is 64, 47 and 78 of 1,202, against a bar of 36 (⌊0.03 n⌋). That is 1.8×, 1.3× and 2.2× the bar. The row is **MISS**, and the item stays `ready`.
- **The margin is not measured.** The 3% bar was kept, not re-derived (the card). The raw export's rounding residual against the trained forward has not been measured, so part of this MISS may be that margin. Any cell over the bar is a MISS however narrow, and this one is not narrow at s1 and s3.
- **The gap is not localized.** An unregistered diagnostic found the disagreements diffuse, with no over-concentration like S's at p-w s3:
  - R's most common token among them covers 8 of 64, 5 of 47 and 15 of 78 disagreements;
  - no reply is empty;
  - R's first-token mix over all 1,202 prompts follows F′'s (token 318 opens 223 against 219 replies at s1, 216 against 217 at s2, 220 against 219 at s3).
- **`uor-pod down` could not archive the pod to the laptop** (`uor-r4/pods: Is a directory`, then no SSH). The results were already in the store, with 136 files listed. `--confirm-archived` then deleted the pod. Logged in FRICTION.md; no tooling was changed.

## What worked

- **DEV meets the bar at every seed:** 4, 4 and 5 of 310 first tokens disagree, against 9.
- **The raw export closes most of claude-232's gap.** With both sides on the same rules, R against F′ agrees on far more first tokens than claude-232's snapped S did:

| First tokens agreeing | DEV s1 / s2 / s3 (of 310) | p-w s1 / s2 / s3 (of 1,202) |
|---|---|---|
| **R against F′ (the bar)** | **306 / 306 / 305** | **1,138 / 1,155 / 1,124** |
| S against F′ (the snap's control) | 291 / 285 / 284 | 909 / 777 / 64 |
| R against F | 306 / 306 / 304 | 1,138 / 1,149 / 1,122 |
| F′ against F | 310 / 310 / 309 | 1,190 / 1,190 / 1,191 |

- **So the snap was the larger difference behind claude-232's MISS.** The other side's change is smaller.
  - F′ against F, the float code's change on G's path since `eb762e602`, moves 0 to 1 of DEV's 310 first tokens and 11 to 12 of p-w's 1,202.
  - It moves more of the whole replies: F′ and F agree on 298 to 300 of 310 and 1,071 to 1,130 of 1,202.
  - By the code that change is the tie rule alone. `train-in-space` has no switch to test it, so the sheet and the card do not credit the reading to the tie rule.
- **Whole-reply agreement:**
  - R against F′: 188, 247 and 201 on DEV; 488, 751 and 547 on p-w.
  - S against F′: 83, 163 and 129 on DEV; 59, 113 and 36 on p-w.
- **GPU greedy is repeatable at this head.** I-Repeat's second seed-3 process wrote both G files equal byte for byte. Of seed 3's 25 files, 22 are equal. The three keys files differ only because they carry wall-clock fields.
- **Every instrument held:**
  - I-Float-Inputs, I-Inputs, I-Flags, I-Raw, I-Identity and I-Prompts, on the laptop before the pod;
  - the pod's checks:
    - the CUDA library tests at the head, the tie rule's four by name;
    - every input against its sums;
    - every expected file written;
    - the sums holding the eight G files;
  - I-Repeat;
  - I-Store: 113 of 113 sums on the laptop, 136 files in the store, and the eight G files read back from it equal;
  - I-Prompts-F′: 6 of 6 cells equal at every index;
  - I-Decode: every decode exited 0 with `keyed` 0;
  - R's capped counts, 26 of 310 and 136 of 1,202 per seed, the same as S's;
  - the probes:
    - S against F gives claude-232's six grade files exactly;
    - F′ against itself reads n in every cell (F′ has no empty reply);
    - I-Positions: n in every pair.

## What changed

- [DECISION.md](DECISION.md): the card. The bar is read on R against F′; S and F are kept as readings; "done" applies to the raw export.
- [ORDER.md](ORDER.md), and the prep sheet with its FIT (§9) and pre-run review (§10).
- The item: done-when 3 amended under the card; this unit's row and speed recorded; it stays `ready`.
- One row in [m4-serving](../../../../research/notebook/m4-serving.md).
- `docs/roadmap/README.md`, re-rendered. The render also moves `integer-hamming-read` to done: its item was already `status: done` on main, and main's render was stale.

## Measurements

- **The pod:** one RTX 5090 (Vast `vast-58660117`, $0.563/h). Up 12:55Z to 16:33Z, about $2.04. The four processes took 12,312 to 12,364 s side by side.
- **R's export:** `fbs-export --raw-gates --raw-transition --run-config in/results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/sN/r-drop-sN/config.json in/results/claude/integer-retention-rates-parity-2026-10-31/out/full/snapshot-sN.json R/a7/a7r-sN.uorlut`, run from `local/claude/claude-243/`. It gave `f86a7856…`, `0a69a009…` and `66ae49de…`, 51,310,784 bytes each.
- **R's decodes:** `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64` on claude-232's prompt files. About 25 s per DEV cell and 104 s per p-w cell.
- **The sealed runs:** `results/claude/integer-nearest-readout-tie-parity-2026-11-01/`:
  - `tie-parity/`: the pod's outputs, logs and sums.
  - `laptop/`: R's artifacts, replies and logs, the grades, the speed files, `grade.log`, and the three scripts. It has its own SHA256SUMS, and 71 of 71 read back equal.
- **The scripts** (`job.sh` `a52295ee…`, `watch.sh` `de6a68f9…`, `grade.sh` `41bbeb35…`) are the ones the pre-run review recorded.

## Checks run at the head

- The sheet's §4 checks, each read from the run's files, as above.
- `loop gates` and `loop pr-check`, as recorded on the PR.

## Skills used

None.

## Still open

- **Locate the p-w gap on R against F′,** at the first disagreeing prompts, layer by layer (trunk, Hamming read, memory, readout). Do it under a sheet. Three leads:
  - **Measure the bar's margin first.** Run claude-227's trunk parity on the raw export against the trained forward, on p-w windows. It gives the rounding residual the bar was never re-derived against.
  - **Split the disagreements by exact-key status,** as claude-232 did on DEV s1 with claude-229's served decode. DEV passes and p-w does not. p-w is the held-out-phrasing panel, where the memory names fewer replies and the trunk decides more, so this split tests whether the gap sits there.
  - **The near-tie readout:** the diffuse disagreements suggest near-equal nearest distances. Count, at the disagreeing openings, how close F′'s and R's top two nearest rows are.
- **Making raw the default, or the snap faithful,** is a follow-up. S against F′ now measures what the snap costs.
- **Speed:** the nearest scan is 65% of a decode step, so a cheaper exact nearest is the speed lead.
- **Carried from earlier units:**
  - the read's selection by counting, then a re-time;
  - claude-240's code minors;
  - claude-241's Lows;
  - the long_sweep_prompt vocabulary check.

## Plain summary

Converting the model without rounding its gate and transition weights to the nearest of 120 fixed shapes fixed most of the mismatch the last check found. On the main check set, the converted model now starts its answer the way the trained model does on 305 or 306 of 310 prompts at each seed, inside the bar. On the panel of new phrasings it matches on 1,124 to 1,155 of 1,202, up from as few as 64. That is still short of the bar's 1,166.

The remaining mismatches are spread thin over many words, not stuck on one. The next step is to measure how much of that comes from the conversion's own rounding, and where in the model the rest begins.
