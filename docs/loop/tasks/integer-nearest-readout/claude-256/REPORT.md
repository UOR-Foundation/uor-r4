# REPORT: integer-nearest-readout, claude-256 (the replay driver, arithmetic step)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md), as amended by claude-254's [decision card](../claude-254/DECISION.md). Q5 chain step 6, part 1. References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. The one serving-crate change is a read-only accessor, `IntegerStackModel::fbs_writer_table`. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.

The driver runs offline. On the laptop (Apple M1), with the release `fbs-toks` built at `3aa60ad71` (sha256 `f8ff0e6d…`):

| Reading on DEV s1 (T's 621 MB trace, 308 windows) | Wall | Peak resident set (`time -l`) |
|---|---|---|
| `--decision-check` of T's trace | 4 s | 1.00 GB |
| `--decision-deciles` of T's trace | 3 s | 1.39 GB |
| `--decision-replay` of (T, R): 308 unforced R windows, then 2 replays of 14 passes | 18 s (the 14 passes: 733 ms, about 52 ms each) | 1.15 GB |

## What did not work, first

- **The driver's first comparison would have refused every DEV window.** T's trace reads on past the prompts file's reference: its first window holds 99 ids, against R's 50. The first draft compared the reference's picks with R's unforced picks only when their counts were equal, and called anything else I-Align.
  - It was found by reading the inputs' first bytes, before any reading ran. Class: the instrument (this unit's driver), not a mechanism.
  - The repair is `differing_offsets`. It compares the reference's offsets inside R's window (the row's prompt and its reference), refuses a window whose tokens there are not R's or that stops short of R's last offset, and counts the reference's picks past R's window, not comparing them.
  - The ORDER records the amendment, made before any reading ran. A host test pins it, with negative gates G9 and G10.
- **The first reading's "beyond" count was not a count of picks.** It counted 19,050 reply steps past R's window, steps with no nearest among them. The count is now of steps with a nearest: 6,458. With the 4,382 compared, that is 10,840, which is T's nearest count in its own check (negative gate G11). That first run is kept in `read/run1/`. Its check and deciles files are byte for byte the second run's.
- **The ORDER's first done-when 1 asked for a refusal no replay can reach:** a replay that runs past its prefix's decision count. `grade_pass` refuses a pass that does not take Φ's values, so each pass ends at a new decision of the prefix. It was amended before any code was written, and the test of the replay's end replaced it.
- **Two of claude-254's tests asserted what this unit forbids.** They accepted float picks 1,024 and 1,280 behind the other side's in f64. Done-when 3 refuses a pick behind past f32's rounding bound, so both now expect the refusal, and new tests cover the cases inside the bound. The behaviour changed by the ORDER, not to reach green.
- **The 36 failing tests of `uor-r4-integer --features serving-wasm` are still red.** They are the base's own (claude-255's `head-failed-names.txt`, at its head merged as `175fca2a8`): the M1/M2/M3 challenger and timing tests and the frozen-panel CLI test. None is touched here.
- **The structure step (F, T) is not replayed.** It needs Q = T on the GPU: `train-in-space`'s single-window float mode, the next unit. Q5 is not read, and done-when 3 does not move.

## What worked

- **T's loader holds on real data.** `--decision-check` of T's DEV s1 trace (`b1970581…`, claude-252) against T's table, R's artifact divided by 4096, passes on all of it. That is 14,031,744 buckets, 584,656 codes and 10,840 nearests over 36,541 steps in 308 windows.
- **The replay of the arithmetic step accounts for both of its differences.** Of the 4,382 offsets inside R's window, T and R pick the same token at 4,380, as claude-252 read without forcing. The other two each replay to the nearest:
  - **Position 30248, offset 19 (step 80):** 11 passes. Nine rate buckets (layers 1, 2 and 5, steps 8 to 72) and one type-head query code (depth 2, step 80) were forced, each a near tie. The picks then agree (970).
  - **Position 41408, offset 3 (step 22):** 3 passes. Two rate buckets were forced (layers 4 and 3, steps 7 and 20), each a near tie. The picks then differ (T 289, R 3478), but both margins are within G₁₀ = 30,228 (T's 1.13, R's 11), so the step is a near tie.
  - Each forced bucket's value is between 6.3 × 10⁻⁶ and 2.2 × 10⁻⁴, against its family's G₁₀ of 0.0061 to 0.0093. The forced code's value is 2.0 × 10⁻⁴, against 0.354.
  - **Counts:** 2 accounted, 0 unaccounted, 2 ended at the nearest, 0 at a kept set, 0 at a memory lookup.
  - This is a reading outside Q5's set (claude-249's `prompts-dev-s1-outside.json`). It reads no Q5 position, and the bar is unchanged.
- **T's deciles** (14 families): rate buckets 0.0057 to 0.0093, copy-head codes 0.031 to 0.032, type-head keys 0.015 to 0.016, and type-head queries 0.172 and 0.354. These are close to claude-254's F deciles on the same windows.

## What changed

