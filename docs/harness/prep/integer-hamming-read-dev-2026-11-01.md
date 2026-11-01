# Prep sheet: the Hamming read's parity on claude-146's checkpoints, on DEV (integer-hamming-read, claude-240), 2026-11-01

- Item: [integer-hamming-read](../../roadmap/items/integer-hamming-read.md) (#3385), step 5, after claude-238 and claude-239. ORDER: [claude-240](../../loop/tasks/integer-hamming-read/claude-240/ORDER.md).
- **The run:** one GPU job on one leased RTX 5090, `local/claude/claude-240/job.sh`.
  - It is not in git. The job copies itself into the outputs, so the seal covers it.
  - It runs against the pushed head.
- **The laptop side:** the integer traces, the comparisons, the copy-code checks and the timing run on the laptop from the stored outputs, as serving does.

## 1. Question

claude-238 (#3478) measured the read's parity on (r-drop) step 16,384, on the fixed sequences, with one tie rule on both sides (row T). It met both bars at both seeds:
- d was 6 and 27 of 1,536;
- every barred cell was at 0.95 or above.

With the GPU sort's own order, the forward as trained missed both bars: d was 174 and 243, and the copy cells were 47–48% equal. claude-239 (#3479) then made the integer session's rule (ascending bits, then ascending `s`) the float read's default.

**The item's done-when 2 names claude-146's language checkpoints on DEV.** Does the integer session agree with the float D11 forward of claude-146's step 16,384 there, under the float read's default rule, at all three seeds? Two numbers answer it:
- top-1 agreement of the decoded ids;
- the share of (position, head, read) kept sets equal to the float's.

**Two readings beside it:**
- **The gap the rule closes, on real text.** claude-146 trained before claude-239: its `choose_sorted` (`b349e65ca`) is one arg-sort with no tie rule. So `d11`, the rule off, is the forward as trained, and its distance from the parity arm is the trained readout's dependence on an order the trainer never fixed. DEV repeats tokens, and a repeated token's keys carry one copy code, so ties at the 16th place should be more common than on the fixed sequences, whose tokens never repeat.
- **The bridge.** The fixed sequences under `d11-tie` on the same checkpoints, against claude-238's bars. If DEV misses and the bridge meets, the gap is in the text. If both miss, it is in the checkpoints.

**Out of the question:**
- the turn memory;
- real prompts;
- the older `lang-6d8bac25` form;
- training with the rule on;
- the timing as a bar. It is a reading (§2).

## 2. Arms

**Per seed** (s1, s2, s3), on claude-146's step 16,384 with its Hamming read:

| Arm | Sequences | Tie rule | Role |
|---|---|---|---|
| `d11-tie` | DEV's sample | the integer session's, the default since claude-239 | **parity** |
| `d11` | DEV's sample | the GPU sort's own, as trained | the gap |
| `d11-tie` (fixed) | the 8 fixed sequences of 192 | the integer session's | the bridge |

**On the pod, per seed: the float side only** (CHARTER §3.5).
1. **The snapshot and DEV's readouts:** `fbs-snapshot --dev DATA_ROOT MODEL_DIR snapshot.json readout.json readout-export.json readout-d11.json`.
2. **The fixed sequences:** `fbs-snapshot MODEL_DIR` (no `--dev`) writes `snapshot.json` and the three fixed readouts.
3. **The traces:** `fbs-snapshot --dev DATA_ROOT --hamming-trace ARM MODEL_DIR readout-d11.json trace.json` for `d11-tie` and `d11`, and the same without `--dev` for the bridge.

**On the laptop, from the stored files: the integer side, as serving runs it** (the item's done-when 2). claude-238 ran the export and the comparison on the pod; here they read the integer artifact, so they run where it is served.
- **The writer copy:** the snapshot less exactly its last field, `,"readout_nearest":true` (23 bytes), as claude-233 made its copies.
  - The run's objective is `nearest`, so the snapshot reads nearest (`run_readout_nearest`, claude-231).
  - The D11 readout's token is the native writer's, so the export and every comparison read the writer copy.
- **The artifact:** `fbs-export snapshot-writer.json artifact.uorlut`.
- **The top-1 comparisons:** `fbs-compare --parity-only` of the writer copy, the plain and export readouts on the same sequences, and each arm's D11 readout. Each gives d = positions − agreements.
- **The integer traces:** `fbs-toks ARTIFACT --hamming-trace readout-d11.json trace-integer.json`, once on DEV and once on the fixed sequences, from the `d11-tie` readouts. The tool takes its sequences from the readout.
- **The comparisons:** `fbs-toks --hamming-trace-compare trace-ARM.json trace-integer.json` for the three arms. Per (read, head kind), each gives:
  - the sets judged and equal;
  - ties, query flips, key flips, both, and instrument faults;
  - the type-head sets downstream of a stream-read difference.
- **The copy codes:** `fbs-toks ARTIFACT --copy-codes trace-ARM.json trace-integer.json copy-codes.json` for the three arms.
- **The tie dependence:** the positions where the `d11` and `d11-tie` DEV readouts' `next` differ, counted with `jq`, as claude-238 read it.
- **The timing, a reading, not a bar:** `fbs-toks ARTIFACT --context-sweep` on s1's artifact, with the read and without, on the same artifact (the item's done-when 3).

## 3. Bars and decision table

**DEV's sample** (§5): 54 windows, 5,795 positions, 4,823 of them past t = 17. Every window reaches t = 17, so the 18 positions up to it number 54 × 18 = 972.

**The two bars, judged on `d11-tie` (DEV) at each seed:**
- **P1, top-1:** d ≤ 173 of 5,795 sequence positions, so top-1 agreement ≥ 97.0%. This is claude-233's 3% (46 of 1,536), on DEV's count.
  - d counts every position the forward reads, the prompt positions included, as claude-238's parity did. About 1,800 of the 5,795 are scored targets.
  - d on the scored positions alone is not reported: `fbs-compare` gives counts, not the differing positions, and it is outside this unit's scope.
- **P2, kept sets:** in each of the three cells the comparison judges at every position, the share of sets past t = 17 equal to the float's is ≥ 0.95. The three cells are the copy heads at both reads and the type heads at the first.
  - The share is (equal − 1,944) / (judged − 1,944).
  - The 1,944 are each cell's sets at t ≤ 17 (2 heads × 972). Up to t = 17 a read keeps every earlier position, so these are equal by construction.
  - That leaves 9,646 sets past t = 17 per cell, and the bar allows at most 482 unequal.
  - **The second read's type heads,** the fourth cell, are judged only where every earlier read's sets agree at that position and before it (`compare_hamming_traces`). Its condition: its judged share past t = 17 is ≥ 0.95 on at least 965 judged sets past t = 17 (10% of 9,646), as claude-238 set it.
  - With fewer judged sets, the fourth cell is unmeasured, and the arm cannot meet parity. These heads feed the readout, so a parity call needs them measured.

**"Meets" always means at all three seeds.**
- `d11-tie` **meets P1** when d ≤ 173 at s1, s2 and s3. It **meets P2** when all four cells' conditions hold at each.
- It **meets parity** when it meets P1 and P2.
- Holding at fewer seeds is not meeting. Every seed's numbers are reported.

The single tokens reach no read, so their agreement is reported only.

**The bridge is held to claude-238's bars on its 1,536 positions:** d ≤ 46; each barred cell ≥ 0.95 past t = 17 with 288 sets up to it; the fourth cell on at least 278. It is reported, and it informs rows K and G; it decides no row.

**The rows are read in order, and the first that holds applies.** Outside rows 0 and S they are exhaustive.

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4) | Nothing. | Repair, then rerun. |
| S | **`I-Copy` fails** (§4.7): a `d11-tie` copy bit differs from the CPU control where the control's value is 10⁻⁴ or more, or a pair is inconsistent | A stage remains between the read as trained and the float reference. No parity call is made. | Classify the stage (instrument or object), then trace it, as claude-237 did. |
| P | `d11-tie` meets parity | **The item's done-when 2 holds:** claude-146's checkpoints on DEV, against the float forward under its default rule. | The item's remaining done-when, the timing (step 6), from this unit's reading if it serves. |
| K | `d11-tie` meets P1 | Top-1 holds; the kept sets miss or go unmeasured. | A missed cell is read from its flips: type-head query flips are the residual near a code's sign, and copy flips are the codes (`I-Copy`'s route). An unmeasured fourth cell gets a comparison that feeds both sides the float's first-read outputs. |
| G | otherwise: `d11-tie` misses P1 at some seed | The read on DEV does not reach parity. | The bridge names where. If it meets claude-238's bars, the gap is in the text: read the differing positions' windows. If it misses, the gap is in the checkpoints: compare with (r-drop)'s run config and read. |

