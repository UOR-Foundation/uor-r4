<!-- rendered by uor-harness loop render from 640367b302af5a0aecd09d78fc66c9c1c23437c6; do not edit -->
# STATE

Rendered from committed inputs at the commit in the stamp above.

## Milestone bars and holds

Source: `docs/harness/milestones.json`.

| Milestone | Status | Headline bar | Training hold |
|---|---|---|---|
| M1 Language base: a geometric stack language model good enough to carry chat | complete | M1 closing bars: 6 of target 6 (out of 6 closing bars) | released |
| M2 Grounded reply from exact memory: the native learner completes correct replies | in-progress | complete correct replies on one accepted saved model (dev): 0 of target 256 (out of 512 replies) (direction: open) | released |
| M3 Durable conversation memory: provenance, conflict, abstention, reset and restart | in-progress | typed outcomes qualified on the saved model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M4 One served model: the chosen model through D11 integer serving and a CLI | not-started | accepted model served through D11 and CLI: 0 of target 1 (out of 1 qualified milestone) | released |
| M5 Laptop cost (D5): full-path M1 cost and selected parameter access | not-started | full-path cost and selected parameter access qualified: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M6 Reasoning and coding on the same native model | not-started | unseen compositions and changed constraints on one model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M7 Distribution: capability API, WASM runtime and the Pages Studio | in-progress | native model served through the API, WASM and Studio: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M8 Alpha release: the same model for conversation, memory and coding | not-started | one labelled release on the same native path: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |

## Roadmap items by status

Source: `docs/roadmap/items/*.md` frontmatter.