- **`replay_step`** (`fbs_snapshot.rs`) runs one step's whole forced replay. Each pass goes through `grade_pass`, and a code or bucket the pass ends at joins Φ for the next. The replay stops at a kept set, a memory lookup or the nearest, and is accounted for by `replay_accounts`. A pass error is returned as "pass i: …".
- **f32's rounding bound on a float pick** (`NearestTable::behind_bound`): γₙ(|s(u)| + |s(w)|) at u = 2⁻²⁴, plus the f64 term at 2⁻⁵³, where n is the table width. R's bound is 0.
  - In `grade_pass`, a float pick behind its rival by more than the bound is refused on either side.
  - In `check_nearest`, a float pick may trail its runner-up by at most the bound. The base took any order.
- **R's tie rule:** in `grade_pass`, R's pick that ties the reference's but has the larger id is refused.
- **`grade_pass`'s doc** now says the reference's entries and codes are checked even where forced.
- **`fbs-toks ARTIFACT --decision-replay`** runs the arithmetic step (T, R) with Q = R from the artifact. R's table is the model's own writer table (`fbs_writer_table`). Each pass steps the row's prompt and its reference cut at p, under Φ, through `forced_picks`. It writes `uor-r4.decision-replay/1` new.
- **The decision commands' refusals,** before any file is read: an unknown or doubled flag, an `--at` named twice, a side with no table (only F, R and T have one), an offset past the window, and a deciles file that names no trace.
- **Size:** code +521 −70, tests +541 −10, in `fbs_snapshot.rs`, `bin/fbs-toks.rs` and `stack/session.rs`. The docs are this REPORT, the ORDER, one notebook row and one item bullet. The ORDER gave no size estimate.

## Measurements

- **Inputs** (`read/inputs.sha256`):
  - R's artifact `a7r-s1.uorlut` (`f86a7856…`, claude-243's seal);
  - the prompts `prompts-dev-s1-outside.json` (`5e0bbf75…`, claude-249);
  - T's trace (`b1970581…`, claude-252, in the HF store at `results/claude/integer-nearest-readout-trace-t-2026-11-02/`).
- **Outputs** (`local/claude/claude-256/read/`, run by `run-readings.zsh` at `3aa60ad71`):
  - `check-t.json` `e4123f04…`;
  - `deciles-t.json` `d303c4b3…`, naming T's trace;
  - `replay-t-r.json` `0a12291d…`, with G₁₀ 30,228 (claude-244's cell).
- **Negative gates,** each setting aside one fix's own lines, running its test, expecting a failure and restoring the file byte for byte:
  - **The library** (`gates/run-gates.zsh`, `gates-lib.txt`):
    - G1: the replay forces the decision each pass ends at.
    - G2: the grade refuses a float pick behind past the bound (2 tests).
    - G3: the trace check holds a float pick to its runner-up within the bound (2 tests).
    - G4: R's larger-id tie is refused.
    - G5: a negative reference margin inside the bound is accepted.
  - **The CLI** (`gates/run-gates-bin.zsh`, `gates-bin.txt`):
    - G6: a deciles file with no trace is refused.
    - G7: a side with no table is refused.
    - G8: each pass hands Φ to R.
    - G9: a longer reference window is compared over R's offsets.
    - G10: other tokens inside R's window are refused.
    - G11: only picks are counted past R's window.
  - All eleven fail with their fix set aside, and pass restored.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast` at `3aa60ad71`: 636 passed, 36 failed and 4 ignored (`suite-head.log`). The 36 names are the base's, with none new and none fixed. The 24 `fbs_snapshot` tests all pass. The run at `b07851a2b`, before the two alignment commits, gave the same counts (`suite-b07851a2b.log`).
- `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --bin fbs-toks`: 14 passed. The `forced` module builds only with `fbs-synthetic`, so the suite above does not run it.
- `cargo clippy -p uor-r4-integer --features serving-wasm,fbs-synthetic --all-targets`: no warning in `fbs_snapshot.rs` or `bin/fbs-toks.rs`. The one in `stack/session.rs` (line 736, a manual `Option::map`) is the base's, not on this unit's lines.
- `cargo fmt -p uor-r4-integer`: applied, no other file changed.
- `uor-harness loop gates --full` and `loop pr-check --local`: listed in the PR's review record.

## Skills used

None.

## Still open

- **The structure step (F, T)** (Q5 chain step 6, part 2): `train-in-space`'s single-window float mode, so that Q = T runs one window per pass on the GPU, driving the same `replay_step`. Each float pass is now a whole `score_rows` run (204 s on F's s1).
- **Then the sheet** registers claude-247's rule with claude-254's amendment verbatim, with a fresh FIT and a pre-run review, and the GPU run reads Q5. Tracked in the item's done-when 3 (#3462).
- **The WASM build** in done-when 2 was not run here. This change touches no WASM-only code.

## Plain summary

The integer model and the float model it was exported from sometimes pick different words. Q5's rule asks whether each such difference is only a near tie. To answer it, the step is replayed: the integer model runs again with its internal choices forced, one at a time, to the float model's, until the two either agree or one choice turns out not to be close.

This unit builds that replay loop and runs the integer half on the laptop. On 308 test windows, the integer model and its float twin with the same rounded numbers disagreed at 2 of 4,382 words. Both replays ended in near ties. One needed ten tiny forced choices before the words matched; in the other, two forced choices left the two words essentially tied. Along the way the replay found that the float records run longer than the integer ones, and now compares only the part both cover.
