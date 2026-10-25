# Carrier credit by hard selection on the live discrete learner (`carrier-credit-hard-selection`, #2819, claude-103)

- **Status:** pre-registration, written before any arm of this schedule has run. A fresh FIT and a D67 pre-run review follow on #2819. Lab: claude.
- **Decision record:** the D68 card [`DECISION.md`](../../loop/tasks/carrier-credit-hard-selection/claude-103/DECISION.md), owner log #2819 `6395405669`.
- **Supersedes nothing.** It is a new sheet (§4.5). The archived sheet's objective is banned and its code is compiled out.

## 1. Question

At a matched full dose, does choosing which vector block the exact counterfactual move tries, by the objective's own carrier credit, beat two model-blind in-batch choices on fresh rows? Does a model-blind in-batch choice beat the interleaved sweep that #2831 runs?

## 2. Arms (8 = 4 schedules × seeds 1001 and 2001)

Every arm uses the same settings:
- `uor-r4.discrete-corpus-driver/1`, V 4,096, B 4, context 192, kept_cells 8;
- Relation read (kept 8, sweeps 1), `zero_gates_v1`, `learn_v1`;
- `nearest_address_margin_v1`, `rows_v1`, 64 rows per batch, coordinate block 1;
- seeded-device codebook at the arm's seed;
- 16,404 batches from cursor 0.

