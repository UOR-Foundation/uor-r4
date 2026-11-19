# REPORT: written-state-count-model, claude-438 (P1 gate (c): the KN start and a bound on the fit's distance)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet:
[kn-ceiling-kn-start-2026-11-19](../../../../harness/prep/kn-ceiling-kn-start-2026-11-19.md) (FIT at round 1).
Code: every run at `6c66a76d5` on `claude/kn-start-438` (code as at `d366c829f`; later commits change
documents only). Release: Casey, 19 Nov, "Yes, rent it", for one RTX 5090.

## What did not work, first

- **Gate (c) is Unsettled** [claude]. At 100% of TRAIN the converged ceiling lies in
  [L, U] = [1,284,092, 1,431,925] micro-BPB, and that band holds 0.90 × R\* = 1,289,086: L misses
  it by 4,994 micro-BPB. The sheet's reading map fixed this case before the run: the 96 passes do
  not decide between "Between" and "Geometry critical", and more passes or another start need a
  new sheet. What the band does settle is in What the numbers say.
- **The KN start barely left KN** [claude]. After 96 passes at 100%, U_count is 393 micro-BPB
  (8,609 bits) below KN-only and still falling 9.5 bits a pass. KN holds 997‰ of the
  target-weighted weight (claude-433's uniform start ended at 600‰ after 24 passes). On average
  7.1 of the 12 other experts sit at exactly weight 1 of 2^32, and about one other than KN rises
  above 2^20. At every cut size the KN start ended above claude-433's 24 passes from uniform
  (U_count 2,293,695 against 2,283,303 at 1‰, 2,138,432 against 2,132,625 at 1%, 1,691,107
  against 1,690,355 at 10%).
- **The ORDER's reason for the start was wrong for this integer EM** [claude]. It said every
  present expert starts above 0, so EM can still move weight onto it, and that the start cannot
  change EM's fixed point. Reading the M step (`kn_ceiling.rs`, `fit_em_from` and
  `update_bucket`): each target's responsibility `(w·p << 32) / m` is truncated to an integer, and
  the new weight is `R_e · 2^32 / ΣR`, floored. For a weight of 1, each target's term is about
  p/m, truncated, so the weight stays at 1 unless the expert's ratios are mostly at least 2, and
  a weight at 1 is lifted back to 1 if it falls to 0. An expert whose mean ratio g lies in (1, 2)
  is held at 1, while the bound still counts what it could save. **This is a hypothesis drawn from
  the code and the weight census above; no test isolates it yet** (Still open). Real-valued EM
  would have no such trap.
- **The first pod job stopped on a bug in my job script** [claude]. In `b1`, the loop
  `for c in all novel` overwrote the global `c` that held the ceiling run's name, so `bars` then
  read `novel.json` and the job exited 3 after B1 at 1%. Every check it reached held: step 1
  39/39, step 2 B2 and B3 true at 1‰, B1 at 1%. The loop variable was renamed (`part`) and the
  output folder moved to `run2`; nothing else changed. Under the stop rule every step re-ran from
  step 1 in the second job. The first job's 1‰ and 1% JSONs are those sizes' records, and the
  second job's are identical to them (whole JSON, timings aside).
- **The memory sampler's 1‰ device peak is not reliable.** It read 796 MiB in the first job and
  8,052 MiB in the second for the same 12 s run: one sample a second misses short allocations.

## What changed

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: `EmStart::{Uniform, Kn}` and `fit_em_from` (the KN
  start: 2^32 − (L − 1) on KN and 1 on each other present expert, a mask without KN refused);
  `gap_bound_q12`, the bound on how many bits the best real-valued bucketed weights can still save
  over given weights on the fit set: per bucket N_b · log2(max_e g_e), rounded up one Q12 unit per
  target, plus the integer floors' slack of 2 + ⌈8192 / mixed mass⌉ Q12 units per target.
  `fit_em` and `fit_em_with` keep the uniform start. New host tests: the start's weights, the
  start's bits against KN-only's, and the bound against what EM saves from both starts.
- `docs/labs/kn-reference/instrument`: every ceiling mode fits from the KN start with 96 passes;
  the JSON gains `em_start`, each fit's `gap_bound_q12` (per fold for U_split) and, for U_count,
  `lower_bpb_micro`; `unit` reads `claude-438`.
- The prep sheet [kn-ceiling-kn-start-2026-11-19](../../../../harness/prep/kn-ceiling-kn-start-2026-11-19.md).
- Run packets: [`run1/c438-job1/`](run1/c438-job1/) (the stopped job) and
  [`run1/c438-job2/`](run1/c438-job2/) (every step). No stream, mask, id list or count table is
  committed.

## Measurements

