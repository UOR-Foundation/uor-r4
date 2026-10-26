<!-- rendered by uor-harness loop render from c695ad21a72c5b91def22458eebbe796e07d5392; do not edit -->
# STATE

Rendered from committed inputs at the commit in the stamp above.

## Milestone bars and holds

Source: `docs/harness/milestones.json`.

| Milestone | Status | Headline bar | Training hold |
|---|---|---|---|
| M1 Language base: a geometric stack language model good enough to carry chat | in-progress | reply acceptable: 37 of target 116 (out of 232 acceptable replies) (suspended) | released |
| M2 Grounded reply from exact memory: the native learner completes correct replies | blocked-on-owner | complete correct replies (dev): 437 of target 256 (out of 512 replies) (direction: open) | HELD |
| M3 Durable conversation memory: provenance, conflict, abstention, reset and restart | paused | typed outcomes qualified on the saved model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M4 One served model: the chosen model through D11 integer serving and a CLI | not-started | softmax at runtime: yes; target no (D23 prohibits softmax project-wide) | released |
| M5 Laptop cost (D5): full-path M1 cost and selected parameter access | not-started | full-path cost and selected parameter access qualified: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M6 Reasoning and coding on the same native model | not-started | unseen compositions and changed constraints on one model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M7 Distribution: capability API, WASM runtime and the Pages Studio | not-started | native model served through the API, WASM and Studio: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M8 Alpha release: the same model for conversation, memory and coding | not-started | one labelled release on the same native path: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |

## Roadmap items by status

Source: `docs/roadmap/items/*.md` frontmatter.

| Status | Items |
|---|---|
| decision | 1 |
| done | 56 |
| held | 15 |
| ready | 21 |
| **total** | 93 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| data-tokenizer | no rows yet | | | |
| float-training | 2026-10-25 | 19 of 19 CPU tests pass (`5` lib contract tests, `2` dependency/D29 entry-point guards, `12` finite control tests covering all 24 kernels); 0 `uor-r4-training` or `candle-nn` edges in `cargo tree`; 0 D23 sites and 0 D25 sites in `crates/uor-r4-float-train/`; stub-`nvcc` `--features cuda --all-targets` exits 0 | Standalone crate builds CPU-only with CPU tests, passes the banned-token scan under its own scanned entry (`D37_SCANNED_FLOAT_CRATES`), carries the 24-kernel extract with 0 dependency edges on `crates/uor-r4-training`, and passes the stub-`nvcc` `--features cuda` type check (`#2823`) | positive (finite CPU controls and static module boundary only; GPU qualification and W4 training are separate downstream items) |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
| m1-discrete-attention | 2026-10-25 | Step 0 `5/5 PASS` (`AKd` `32/32`, `V1d` `24/24`, `KId` `128/128`, `V8d` `128/128`, `V9d` bitwise resume `3545fe65…`; `57,156 context tok/s`). `T16_fz` (Seeds 1 & 2, `3,008` batches, `epoch = 420`, `854` moves, `123` joint commits, `50,910` / `50,860 context tok/s`): TRAIN anchor peak `1,823/3,847` (final `1,342`), DEV admitted **`5,009 / 14,287`** (`G-Adm` PASS, `+765` over baseline `4,244`, `+717` over `K=1` Q1 `4,292`; `P_succ` `1,216/3,847`), writer `top1 = 455`, `top8 = 1,484`, distinct read codes `19,385` TRAIN / `4,067` DEV, full-panel $T(c(x)) = 4,085$, checkpoint-restricted **`T_restr(A_W) = 2,140`** (`top8 = 4,006`) and **`G_restr(A_W) = 3,084`** (`top8 = 4,674`). `T16_rel` (Seeds 1 & 2, `4,096` batches to Q1, `epoch = 678`, `1,505` moves = `545` vector + `535` combine + `425` relation, `199` joint commits): TRAIN anchor peak `1,849/3,847` (final `1,286`), DEV admitted `3,827 / 14,287` (`P_succ` `1,144/3,847`), writer `top1 = 371` (`+238` over `K=1` `T1_rel` `133`; `relation_off` ablation `327` `top1`, `-44` rows), `top8 = 1,124`, distinct read codes `27,132` TRAIN / `4,429` DEV, `T_restr(A_W) = 1,602`, `G_restr(A_W) = 2,364`. `fresh_evaluations = 0` | Stage 2 bar (`#2836`): Step 0 `5/5 PASS`, `G-Adm` (`>= 4,220` full DEV & `>= 676` `P_succ`), and writer `top-1` beats unigram (`> 1,226`, `>= 1,344`) and W2 admission-matched tables (`> 1,138`, `> 1,482`, and `> T_restr(A_W)`) on two seeds | diagnostic stop (`DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS`; `classification: mechanism` configuration negative on the parameterless `OuterL1V1` writer under `K = 16` greedy 64-row block proposals; `BVG-K` expands DEV cell admission to `5,009 / 14,287` and lifts the checkpoint-restricted read-code table `T_restr(A_W)` to `2,140` and bigram `G_restr(A_W)` to `3,084`, isolating the remaining M1 gap to within-cell writer selection; `fresh_v2` remains sealed) |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | no rows yet | | | |
| m3-durable-memory | 2026-10-25 | **896,898 tokens/s** (732 turns, 4,280 words, 4.8 ms upload to download). Panel: false_assert **0/50**, false_overwrite **0/50**, clean **50/50**. R1 **462/462** exact. R2 **0/220** (every miss `Unresolved`, no wrong write). Finite CUDA controls 12/12. A second build's decisions are byte-identical (`d3f05ecf…`) | Owner bar (#2028 comment `6356034555`): 0/50, 0/50, 50/50, counted only together with R1 ≥ 95% (decision review on #2856). R2 has no bar | positive, at this scope only: the sieve compiler on world=v2. It is not a pass of `m3-memory-turn-discriminator-gate` |
| m4-serving | 2026-10-25 | **Decode:** 13,259 tok/s median, which is 4,826 µs per 64 tokens; p95 13,176 tok/s. **Prefill:** 2,967,175 tok/s median, which is 64 µs per 192-token prompt; p95 2,765,893 tok/s. All 21 decode runs wrote the same reply. | none registered (qualification check) | inconclusive (no bar registered; it qualifies the protocol and is not step 6's measurement) |
| m6-reasoning-coding | no rows yet | | | |

## Mechanisms

Source: the ACTIVE / READY / PARKED tables in `docs/mechanisms/README.md`.

- ACTIVE: Address cells (A8)
- ACTIVE: Admission
- ACTIVE: Discrete geometric learning
- ACTIVE: Flock read
- ACTIVE: Prime-zeta codebook
- ACTIVE: Rank-margin hinge
- ACTIVE: Relation read
- ACTIVE: Routed read
- ACTIVE: Tokenizer
- READY: D11 integer serving
- READY: Hamilton gate (G2)
- READY: Hopf observation: kept-fibre map
- READY: Object store (D19)
- READY: Recurrence gates
- READY: Spin-native weights
- READY: SpiralCore/FBS projection
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
| codex | OpenAI | Codex | 3 |
| deepseek | DeepSeek | DeepSeek harness | 3 |
| kimi | Moonshot AI | Kimi Code | 3 |