**In every row, the REPORT gives:**
- each arm's d and top-1 share, its three barred cell shares, and the fourth cell's judged share and downstream count, at all three seeds;
- the gap: `d11`'s d and cell shares against `d11-tie`'s, with each comparison's tie count, and the positions where the two readouts' `next` differ;
- the bridge against claude-238's bars, next to (r-drop)'s 6 and 27.

## 4. Instrument checks (any failure is row 0, except `I-Copy`, which is row S)

1. **`I-Data`:**
   - the 12 checkpoint files pass the pins in §5, taken from claude-146's sealed `SHA256SUMS-claude-146-lang`;
   - DEV's two files pass the pins in §5.
   - Every `--dev` step prints `dev: 428 windows, 54 sampled, 5795 positions, 4823 past t = 17`. The binary loads DEV with its pins, checks its 14,287 scored targets and prints the line before the GPU opens.
2. **`I-GPU-Tests`, at the head:** the `flock_read_hamming::` and `fbs_export::` CUDA tests pass, with at least one run, 0 failed. These six must run and pass by name:
   - `the_tie_rule_keeps_the_integer_sessions_order`;
   - `a_read_breaks_its_ties_by_the_integer_sessions_rule_by_default` (claude-239's default);
   - `recording_the_trace_leaves_the_read_unchanged`;
   - `the_copy_code_snap_is_the_gpu_hooks_own`;
   - `the_read_runs_as_trained_under_the_d11_and_export_gates`;
   - `the_trunks_gates_after_the_read_still_snap`.
3. **`I-Snapshot`, per seed:**
   - on the pod: the DEV form's `snapshot.json` is byte-identical to the fixed form's (the snapshot does not depend on `--dev`), and it carries the read (`"hamming":{`), the retained reading once, and `"readout_nearest":true` once;
   - on the laptop: the writer copy is it less exactly 23 bytes and carries no flag; the artifact does not read nearest; and every `compare.json`'s `artifact_sha256` is the exported artifact's.
4. **`I-Default`, per seed and sequences:** the snapshot form's `readout-d11.json` is byte-identical to the `d11-tie` trace form's. So the parity arm is the float read's default forward, and recording the trace leaves it unchanged.
   - **`d11`, a reading:** it reached the sort when its comparison counts ties, or its kept sets differ from `d11-tie`'s (the traces' `windows`, compared on the laptop).
   - Its readout can still equal the parity arm's: a tie at or past a head's margin has weight max(0, m − d)/m = 0. So an equal readout alone is not a fault.
   - If neither holds, DEV had no tie at a 16th place, and the REPORT says so. Each arm's rule is pinned by `I-GPU-Tests`.
