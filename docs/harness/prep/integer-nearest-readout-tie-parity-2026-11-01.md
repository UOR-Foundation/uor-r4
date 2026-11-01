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
- **S against F′:** the snap's share, with the ties now equal. It is the card's control for the snap. On a PASS row, if S against F′ also meets ⌊0.03 n⌋ in every cell, the card's line holds: the snap is shown harmless here, and either export is a served recipe. The item then records both, the raw one first. S never decides the row.
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
10. **`I-Store`:** the run's outputs are copied off the pod to the laptop, then stored at `results/claude/integer-nearest-readout-tie-parity-2026-11-01/`. Every file in the pod's `SHA256SUMS` matches on the laptop: the outputs, the binary's and tables' sums, and the logs other than the live `job.log`. The job fails (`EXIT 9`) unless the sums hold the eight G files.
11. **`I-Prompts-F′`:** F′'s G files carry R's prompts. At every index of every cell, F′'s `position`, `prompt` and `reference` equal the prompt file's. Without this, a change in F′'s sampling at this head would read as a MISS (L-007). On F, the same check reads 310 of 310 on DEV s1.
12. **`I-Decode`:** every `fbs-toks` exits 0, and its `keyed` is 0.
13. **`I-Positions`:** `positions_match` = n in every graded pair.
14. **The grader's positive probe:** F′ against itself reads n positions, and n less F′'s empty replies for `first_agree` and `whole_agree`, in every cell (F's shortest reply has 8 tokens). With `I-S-Reproduces` re-run in the same script, the grader reads known answers at both ends.

Steps 4 and 5 run as one script, `local/claude/claude-243/grade.sh`, adapted from claude-232's `log/run.sh` (its sha256 is in the pre-run review). It hard-codes every pair, reads the bar on R against F′ only, stops at the first failed check, never writes into an existing folder, and prints the row only after every check holds.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384, seeds 1–3: the sealed checkpoints for F′, and claude-227's full snapshots of them for R.
- **Data split:** DEV (`ft/dev`) and p-w (`inputs/claude/heldout-panel-p-w1200-2026-10-31/`), the prompts claude-220 scored. p-v runs inside F′'s unchanged command and is stored, not read. All three are opened panels; none decides a mechanism. p-x is not touched.

## 6. Steps

1. The checks of §4.1–7 (done).
2. **The pod:** `uor-pod status`, then one RTX 5090 up and leased for this session; `job.sh <head>`, with `local/claude/claude-243/watch.sh` armed on it.
3. **Off the pod, then the store:**
   - `uor-pod ssh POD 'tar -C /workspace --exclude tie-parity/in -cf - tie-parity' > local/claude/claude-243/run.tar` (GNU tar applies `--exclude` only to the paths after it) comes first. It carries `out/`, `repeat/`, the logs, `binary.sha256`, `tables.sha256` and `SHA256SUMS`, and leaves the fetched inputs.
   - Then it is unpacked under `local/claude/claude-243/run/` and checked with `sha256sum -c` (`I-Store`).
   - Then `scripts/pod/uor-store put` to the store folder and `uor-store ls` to confirm.
   - The pod is released and taken down only after that.
4. **R:** `grade.sh` runs `I-Prompts-F′`, then the six decodes on the laptop, one after another, with `/usr/bin/time -l` (`I-Decode`).
5. **The grades:** `grade.sh` runs the probes, then R against F′ (the bar), then the readings, each with `I-Positions`.
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
- **The probe ladder** (run file, Part 4 A.4):
  - (a) The mechanism's own check is the pod's CUDA library tests, the tie rule's by name, in minutes, before any decode.
  - (b) The throughput projection is claude-220's run of this exact command on the same GPU type.
  - Each seed runs about 2 h 45 min, past the ladder's hour per arm. Arm G itself is about 6 min on DEV and 23 min on p-w per seed (claude-220's `main-s1.out`). The rest is arms that run because the command is claude-220's unchanged.
  - Cutting them needs a new switch in `train-in-space.rs`. That is outside this item's scope, and it would end "the same command" that I-Flags and the card rest on. The lab's call is to run it whole, as claude-220 did after its own projection review.
- **The watcher** (`watch.sh`): a spend stop at $4, a time stop at 6 hours, and a stall stop at 60 unchanged minutes. Its leases are renewed every 25 minutes, and for 2 hours before any stop, so a stopped healthy run or a finished one is never reaped while the session answers. The longest silent stretch of a process is one p-w decode file, about 32 minutes with four processes. The job's trap leaves an `EXIT` file on every way out, so a crash is seen at the next check.
  - A spend or time stop wakes the session, which extends the stop for a healthy run and notes it. A healthy run is never cancelled for a self-set budget.
- **Stop rule:** any §4 check that fails stops the run as row 0, never counted. The bar is read only after every check holds.

## 9. Non-author TEST FITNESS review

TEST FITNESS: FIT

