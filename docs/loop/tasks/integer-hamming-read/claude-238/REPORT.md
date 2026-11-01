# REPORT: integer-hamming-read, claude-238 (the read run as trained, and its parity)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). Sheet: [integer-hamming-read-astrained-2026-11-01](../../../harness/prep/integer-hamming-read-astrained-2026-11-01.md). References #3385.

## Tokens per second

`fbs-toks --context-sweep` on claude-233's h1 artifact (`28b01bba…`), on this laptop: an Apple M1, 8 cores, 16 GiB. It ran on mains power at a load average of 3.3–4.8 (`sweep-load.txt` and the run's own `uptime`), so read these as this machine's numbers under load, not its best. Each point decodes 64 tokens after its prefill; the figures are medians of 20 timed runs.

| Context (prefill + 64) | Decode, with the read | Decode, without | The read's cost | Prefill, with / without |
|---|---|---|---|---|
| 96 | 3,440 tok/s | 4,306 tok/s | 20% | 3,515 / 4,314 |
| 128 | 3,068 | 4,563 | 33% | 3,237 / 4,672 |
| 160 | 4,085 | 4,857 | 16% | 4,240 / 4,855 |
| 192 | 3,998 | 4,826 | 17% | 4,228 / 4,792 |

The artifact is claude-235's, so this repeats its reading (13–17%, 3,812–4,109 tok/s with the read) under today's load. The 128 point is noisy: its prefill's p95 is 35 ms against a median of 20 ms. 192 is the readout window, so the sweep says nothing about long contexts. The guard changes the GPU's float reference only; the served artifact and its speed are untouched.

## What did not work, first

- **The float forward as trained does not meet parity** (the d11 arm). d is 174 of 1,536 at s1 and 243 at s3, against a bar of 46. Its copy-head cells keep 47–48% of their sets past t = 17 equal, against a bar of 95%. So the item's done-when 2 is not met against the checkpoint's own forward. It holds only with one tie rule on both sides (row T, below).
- **The cause is the order among ties, and it is large.** With every copy code now equal on both sides, 1,448 to 1,567 of each cell's 3,072 sets in the d11 arm hold a tie across the 16th place. The GPU arg-sort, whose order among ties is not specified, breaks them differently from the integer session's rule (ascending bits, then ascending `s`). The two float arms differ in top-1 at no fewer than 168 positions (s1, 174 − 6) and 216 (s3, 243 − 27). So the trained readout depends, at 11–14% of positions, on an order the trainer never fixed.
- **Small differences remain with the tie rule matched**, all type-head query flips: 8 sets at the first read at both seeds, and 11 (s1) and 26 (s3) at the second. The arms count 0 ties, 0 key flips and 0 faults, and every copy code is equal. So they come from the type heads' only trunk-dependent input, the query from the residual. The trunk alone already differs at 5 and 11 positions. This is read from the counts; no residual coordinate was traced.
- **The pod had two GPUs; the sheet registered one.** I left `--count 1` off `uor-pod up`, so Vast billed 2 × $0.563/h. The job used one. Up about 15 minutes, so about $0.28 against the sheet's $2 ceiling.
- **claude-235's two tie-switch tests were not written.** The item's step 5 asked this pod to run them, but they never existed (claude-235's Still open). The run shows the switch acts: the tie arms count 0 ties where d11 counts about 1,450 per cell. Still, no CUDA test runs `read()` with the rule on and checks the traced order, and `the_tie_rule_keeps_the_integer_sessions_order` has had no set-aside check.
- **Two record slips on the laptop, neither touching a number.**
  - `read.sh`'s per-seed log line says "two copy-code checks written" while it wrote four, one per arm (the files and `SHA256SUMS` hold all four).
  - `reading/read.log` took its last line ("done") after the folder's sums were taken, as claude-235's did. It is the one file that fails them.
- **Out of reach, as registered:** claude-146's checkpoints (done-when 2's named reference), DEV, real prompts and the turn memory. This is (r-drop) step 16,384 on claude-233's 8 fixed sequences.