5. **`I-Shape`:**
   - each DEV trace and readout holds 54 windows with lengths summing to 5,795;
   - each fixed one holds 8 of 192;
   - the integer traces carry the same ids (the comparison refuses otherwise).
6. **`I-Cut`, already run on the laptop (1 Nov), before the run.** No integer tool had read windows shorter than the context before this unit. On claude-238's s1 `d11-tie` files (claude-233's h1 artifact), with the windows cut to 192, 131, 64, 37, 18, 17, 5 and 1 positions:
   - `fbs-toks --hamming-trace` on the cut readout gave exactly the cut of the full integer trace (equal under `jq -S`);
   - `--hamming-trace-compare` of the cut float and integer traces judged 930 sets per cell (2 × 465), with 0 faults;
   - `fbs-compare --parity-only` on the cut readouts gave 465 positions and 463 agreeing (the full run: 1,530 of 1,536). The positions it lists as worst carry the full run's gaps (166, 164, 200, 165).
   - The model is causal, so a cut window is the prefix of the full one, and the tools agree.
   - The files are in `local/claude/claude-240/cut/`.
7. **`I-Copy`, the reference computes the read as trained,** on `d11-tie` (DEV) at each seed and read:
   - 0 inconsistent pairs: each (token, copy head) carries one code across the trace;
   - no differing query or key bit where the control's value is 10⁻⁴ or more (`COPY_VALUE_BINS` bins 3 to 6 empty).
   - **The allowance:** claude-238's arithmetic bound, CPU against GPU, is about 1.3 × 10⁻⁵. `d11-tie`'s f32 rotations differ from the artifact's rounded ones by at most 2⁻²⁵, which adds under 10⁻⁶ (4 terms × |p| ≤ 8).
   - **Bits under 10⁻⁴ are counted and reported, with no cap.** DEV repeats tokens, so one flipped (token, head, bit) counts once per occurrence, and a count of samples measures frequency, not stages.
   - Bins 0 to 2 are reported apart, so a bit between the allowance and 10⁻⁴ (bin 2) shows as such. It is not row S: that rule is the 10⁻⁴ line above.
   - A real stage shows at 0.1 or more: claude-237's snap moved 57 to 695 bits per cell there. claude-238 found 0 differing bits on every arm.
   - The other two arms' counts, and the control against the integer session at all 8,192 codes, are reported.