| Status | Items |
|---|---|
| decision | 1 |
| done | 101 |
| held | 11 |
| ready | 23 |
| **total** | 136 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| cpu-training | 2026-11-03 | **6/6 exact-export receipts; 5 processes exited 0 and seed-1 K4 exited 2 at the required admission gate:** 262,144 Q12 query coordinates and 8,192 kept cells per arm had zero mismatches. Seed 1 baseline 278/384/396; K1 278/383/395; K4 278/383/395. Seed 2 baseline and both trained arms were 224/285/292. K4 accepted 437/238 blocks, all four-coordinate; total wall 621.118 s and peak RSS 53,992 KiB. | pre-registered two-seed same-amplitude comparison: K4 admission preserved, at least one four-coordinate block, zero export mismatch, top-1 gain over baseline at least 43/40 rows per seed, top-8 non-falling and top-1 strictly above K1. | **negative (bounded coordinate-breadth configuration):** four coordinates reduced the margin objective but did not improve ranking; seed 1 also lost admission and top-8. This closes the tested same-amplitude K1/K4 configuration, not BVG-K or CPU training generally. |
| data-tokenizer | no rows yet | | | |
| float-training | 2026-11-03 | **Instrument PASS:** all four finite and served-evaluator loadable; both served checkpoints unchanged by export rounding, all plain files changed. **Operational FAIL:** served training 355 s vs 51/52 s plain = 6.96×/6.83×, ceiling 2×. **Quality INCONCLUSIVE:** top-1 served − plain = −25/−55; two-seed spread 30, so seed 1 is inside the max(24,30) band. Served decode 49.2–49.4/40.8–41.1/28.6–28.8 tok/s at contexts 16/64/192. | registered instrument, operational and quality doors | **negative (operational gate):** served-surrogate wiring is finite and export-consistent but 6.8–7.0× too slow for a longer run; the 49-pass activation-scale loop is an unmeasured cost hypothesis |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
| m1-discrete-attention | 2026-10-30 | Apple M1 laptop **`10,876` / `10,566 tok/s`** decode median (prefill 2.67M tok/s); top-1 **`7,850` / `7,872`** vs T2 `7,375` and unigram `1,409` (b/c vs T2 `1,038`/`563`, `1,051`/`554`); P_succ `1,903` / `1,904` of 4,077 vs T2 `1,260` (reported) | at each seed, beat unigram and T2 by `max(24, ⌈2√(b+c)⌉, \|M(1) − M(2)\|)` (81 vs T2, 162 vs unigram), 0 refused, 0 mismatches, B4 pass; builds A and B byte-identical; DEV top-1 reproduced first (`7,002` / `7,001`) | positive (**`PASS_BAR_F_HELDOUT_PHRASING`**; M1 close grade now needs (a)–(f): `PASS_M1_ALL_SIX_BARS`). Negative first: the first attempt did not compile without `--target` (E0463), an instrument fault before the panel was read |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | 2026-10-26 | `TEST FITNESS: NOT_FIT`: Stage T/trunk required arguments and binary staging absent, trunk Step 0 precedes its checkpoint, anchor build/Stage T grade absent, no M2 student, no candidate-specific GPU reply evaluator or D11/native-bank bridge. Bar 1 is an M1 composite, not a faithful M2/Amendment bar; best antecedent top-8 `4,674 < 4,775`. M2 student cost/power `UNDEFINED`. Superseded recipe matches the #3300 base-blob sections with zero diff. | A FIT must describe an executable end-to-end configuration; no compute authorized and no mechanism/bar/hold/source change. | **negative** (`INSTRUMENT_NOT_FIT`; #2851 remains a pins-delivery close only; #2853 must provide student/evaluator/bridge under a new ORDER; not a model or mechanism negative) |
| m3-durable-memory | 2026-11-02 | Both seeds emitted `Memory: Pickle.` and `Memory: tomatoes.`. `6/6` exits 0, empty stderr, `15/15` valid protocol-2 stops (10 EOS, 5 token-cap), exact normalized seed-1 repeat; **0/6** factual replies contained the supplied value. Peak RSS 11,888 KiB; 0.17–0.22 s per process. | FIT/PRE-RUN registered instrument: both expected memory lines on both seeds, deterministic repeat and valid processes; model quality reported separately with no bar. | mixed at serving scope: `AUTO_MEMORY_OBSERVED`, `MODEL_RECALL_NOT_OBSERVED`; not M3 qualification or useful chat |
| m4-serving | 2026-11-02 | **F against itself:** all 10,840 passes reach the nearest with equal picks, in 11.9 s for the 621 MB trace. **Table checks:** F's trace holds against F's table on 14,031,744 buckets, 584,656 codes and 10,840 nearests, and R's against R's on 6,716,544 buckets and 4,382 nearests. **G₁₀ on F's base:** 0.0055 to 0.0094 for the six bucket families, 0.031 to 0.032 for the copy heads' codes, and 0.015 to 0.016 for the type heads' keys against 0.17 to 0.36 for their queries. **The one-forced window 41:** steps 40 to 48 are equal. At step 49 the grader stops at bucket (layer 0, lane 0), entries 1 and 9, as an instrument fault: no score selects the forced 9 (I-Force). R's first check found the grader's own −0 fault, now fixed and pinned by a test. Review round 1 narrowed the "pick behind" refusal to R (a float side picks on GPU f32 scores, so its f64 order may reverse at a near tie); every reading re-ran equal at `768c39ff3` | none registered (instrument qualification, the unit's done-when 5) | **positive** (instrument qualification): the grader holds on real traces and refuses the one input that is not a replay. Q5 is not read. Done-when 3 still waits on the replay driver, the sheet and its GPU run |
| m6-reasoning-coding | no rows yet | | | |

## Mechanisms

Source: the ACTIVE / READY / PARKED tables in `docs/mechanisms/README.md`.

- ACTIVE: Address cells (A8)
- ACTIVE: Admission
- ACTIVE: D11 integer serving
- ACTIVE: Discrete geometric learning
- ACTIVE: Flock read
- ACTIVE: Hamilton gate (G2)
- ACTIVE: Prime-zeta codebook
- ACTIVE: Rank-margin hinge
- ACTIVE: Relation read
- ACTIVE: Routed read
- ACTIVE: SpiralCore/FBS routes
- ACTIVE: Tokenizer
- READY: Hopf observation: kept-fibre map
- READY: Object store (D19)
- READY: Recurrence gates
- READY: Spin-native weights
- READY: Ternary/XOR
- PARKED: Carrier credit
- PARKED: E8 orbit memory
- PARKED: Exact memory
- PARKED: Geometric stack
- PARKED: Hopf observation: sector routing
- PARKED: Legal construction
- PARKED: Native geometric engine
- PARKED: Pointer key
- PARKED: R4G1 graph
- PARKED: Read binding
- PARKED: VSA

## Lab registry

Source: `docs/harness/labs.json`.

| Lab | Vendor | Client | Open PR limit |
|---|---|---|---|
| antigravity | Google | Antigravity | 3 |
| claude | Anthropic | Claude Code | 3 |
| deepseek | DeepSeek | DeepSeek harness | 3 |
| kimi | Moonshot AI | Kimi Code | 3 |