These are the A8 dose settings (#2831), except for the codebook start and the schedule. Reads land on `completed_batches % 8 == 7` in every arm, exactly as `interleaved_read_every8_v1` places them. The schedules differ only on the other 7 of every 8 batches:

| Schedule | Vector block tried |
|---|---|
| **OFF** `interleaved_read_every8_v1` | the fixed sweep `(n − ⌊n/8⌋) mod 16,384` |
| **ON** `carrier_credit_v1` | `token·4 + (n mod 4)`. The token is the argmax, ties to the lowest id, of per-token credit. A row with integer hinge `h` = token + address > 0 adds `h` to its target, and also to its strongest admitted rival when top-1 is wrong. Zero credit falls back to the lowest target id |
| **CONTROL-U** `carrier_uniform_target_v1` | `token·4 + (n mod 4)`, where the token is the k-th distinct batch target and `k = SplitMix64(seed≪32 ⊕ n) mod D` |
| **CONTROL-F** `carrier_uniform_row_v1` | `token·4 + (n mod 4)`, where the token is the target of row `SplitMix64(seed≪32 ⊕ n) mod 64` |

Here `n` is `completed_batches`. ON and both controls run the same CUDA evaluation and selection kernel (`discrete_carrier_select`, integer only) every batch. The host receives 2 words. Credit and draws read the training batch only.

## 3. Data

| Object | Pin |
|---|---|
| TRAIN | `data/ft-balp.tar` (HF store), sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`, member `mw-balp/train/`: tokens `3dee858f…a4c8` (7,512,797 u16), mask `b5e7352d…49e3`, 2,425,925 eligible targets |
| DEV | `data/ft-dev.tar`, tokens `0919e90f…555b`, mask `a838bb1a…b078`, 14,287 targets |
| Tokenizer | `d36d3e87…0f89` |
| FRESH | tokens `4d9ead83…ef61`, mask `557131bc…5e9b`, 65,536 targets (§4) |
| Base config | [`base-config.json`](../../labs/carrier-credit-hard-selection/base-config.json): the A8 dose config (#2831) byte for byte. `carrier-arms` sets each arm's schedule, seed and a seeded-device codebook (`carrier_arms::arm_config`) |

## 4. Fresh rows

- Batches walk TRAIN's eligible targets in order from cursor 0 (`corpus.rs`:254–279), and each window stays inside its document. A 16,404 × 64 dose reads eligible indices 0 to 1,049,855 only.
- **The FRESH slice** is the first 65,536 eligible TRAIN targets in documents that begin after the document holding index 1,049,855, less any target whose in-document window (at most 192 tokens) and target exactly match a visited row's.
- **How it is stored.** `carrier-fresh-slice` copies those whole documents into a separate, pinned `Split::Fresh` corpus, so each window equals its TRAIN window (fixture test, `carrier_fresh.rs`). The slice is scored once, after the dose, through `evaluate_transfer`.
- **The slice as cut and sealed, 25 Oct, before any arm** (HF `results/claude/carrier-credit-2026-10-25/fresh/`, with SHA256SUMS; manifest in [`docs/labs/carrier-credit-hard-selection/fresh-slice-manifest.json`](../../labs/carrier-credit-hard-selection/fresh-slice-manifest.json)):
  - the last visited TRAIN position is 3,251,164, and the first fresh document starts at 3,251,242;
  - 68,773 candidates were scanned, 3,237 dropped as exact repeats, and 65,536 kept;
  - the slice is 211,846 tokens;
  - tokens `4d9ead83fa05f959502c6d4b1a176cc34b7134b9adef41e13d1bf7f4ba9eef61`, mask `557131bca30514b6728e00e66bc945b43abbcbf6b7a96ed9ec3e817057915e9b`.
- The TRAIN binding stays byte-identical to #2831's. The M1 sealed fresh panel (`4bd108ad…`, `df5d074a…`) and frozen `ft/dev` stay unopened as FRESH.
- `run_arm` refuses any batch that would read past TRAIN index 1,049,855. The D67 release pins `visited_targets` = 16,404 × 64.

## 5. Bars and decision (card D/E, fixed here)

- **Metric:** integer top-1 hits on FRESH (N = 65,536). Top-8 hits and the token and address hinge sums are reported alongside. DEV is reported, and a DEV-only gain is labelled panel fit.
- **Win rule, per seed.** Arm A beats arm B when `a > b` and `(a − b)²·N > 4·h·(N − h)`, where `h = ⌊(a + b)/2⌋`. This is an exact integer two-sigma test.
  - **Credit:** a seed counts when ON beats both CONTROL-U and CONTROL-F.
  - **Selection:** a seed counts when CONTROL-F beats OFF.
  - Each comparison is a **win** (both seeds count), a **tie or loss** (neither does) or a **split** (one does).
- **Decisions:** as tabulated in the card.
  - A credit win: reopen conditions 1 and 5 are met, the mechanism goes READY, and ON becomes a candidate arm for #2831 and #2836.
  - A credit tie or loss: configuration negative (D22). A split: no change, and the next configuration adds seeds.
  - A selection win: CONTROL-F's rule becomes a candidate arm. A selection tie or loss or split: #2831 and #2836 are unchanged.
  - ON against OFF is reported beside any ON candidacy.
- **Count tables (§4.4):** the unigram table and the previous-token next-word table, both built from the visited TRAIN targets 0 to 1,049,855, are scored on FRESH and on DEV beside every arm.

## 6. Instrument checks (an instrument failure is repaired, never read as a verdict)

1. **The selector's finite CUDA controls** (8 cases, with literal expected results): the credit argmax, a tie going to the lowest id, the zero-credit fallback, a rival credited only when top-1 is wrong, an out-of-range rival skipped, CONTROL-U's k-th distinct target, CONTROL-F's row, and two identical launches.
2. **Default bytes:** the config bytes of both existing schedules, and their lineage bytes with a fixed driver-source string, are unchanged.
3. **The selection downloads only its 2 words** (source-scan test, shown to fail when broken).
4. **The full host suite and the stub-nvcc CUDA type-check pass,** `ratchet d23` and `d25` are at baseline, and the driver-source pin is updated.
5. **On the pod:** the discrete crate's own CUDA suite passes. Each arm's receipts show the selection on every non-read batch. A second launch of the first 64 batches of ON at seed 1001 gives identical receipts.

## 7. Cost, probes and stops

- **Probes:** (a) the controls and the CUDA suite; (b) 164 batches per schedule, to project wall time; (c) the eight arms, four at a time on each of two RTX 5090 pods.
- **Throughput:** 0.12 s per batch was measured on A8 (16,404 batches in 1,967 s). The selection adds one evaluation per batch, which probe (b) measures.
- **Budget:** at most 2 pods × 3 h at about $1/h.
- **Stops:**
  - A pin mismatch stops the run before launch.
  - A failed control stops it, and the fault is repaired.
  - If probe (b) projects more than 90 minutes per arm, the code is fixed before (c).
  - Outputs go to the HF store under `results/claude/carrier-credit-2026-10-25/`, with a SHA256SUMS, before any pod comes down.
- **Lead metric:** tokens per second through the dose, reported per arm.
