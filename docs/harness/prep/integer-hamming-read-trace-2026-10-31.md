# Prep sheet: trace the Hamming read, integer against its float D11 forward, with the tie and rounding arms (integer-hamming-read, claude-235), 2026-10-31

- Item: [integer-hamming-read](../../roadmap/items/integer-hamming-read.md) (#3385), step 5, as claude-234 recorded it. ORDER: [claude-235](../../loop/tasks/integer-hamming-read/claude-235/ORDER.md).
- **The run:** one GPU job on one leased RTX 5090, `local/claude/claude-235/job.sh`. It is not in git, and its copy is stored with the outputs. It runs against the pushed head. The integer side and the timing run on the laptop, from the stored outputs, as serving does.

## 1. Question

claude-233 (#3465, row R) measured the gap on (r-drop) step 16,384. The integer trunk alone agrees with its float D11 forward on all but 5 (s1) and 11 (s3) of 1,536 sequence positions. With the Hamming read, 694 and 823 differ.

**Which of the read's kept sets differ, per (position, head, read), and why?** Each differing set is a tie across the 16th place or a sign flip, of the query or of a key. **Does either lead close the gap?** The two leads are:
- the float read breaking ties as the integer session does;
- the read's parameters rounded as the artifact stores them.

**Out of the question:**
- claude-233's second gap, the snapped D11 forward against the trained forward. Both sides here are the snapped D11 forward, so it cancels.
- The turn memory.
- Real prompts.

## 2. Arms

Two seeds × four arms on the GPU. Each arm is a D11 readout of (r-drop) step 16,384 with its Hamming read and without the turn memory, from the same copies as claude-233's h1 and h3:

| Arm | Tie rule | Read parameters |
|---|---|---|
| d11 | the GPU sort's own (main's read) | f32 as stored (main's read) |
| d11-tie | the integer session's: ascending bits, then ascending `s` (key `h·T + s`) | f32 |
| d11-rounded | the sort's own | rounded as exported: rotations and weights at 2^-24, margins at 2^-16 |
| d11-tie-rounded | the integer session's | rounded |

**Per arm and seed, on the pod:**
1. `fbs-snapshot --hamming-trace ARM MODEL_DIR readout.json trace.json` writes the D11 readout and the float trace of its 8 sequences. The trace holds each position's query and key codes and kept positions, per head and read.
2. `fbs-compare --parity-only` of claude-233's writer snapshot, plain and export readouts, with this arm's D11 readout. This gives claude-233's count, d = 1,536 − agreements.

**On the laptop, from the stored files:**
- **The integer side has no arms**, since its tie rule and its rounding are fixed. Per seed, `fbs-toks ARTIFACT --hamming-trace readout-d11.json trace-integer.json`, on claude-233's h_s artifact, steps the same 8 sequences.
- **The comparison:** `fbs-toks --hamming-trace-compare trace-ARM.json trace-integer.json` for each arm. It reports, per (read, head kind):
  - the sets judged and equal, and top-1;
  - ties, query flips, key flips, both, and instrument faults;
  - the readout type-head sets downstream of a stream-read difference.

  It follows the item's step 5 rule.
- **The timing:** `fbs-toks ARTIFACT --context-sweep` on h1's artifact. It times decode tokens/s against context, with the read and without (`IntegerStackModel::without_hamming_read`). The prompts are its own default prompt's first 32, 64, 96 and 128 tokens, each followed by 64 greedy decode steps, with 1 warm-up and 20 timed runs (median and p95). This is a reading, not a bar.

## 3. Bar and decision table

**Bar:** claude-233's, d ≤ 46 of 1,536 sequence positions (3%).
- An arm **closes** the gap at a seed when its d ≤ 46.
- It **moves** the gap when d(d11) − d(arm) > 2√(d(d11) + d(arm)), claude-227's test (b).

The single tokens reach no read (claude-233: 0 of 4,096 moved), so they are reported only.

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4) | Nothing. | Repair, then rerun. |
| T | d11-tie closes at both seeds | The cause is the order among ties at the 16th place. | The tie rule becomes the float readout's, in the port (`flock_read_hamming.rs`). Then the parity bar on claude-146's checkpoints and on (r-drop), the item's done-when 2. |
| Q | d11-rounded closes at both seeds, and d11-tie does not | The cause is the read's rounding in the artifact. | The exporter stores the read on finer grids, or the readout rounds as exported. Then done-when 2. |
| B | only d11-tie-rounded closes at both seeds | Both causes. | Both fixes, then done-when 2. |
| F | no arm closes at both seeds | No lead alone closes it. | The cause comes from d11-tie-rounded's judged differing sets (see below). |

**Row F's next step,** by the most common cause:
- **Type-head query flips:** the trunk residual sits near a code's sign. The next step is the BitNet b1.58 lesson: train (r-drop) with the D11 forward in the loop (`with_d11_training`). claude-130 and claude-131 closed only those configurations (§4.1).
- **Copy-head flips or key flips,** which depend only on the table and the read's rotations: the rotation arithmetic, or a fault. The rounded arm puts only the parameters on the integer grid. The float still normalizes `g/|g|` in f32 and rotates by `hamilton_gate`, while the integer uses `stack_rotation` and `stack_hamilton` through D11's tables, so a coordinate near 0 can flip from arithmetic alone. The next unit tells the two apart.

The rows are read in order, and the first that holds applies. Outside row 0 they are exhaustive.
- **In every row, each arm's d and whether it moves the gap are reported.**
- **A lead that moves the gap without closing it** is named a partial cause.
- **An arm that closes at one seed only** does not count as closing. The rows are then read as usual, and both seeds' counts are reported.

## 4. Instrument checks (any failure is row 0)

1. **`I-Data`:**
   - each seed's six checkpoint files pass the run's own sums;
   - claude-233's h1 and h3 files pass its sealed `SHA256SUMS` (§5). Those files are the writer snapshot, the plain, export and D11 readouts, and the artifact.
2. **`I-GPU-Tests`:** the `flock_read_hamming::` and `fbs_export::` CUDA tests pass at the head, with at least one run. They include:
   - `recording_the_trace_leaves_the_read_unchanged`;
   - `the_tie_rule_keeps_the_integer_sessions_order`.
3. **`I-Repro`, the ORDER's done-when 1 end to end.** Each seed's d11 readout is byte-identical to claude-233's h_s `readout-d11.json`. Its `compare.json` is byte-identical to claude-233's (`ef617640…`, `0d9484bf…`), which carries d = 694 and 823 and claude-233's `artifact_sha256`.
   - The code is unchanged between claude-233's run head (`ecabcc93b`) and this unit's base (`7a1cbcffc`): only three crate READMEs differ. So the float readout pins the float read, and the comparison pins the integer read, each with this unit's switches off.
   - A mismatch means the capture or a switch is not bit-identical when off. The job stops there (the ORDER's stop rule).
   - The d11 comparison runs before the check, so the mismatch is recorded.
4. **`I-Trace-Shape`:**
   - each float trace holds 8 windows at the read depths, with 4 heads keeping 16 (`fbs-snapshot` refuses otherwise);
   - the integer trace's windows carry the same ids, reads and lengths (the comparison refuses otherwise).
5. **`I-Faults`:**
   - every arm's comparison counts 0 instrument faults;
   - the two tie arms count 0 ties. With one tie rule on both sides, equal codes give equal sets, so a tie there means the float rule is not the integer's.
   - **Two known ways to reach row 0 here, neither a false row:**
     - the comparison expects min(16, t − 1) kept positions, every earlier position a candidate. `fbs-snapshot` sets neither `set_read_cut` nor `set_row_cut`, so `row_limits` gives every row its full window;
     - a float coordinate of exactly 0 packs as bit 1, while its distance sits on a half step, so a tie arm could count a tie. That is very unlikely.
6. **`I-Finite`:** every tool exits 0.
   - An arm whose tool fails is logged, and the others still run. The job ends with EXIT 7.
   - A missing comparison or trace is row 0.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384 at s1 and s3, from `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/`, with its run configs (`retain_rates: true`, `hamming_read: 4`). Each is copied without the turn memory, as claude-233's h1 and h3 were.
- **claude-233's sealed outputs,** `results/claude/integer-nearest-readout-locate-2026-10-31/out/`:

  | File | h1 | h3 |
  |---|---|---|
  | `snapshot-writer.json` | `b1b534a0ebeecda270293097ce345d0b5b41e816a1cd4496c882407989d2308f` | `238d10cdb5e75be2b81b6e8a93e61c62cfc2f8ceb51b73e7ec655b37c7f367ae` |
  | `readout.json` | `aa498cdcb86e33e8fb7e1850e29d63070f757f0d67a5c07ce1fbfbf00dac376c` | `87da7d87a7f9fbb54c4aa541e62b3b45a3e79fa46a5b8c4828fc1012c43b22d9` |
  | `readout-export.json` | `2380842b4d8293f8a92ab595cea15ea559b4a37a492df657bff3ec59845c14d4` | `538fe38d6467298006e621130e90c8069bd976603068398cc866f3fa75a11064` |
  | `readout-d11.json` | `6b3d9d7d72e30a78e7801bf27fd0748283b6e782a1ff39bc7742f8256ec239d6` | `bf34b819a13578653b10078ad838f92986a362bb516a9c06a2030d3d02d76409` |
  | `artifact.uorlut` | `28b01bba93bc94971a1f83c18c82b92cd4dc7f6e66af559807b7931197609054` | `1d1d2dc84e68f6b2e74eb01c43ffad011fd06e45c8556a9e78983f77aaebc566` |

- **Data split:** none, and no corpus is read. The readouts' inputs are `fbs_float_readout`'s 4,096 single tokens and 8 fixed sequences of 192, token (37s + 11t + 1) mod 4,096. No TRAIN, DEV or panel split is opened.

## 6. Steps

**On the pod:**
1. The data, every file checked (`I-Data`).
2. The source pin at the pushed head.
3. The CUDA tests (`I-GPU-Tests`).
4. The builds: `fbs-snapshot` (cuda), and `fbs-compare` (serving-wasm).
5. Per seed: arm d11, its comparison, then `I-Repro`; then the other three arms, each with its comparison (`I-Finite`).
6. `SHA256SUMS`; the output is stored to `results/claude/integer-hamming-read-trace-2026-10-31/` and listed back; then the pod goes down.

**On the laptop:**

7. The stored output fetched, and checked against its sums.
8. Per seed, the integer trace, then the four comparisons (`I-Trace-Shape`, `I-Faults`).
9. The context sweep on h1's artifact.
10. The rows, read from the stored comparisons.

## 7. Minimum test power

- **1,536 sequence positions per seed,** against a bar of 46. claude-233's gap of 694 and 823 is 15 to 18 times the bar.
  - A lead that removes the cause shows as d falling toward the trunk's own 5 and 11.
  - A cause half removed moves the gap by 347 and 412, far past 2√(d(d11) + d(arm)), about 65 and 70.
- **Sets per seed:** 8 windows × 192 positions × 2 reads × 4 heads = 12,288 kept sets. Of these, the readout's type-head sets after a stream-read difference are counted apart. Positions 0 and 1 have empty sets and count as equal.
- **Out of reach:** a fault seen only on real prompts, or only through the turn memory, cannot show here.

## 8. Cost and stop rule

- **Cost:** one RTX 5090. Vast's cheapest verified 5090 was $0.563/h on 31 Oct.
  - The bootstrap and builds take about 10 minutes, and the CUDA tests a few more.
  - Eight readouts and eight comparisons take under a minute each. claude-233's four took 16 minutes of pod time in all.
  - That is about 40 minutes, under $1, with `--hours 1` and a $2 ceiling.
- **Stop rule:**
  - a data, build, test or `I-Repro` check that fails stops the job (exit 3, 4 or 6) and is never counted;
  - an arm whose tool fails is logged, and the job ends with EXIT 7 (row 0);
  - every exit copies the logs and `EXIT` into the output and seals it;
  - a watcher runs from launch to down, and a healthy run is never cancelled for a self-set budget;
  - the laptop steps begin only after the seal is read back.

## 9. Non-author TEST FITNESS review

**Round 1** (a fresh reviewer, at `b1d4a9ad2`): no blocker.
- **It checked:**
  - the arms against `trace_arm` and main's D11 path, so I-Repro's byte identity is sound;
  - the comparison rule against step 5;
  - that the tie arms must show 0 ties. Equal codes on both sides and one key (bits, s) cannot keep a position on one side only, and the float's mean L1 between ±1 codes is exactly 2h/width.
- **Its six minor points, all taken:**
  1. the power figure is now 65 and 70, not 50 (§7);
  2. an arm closing at one seed only no longer forces row F, so row T stays reachable (§3);
  3. row F's copy and key flips now name the rotation arithmetic as well as a fault (§3);
  4. and 5. the full-window assumption and exactly-zero codes are noted under `I-Faults` (§4);
  6. the REPORT leads with tokens/s.
- **Added after it:** I-Repro also pins each comparison byte for byte (§4.3).
- **Left to the pre-run review:**
  - `fbs-toks`'s argument parsing;
  - `stack/hamming.rs`'s `record`;
  - claude-233's 0 of 4,096 single tokens moved.

TEST FITNESS: FIT

## 10. Pre-run review

Pending.
