# Prep sheet: the Hamming read's parity on claude-146's checkpoints, on DEV (integer-hamming-read, claude-240), 2026-11-01

- Item: [integer-hamming-read](../../roadmap/items/integer-hamming-read.md) (#3385), step 5, after claude-238 and claude-239. ORDER: [claude-240](../../loop/tasks/integer-hamming-read/claude-240/ORDER.md).
- **The run:** one GPU job on one leased RTX 5090, `local/claude/claude-240/job.sh`.
  - It is not in git. The job copies itself into the outputs, so the seal covers it.
  - It runs against the pushed head.
  - **Attempt 2.** Attempt 1 stopped at its first snapshot step: `StackModel::load` refused claude-146's checkpoints, whose saved kernel identity predates claude-179. The repair is an evaluation copy, held to its run's sealed inspection (§4.10, §11).
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
0. **The evaluation copy and its inspections** (attempt 2, §11):
   - `fbs-snapshot --rebind-fbs-backward MODEL_DIR COPY_DIR` writes the copy (§4.10);
   - `fbs-snapshot --inspect as-trained DATA_ROOT COPY_RUN` and `--inspect tie-rule` run the trainer's `--inspect` of the copy, with the read's ties broken as trained and by the rule.

   Every form below reads the copy as `MODEL_DIR`.
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
10. **`I-Rebind`, the copy is the checkpoint as claude-146 evaluated it** (attempt 2, §11), per seed:
   - **The static half, on the pod.** `--rebind-fbs-backward` refuses unless:
     - the kernel's text through `spiralcore_fbs_fwd` hashes to the text before claude-179;
     - each of the six saved fingerprints (five plans and the vector head's 8/8 `geometry`) is this build's plan with only the kernel's hash put back;
     - the copy then loads, with every check `StackModel::load` makes.

     Then the job checks that the copy's tensors and run config are the pinned files byte for byte; that it holds 4 files; and that its step config is the checkpoint's with exactly the six printed fingerprints replaced. That replay checks that nothing else changed, not that each new fingerprint is right; only the tool's own load of the copy checks that.
   - **The Gate R half, read on the laptop.** Two inspections are held to claude-146's sealed inspection at step 16,384, over all 14,287 of DEV's scored rows, field by field over the sealed line's own 22 fields, nulls included. Two of them, `flock_weight_mean_abs` and `content_weight_mean_abs`, are null in the sealed line, since neither the Flock read nor the content read is set; each inspection must report null there too. Fields the head's inspection gained after claude-146 are not compared.
     - **H:** the copy's inspection as trained, at this head.
     - **C, the control:** claude-146's own build, `train-in-space` at `df1d7892`, inspecting the original checkpoint with claude-146's command. It loads there unchanged. C only classifies a miss of H, so a C that does not build or run is recorded and the job goes on.
     - **The counts:** `scored_rows`, `top1_hits`, `nearest.hits`, `nearest.opening_rows`, `nearest.opening_hits`, `nearest.commonest_opening_rows`, `nearest.opening_distinct_picks`, `admitted_hits`, `token_unviolated_hits`, `probe_rows` and `distinct_first_cells`. The other eleven fields are floats (margins, spreads, means), the two nulls and a string.
   - **The rows, per seed, read in order:**

     | Row | H | C | The seed |
     |---|---|---|---|
     | holds | every field equal | (recorded) | the copy is the checkpoint as claude-146 evaluated it |
     | holds on counts | every count equal, a float differs | every count equal, a float differs | holds; the pod's floats do not reproduce claude-146's own build either. The REPORT says so, and says whether H's floats equal C's on the same pod |
     | 0, the head | otherwise | every count equal, and H differs in a count; or every field equal | row 0: this head's forward or evaluator differs from claude-146's build. `inspect_run` itself has changed since `df1d7892`, so a decision review classifies which |
     | 0, unclassified | otherwise | a count differs, or C did not run | row 0: the pod or the build. The control is repeated on another pod |

   - **Every seed is held to it.** "Meets" is at all three seeds (§3), so a seed in a row 0 means no parity call. Its numbers, and the other seeds', are still computed and reported as readings.
   - **The rule:** the REPORT says whether H's line and the inspection under the rule differ (unknown if either line is missing). If they are equal, DEV has no tie that moves the inspection, and the gate cannot see which rule ran. The rule is then pinned only by `I-GPU-Tests` and the form's own test.
   - **What the class rests on:** the static half proves only that the copy is the checkpoint, byte for byte, with the forward kernel's text unchanged. The Hamming read's Rust forward and the native stack have changed since `df1d7892`: hooks that are off unless set, an `as_trained` path, and the tie rule. That the copy computes claude-146's forward at this head rests on H and C, not on the rebind.
   - **The sealed inspections' pins** are claude-146's seal lines (§5).

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
  - **The evaluation copies (attempt 2).** The forms read each seed's copy (§4.10). On the laptop (1 Nov), the three copies loaded through `StackModel::load` and `HammingReads::load`. Their tensors and run configs equal the originals byte for byte, and each step config differs only in its six fingerprints.
  - **claude-146's sealed inspections**, its `train-in-space --inspect` of each run at `df1d7892` (`target-sm120`), pinned by its seal:

    | Seed | File | sha256 |
    |---|---|---|
    | s1 | `lang-df1d7892/inspect-hamming-s1.out` | `c6223dbfac14aec7d2ba1ecfcaaf382d90ed7b858fbbd601fa571094db86e624` |
    | s2 | `lang-df1d7892/inspect-hamming-s2.out` | `17d86bb7787e6c03402774af6ee739b039ec9161578c53f0ad740a3912ebfeba` |
    | s3 | `lang-s3-df1d7892/inspect-hamming-s3.out` | `0bec386e2555fa204b0f2872bd981598fbcaf7a0262b9344657c870619aa0a1d` |

    s1's step-16,384 line reads, among its counts, 368 top-1 hits and 8,687 nearest hits of 14,287.
  - **The control's source:** `df1d7892542dcdba1a45dd64387d4a123868055c`. Its `rust-toolchain.toml` and `Cargo.lock` equal this head's.
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
1. The data, every file checked (`I-Data`), and claude-146's three sealed inspections against their pins and its seal.
2. The source pin at the pushed head, clean.
3. `I-GPU-Tests` at the head.
4. The build: `fbs-snapshot` (cuda); the GPU's name and compute capability recorded. Then the control: `train-in-space` (cuda) at `df1d7892`, in its own source and target folders.
5. Per seed: the copy and `I-Rebind`'s static half; the copy's two inspections and the control's; then, on the copy, the DEV snapshot form, the fixed snapshot form, the pod's half of `I-Snapshot`; the three traces; `I-Default`.
6. `SHA256SUMS`; the output is stored to `results/claude/integer-hamming-read-dev-2026-11-01/` and listed back; then release and down.

**On the laptop,** with `fbs-export`, `fbs-compare` and `fbs-toks` built at the same head (serving-wasm):

7. The stored output fetched, and checked against its sums. The laptop's head, its clean tree and each tool's sha256 are recorded in `reading/`. A crate or Cargo file that differs from the pod's head (its `head.txt`), or a tree not clean there, is row 0.
8. Per seed: `I-Rebind`'s Gate R half (§4.10); the writer copy, the artifact and the laptop's half of `I-Snapshot`; the three top-1 comparisons.
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
  - **Attempt 2 adds:** the control's build, a few minutes in its own target folder; per seed, the copy (seconds) and three inspections of one checkpoint each. claude-146's inspection of nine checkpoints took 17 s.
  - The watcher's seed-1 time now starts at the copy.
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

**Attempt 2's delta: FIT** (claude, a fresh reviewer, 1 Nov, at `de83faabf`, with decision round 2, §11). It read §4.10's `I-Rebind`, the amended §2, §5, §6 and §8, and `read.sh`'s Gate R half.

## 10. Pre-run review

**PASS** (claude, 1 Nov, at `5ab73fc01`). Two fresh read-only reviewers, neither of which wrote the sheet or the scripts, each took one side.

**The integer side** (`read.sh` against `fbs-compare` and `fbs-toks`): PASS, nothing blocking.
- `fbs-compare --parity-only` takes the four files in read.sh's order. Its `sequences` section counts the readout's own sequence positions (5,795), with the single tokens apart.
- It accepts any length from 1 to the context, and refuses readouts that hold different sequences. It prints compact JSON, so the `artifact_sha256` match works.
- The four `fbs-toks` forms match read.sh, float before integer. Each takes its windows from its inputs; the comparison checks counts, ids and depths, with no fixed 8 or 192.
- The jq field names are the writers' own. The 23-byte writer copy holds: the snapshot is compact, with no final newline, and the flag is its last field.

**The pod side** (`job.sh` and `watch.sh` against `fbs-snapshot`, the sheet and `uor-pod`): PASS, nothing blocking.
- The four `fbs-snapshot` forms match its parse, `--dev` before `--hamming-trace` included.
- The DEV line matches the binary's print byte for byte.
- The six tests exist under `--features cuda`. All 14 pins are the sheet's.
- Every failure path seals through `end()`, with the sheet's exit codes.
- The watcher's greps match the job's log lines, and the `uor-pod` flags exist.
- The bootstrap leaves the source as a git checkout at `src-SHA` (`uor-pod-bootstrap.sh:447`).

**The minor findings, each applied to the scripts:**
- the laptop fetches the pod's head if it is missing, and exits 8 when a fault was logged;
- the d11 reading compares kept sets only;
- the watcher reports whether the seal is present, flags a projection over 2 hours, and renews the lease once before a time stop;
- `head.txt` is written first, and the seal grep is anchored. It still picks out the 12 lines of the local copy of the seal.

**What the reviewers were not asked:**
- I-Faults' and I-Copy's counts are read from the stored comparisons at step 13, not inside read.sh.
- The output is stored from the laptop (`uor-store put` over ssh) and listed back before `down`.

**The chain and the lease:**
- This run is the item's step 5, in PLACE.md's order: serve the working model's parts in integers, the read's parity before its speed.
- Each row of §3 names what it unlocks.
- `uor-pod status`, 1 Nov: 0 pods, $0/h of $8/h; a 5090 at $0.563/h, 2 h leased, `--count 1`.

**Attempt 2's delta: PASS** (claude, 1 Nov, at `de83faabf`). A fresh read-only reviewer that wrote neither the scripts nor the tool took the job's new steps against `fbs-snapshot.rs`, `inspect_run` at `df1d7892`, claude-146's job log and sealed inspection, and `read.sh`. No blocker, no major.
- The two new forms' parses, and the job's grep and sed, match the tool's prints byte for byte. The control's command is claude-146's, apart from its paths.
- Every failure path seals through `end()`. A failed control is only recorded.
- **Its minor findings:**
  - the sed replay checks that nothing else changed, not each new fingerprint (§4.10 now says so);
  - the tie-rule setting is seen on the pod only through `rule_changes_the_line`, which §4.10's "The rule" already reports;
  - the control is a cold CUDA build, a guess of 5 to 15 minutes, still well inside the 2-hour lease.

## 11. Attempt 1, and the decision review

**Attempt 1 (1 Nov, at `50a824385`): row 0, an instrument fault, before any measurement.**
- The pod passed `I-Data`, the source pin, `I-GPU-Tests` (44 passed, the six named among them) and the build.
- The first snapshot step then refused s1: `Invalid("native projection compiler/operator/FBS identity is missing or stale")`. The job ended with EXIT 6.
- It was up about 5 minutes, about $0.05. Its output is stored at `results/claude/integer-hamming-read-dev-2026-11-01-fail1/`. Attempt 2 stores to the unused main root (L-002).
- **The cause.** `StackModel::load` requires the saved projection identity, and the vector head's, to equal this build's. Each plan's fingerprint hashes its plan, which carries the sha256 of the whole `spiralcore_fbs.cu`. claude-179 (`bcf2f57f7`, #3367) changed that file after claude-146 trained: only the backward kernel and two new reduce kernels. The forward's text is unchanged.

**The decision (the ORDER's stop rule "`fbs-snapshot` refuses it"), by a fresh decision reviewer that did not write the work:**
- **Round 1 (at `e7dd3e40e`): REVISE.** The rebind is sound and minimal. On the laptop, its three copies equal the originals byte for byte, less the six fingerprints. The reviewer's four findings, each applied here (§4.10):
  1. *Major:* claude-146 inspected on `target-sm120`, and the pod is the same architecture, so the gate holds every sealed field, floats included. Counts alone are the gate only where the control's floats do not reproduce either.
  2. *Major:* `inspect_run` has changed since `df1d7892`, and the pod could be the cause, so a miss is classified with a control: claude-146's own build on the original checkpoint, on the same pod. Each seed has its own row.
  3. *Minor:* the static half proves only the copy and the forward kernel's text. That the copy computes claude-146's forward rests on H and C.
  4. *Minor:* the gate cannot see the rule if DEV has no tie that moves the inspection, so the REPORT says whether the two inspections differ.
- **Round 2 (at `de83faabf`): APPROVE, and the delta FIT: FIT.** A fresh reviewer found all four findings fixed. It enumerated the 16 pairs of H and C (all equal, a float differs, a count differs, missing) and found that each falls in exactly one row read top-down, as the jq in `read.sh` does. It checked the three sealed pins against the seal, and attempt 1's facts against its stored log. Its three minor findings, each applied here and in `read.sh`:
  1. the comparison skipped the sealed line's two null fields. It now compares all 22, nulls included (§4.10);
  2. in the row "holds on counts", the REPORT also says whether H's floats equal C's, which `read.sh` records as `as_trained_against_control`;
  3. a missing tie-rule line now reads as unknown, not as "the rule changes the line".

  On eight synthetic cases plus one with a changed null field, the edited jq gives the intended row. The changed null field now lands in 0-head, where the old jq passed it.
