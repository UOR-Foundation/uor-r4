# Prep sheet: end-to-end token parity with both sides on the same rules (integer-nearest-readout done-when 3 and 4, claude-243), 2026-11-01

- Item: [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). ORDER: [claude-243](../../loop/tasks/integer-nearest-readout/claude-243/ORDER.md). Decision card: [DECISION.md](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), approved after two fresh rounds.
- **The run:**
  - the float reference F′ is decoded on one Vast RTX 5090 (CHARTER §3.5);
  - the integer decodes run on the laptop (Apple M1), as product serving (§3.6) under the owner's 31 Oct allowance for this chain.
  - No code changes.

## 1. Question

**With the float read keeping ties as the integer read does, and the integer trunk unsnapped, does the integer artifact of (r-drop) decode as the trained float model does?**
- claude-232's MISS (S against F) mixed two known differences with any integer fault: the float reference's tie order and the integer side's icosian snap (the card's "What is measured").
- This run removes both, and reads each one's share beside the bar.

## 2. Arms

- **R, integer greedy, raw:** (r-drop) step 16,384 at seeds 1–3.
  - Exported by `fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT a7r-sN.uorlut` from claude-227's full snapshots (`results/claude/integer-retention-rates-parity-2026-10-31/out/full/snapshot-sN.json`: `dcbfb69b…`, `d0522d02…`, `9744d385…`), with the configs at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/sN/r-drop-sN/config.json`. Done on 1 Nov as the command-form check, before any decode: `f86a7856…`, `0a69a009…` and `66ae49de…`, 51,310,784 bytes each.
  - Decoded by `fbs-toks a7r-sN.uorlut --replies PROMPTS --out FILE --reply-tokens 64` on claude-232's sealed prompt files (`prompts/prompts-{dev,p-w}-sN.json`), as claude-232 decoded S.
- **S, integer greedy, snapped:** claude-232's sealed replies, `results/claude/integer-nearest-readout-parity-2026-10-31/R/r-drop-sN/integer-greedy-{dev,p-w}.json`, read as stored.
- **F′, float greedy at this unit's head:** claude-220's arm G ("greedy, memory on") by claude-220's command, on the GPU:
  - `train-in-space --data-root DATA --score-rows CKPT --out DIR/r-drop-sS --samples 1536 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1 --served-key 1 --panel p-w=PW --panel p-v=PV --seed S`;
  - on the sealed checkpoints `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/sS/r-drop-sS/models/step-016384` and the same panels, every sha checked as claude-220 checked them;
  - only `samples-memory.json` (DEV) and `samples-p-w-greedy-memory.json` (p-w) are read. The other arms run because the command is claude-220's unchanged; they are stored, not read.
  - At this head the read's tie rule is on (`flock_read_hamming.rs`, `TIE`, since claude-239).
- **F, float greedy as stored:** claude-220's G files, `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/out/r-drop-sN/`, on the laptop at `local/claude/claude-220/run-eb762e60/served-key/out/r-drop-sN/`.

## 3. Bar and decision table

For each cell (split ∈ {DEV, p-w}, seed ∈ {1, 2, 3}), two arms are paired by index; their `position`s must match (§4). Then:
- **First-token agreement:** the integer `reply[0]` equals the float `reply[0]`. An empty integer reply disagrees.
- **Whole-reply agreement (reported, no bar):** the integer reply is non-empty and equals the float reply cut to its length.

The registered grader, claude-232's unchanged, run per cell on the stored files (`I` the integer or first arm, `F` the float or second):

```text
jq -n --slurpfile i I --slurpfile f F '($i[0]|length) as $n
  | if ($f[0]|length) != $n then error("counts differ") else . end
  | [range(0; $n) as $k | $i[0][$k] as $a | $f[0][$k] as $b
     | {p: ($a.position == $b.position), first: (($a.reply|length) > 0 and $a.reply[0] == $b.reply[0]),
        whole: (($a.reply|length) > 0 and $a.reply == $b.reply[0:($a.reply|length)])}]
  | {n: $n, positions_match: ([.[]|select(.p)]|length), first_agree: ([.[]|select(.first)]|length),
     whole_agree: ([.[]|select(.whole)]|length)}'
```

**The bar is read on R against F′ only.**

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4) | Nothing is counted. | Repair, then rerun. |
| PASS | In **every** cell, R's first-token disagreement with F′ (n − `first_agree`) is at most ⌊0.03 n⌋: 9 of 310 on DEV, 36 of 1,202 on p-w | Done-when 3 is met for the raw export; the item is done with done-when 4 and 5's records, and names `fbs-export --raw-gates --raw-transition` as the served artifact's recipe. | point-key-integer's rerun on p-x, on the raw export; a follow-up on making raw the default or the snap faithful. |
| MISS | Any cell over, however narrow | The item stays `ready`, and the row is recorded. Never a mechanism verdict. | The next unit locates the gap on R against F′, now on the same rules, at the first disagreeing prompts, layer by layer (trunk, Hamming read, memory, readout). |

**Why 3%, and its margin without the snap:** it is claude-232's bar, registered against claude-227's snapped residual (0.33% of sequence positions, 1.78% of single tokens, against a D11 forward that snaps too). The raw export's rounding residual against the trained forward is not measured. The bar is kept, not re-derived (the card's constraint). A first token follows up to 186 positions, with the memory and the Hamming read on top.

**Readings, no bar,** each graded with the same grader:
- **S against F′:** the snap's share, with the ties now equal. It is the card's control for the snap.
- **F′ against F:** the code's change on G's path since `eb762e602`. By the code that is the tie rule alone (the card lists the six commits), but `train-in-space` has no switch to test it, so the reading is not credited to the tie rule.
- **R against F:** the raw export against the stored reference.
- **Whole-reply agreement** of every pair, and R's capped count (expected 26 of 310 on DEV and 136 of 1,202 on p-w per seed, as S's).