## What changed

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`:
  - **The guard** (`as_trained`, commit `148f6cb5a`): every gate call of the read runs inside `hamilton_sheet::with_d11_snaps(D11Snaps { gates: false, transition: is_snapping_transition() }, …)`. These are the key, query, value and step `rotate`s, and the step path's direct `hamilton_gate`. Inside, the arm is `HamiltonActive`, the trainer's pass-through, and the export flag is off. `with_d11_snaps` gives back the arm, call index, export flag, transition snap and telemetry flag afterwards, so the trunk's later gates snap and number as before.
  - **Two CUDA tests:**
    - **T1,** `the_read_runs_as_trained_under_the_d11_and_export_gates`: a read with off-icosian rotations gives the same output, bit for bit, under `with_d11_mode`, under `with_d11_export_gates` and under `with_gate_mode(HamiltonActive, …)`. The test also asserts that a plain gate under D11 differs from the pass-through, so it cannot pass vacuously.
    - **T2,** `the_trunks_gates_after_the_read_still_snap`: under D11, a gate after the read still snaps, and the snap capture holds 1 entry, where it holds 5 without the guard.
  - **The doc comment** names every arm (review finding 1, commit `829bd87dd`).
- **The prep sheet**, its two FIT rounds (NOT FIT, then FIT) and its pre-run review (GO), each a fresh reviewer, with every point taken (`3ff95f8b5`, `ab1f713ec`, `d4af1adec`; #3385 6554654953).
- **The ORDER's stop-rule line** on the trunk-only d, amended after the FIT. It now reads it as a path or toolchain fault, not a guard leak, and names T2 as the leak check.
- **The item's step 5** records this result and the next step. **The notebook row** is in `m4-serving.md`.

## Measurements

**d of 1,536** (the comparison's `sequences`; top-1 agreement in brackets):

| Arm | s1 | s3 |
|---|---|---|
| d11 | 174 (88.7%) | 243 (84.2%) |
| d11-tie | **6 (99.6%)** | **27 (98.2%)** |
| d11-rounded | 174 | 242 |
| d11-tie-rounded | **6** | **27** |
| trunk alone (t) | 5 | 11 |

The single tokens agree at 4,023 and 3,992 of 4,096 in every arm, the same as the trunk alone. So the read moves none of them, as claude-233 found.

**The move test** (claude-227: d(old) − d(new) > 2√(d(old) + d(new)), against claude-235's figures):
- **d11:** s1 694 → 174 moves by 520 against 59, and s3 823 → 243 by 580 against 65. Both moved.
- **d11-tie:** s1 696 → 6 moves by 690 against 53, and s3 816 → 27 by 789 against 58.

**The kept-set share past t = 17**, (equal − 288) / (judged − 288). The bar is 0.95 per cell and seed; the fourth cell also needs at least 278 judged sets past t = 17.

| Cell | d11 s1 | d11 s3 | d11-tie s1 | d11-tie s3 |
|---|---|---|---|---|
| first read, copy | 0.480 | 0.477 | **1.000** | **1.000** |
| first read, type | 0.461 | 0.434 | **0.997** | **0.997** |
| second read, copy | 0.475 | 0.475 | **1.000** | **1.000** |
| second read, type (judged past 17; downstream) | 0.889 (18; 2,766) | 0.875 (16; 2,768) | **0.993 (1,570; 1,214)** | **0.983 (1,494; 1,290)** |

The rounded arms give their unrounded twins' counts in every cell, except that s3 d11-rounded's d is 242. The comparisons' own top-1 (the nearest kept member) for d11-tie is 3,040 of 3,040 at both copy cells, 3,037 of 3,040 at the first read's type heads, and 1,821 of 1,826 (s1) and 1,741 of 1,750 (s3) at the second's.

**The causes** of the judged sets that differ:
- **d11:** ties (1,448–1,567 per cell), plus the type-head query flips below. 0 key flips, 0 faults.
- **The tie arms:** query flips only: 8 at the first read's type heads at both seeds, and 11 (s1) and 26 (s3) at the second's. 0 ties, 0 key flips, 0 faults.

### The instrument checks (the sheet's §4): all hold

- **`I-Data`:** every checkpoint file passed the run's sums, and all 24 of claude-233's h and t files passed the sheet's pins, on the pod. On the laptop, the pod's 155 output files and claude-233's two artifacts passed again.
- **`I-GPU-Tests`:** at `d4af1adec`, after the restore, 43 `flock_read_hamming::` and `fbs_export::` tests passed and 0 failed. The five named tests all ran ok: T1, T2, `the_copy_code_snap_is_the_gpu_hooks_own`, `recording_the_trace_leaves_the_read_unchanged` and `the_tie_rule_keeps_the_integer_sessions_order`.
- **`I-Negative`:** with `guard.patch` (`f0e91ea6…7d40`) reversed, T1 and T2 ran alone and both failed by name, in a run of 2 after a rebuild of `uor-r4-float-train`. T1 failed at "the D11 mode" (the output under D11 differed from the pass-through), T2 at its capture count. The guard was restored, the tree was clean against `HEAD`, and `HEAD` was the pinned head before the tests and the builds.
- **`I-Trunk`:** at both seeds, the trunk alone's `snapshot.json`, its three readouts and its `compare.json` are claude-233's byte for byte (`out/i-trunk.txt`). So the trunk's path and the pod's toolchain are claude-233's.
- **`I-Guard-Effect`:** each seed's d11 readout differs from claude-233's (`8c0c1b92…` against `6b3d9d7d…`, and `9487a1a6…` against `bf34b819…`; `out/i-guard-effect.txt`).
- **`I-Integer`:** both integer traces are claude-235's byte for byte (`a8dbc19c…`, `cca428db…`; `reading/i-integer.txt`).
- **`I-Copy`, on d11-rounded and on every arm:** at both seeds and both reads, every copy-head query (3,072) and key (3,056) in the float trace equals the CPU float control (a). That is 0 differing bits, every value bin empty, and 0 inconsistent pairs. The control equals the integer session's codes at all 8,192. Against the snapped control (a′), the float queries now agree at only 172 and 10 (s1) and 328 and 15 (s3) of 3,072, claude-237's counts for (a) before the guard. So the reference computes the read as trained, and the ORDER's done-when 3 instrument check holds with no arithmetic allowance used.
- **`I-Faults`:** 0 instrument faults in all 8 comparisons; 0 ties in both tie arms.
- **`I-Finite`:** every tool exited 0; the job ended with EXIT 0, and `read.sh` exited 0.

### The sheet's decision: row T

Rows 0 and S do not hold: every instrument check passed, and `I-Copy` found no difference. Row P does not hold: d11 misses P1 at both seeds, and P2 in every cell. **Row T holds:** d11-tie meets P1 (d 6 and 27 ≤ 46) and P2 (all four cells ≥ 0.95, the fourth on 1,570 and 1,494 judged sets) at both seeds. d11-tie-rounded meets parity too, with the same counts. Those are the two arms that meet parity.

So the integer port computes the read the float forward computes, once the two sides break ties the same way. The rounded arms change no code, and the read's grid is not a cause.

**Next, the sheet's row T:** one tie rule for both sides.
- The trainer takes the integer session's deterministic rule, ascending bits then ascending `s`, by default. The switch exists, off by default, as `set_hamming_tie_rule`.
- The two tie-switch CUDA tests owed since claude-235 come with it.
- Then the bars run again. That means claude-146's checkpoints (done-when 2's named reference, row P's next) and (r-drop), whose readout with the rule on is the d11-tie arm measured here.

Retraining is not in that step. A checkpoint trained under the GPU's tie order moved 11–14% of its top-1 decodes when the order changed. Whether it loses quality that way is a separate question, measured by the one-model path's own scoring, not by parity.

### Cost and records

- **The pod:** vast-58628933, two RTX 5090s at $0.563/h each, up about 15 minutes (lease from 06:54:07Z, deleted about 07:09Z), so about $0.28.
  - The job ran 06:56:59–07:07:11Z (`job.log`, `gpu.csv`), and the watcher saw it finish at 07:07:36Z (`watch.log`).
  - The lease was released, and the pod was deleted once its results were listed back from the store. `uor-pod down` could not archive over SSH, so it ran again with `--confirm-archived`. No pod is running.
- **The store,** `results/claude/integer-hamming-read-astrained-2026-11-01/`:
  - `out/`, the pod's sealed output, with `job.sh` and `guard.patch` inside it. Its `SHA256SUMS` is `c1d19bec…75d0`, and it read back 155 of 155 OK.
  - `bin/`, the two binaries (`fbs-snapshot` `2781763a…`, `fbs-compare` `e6a02bff…`), matching `out/binaries.sha256`.
  - `laptop/`: `reading/`, `read.sh`, `watch.sh`, `watch.log` and the review note. Its `SHA256SUMS` is `b57c2ae5…a654`.
- **The reviews:** the FIT, its two rounds, and the pre-run review, GO (#3385, 6554654953).

## Checks run at the head

- **On the pod, at `d4af1adec`:** `I-Negative`, then `I-GPU-Tests` (43 passed), then the builds and the ten readouts (`out/neg-tests.out`, `out/tests.out`).
- **On the laptop, at `d4af1adec`:**
  - `cargo test --release --locked -p uor-r4-float-train --lib --no-fail-fast`: 194 passed, 0 failed;
  - `uor-harness loop gates --full`: 11 gates, all GREEN (`local/claude/gates/1793516442-10955`);
  - the CUDA code type-checks with the stub nvcc (`cargo check --features cuda --bin fbs-snapshot --lib --tests`): 0 errors;
  - `cargo clippy -p uor-r4-float-train --lib --tests`: no warning in `flock_read_hamming.rs`. The crate's six are in other files, on main;
  - `rustfmt --check` on `flock_read_hamming.rs`: 20 diffs, the same 20 as main's copy of the file, line for line after this unit's insertions. None is in this unit's lines.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`.

## Skills used

None.

## Still open

- **One tie rule for both sides** (row T's next, the item's next step): the trainer and every float reference take the integer session's rule by default. The two tie-switch CUDA tests come with it.
- **Then parity on claude-146's checkpoints,** done-when 2's named reference, and on DEV.
- **The type heads' residual query flips** (8 at the first read, 11 and 26 at the second) are not traced to a coordinate. They are well inside the bars, and the trunk alone differs at 5 and 11 positions.
- **The timing at long contexts:** the sweep stops at 192, the readout window. A longer sweep on an idle machine is still owed (done-when 3).
- **The item stays `ready`;** done-when 2 and 3 remain open.

## Plain summary

The float reference now runs the model's look-back read the way training ran it. The integer version then picks the same earlier positions in almost every case, but only when both sides use the same rule for breaking ties. With matched tie rules, the integer and float models agree on the next word at 99.6% and 98.2% of positions, above the 97% bar. With the GPU's own unspecified tie order, the agreement falls to 89% and 84%. So the last gap is a tie rule, not arithmetic. The next step makes the trainer use the integer version's rule, then measures again on the checkpoints the item names. On this laptop the read costs 16–33% of decode speed under load: about 3,100–4,100 tokens per second with it, 4,300–4,900 without.
