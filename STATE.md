<!-- rendered by uor-harness loop render from 149e5f422f805c82622597bc06f7fd37eacb8f80; do not edit -->
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
| done | 71 |
| held | 11 |
| ready | 12 |
| **total** | 95 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| data-tokenizer | no rows yet | | | |
| float-training | 2026-10-25 | 19 of 19 CPU tests pass (`5` lib contract tests, `2` dependency/D29 entry-point guards, `12` finite control tests covering all 24 kernels); 0 `uor-r4-training` or `candle-nn` edges in `cargo tree`; 0 D23 sites and 0 D25 sites in `crates/uor-r4-float-train/`; stub-`nvcc` `--features cuda --all-targets` exits 0 | Standalone crate builds CPU-only with CPU tests, passes the banned-token scan under its own scanned entry (`D37_SCANNED_FLOAT_CRATES`), carries the 24-kernel extract with 0 dependency edges on `crates/uor-r4-training`, and passes the stub-`nvcc` `--features cuda` type check (`#2823`) | positive (finite CPU controls and static module boundary only; GPU qualification and W4 training are separate downstream items) |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
| m1-discrete-attention | 2026-10-26 | Serving throughput (`d11-scorer toks`, 1 warmup, 20 runs, 192 prefill / 64 decode): Apple M1 laptop **`11,026` / `10,573 tok/s`** decode median (`2,596,805` / `2,764,260 tok/s` prefill median); RTX 5090 host CPU **`11,017` / `10,591 tok/s`** decode median; RTX 5090 GPU `d11-scorer score` **`183,206` / `181,086`** response `tok/s` (**`34,625,934` / `34,225,309`** context `tok/s`, `77,652`–`77,983` / `78,896 us`). Exported 19-table `UORLUT01` (`uor-r4.lut-discrete-trunk/1`, byte-identical across `Build A` and `Build B`): Seed 1 `04652a3d…` (`86,536,132` B) and Seed 2 `1b49e8bd…` (`84,943,328` B), accepted by `crates/uor-r4-integer::IntegerDiscreteModel::from_bytes` (`Bar (a) PASS`), with `14,287 / 14,287` exact agreement (`0` export gap) and `B4` pass (`1,260` / `1,262` cell flips, `656` / `682` route/gate flips; `Bar (b) PASS`). On `ft/dev` (`14,287` rows, `Unigram = 1,226`, `T2 = 6,300`, target `>= 6,324`): **`7,002` / `7,001`** `top-1` (`49.01%` / `49.00%`, **`+702` / `+701` over `T2`**, `top-8 = 7,485` / `7,478`, `admitted = 9,012` / `9,010`) and **`1,779` / `1,770`** on `P_succ` (`/ 3,847`, **`+534` / `+525` over `T2_psucc = 1,245`**, `Bar (d) PASS`). On sealed `Fresh V2` (`0b7e754c…` / `a1cda529…`, `14,479` rows, `3,594` `P_succ` rows; `Unigram = 1,279`, `T2 = 6,582`, `T2_psucc = 1,240`): **`7,215` / `7,228`** `top-1` (`49.83%` / `49.92%`, **`+5,936` / `+5,949` over Unigram** vs floor `156`; **`+633` / `+646` over `T2`** vs floor `82`; `top-8 = 7,688` / `7,696`) and **`1,713` / `1,711`** on `P_succ` (`47.66%` / `47.61%`, **`+473` / `+471` over `T2_psucc`** vs floor `52`, `0` mismatches, `B4` pass; `Bar (e) PASS`) | All 5 M1 closing bars (a)–(e) pass simultaneously across `>= 2` seeds (`d11_dev_top1 >= 6,324` on `ft/dev`, `psucc_d11_top1 >= 1,265`, and `PASS_BAR_E_FRESH_TRANSFER` on sealed `Fresh V2`) | positive (**`PASS_M1_ALL_FIVE_BARS`**, `m1_close_pass = true`, `m1_milestone_status = "CLOSED_PASS"`; closes `#2837` and satisfies `#2841` and Milestone M1 `#2029`) |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | 2026-10-26 | `TEST FITNESS: NOT_FIT`: Stage T/trunk required arguments and binary staging absent, trunk Step 0 precedes its checkpoint, anchor build/Stage T grade absent, no M2 student, no candidate-specific GPU reply evaluator or D11/native-bank bridge. Bar 1 is an M1 composite, not a faithful M2/Amendment bar; best antecedent top-8 `4,674 < 4,775`. M2 student cost/power `UNDEFINED`. Superseded recipe matches the #3300 base-blob sections with zero diff. | A FIT must describe an executable end-to-end configuration; no compute authorized and no mechanism/bar/hold/source change. | **negative** (`INSTRUMENT_NOT_FIT`; #2851 remains a pins-delivery close only; #2853 must provide student/evaluator/bridge under a new ORDER; not a model or mechanism negative) |
| m3-durable-memory | 2026-10-26 | True address **80/80** (40 current, 20 previous, 20 typed abstention), no-memory/no-address **0/80**, sieve **20/80**. True-vs-no-memory `40/0` current, `20/0` previous, `20/0` abstention; true-vs-sieve `20/0` current and `20/0` abstention. D11 parity exact on 14,287/14,287 rows; random control held. Final pod cost **$0.135**. | Exact 40/40 current, 20/20 previous, 20/20 typed abstention (10 Absent + 10 Evicted), zero substitutions; composed parity/leak/identity/baseline/random gate. | positive at the M1 exact-read/oracle scope; closes #2833, not learned address formation, default serving or M3 qualification |
| m4-serving | 2026-10-27 | Training: **4,470.49 / 4,470.82 tokens/s**, 351.83 / 351.81 s per seed. At step 0 both seeds match claude-131 exactly. At step 1,024 plain/export-mode top-1 changed from 3,845/3,511 to 1,472/1,048 (seed 1) and 3,128/3,023 to 3,258/3,209 (seed 2). No artifact, A/D gap or serving measurement was produced. | I-Prefix requires exact top-1/top-8/admitted counts and losses within 1e-6 against claude-131 at step 1,024. The gate exited 1; no later registered adjudication was made. Step-2,048 metrics exist in `train.json` but were not scored against the export-gap bar. | blocked (instrument): training reproducibility across pods is unresolved; this is not a model or export-gap verdict and #2820 stays open. A separately FIT'd diagnostic must distinguish same-device repeatability from cross-device/source-version drift before changing the gate. |
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
| deepseek | DeepSeek | DeepSeek harness | 3 |
| kimi | Moonshot AI | Kimi Code | 3 |