8. **`I-Faults`:**
   - every comparison counts 0 instrument faults;
   - the two `d11-tie` comparisons count 0 ties. With one rule on both sides, equal codes give equal sets.
   - claude-235's two known routes to row 0 still apply:
     - the comparison expects every earlier position to be a candidate, and `fbs-snapshot` sets no read or row cut;
     - exact-zero coordinates.
9. **`I-Finite`:** every tool exits 0.
   - An arm whose tool fails is logged, the others still run, and the job ends with EXIT 7.
   - A missing comparison or trace is row 0.

## 5. Base artifact and data split

- **Base:** claude-146's merged-form language runs, step 16,384, at `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`.
  - Each run config reads `objective: nearest`, `retain_rates: true`, `hamming_read: 4`, `steps: 16384`, with seeds 1, 2 and 3.
  - Each model config reads context 192 and vocab 4,096.
  - The pins come from `SHA256SUMS-claude-146-lang`. All 12 were checked on the laptop on 1 Nov.

  | File | s1 (`lang-df1d7892/hamming-s1`) | s2 (`lang-df1d7892/hamming-s2`) | s3 (`lang-s3-df1d7892/hamming-s3`) |
  |---|---|---|---|
  | `config.json` | `031cd39d22e84bedcb62f1d7fdc933e65aa3cd698a6ae3ab2a410d33db833264` | `91bb65d77e5557f988c7583b442605f153427fd85e63eb88424897b2e6e4a40f` | `8e2d98522b199903c64cdc02f7b2a4f19371adce5b89798fcb093265b1184535` |
  | `models/step-016384/config.json` | `f1674fba2481a6520adddcea85e92e34fa2e059761e4bd307af5124e1acbc6c4` | `a9ba99b3239df7108158521a78958cf90288134f36b6c4858f186e387e4abdd2` | `adee67596a779136a24848eaa1ccc28bcc801cd39e9bdb702a1c1e6a002010d2` |
  | `models/step-016384/hamming_read.safetensors` | `bde8174f3198de780de90363c46e82b157d70d8382192eef3882550599b98bba` | `c0f34c0510ce6b5b5ae273f4f6d943605dbd5230816043c2de09099ada0b3300` | `a6cacff589f031eae6268192259f56b39fa5b41827d66a88eb00107e4a7eab35` |
  | `models/step-016384/model.safetensors` | `cad51f17104025c18a194a2394d0c0f5d9464ea56c7c8e5bbaaa41d2cec4073d` | `42e327b020358651dc12a1385f1c8bc82022dd6a1fd4cdb419f11408b87561fc` | `a55a5318fc3592c2a2559ae9706980f63a64e10be18c766415b919372385d0cb` |

  - On the laptop (1 Nov), `fbs-snapshot --dev` read each checkpoint through its read check (`check_checkpoint_reads`), its objective and its rate reading, up to the GPU. The read's own load comes after the GPU opens and is first exercised on the pod.