## 4. Instrument checks (any failure is row 0)

Checked on 1 Nov, before this sheet's FIT:
1. **`I-Float-Inputs`:** F's six files on the laptop equal claude-220's `SHA256SUMS`: 6 of 6 OK.
2. **`I-Inputs`:** claude-227's three snapshots, the three configs, and claude-232's 37 sealed files other than its three `a7` artifacts equal their store `SHA256SUMS`: all OK.
3. **`I-S-Reproduces`:** the grader on S against F gives claude-232's six sealed grade files exactly (for example DEV s1: n 310, 310, 291, 83): 6 of 6.
4. **`I-Flags`:** `git diff eb762e602 <head> -- crates/uor-r4-float-train/src/bin/train-in-space.rs` changes no parse line of the flags F′'s command passes (`command grep` over the diff for each flag's parse arm: no hits). It removes `--opening-table`, which claude-220's command never passed, and adds two modes that are off by default.
5. **`I-Raw`:** each R artifact's header carries `"head":"native_vector_nearest"`, `"rec_gates":"geometric-retention"`, `"raw_gates":true`, `"raw_transition":true` and `"schema":"uor-r4.lut-stack/7"`: 3 of 3.
6. **`I-Identity`:** each export's printed sha256 equals its file's: 3 of 3.
7. **`I-Prompts`:** 310 DEV and 1,202 p-w prompts at every seed: 3 of 3.

Checked in the run:
8. **`I-Pod`:** the pod job (`local/claude/claude-243/job.sh`, its sha256 recorded in the REPORT) ends `EXIT 0`:
   - the source is pinned to the sheet's head and clean;
   - the crate's CUDA library tests pass, the read's four tie tests and claude-220's four served, table, forced and rank tests among them by name;
   - every panel and checkpoint matches its sums, and the panels match the pinned shas;
   - every file claude-220's run wrote is written at every seed.
9. **`I-Repeat`:** seed 3's second process, beside the three on the same GPU, writes `samples-memory.json` and `samples-p-w-greedy-memory.json` equal to the main run's, byte for byte. A difference is row 0: GPU greedy is then no reference.
10. **`I-Store`:** the run's outputs are stored at `results/claude/integer-nearest-readout-tie-parity-2026-11-01/` and read back to the laptop, every sha256 equal to the pod's `SHA256SUMS`.
11. **`I-Decode`:** every `fbs-toks` exits 0, and its `keyed` is 0.
12. **`I-Positions`:** `positions_match` = n in every graded pair.
13. **The grader's positive probe:** F′ against itself reads n, n, n and n in every cell. With `I-S-Reproduces`, the grader reads known answers at both ends.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384, seeds 1–3: the sealed checkpoints for F′, and claude-227's full snapshots of them for R.
- **Data split:** DEV (`ft/dev`) and p-w (`inputs/claude/heldout-panel-p-w1200-2026-10-31/`), the prompts claude-220 scored. p-v runs inside F′'s unchanged command and is stored, not read. All three are opened panels; none decides a mechanism. p-x is not touched.

## 6. Steps

1. The checks of §4.1–7 (done).
2. **The pod:** `uor-pod status`, then one RTX 5090 up and leased for this session; `job.sh <head>`, with `local/claude/claude-243/watch.sh` armed on it.
3. **The store:** `/root/uor-store put` of the job's `out/`, `repeat/`, logs, `binary.sha256`, `tables.sha256` and `SHA256SUMS`; then the G files and the sums fetched to the laptop and checked (`I-Store`). The pod is released and taken down only after that.
4. **R:** the six decodes on the laptop, one after another (`I-Decode`), with `/usr/bin/time -l`.
5. **The grades:** R against F′ (the bar), then the readings, each with `I-Positions`, and the positive probe.
6. **Speed (done-when 4):** `fbs-toks a7r-s1.uorlut` alone, the plain protocol (1 warmup and 20 timed runs of a 128-token prefill and 64 decode steps, the nearest scan inside each step), under `/usr/bin/time -l` for the peak resident memory, on AC power, with the machine's load recorded.
7. **Seal:** R's artifacts' sums, its replies, the grades and the logs to the same store folder, read back.

## 7. Minimum test power

- 310 and 1,202 prompts per seed, at three seeds: 4,536 first tokens.
- At a true residual of 1%, a cell expects about 3 (DEV) and 12 (p-w) disagreements, against bars of 9 and 36.
- S against F′ and R against F′ are paired on the same prompts, so the snap's share is read prompt by prompt.

## 8. Cost and stop rule

- **Cost:**
  - claude-220's three seeds took 9,840 to 9,995 s side by side on one 5090, at 92.6% GPU use over its busy rows and 11,354 MiB at the peak.
  - I-Repeat's fourth process puts the run near 3 h 40 min, plus about 25 min for tests, build and data: about 4 h 05 min.
  - At about $0.56/h, that is about $2.30. The laptop side takes minutes.
- **The watcher** (`watch.sh`): a spend stop at $4, a time stop at 6 hours, and a stall stop at 60 unchanged minutes. Its leases are renewed every 25 minutes.
  - A spend or time stop wakes the session, which extends the stop for a healthy run and notes it. A healthy run is never cancelled for a self-set budget.
- **Stop rule:** any §4 check that fails stops the run as row 0, never counted. The bar is read only after every check holds.

## 9. Non-author TEST FITNESS review

To be filled in.

## 10. Pre-run review

To be filled in.