N = 10, budget 2^24, DEV-tune as query and, for U_count, fit set. Micro-BPB, overall and novel. All
on one Vast RTX 5090 (`vast-61370413`, 96 CPUs, 503 GiB), one build, stream sha256 values checked
on the pod against claude-433's.

| TRAIN | KN-only | U_count (U) | L | U_split | KN + U |
|---|---|---|---|---|---|
| 1‰ | 2,305,596 / 2,445,080 | 2,293,695 / 2,433,675 | 2,116,747 | 2,321,475 / 2,458,528 | 2,304,617 / 2,444,479 |
| 1% | 2,145,471 / 2,404,087 | 2,138,432 / 2,401,227 | 1,878,820 | 2,147,941 / 2,406,914 | 2,144,110 / 2,403,792 |
| 10% | 1,691,806 / 1,986,542 | 1,691,107 / 1,985,858 | 1,488,912 | 1,691,566 / 1,986,250 | 1,691,589 / 1,986,365 |
| 100% | **1,432,318 / 1,747,150** | **1,431,925** / 1,746,667 | **1,284,092** | 1,432,030 / 1,746,774 | 1,432,196 / 1,747,012 |

Parts per million of KN-only on the same cut (overall / novel; L overall only). At 100% KN-only is
R\*, so that row is also the ratio to R\*. Ratios to R\* at the cut sizes are not comparable.

| TRAIN | U_count | L | U_split | KN + U |
|---|---|---|---|---|
| 1‰ | 994,838 / 995,335 | 918,091 | 1,006,887 / 1,005,500 | 999,575 / 999,754 |
| 1% | 996,719 / 998,810 | 875,714 | 1,001,151 / 1,001,175 | 999,365 / 999,877 |
| 10% | 999,586 / 999,655 | 880,072 | 999,858 / 999,853 | 999,871 / 999,910 |
| 100% | 999,725 / 999,723 | 896,513 | 999,798 / 999,784 | 999,914 / 999,921 |

- **Gate (c) inputs at 100%:** U = 1,431,925, L = 1,284,092; 0.85 × R\* = 1,217,470 and
  0.90 × R\* = 1,289,086. L / R\* = 0.896513 and U / R\* = 0.999726; novel U_count / R\*'s novel
  = 0.999724 (no bound on the novel ratio).
- **The 100% fit:** fit bits 128,649,429,056 Q12 at the start (KN-only 128,649,439,118),
  128,627,655,182 after pass 24 and 128,614,178,256 after pass 96 (equal to the scored
  `bits_q12`); pass 96 saved 39,065 Q12 (9.5 bits). The bound is 13,278,202,590 Q12
  (3,241,749 bits), 377 times what the 96 passes saved.
- **Bounds of the other fits at 100% (Q12, recorded, no bar):** U_split's even and odd fit halves
  7,916,061,244 and 7,339,621,169, each bounding its own fit, not the cross-scored score; KN + U
  3,352,888,438.
- **B1:** KN-only equals the plain run's `all` and `novel` at 1% and 10%, and claude-422's
  1,432,318 / 1,747,150 at 100%; at 1‰ the host check passed (device lookups and host-checked KN
  masses).
- **B2:** true at every size. Start and last fit bits are at most KN-only's bits plus the target
  count (at 100% the start is 10,062 Q12 below KN-only), and the scored bits equal the last fit
  bits.
- **B3:** true at every size (L ≤ U, bound at least four units per target).
- **B4 (rises between passes, Q12):** U_count 8 rises at 1‰ (largest 4), 2 at 1% (23), 1 at 10%
  (20), none at 100%. KN + U 5, 8, 17 and 23 rises (largest 15, 52, 51, 316). U_split's folds:
  15 and 10 at 1‰ (21, 4), 25 and 0 at 1% (23), 0 and 3 at 10% (29), none at 100%. At most
  316 Q12 (0.08 bits), the size of the integer floors.
- **Weights at 100% (target-weighted, ‰; KN, ML_0..ML_10, uniform):** 997, 0, 0, 1, 0, 0, 0, 0,
  0, 0, 0, 0, 0. 154 buckets populated.
- **Cost:**

| TRAIN | wall | raw count | lookup | EM (four fits) | peak RSS | peak device (whole card) |
|---|---|---|---|---|---|---|
| 1‰ (host check) | 12 s | — | — | — | 8.2 GB | see above |
| 1% | 29 s | 7.1 s | 0.26 s | 5.3 s | 8.2 GB | 7,908 MiB |
| 10% | 175 s | 50.7 s | 0.42 s | 52.2 s | 8.2 GB | 8,696 MiB |
| 100% | 1,354 s | 352.5 s | 2.2 s | 546.6 s | 10.3 GB | 16,028 MiB |

  The pod ran 2,923 s (03:44:43Z to 04:33:26Z) at $0.908/h, about $0.74, of which the stream copy
  was about 5 minutes and the stopped first job 9 minutes.