- **Data split: DEV only.** `w2-inputs/2026-10-25/driver-data/ft/dev/`:
  - `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
  - `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

  These are `hamilton_sheet`'s pins. The sample:
  - DEV is cut into the trainer's windows (`build_document_windows` at 192), 428 of them, scoring 14,287 targets;
  - every 8th window is kept from the first (claude-126's `PARITY_WINDOW_STRIDE`), so 54;
  - each is cut after its last scored position. The trainer pads every window with BOS to 192, and the model is causal, so the cut changes no scored prediction. Padded, the 54 would be 10,368 positions; cut, they are 5,795.
  - TRAIN and the panel are not opened.
- **The bridge's sequences:** `fbs_float_readout`'s 8 fixed sequences of 192, with token (37s + 11t + 1) mod 4,096.

## 6. Steps

**On the pod:**
1. The data, every file checked (`I-Data`).
2. The source pin at the pushed head, clean.
3. `I-GPU-Tests` at the head.
4. The build: `fbs-snapshot` (cuda).
5. Per seed: the DEV snapshot form, the fixed snapshot form, the pod's half of `I-Snapshot`; the three traces; `I-Default`.
6. `SHA256SUMS`; the output is stored to `results/claude/integer-hamming-read-dev-2026-11-01/` and listed back; then release and down.

**On the laptop,** with `fbs-export`, `fbs-compare` and `fbs-toks` built at the same head (serving-wasm):

7. The stored output fetched, and checked against its sums. The laptop's head, its clean tree and each tool's sha256 are recorded in `reading/`. A crate or Cargo file that differs from the pod's head (its `head.txt`), or a tree not clean there, is row 0.
8. Per seed: the writer copy, the artifact and the laptop's half of `I-Snapshot`; the three top-1 comparisons.
9. Per seed: the two integer traces, then the three trace comparisons (`I-Shape`, `I-Faults`).
10. The copy codes for the three arms (`I-Copy` on `d11-tie`).
11. The tie dependence from the two DEV readouts.
12. The context sweep on s1's artifact.
13. The rows, read from the stored comparisons. The laptop's outputs are sealed and stored to `results/claude/integer-hamming-read-dev-2026-11-01/reading/`.

## 7. Minimum test power

- **P1:** 5,795 positions per seed against a bar of 173.
  - On (r-drop) and the fixed sequences, `d11-tie` sat at 0.4% and 1.8% (6 and 27 of 1,536), and `d11` at 11% and 16%.
  - If DEV behaves like that, `d11-tie` sits near 25 to 100, and `d11` far above the bar. The bar separates the two by an order of magnitude.
- **P2:** per cell and seed, 9,646 sets past t = 17, against at most 482 unequal. On (r-drop), `d11-tie`'s copy cells were exact, and `d11`'s were 47–48% equal.
- **The fourth cell's floor is about as tight as P2's bar.**
  - Reaching 965 judged sets over 54 windows needs about 8.9 positions past t = 17 per window, of the 89 each window has on average.
  - With a first-read type-head difference rate p per set, the mean judged run before the first difference is about (1 − 2p)/(2p) positions. So the floor needs p ≤ about 5.0%, P2's own 5%. Short windows tighten it further.
  - So an arm near P2's bar may land in row K with the fourth cell unmeasured. Row K says what that needs.
  - On the fixed sequences claude-238's floor needed about 3%, because each window had only 174 positions past t = 17.
- **Seeds:** three, each with its own 5,795 positions. "Meets" at all three.
- **Out of reach:** a fault seen only on real prompts, only through the turn memory, or only past DEV's distribution.

## 8. Cost and stop rule

- **Cost:** one RTX 5090. Vast's cheapest verified 5090 was $0.563/h on 31 Oct; the price is read again at launch.
  - Bootstrap and builds take about 10 minutes, and the CUDA tests about 5.
  - Per seed on the pod: two snapshot forms and three traces.
  - **claude-238's measured steps, on the fixed sequences** (`job.log`): a snapshot form with three readouts took 78–79 s, and a trace 27–30 s. Its pod was up about 15 minutes in all.
  - DEV has 3.8 times the sequence positions, but each sequence is one forward, and the 4,096 single tokens are read in every readout either way. Even at twice claude-238's step times, a seed takes about 6 minutes.
  - So about 35 minutes in all, under $0.50. The job runs with `--hours 2` and a $2 ceiling.
  - `uor-pod up` takes `--count 1`: claude-238's pod was billed for two GPUs because it was left off.
- **The throughput check:**
  - the job logs each step's start and end;
  - after seed 1, the projection is the elapsed time plus twice seed 1's;
  - past 2 hours is the ORDER's stop rule: a decision review, never a wait. A healthy run is never cancelled for a self-set budget; the lease is renewed and the extension noted.
- **Stop rule:**
  - a failed data, build, test or `I-Snapshot` check stops the job with exit 3, 4 or 6, and is never counted;
  - an arm whose tool fails is logged, and the job ends with EXIT 7 (row 0);
  - every exit copies the logs and `EXIT` into the output and seals it;
  - a watcher runs from launch to down, and the lease is renewed while the job runs;
  - the laptop steps begin only after the seal is read back.

## 9. Non-author TEST FITNESS review

**FIT** (claude, a fresh reviewer that did not write the sheet, 1 Nov, at `ec06f1318`).

It read the sheet, `fbs-snapshot.rs`, `fbs_export.rs`, `hamilton_sheet.rs`, `flock_read_hamming.rs` and the integer `fbs_snapshot.rs`. It checked:
- the bars' arithmetic: 173 of 5,795; 9,646 sets past t = 17, at most 482 unequal; the floor of 965;
- that every window reaches t = 17: 5,795 − 4,823 = 972 = 54 × 18, and `I-Data` pins the printed line;
- that sets at t ≤ 17 are equal by construction (`kept_set` wants min(16, t − 1));
- that the cut keeps real tokens only, and both sides read the same cut inputs;
- that the tie key (h·T + s) is exact in f32 and orders by (bits, s);
- that rows P, K and G are exhaustive, and the bridge decides no row;
- the writer copy's 23 bytes, and the value bins.

**Its five minor findings, each applied here:**
1. §7: the mean run is (1 − 2p)/(2p), so the floor needs p ≤ about 5.0%, not 5.6%.
2. §4.4: a `d11` readout equal to the parity arm's is not a fault when every tie has weight 0. The rule now reads ties and kept sets.
3. Row S now says what `I-Copy` triggers on (10⁻⁴), and bin 2 is reported apart.
4. §6 step 7: the laptop's tools are pinned to the pod's head.
5. §3 P1: d counts every position. Scored-only d is not reported, and the reason is given.

**Left for the pre-run review:** `fbs-compare --parity-only` on DEV's variable-length sequences (that d counts sequence positions only), and the `fbs-toks` command forms.

## 10. Pre-run review

Pending.
