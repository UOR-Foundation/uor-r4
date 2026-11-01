# Prep sheet: the Hamming read's parity against a float reference that runs the read as trained (integer-hamming-read, claude-238), 2026-11-01

- Item: [integer-hamming-read](../../roadmap/items/integer-hamming-read.md) (#3385), step 5, after claude-237. ORDER: [claude-238](../../loop/tasks/integer-hamming-read/claude-238/ORDER.md).
- **The run:** one GPU job on one leased RTX 5090, `local/claude/claude-238/job.sh`. It is not in git; the job copies itself into the outputs, so the seal covers it. It runs against the pushed head. The integer side, the comparisons and the copy-code check run on the laptop, from the stored outputs, as serving does.

## 1. Question

claude-237 (#3474) found the stage where the read's two sides part. In D11 mode the CUDA gate hook snaps the read's rotations to icosians. The trainer (`HamiltonActive`) and the artifact (`lut-stack/5`'s raw gates) do not. So claude-233's d of 694 (s1) and 823 (s3) of 1,536 measured the snap, not the port. This unit's guard (`as_trained` in `flock_read_hamming.rs`) runs the read's gates through the hook's pass-through inside any D11 or export scope, and gives the hook back whole afterwards.

**With the read run as trained, does the integer session agree with the float D11 forward of (r-drop) step 16,384?** Two numbers answer it, the item's done-when 2:
- top-1 agreement of the decoded ids;
- the share of (position, head, read) kept sets equal to the float's.

**If it does not agree,** which of the remaining known causes is it: the order among ties at the 16th place, the read's parameter grid, or the trunk residual near a code's sign?

**Out of the question:**
- claude-146's checkpoints. The item names them as done-when 2's reference; they are the next unit, after this one settles the reference on (r-drop).
- The turn memory.
- Real prompts.
- A trainer that runs the read's backward through D11 (`with_d11_training`). The guard covers the forward only, and no trainer runs one with a read.

## 2. Arms

**The read, with the guard.** Two seeds × claude-235's four arms, each the D11 readout of (r-drop) step 16,384 with its Hamming read and without the turn memory. The copies are claude-233's h1 and h3.

| Arm | Tie rule | Read parameters |
|---|---|---|
| d11 | the GPU sort's own, as trained | f32 as stored |
| d11-tie | the integer session's: ascending bits, then ascending `s` | f32 |
| d11-rounded | the sort's own | rounded as exported: rotations and weights at 2^-24, margins at 2^-16 |
| d11-tie-rounded | the integer session's | rounded |

**d11 is the parity arm:** the float forward of the stored checkpoint, the read as trained. The other three name a cause if d11 misses.
- With the copy-head codes equal on both sides, ties at the 16th place now decide sets that flips decided before. That is why the two tie arms run again, though claude-235 counted 0 ties.

**Per arm and seed, on the pod:**
1. `fbs-snapshot --hamming-trace ARM MODEL_DIR readout-d11.json trace.json` writes the D11 readout and the float trace.
2. `fbs-compare --parity-only` of claude-233's h_s writer snapshot, its plain and export readouts, and this arm's D11 readout. It gives d = 1,536 − agreements.
   - `--parity-only` reads only the snapshot (for the artifact) and the D11 readout. The plain and export readouts are parsed for shape alone.
   - The snapshot is written from the parameters before any read is set (`fbs-snapshot.rs`), so the artifact is claude-233's, byte for byte, and the integer side does not move.

**The trunk alone, per seed, on the pod:** claude-233's t1 and t3 copies (no `hamming_read.safetensors`), through `fbs-snapshot`'s first form, then `fbs-compare --parity-only`. This is `I-Trunk`.

**On the laptop, from the stored files:**
- **The integer traces:** `fbs-toks ARTIFACT --hamming-trace readout-d11.json trace-integer.json` per seed, on claude-233's h_s artifact (`I-Integer`).
- **The comparisons:** `fbs-toks --hamming-trace-compare trace-ARM.json trace-integer.json` for each arm and seed. Per (read, head kind), each gives:
  - the sets judged and equal;
  - ties, query flips, key flips, both, and instrument faults;
  - the type-head sets downstream of a stream-read difference.
- **The copy codes:** `fbs-toks ARTIFACT --copy-codes trace-ARM.json trace-integer.json copy-codes.json` for d11 and d11-rounded at each seed (`I-Copy`). Both tie arms share their parameter arm's codes.
- **The timing, a reading, not a bar:** `fbs-toks ARTIFACT --context-sweep` on h1's artifact, as claude-235 ran it. The artifact is unchanged, so this repeats claude-235's reading under today's load.

## 3. Bars and decision table

**The two bars, judged per arm at each seed:**
- **P1, top-1:** d ≤ 46 of 1,536 sequence positions, so top-1 agreement ≥ 97.0%. This is claude-233's bar.
- **P2, kept sets:** in each of the three cells the comparison judges at every position, the share of sets past t = 17 equal to the float's is ≥ 0.95. The cells are copy heads at both reads, and type heads at the first read.
  - The share is (equal − 288) / (judged − 288).
  - The 288 are each cell's sets at t ≤ 17 (2 heads × 8 windows × 18 positions). They keep every earlier position, so they are equal by construction, and every earlier read agrees there.
  - That leaves 2,784 sets past t = 17 per cell, and the bar allows at most 139 unequal.
  - **The second read's type heads,** the fourth cell, are judged only where every earlier read's sets agree at that position and before it (`compare_hamming_traces`). One differing first-read set makes the rest of its window downstream. So that cell's judged share and downstream count are reported, not barred: its inputs already differ wherever it is not judged.

An arm **meets parity** at a seed when P1 and P2 both hold. **Closing** at one seed only counts as not closing; both seeds' numbers are reported.

The single tokens reach no read (claude-233: 0 of 4,096 moved), so they are reported only.

**The rows are read in order, and the first that holds applies.** Outside rows 0 and S they are exhaustive.

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4) | Nothing. | Repair, then rerun. |
| S | **`I-Copy` fails:** a float copy code differs from the CPU control at a bit whose control value is 10⁻⁶ or more | A second stage remains between the trained read and the float reference. No parity call is made. | Classify the stage (instrument or object), then trace it. The ORDER's stop rule. |
| P | d11 meets parity at both seeds | **The item's done-when 2 holds on (r-drop)**, against the read as trained. | The same bars on claude-146's checkpoints, done-when 2's named reference, in the next unit. |
| T | d11 misses, and d11-tie meets parity at both seeds | The remaining gap is the order among ties. | One tie rule for both sides. The trainer takes the integer session's deterministic rule by default, and the checkpoints are checked again. |
| Q | d11-rounded meets parity at both seeds, and d11-tie does not | The cause is the read's grid in the artifact. | The exporter stores the read on finer grids. |
| B | only d11-tie-rounded meets parity at both seeds | Both causes. | Both fixes. |
| K | some arm meets P1 at both seeds, and none meets P2 | Top-1 holds, but the kept sets miss. | A laptop host check of the type heads' codes by coordinate magnitude, as claude-237 checked the copy heads'. |
| G | no arm meets P1 at both seeds | The snap, ties and grid do not explain the gap. | Taken from d11-tie-rounded's judged differing sets, as claude-235's row F named them. For type-head query flips: training with the D11 forward in the loop (`with_d11_training`, claude-130). |

**In every row, the REPORT gives:**
- each arm's d, its top-1 share, its three barred cell shares, and the fourth cell's judged share and downstream count, at both seeds;
- the guard's move against claude-235's d11, which was d = 694 and 823. The move test is claude-227's: d(old) − d(new) > 2√(d(old) + d(new)).

## 4. Instrument checks (any failure is row 0, except `I-Copy`, which is row S)

1. **`I-Data`:**
   - each seed's six checkpoint files pass the run's own sums;
   - claude-233's h1, h3, t1 and t3 files pass the sheet's pins (§5), taken from its sealed `SHA256SUMS`.
2. **`I-GPU-Tests`, at the head:** the `flock_read_hamming::` and `fbs_export::` CUDA tests pass, with at least one run, 0 failed. These five must run and pass by name:
   - `the_read_runs_as_trained_under_the_d11_and_export_gates` (T1);
   - `the_trunks_gates_after_the_read_still_snap` (T2);
   - `the_copy_code_snap_is_the_gpu_hooks_own`;
   - `recording_the_trace_leaves_the_read_unchanged`;
   - `the_tie_rule_keeps_the_integer_sessions_order`.
3. **`I-Negative`, T1 and T2 with the guard set aside.**
   - The job reverses `guard.patch`, the guard's own two hunks and none of the tests (`local/claude/claude-238/guard.patch`, sha256 `f0e91ea6…7d40`, embedded in the job). It then runs T1 and T2 alone.
   - Both must fail by name, in a run of 2 tests, after a build line that shows the test binary was rebuilt.
   - The job restores the patch and checks that the source is clean. Only then does it run `I-GPU-Tests`, which rebuilds the restored library before anything is measured.
   - The reversed source type-checks with the stub nvcc on the laptop (1 Nov).
4. **`I-Trunk`, the guard does not leak into the trunk.** Each seed's trunk-alone outputs must be byte-identical to claude-233's t_s files:
   - `snapshot.json`, and the plain, export and D11 readouts;
   - `compare.json` (`8300b7aa…`, `8196d5f3…`), which carries d = 5 and 11 and claude-233's `artifact_sha256`.

   No read is installed there, so the guard never runs. The check pins the trunk path and the toolchain between claude-233's run head and this one.
5. **`I-Guard-Effect`, the guard is on the readout's path.** Each seed's d11 readout must differ from claude-233's h_s `readout-d11.json` (`6b3d9d7d…`, `bf34b819…`), which claude-235's d11 arm reproduced byte for byte. If it is identical, the readout never reached `as_trained`.
6. **`I-Integer`:** each seed's integer trace is byte-identical to claude-235's (`a8dbc19c…`, `cca428db…`). Same artifact, same sequences, same session: the integer side is unchanged.
7. **`I-Copy`, the reference computes the read as trained** (the ORDER's done-when 3). For d11-rounded at each seed and read, every copy code in the float trace equals the CPU float control (a), queries and keys, and 0 pairs are inconsistent.
   - (a) reads the artifact's own rotations and its Q12 table, and d11-rounded reads the same rounded values. So its codes can part from (a) only by arithmetic: CPU against the GPU kernel, at most a few ulps.
   - **A differing bit whose control value is under 10⁻⁶** (the check's first value bin) is classified as that arithmetic. It is counted and reported, and it does not fail the check. A 4-term f32 sum of unit terms rounds by about 2.4 × 10⁻⁷.
   - **Any differing bit at 10⁻⁶ or more** is row S.
   - d11's counts against (a) are reported too. Its rotations differ from the artifact's by at most 2^-25.
   - `--copy-codes` also re-shows claude-237's (a) = (b) at all 8,192 codes.
8. **`I-Faults`:**
   - every comparison counts 0 instrument faults;
   - the two tie arms count 0 ties. With one rule on both sides, equal codes give equal sets.
   - claude-235's two known routes to row 0 still apply: the full-window assumption, and exact-zero coordinates.
9. **`I-Finite`:** every tool exits 0.
   - An arm whose tool fails is logged, the others still run, and the job ends with EXIT 7.
   - A missing comparison or trace is row 0.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384 at s1 and s3, from `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/`, with its run configs (`retain_rates: true`, `hamming_read: 4`).
  - The h copies carry `hamming_read.safetensors`, and the t copies do not. Neither carries the turn memory.
- **claude-233's sealed outputs,** `results/claude/integer-nearest-readout-locate-2026-10-31/out/`. The pins come from its `SHA256SUMS`:

  | File | h1 | h3 |
  |---|---|---|
  | `snapshot-writer.json` | `b1b534a0ebeecda270293097ce345d0b5b41e816a1cd4496c882407989d2308f` | `238d10cdb5e75be2b81b6e8a93e61c62cfc2f8ceb51b73e7ec655b37c7f367ae` |
  | `readout.json` | `aa498cdcb86e33e8fb7e1850e29d63070f757f0d67a5c07ce1fbfbf00dac376c` | `87da7d87a7f9fbb54c4aa541e62b3b45a3e79fa46a5b8c4828fc1012c43b22d9` |
  | `readout-export.json` | `2380842b4d8293f8a92ab595cea15ea559b4a37a492df657bff3ec59845c14d4` | `538fe38d6467298006e621130e90c8069bd976603068398cc866f3fa75a11064` |
  | `readout-d11.json` | `6b3d9d7d72e30a78e7801bf27fd0748283b6e782a1ff39bc7742f8256ec239d6` | `bf34b819a13578653b10078ad838f92986a362bb516a9c06a2030d3d02d76409` |
  | `artifact.uorlut` | `28b01bba93bc94971a1f83c18c82b92cd4dc7f6e66af559807b7931197609054` | `1d1d2dc84e68f6b2e74eb01c43ffad011fd06e45c8556a9e78983f77aaebc566` |
  | `compare.json` | `ef61764025486123480bce6839618ae37e73b3568c2ba1fb95a57a8a98d03bc5` | `0d9484bf7697ec5b031ca2f213263d0501c8f8afa462bedef4f381bd3291fe5b` |

  | File | t1 | t3 |
  |---|---|---|
  | `snapshot.json` | `ad0e6e7efc4943b22a5741ab977dc347db25e38bf98b4c53c9a7eda6b5df352d` | `4b573575b3e1825bf69699acdd679681308fc2b2b217cf01e51d8b77b52ab296` |
  | `snapshot-writer.json` | `e8f2f2e6dc123f46f43cac1e0841ece67d5bb87290faea1f635ca139658cf710` | `0f56e872f6e52992794bed5478afca4d2d15da25e4b763f9b8dcba812e20ed4b` |
  | `readout.json` | `b668026ba00f1e1fb2cc06098990f39d9ba535107ddba6af4804cfe8b7eab64e` | `ad9b99cfb667b47fc087b36cc71d90fdc559af51f2b8e0cd658d234ca20e3f83` |
  | `readout-export.json` | `2cfa9811708384cab29abf5e99571090326341cde691db01dad56658229531d9` | `8e0bc594e61ed016e4bcd971b767b77e2b6f8e81abac13a9b245840edfce2ea4` |
  | `readout-d11.json` | `e7a50dba2495943e79e4e2199b8c9c47fa5de40e8bfb75a7059a82aba129c582` | `9d69f35e131199c599b459928ae445c10511d64cb15071c9d92ab484c0490d23` |
  | `compare.json` | `8300b7aaa87ac18f2379949118778b94f885c9350f8ff15b92d27b071632ca2b` | `8196d5f37c40d7bf9c07bce51cff331ba842a57acfcd88da3062d9bb873e6892` |

- **claude-235's integer traces,** kept on the laptop and pinned in its `reading/SHA256SUMS`: `trace-integer-s1.json` `a8dbc19ca0594182db3270e34d6ce7e27ce71085a3ddf4cb9388fb719f9bbdfc`, and `trace-integer-s3.json` `cca428dbc19948e586db002b0e01971d8feeb520750ace4796d189618a06d8cc`.
- **Data split:** none, and no corpus is read. The readouts' inputs are `fbs_float_readout`'s 4,096 single tokens and 8 fixed sequences of 192, token (37s + 11t + 1) mod 4,096. No TRAIN, DEV or panel split is opened.

## 6. Steps

**On the pod:**
1. The data, every file checked (`I-Data`).
2. The source pin at the pushed head, clean.
3. `I-Negative`: the guard set aside, T1 and T2 run and fail, the guard restored, the source clean.
4. `I-GPU-Tests` at the restored head.
5. The builds: `fbs-snapshot` (cuda), `fbs-compare` (serving-wasm).
6. Per seed: the trunk alone, its comparison, then `I-Trunk`. Then the four h arms, each with its comparison. After d11, `I-Guard-Effect`.
7. `SHA256SUMS`; the output is stored to `results/claude/integer-hamming-read-astrained-2026-11-01/` and listed back; then release and down.

**On the laptop:**

8. The stored output fetched, and checked against its sums.
9. Per seed, the integer trace (`I-Integer`), then the eight comparisons (`I-Faults`).
10. The copy codes for d11 and d11-rounded (`I-Copy`).
11. The context sweep on h1's artifact.
12. The rows, read from the stored comparisons.

## 7. Minimum test power

- **P1:** 1,536 sequence positions per seed against a bar of 46.
  - The trunk alone sits at 5 and 11.
  - claude-235's d11 sat at 694 and 823. A guard that removes the snap moves d by hundreds, far past claude-227's 2√(d(old) + d(new)) of about 54 and 59 at d = 46.
- **P2:** per cell and seed, 2,784 sets past t = 17, against at most 139 unequal.
  - claude-235's d11 counted 2,741 to 2,774 unequal sets past t = 17 in each barred cell, so the bar separates the snapped reference from parity by an order of magnitude.
  - Copy-head cells can only fall short through ties or flips at codes (a) and (b) agree on (`I-Copy`, claude-237). So a copy cell below 0.95 in d11, with its tie arm above it, reads directly as row T.
- **`I-Negative`'s power:**
  - T1 asserts that the D11 and export outputs equal the pass-through's bit for bit, on off-icosian rotations that every snap moves. It also asserts a plain hook gate under D11 differs from the pass-through, so it cannot pass vacuously.
  - T2 counts the snap captures: 1 with the guard, 5 without (the read's four rotations and the trunk's gate).
- **Out of reach:** a fault seen only on real prompts, only through the turn memory, or only on claude-146's checkpoints.

## 8. Cost and stop rule

- **Cost:** one RTX 5090. Vast's cheapest verified 5090 was $0.563/h on 31 Oct.
  - Bootstrap and builds take about 10 minutes.
  - The tests build twice, because the set-aside rebuilds the library: about 8 minutes.
  - Two trunk readouts, eight traces and ten comparisons take about 15 minutes; claude-235's eight took 11 pod-minutes in all.
  - That is about 40 minutes, under $1, with `--hours 1` and a $2 ceiling.
- **Stop rule:**
  - a failed data, build, test, `I-Negative`, `I-Trunk` or `I-Guard-Effect` check stops the job with exit 3, 4 or 6, and is never counted;
  - an arm whose tool fails is logged, and the job ends with EXIT 7 (row 0);
  - every exit copies the logs and `EXIT` into the output and seals it;
  - a watcher runs from launch to down, and the lease is renewed while the job runs. A healthy run is never cancelled for a self-set budget;
  - the laptop steps begin only after the seal is read back.

## 9. Non-author TEST FITNESS review

Pending.

## 10. Pre-run review

Pending.
