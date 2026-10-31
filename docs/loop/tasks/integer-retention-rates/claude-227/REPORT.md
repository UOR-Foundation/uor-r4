# REPORT: integer-retention-rates, claude-227 (parity of the retained trunk on the GPU)

Lab: claude. Item: [integer-retention-rates](../../../../roadmap/items/integer-retention-rates.md) (#3456). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap.

This unit is the item's done-when 3 (parity, under a prep sheet with a fresh FIT and a pre-run review) and 5 (one notebook row). claude-226 met 1, 2 and 4, so the item is done.

## Tokens per second

No measurement. This run is parity only; the served model's tok/s comes with point-key-integer's run.

## What did not work, first

- **The sheet first failed the harness's shape rule in CI.** Its section 5 was titled "Base artifact and data" with no "Data split" block (`prep.sheet_shape`).
  - It now reads "Base artifact and data split" and states the split: none, with no corpus read.
  - The local `the_real_sheets_pass_the_shape_rule` passes. The job does not read the sheet, so the run at `9e6836242` is unaffected.
- **The first FIT reviewer stopped at its context budget** before three checks. Its verdict was provisional. A second fresh non-author reviewer ran those three checks and gave FIT.
- **`uor-pod up` refused in the fresh worktree:** its Rust provider adapter was not built there. The main checkout's build was used (`UOR_POD_VAST_BIN`), after checking that `crates/uor-r4-pod-provider` has no commit since that build (23 Oct).
- **`uor-pod down` failed its laptop archive** ("uor-r4/pods: Is a directory") and kept the pod, billing.
  - The run's outputs were already in the store and listed back. I also stored the pod's `/workspace/uor-r4`, then destroyed the pod with `--confirm-archived`.
  - This is tooling; it is logged in `local/claude/FRICTION.md`.
- **The single tokens' residual, 1.78%, is the larger one.** It is inside the bar, but above claude-126's 1.4% single-token reading and above this run's 0.33% on sequences. Its cause is not located.
  - Retention does act at position 0: from the zero state, keep = √(1 − λ²) differs between the readings at every index past 0, which is why the decay control misses 97.9% of single tokens.
  - The 8 sequences hold only 8 first positions, so they cannot say where the single-token residual comes from (this unit's review).
- **The by-product snapshots' `retain_rates` was not checked in the files.** `fbs-snapshot` read it from each run's config as in the trunk arm. Point-key-integer's run checks it when it exports them.
- **CI's "harness rules and tests" also fails `harness.documented_commands`** on three of DeepSeek's REPORTs on main (deepseek-75, deepseek-1, deepseek-2): each documents a `pr-check` call with a stray arrow as a flag. That is not this unit's, and not a required check.

## What worked

- **`G-Ret-Parity`: PASS,** read from the stored comparisons.

  | Arm | Sequences agree | Single tokens agree |
  |---|---|---|
  | A_ret, the retained artifact | 1,531 of 1,536 (d = 5, 0.33%) | 4,023 of 4,096 (d = 73, 1.78%) |
  | A_dec, the decay control | 17 of 1,536 (d = 1,519) | 84 of 4,096 (d = 4,012) |

  - **(a):** both of the retained artifact's rates are within 3%.
  - **(b):** d_dec − d_ret = 1,514 on the sequences, against 2√1,524 ≈ 78.1.
  - So the retained artifact computes the trained trunk within the exporter's known residual. An engine before claude-226 would have served a different model.
- **The instruments, all green:**
  - 12 `fbs_export::` GPU tests;
  - every input file against the run's sums;
  - the flag carried once, with the control differing only in it;
  - each artifact's sha256 equal to its comparison's (`003c65fd…`, `7f2da7cb…`);
  - every tool exiting 0;
  - the readout agreeing with the retained artifact, not the decay one (`I-Readout-Retained`).
- **The by-product:** three full snapshots, seeds 1–3, with the memory and the Hamming read, about 60.8 MB each. Point-key-integer's run needs no pod for them.
- **The cost:** about $0.25 against the sheet's $2 ceiling, by the session's estimate. That is $0.95/h from bootstrap at 07:07:38Z to the deletion after the upload. The job ran 07:09:16–07:11:01Z, and no stored file records the deletion time. The builds were warm, since bootstrap had built the same code.

## What changed

- `docs/harness/prep/integer-retention-rates-parity-2026-10-31.md`: the sheet, its two FIT rounds and the pre-run review.
- `docs/labs/integer-retention-rates/README.md`: the lab record.
- `docs/research/notebook/m4-serving.md`: one row.
- `docs/roadmap/items/integer-retention-rates.md`: `status: done`, with the README re-rendered.

## Measurements

- **The store:** `caseyallard/uor-r4-store:results/claude/integer-retention-rates-parity-2026-10-31/`. `out/` holds 46 files: 45 under `SHA256SUMS`, and that file itself. `job.sh` and `pod-workspace-uor-r4/` sit beside it.
- **The largest query gaps** (retained): 3.89 on the sequences and 5.50 on the single tokens. The decay control's are 16.00 in both.

## Checks run at the head

- `xtask roadmap check`: 117 items OK.
- `roadmap render`: wrote the README.
- `roadmap sync` dry run: update #3456's body and close it. It is applied after the merge.
- `the_real_sheets_pass_the_shape_rule`: passes locally.

## Skills used

None.

## Still open

- **Point-key-integer's run** (#3450, claude-228), on the laptop:
  - export the three stored snapshots to `/7` with `--point-key`, checking `retain_rates` in each;
  - the fresh panel;
  - the decodes, the grade and tok/s.

  Its first change to `bin/fbs-export.rs` and `bin/fbs-toks.rs` also formats them (claude-224's drift).
- **The Hamming read's own parity** (integer-hamming-read #3385, step 5). This run left the read out.

## Plain summary

The trained model's memory step and the shipped whole-number version now agree. On a GPU, the converted model gave the same next word as the trained one at 1,531 of 1,536 positions. The old conversion, which got the memory step wrong, matched at only 17. The conversions the next step needs were made in the same run.