## What the numbers say

- **Counts carry is excluded.** L = 1,284,092 is above 0.85 × R\* by 66,622 micro-BPB. No bucketed
  linear mix of these 13 experts, fit on DEV-tune itself, can reach 0.85 × R\*; the best one is at
  least 0.8965 × R\*. This rests on the bound alone (Jensen at the fitted weights plus the floor
  slack, pinned by `the_gap_bound_covers_what_em_still_saves` and its negative), not on the fit
  having converged.
- **The two branches left are Between and Geometry critical.** Under Between the roadmap's order
  is unchanged; under Geometry critical P2 moves ahead of 435/436. In both, P1(a), (b) and (d) are
  still built and claude-434 (Pick) comes next. So gate (c) no longer holds up R2; it decides only
  where P2 stands against 435/436, and that is settled by a fit that gets closer than 96 passes
  from this start did.
- **The fit is the loose side, not the bound.** U moved 393 micro-BPB in 96 passes and was still
  moving, and the census shows most experts held at weight 1. If the floor trap is the cause, a
  sheet that removes it (responsibilities kept at higher resolution, or a start with real weight
  on every expert) should shrink the band sharply: on the fixtures, 400 passes left a bound of
  3 to 6 Q12 units per target, where this run's is about 1,229 per target.
- **Recorded, not read as the gate:** U_count's gain over KN-only at 100% is 0.027% overall and
  0.028% on novel targets; U_split stays above U_count at every size, by 0.0073% of KN-only at
  100%, far below the "large split gap" threshold (half of R\* − U).

## Checks run at the head

- **G1, laptop** (at `d366c829f`): `kn_ceiling` host tests 12/12, including
  `the_kn_start_puts_the_rest_on_kn`, `the_kn_start_scores_kn_up_to_floors` and
  `the_gap_bound_covers_what_em_still_saves` (both masks, both starts, passes 0, 1, 3, 10 and 40
  against 400); the instrument's tests 11/11, including
  `the_lower_bound_subtracts_the_gap_and_floors`; stub-nvcc `cargo check`s of the crate and the
  instrument clean; D32 grep of the added lines clean; `kn_ceiling.rs` formatter-clean, the
  instrument's `main.rs` 2 rustfmt hunks as on main.
- **G2, negatives** (set aside against the base, run, restored): the bound taking a w-weighted
  mean of g instead of its maximum fails the bound test (868,534 saved against a bound of 768);
  the KN start leaving the other present experts at 0 fails two start tests.
- **G3, one Vast RTX 5090** (`vast-61370413`, jobs `20261119T035338Z-claude-438-gpu0` and
  `20261119T040330Z-claude-438-gpu0`, at `6c66a76d5`): step 1 39 passed, 0 failed, 4 ignored
  (the real-stream tests), every `kn_lookup_cuda`, `kn_score_cuda`, `ngram_count_cuda`,
  `kn_filter` and `kn_ceiling` test included, in both jobs; steps 2 to 4 rc 0; B1, B2 and B3 true
  at every size; the second job's 1‰ and 1% JSONs equal the first's.
- **Prep sheet FIT** at round 1 (fresh-context non-author), no blocker; its should-fix items are
  folded into the sheet.

## Skills used

None beyond the loop's own steps. The unit measures a bound with count experts; it changes no
model state, operator or artifact.

## Still open

- **Gate (c)'s last step (R1c, a new sheet):** remove the floor trap and read the band again at
  100%. First a host test that isolates it (an expert at weight 1 with mean ratio in (1, 2) never
  grows under `update_bucket`), then the M step kept at higher resolution (for example the
  responsibilities in Q64), and a start with real weight on every expert. Everything else stays
  claude-433's. The pass count is fixed on that sheet before any run.
- **R2 (claude-434, Pick) is not held by gate (c)** and can go ahead.
- **The device peak sampler** should poll faster than once a second, or read the CUDA driver's
  own high-water mark, before device peaks are compared at small sizes.
- Carried from claude-433: the device lookup's 32-bit row index, the raw-order log line's `ms`
  label, and the two REPORT nits.

## Plain summary

This unit asked again how much better than our reference model (R\*) the best blend of simple
counting models can do on the tuning set. It started the blend at R\* itself and computed a proven
floor under the best possible blend. The floor answers part of the question: no such blend gets
within reach of the "counts carry the project" line (85% of R\*); the best one is at least 89.65%
of R\*. Whether it reaches the 90% line, which decides whether the geometric work jumps the queue,
is not settled, because the blend barely moved away from R\* in 96 rounds. Reading the code
suggests why: with whole-number weights, a model holding the smallest possible share cannot grow
unless it is at least twice as useful as the blend, so most of them never moved. The next sheet
fixes that and reads the 90% line. The rented GPU cost about $0.74.
