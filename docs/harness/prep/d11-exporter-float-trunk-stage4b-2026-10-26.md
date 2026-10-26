# Prep sheet: the D11 export gap of a trained SpiralCore/FBS trunk (d11-exporter-float-trunk Stage 4b, claude-126), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-126](../../loop/tasks/d11-exporter-float-trunk/claude-126/ORDER.md).
- Run: `spiralcore-fbs-sheet --mode d11-export-gap`, then `fbs-export` and `fbs-compare`, all inside one GPU job on one leased RTX 5090. Only `fbs-toks` (serving the artifact and timing it) runs on the laptop.

## 1. Question

Stage 3 trained the SpiralCore/FBS trunk (claude-119, #2891; [lab record](../../labs/spiralcore-fbs-offline-float-sheet/README.md)). Its "integer-export" score snapped only the Hamilton gates and the writer, so it was not the artifact. Stage 4a (claude-125, #3264) built a D11 mode that computes what the exported artifact computes, and proved its parity on a random-init trunk: the gates and the transition snapped to the 120 icosians, the tables rounded as exported, and the query at Q12.

**Does a trained trunk keep its DEV score when it is computed as the artifact, inside CHARTER §3.7's two-seed noise band?** And how fast does the artifact serve on a laptop, for decode and for context?

## 2. Arms

All four are trained exactly as in Stage 3, on the same inputs and settings (`spiralcore_fbs_sheet::run_fbs_arm`: 1,024 `StackAdamW` steps, base rate 0.002, decay multiplier 20, weight decay 0.1), on seeds 1 and 2:
- `Fbs_1Atom_Active` is the **exported arm**. Only one-atom routes convert (`fbs_export::decode_coded_routes`), and Stage 3's capacity gate kept one atom.
- `Fbs_2Atom_Mix`, `Fbs_IdentityTransport_Control` and `Fbs_FrozenQueries_Null` are trained only for the noise band. Their definition is Stage 3's.

For the exported arm on each seed, at step 1,024:
- **F (float):** `evaluate_dev_panel` with export off. These are the trained model's plain predictions.
- **D (D11):**
  1. the model is saved (`StackModel::save`) and loaded again;
  2. its tables are rounded as exported (`round_tables_as_exported`). An embedding value outside Q12 in `i16` (±8) is refused, never clamped, and the run stops as an instrument fault;
  3. it is scored with `evaluate_dev_panel_d11`. The D loss is the same `MarginHinge(rivals = 8)` with the trunk in D11 mode and each scored hidden vector rounded to Q12 before the head.
- **The parity sample:** every eighth `ft/dev` window (`PARITY_WINDOW_STRIDE = 8`), fixed here before the run: about 54 windows and 10,368 positions. The plain, export and D11 readouts read these windows whole, besides the 4,096 single tokens.

## 3. Bar and decision table

Definitions:
- **b** counts the `ft/dev` rows that F hits top-1 and D misses; **c** counts the reverse. Both are over the 14,287 eligible targets.
- **F_I** = max(24, ⌈2√(b + c)⌉), the paired top-1 threshold of the Stage 3 sheet.
- **Δ_noise,L** = maxₐ |L_f32(a, 1) − L_f32(a, 2)| over the four arms, the Stage 3 sheet's two-seed DEV loss band.

| Gate | Bar | PASS unlocks | FAIL decides (instruments green) |
|---|---|---|---|
| `G-D11-ExportGap` | On **both** seeds: \|b − c\| ≤ F_I **and** \|L_D − L_F\| ≤ Δ_noise,L, with every instrument check of section 4 green, `I-D11-Artifact-Agreement` included | This trunk's D11 export is inside the band. With the audits and the tok/s, the item's done-when is met, and #2820 closes. | Direct post-training rounding into the artifact is disqualified for this trained configuration (CHARTER §3.7). The next configuration trains in the D11 mode (a root-snapped forward) before export. A negative closes this configuration, never the mechanism. |

**The expected outcome is FAIL.** Stage 3's build A already missed its top-1 export bar by 184 and 481 rows, in a mode that rounded less than the artifact: it snapped only the gates and the writer. A FAIL is still decisive, because it is the first gap measured in the artifact's own computation. It moves this trunk to training in D11 mode. The receipt's `gap_inside_band_on_both_seeds` is only half of the gate, and must not be read as the verdict before `I-D11-Artifact-Agreement` is green.

**Readings, with no bar:**
- the Stage 3 export-mode gap on the same checkpoint (`evaluate_dev_panel`, export on), so the extra rounding the artifact adds shows;
- top-1 and admitted hits for F and D;
- the tok/s figures.

## 4. Instrument checks (each must be green before the gap counts)

1. **`I-Save-Load`:** the loaded model's plain panel predictions equal the trained model's on all 14,287 rows, and its plain panel loss is the same number.
2. **`I-D-Determinism`:** two D11 panel passes on the same rounded model give identical predictions. Training is not reproducible run to run (Stage 3 item 3), but the forward is; the gap compares F and D on one saved checkpoint.
3. **`I-DEV-Denominator`:** the D11 panel scores 14,287 of 14,287 rows.
4. **`I-Q12-Oracle`:** the D11 panel's pure-`i32` writer oracle has 0 mismatches.
5. **`I-D11-Artifact-Agreement`:** this ties D to the artifact itself. It runs inside the GPU job on the pod, never on the laptop: CHARTER §3.5 makes parity reference execution GPU-job compute. It is the item's "bit-exact parity proof" path (D68). Per seed:
   - **m1:** the scored rows of the sampled windows where the panel's D11 prediction differs from the D11 readout's next token at the same position. The panel batches windows and the readout reads one, so this checks the two GPU paths agree.
   - **m2:** the positions where the D11 readout's token differs from the exported artifact's integer session (`fbs-export`, then `fbs-compare`). These are counted over the 4,096 single tokens and every position of the sampled windows, about 14,464 positions, close to the panel's 14,287 rows.
   - **The bar:** m1 + m2 ≤ 6 on each seed. That is F_I's floor of 24 divided by 4, at about the panel's size, so the instrument's disagreement stays well below what the gap bar resolves.
   - Each disagreeing position is traced with `fbs-compare`'s tie margins. Snaps of an all-zero vector are counted over the sample; that is the known edge where the GPU and the integer path pick different roots.
   - **A miss is an instrument fault:** the gap is not counted, and the D11 mode is repaired before any verdict.
6. **`I-Finite`:** finite losses and outputs, with no CUDA error. The D11 entries refuse an unrounded trunk by construction (host-tested).

## 5. Base artifact and data split

- **Base artifact:** none is loaded. Every arm trains from step 0 at the pushed head, so the checkpoints are this run's own. The saved exported-arm models go to the store.
- **Data:** all inputs come from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/` and are checked with `sha256sum -c`. They are Stage 3's:

  | Input | Split | Contents | SHA-256 |
  |---|---|---|---|
  | `tokens.u16` | train, `mw-balp/train` | 7,512,797 tokens | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
  | `response_mask.u8` | train, `mw-balp/train` | | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
  | `tokens.u16` | DEV, `ft/dev` | 42,856 payload tokens, 14,287 eligible targets, 400 documents, 428 windows | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
  | `response_mask.u8` | DEV, `ft/dev` | | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
  | `tokenizer.json` | vocabulary 4,096 | | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

  The sealed `fresh_v2` draw is not used.
- **Pins:** the kernel sources at the pushed head are hashed into the receipt. The binary run is copied with its sha256.

## 6. Steps and minimum test power

1. **Probe a**, about 2 minutes:
   - `--mode synthetic-probe`, which covers `I-Finite-CUDA`, export exactness and an 8-step descent;
   - the `fbs_export::` GPU tests, which re-run Stage 4a's D11 parity on a random-init trunk at the head.
2. **Probe b**, about 2 minutes: `--mode throughput-probe` projects the wall time. If it is projected past about an hour, fix the code before running.
3. **The run:** `--mode d11-export-gap`. It writes a receipt, each seed's saved model, snapshot and readouts, and both prediction sets. In the same job, the serving build on the pod exports each seed's artifact, and `fbs-compare` writes `I-D11-Artifact-Agreement`'s comparison.
4. **Archive** to `results/claude/d11-fbs-export-gap-2026-10-26/` with a `SHA256SUMS`, then release and tear down the pod. Reading the bars off the stored receipt and comparison is analysis of stored outputs.
5. **On the laptop:** only product serving (CHARTER §3.6).
   - Both zero-multiply audits on the serving builds.
   - `fbs-toks` on each exported artifact: 1 warmup and 20 timed runs, a 128-token prefill (context) and a 64-token greedy decode; medians, p95 and the hardware profile.

**Minimum test power:** 2 seeds; 14,287 frozen DEV targets; trained from step 0 (1,024 steps, 786,432 tokens per arm-seed).

## 7. Cost and stop rule

- **Cost:** one RTX 5090 at about $0.90–1.90/h, inside the caps of 4 pods and $8/h.
  - Stage 3 took about 35 s of training per arm-seed, plus its evaluations, so the 8 arm-seeds take about 10 minutes.
  - The D11 passes and readouts take a few minutes per seed. `fbs-compare` on the pod takes a few minutes per seed. The probes and builds take about 15 minutes.
  - The estimate is about $0.50, with a ceiling of $2.00 (`--hours 1.5`).
- **Stop rule:**
  - Stop and repair if any of these fails: `I-Finite`, `I-Save-Load`, `I-D-Determinism`, `I-DEV-Denominator`, `I-Q12-Oracle` or `I-D11-Artifact-Agreement`. These are instrument faults and never count as a gap.
  - A probe-a failure stops before the run.
  - Never cancel a healthy run for a self-set time estimate; extend the lease.

## 8. Non-author TEST FITNESS review

TEST FITNESS: PENDING

**Round 1 (fresh non-author reviewer, 2026-10-26): NOT_FIT.**

The required changes, each made above:
- `I-D11-Artifact-Agreement` ties D to the artifact on DEV text, at a tolerance below what the bar resolves (section 4, item 5). It runs on the pod: CHARTER §3.5 rules out laptop parity reference execution, and the reviewer had proposed the laptop.
- An agreement miss is an instrument fault, and the receipt's gap field is labelled half of the gate.
- Zero-vector snaps are counted on the DEV sample.

Also adopted from the reviewer's suggestions:
- `I-Save-Load` also compares the plain loss.
- The embedding's Q12 range is refused, never clamped.
- The sheet says FAIL is the expected outcome.

TEST FITNESS: PENDING