**Round 1 (a fresh non-author reviewer, 1 Nov, at `ccf1523d3`): FIT.** It decoded nothing and read no outcome.
- **What it checked:**
  - no route to a false PASS: the pairing, the empty-reply rule, the slicing, ⌊0.03 n⌋ (9 and 36), and F′'s files being arm G at this head. A decoder stuck on one token cannot pass: F's most common first token covers at most 113 of 310 and 219 of 1,202.
  - the rows cover every outcome, and match "done only when the bar passes" and "done applies to the raw export";
  - `job.sh` against claude-220's job: the same binary call, flags and panel order. Its `wait` is on the runs' own pids, and a failed run, a missing file or a repeat mismatch each end the job loudly. The eight named tests exist, and `TIE` is never set off in this path.
  - the watcher's stall rule against the longest silent stretch (about 32 against 60 minutes), and the cost;
  - re-run by hand: I-Raw, I-Identity, I-S-Reproduces (6 of 6), I-Float-Inputs, I-Inputs, I-Prompts and I-Flags.
- **Advisories, each made:**
  1. A laptop script with the pairs hard-coded: `grade.sh` (§4).
  2. `I-Prompts-F′` (§4.11).
  3. The code must not move between the checks and the run: the pre-run review records `git diff --stat ccf1523d3 <run head> -- crates Cargo.lock` empty.
  4. `job.sh`'s sums are checked (`EXIT 9`), hold the eight G files, and carry the logs.
  5. A trap leaves an `EXIT` file on every way out.
  6. Nits: the watcher's comment says 60 minutes and its paths are absolute; the positive probe allows for empty replies; the ORDER's spend line matches the sheet's $4; the REPORT records the export command line.
- **Left unchecked:** the CUDA suite at this head (the pod's step 2), the checkpoint sums on the pod (step 4), and the engine's serving of raw gates (claude-181; R's decodes exercise it).

## 10. Pre-run review

PRE-RUN: GO

Two fresh reviewers, 1 Nov, at `4a587a588`. Neither wrote the work, and neither decoded or read an outcome.

**Round 1 (the tools, inputs, scripts, FIT and caps): GO, no blocker.**
- **The code did not move since the laptop checks:** `git diff --stat ccf1523d3 4a587a588 -- crates Cargo.lock` is empty. This branch changes no crate (`git diff origin/main...HEAD -- crates` is empty), so the laptop's `fbs-toks` carries the head's code.
- **The command parses:** every flag `job.sh` passes has its parse line in `train-in-space.rs` at the head. The eight tests it greps exist (`flock_read_hamming.rs:952, 1021, 1043, 1094`; `in_space_long.rs:10787, 10879, 10919, 11113`).
- **The store holds every input** (`uor-store ls`): the driver data (5 of 5), both panels with their sums, and each seed's `SHA256SUMS`, `config.json` and the five files of `step-016384/`. The laptop holds claude-232's prompts, S and grades, R's three artifacts, F's six G files and `fbs-toks`. No output folder exists yet.
- **The scripts:** `bash -n` passes, and the data and sums lines match claude-220's job. The `wait`, the trap, the sums grep, I-Repeat, the split-to-file map and the bar arithmetic (9 and 36) read correctly. Four GPU processes need about 15 GB of the card's 32 GB.
- **The caps:** `uor-pod status`, 0 pods running, $0 of $8.00/h. The cheapest 5090 is $0.563/h, so about $2.80 for 5 hours.
- **Advisories, made:**
  1. Every watcher stop ended its lease renewals, so a stopped healthy run could be reaped about 80 minutes later. `watch.sh` now renews each lease for 2 hours before any stop (§8).
  2. `grade.sh` checks R's artifacts against `R/a7/SHA256SUMS` (the sheet's three sums) before any decode, and logs `fbs-toks`'s sha256 and the worktree head.
  3. The two seed-3 processes cannot collide: every scoring write is under its own `--out` and created new, never replaced (`in_space_long.rs:3061–5870`, `write_new` at `:5177`).
  4. Left as is: `job.sh` refuses an existing `out/` before its trap, so it writes no `EXIT`. That cannot happen on a fresh pod.

**Round 2 (the chain and what each outcome unlocks): CONFIRMED, no blocker.**
- **The chain:** the run serves the order's step 2, the readout served in integers, and step 4, one model served and scored with tok/s. Done-when 3 is the bar; done-when 4 is R s1's speed. It unblocks point-key-integer's rerun on p-x, which the item and PLAN hold until parity is rechecked. Nothing cheaper comes first: R against F alone cannot meet the bar by construction (the card).
- **Each row decides:**
  - Row 0 holds F′ back as the reference until it is repaired.
  - PASS meets done-when 3 for the raw export, and names its recipe.
  - MISS keeps the item `ready`, and moves the localization to R against F′ on the same rules.
  - The ORDER's spend line names the sheet's $4.
- **Advisories, made:**
  - The card's "either export" line on S is now in §3's readings.
  - The ORDER's quote of done-when 3 is marked as the text before the card.
  - A narrow MISS is read as possibly the bar's unmeasured margin, not an integer fault, when the next unit is scoped. The REPORT carries this.

**The scripts as run** (`local/claude/claude-243/`, not in git):
```
a52295ee4bf1104dcbcefe4784a15193547a9908fde849c1d48364284ea3ccf6  job.sh
de6a68f9bab827d7afbc15370e88d67e9967e61c0ea6607d63b9ac20f61ebce6  watch.sh
41bbeb3545aff690c499729b579b49c3b1292dd97bde8cc9b070e88413148291  grade.sh
```
