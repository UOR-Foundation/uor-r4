# REPORT: integer-hamming-read, claude-240 (parity on claude-146's checkpoints, on DEV)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). Prep sheet: [integer-hamming-read-dev-2026-11-01.md](../../../../harness/prep/integer-hamming-read-dev-2026-11-01.md). References #3385.

## Tokens per second

This is a reading, not a bar. It used `fbs-toks --context-sweep` on s1's artifact (`7c82d76b…`), on this M1, on battery, at a load average of 3.7 to 3.8.

| Context | Decode with the read | Decode without | Prefill with | Prefill without |
|---|---|---|---|---|
| 96 | 4,092 | 4,681 | 4,319 | 4,700 |
| 128 | 3,966 | 4,852 | 4,224 | 4,866 |
| 160 | 3,715 | 4,508 | 3,947 | 4,521 |
| 192 | 3,781 | 4,713 | 4,010 | 4,750 |

These are median tokens per second, with 64 decode steps after each prefill. At these contexts the read costs 13 to 20% of decode. The sweep stops at 192, the artifact's whole context, so the long-context cost is not measured here (see Still open).

## What did not work, first

- **Attempt 1 measured nothing.** At `50a824385`, the pod passed the data pins, the source pin, the 44 GPU tests and the build. Then the first snapshot step refused s1: `Invalid("native projection compiler/operator/FBS identity is missing or stale")`.
  - `StackModel::load` wants each saved plan fingerprint to equal this build's. Each fingerprint hashes its plan, and the plan carries the sha256 of the whole `spiralcore_fbs.cu`.
  - claude-179 (`bcf2f57f7`, #3367) changed that file after claude-146 trained, only in the backward kernel and two new reduce kernels.
  - Class: instrument. The run cost about 5 minutes and $0.05. It is stored at `results/claude/integer-hamming-read-dev-2026-11-01-fail1/`.
- **The parity arm runs on an evaluation copy, not on the stored checkpoint.**
  - `fbs-snapshot --rebind-fbs-backward` rewrites only the six fingerprints (five plans and the vector head's 8/8 geometry). It refuses unless:
    - the kernel's text through `spiralcore_fbs_fwd` is the text before claude-179;
    - each saved fingerprint is this build's plan with only the kernel's hash put back;
    - the copy loads.
  - The static check proves only that the copy is the checkpoint byte for byte, less those fingerprints. Its sed replay shows that nothing else changed. It does not show that each new fingerprint is right; only the tool's own load checks that.
  - That the copy computes claude-146's forward rests on `I-Rebind`'s Gate R half below, not on the rebind.
- **`d11`, the GPU sort's own tie order, misses badly on DEV.** d is 612, 843 and 597 of 5,795. The barred cells sit at 0.47 to 0.55 past t = 17, with 4,306 to 5,111 ties in each. The fourth cell is unmeasured: only 100, 100 and 156 sets are judged past t = 17, against a floor of 965.
  - The `d11` and `d11-tie` float readouts differ in `next` at 590, 815 and 578 positions.
  - So on these checkpoints the tie order moves 10 to 14% of next-word choices, a top-1 drop of 9.5 to 13.3 points. Parity holds only with the integer session's rule on both sides, which has been the float read's default since claude-239.
- **Integer rotations still flip some type-head query bits.** In the parity arm:
  - the first read's type heads differ at 21 to 29 sets of 9,646;
  - the second read's type heads differ at 64 to 101 sets of 7,260 to 7,858 judged.

  Every one is a query flip; no key flipped, and no tie is involved. They are inside the bar, and they are the residual near a code's sign, the route the item's design names. They were not traced here.
- **d counts every sequence position,** the prompt positions included, as claude-238's parity did. d on the scored positions alone is not reported, because `fbs-compare` gives counts, not the positions that differ.
- **The item stays `ready`.** Done-when 2 is met. Done-when 3's sweep stops at the artifact's context of 192, so the read's cost at long contexts is still unmeasured.

## The result

**Row P: `d11-tie` meets parity at all three seeds.** The item's done-when 2 holds: claude-146's checkpoints on DEV, against the float forward under its default rule. The rows are read in the sheet's order:
- row 0 does not apply: every instrument check holds (below);
- row S does not apply: `I-Copy` is clean;
- so row P is the first that holds.

**`d11-tie` on DEV** (54 windows, 5,795 positions). Bar P1 is d ≤ 173. Bar P2 is ≥ 0.95 past t = 17, on 9,646 sets in each of the three barred cells; the fourth cell's judged count is in its column:

| Seed | d | Top-1 | Copy heads, read 1 | Type heads, read 1 | Copy heads, read 2 | Type heads, read 2 (judged past t = 17; downstream) | Single tokens |
|---|---|---|---|---|---|---|---|
| s1 | 47 | 99.19% | 1.0 | 0.99699 (29 unequal) | 1.0 | 0.99118 (7,260; 2,386) | 3,942 of 4,096 |
| s2 | 72 | 98.76% | 1.0 | 0.99782 (21) | 1.0 | 0.98715 (7,858; 1,788) | 3,967 |
| s3 | 49 | 99.15% | 1.0 | 0.99730 (26) | 1.0 | 0.98934 (7,784; 1,862) | 4,035 |

Both comparisons count 0 ties and 0 instrument faults at every seed.

**The gap, `d11` on DEV:**

| Seed | d | Top-1 | Copy, read 1 | Type, read 1 | Copy, read 2 | Type, read 2 (judged past t = 17) | Readouts' `next` differ |
|---|---|---|---|---|---|---|---|
| s1 | 612 | 89.44% | 0.548 (4,359 ties) | 0.474 (5,045 ties) | 0.554 (4,306) | 0.85 (100: unmeasured) | 590 |
| s2 | 843 | 85.45% | 0.531 (4,527) | 0.468 (5,111) | 0.546 (4,383) | 0.88 (100: unmeasured) | 815 |
| s3 | 597 | 89.70% | 0.546 (4,382) | 0.484 (4,947) | 0.534 (4,493) | 0.833 (156: unmeasured) | 578 |

**The bridge, `d11-tie` on the fixed 8 × 192 sequences** (claude-238's bars: d ≤ 46, cells ≥ 0.95, the fourth on at least 278). It is reported only; it decides no row:

| Seed | d of 1,536 | Type heads, read 1 | Type heads, read 2 (judged past t = 17) |
|---|---|---|---|
| s1 | 28 | 0.99713 | 0.98700 (1,692) |
| s2 | 19 | 0.99749 | 0.99281 (2,086) |
| s3 | 21 | 0.99605 | 0.98119 (1,382) |

The copy heads are 1.0 at every seed. On the same sequences, (r-drop) gave 6 and 27 (claude-238).

## Instrument checks

- **`I-Rebind`: holds at all three seeds.**
  - **The static half:** each copy's tensors and run config are the pinned files byte for byte. Each copy holds 4 files. Its step config is the checkpoint's with exactly the six printed fingerprints replaced. Each copy loads (`rebound: 6 fingerprints, 3 files; the copy loads`). The same six fingerprints were rebound at every seed.
  - **The Gate R half:** H, the copy's inspection as trained at this head, equals claude-146's sealed inspection at step 16,384 in all 22 fields, nulls included. So does C, claude-146's own build at `df1d7892` on the original checkpoint on the same pod. H equals C.
    - s1: 368 top-1 hits and 8,687 nearest hits of 14,287.
    - s2: 293 and 8,709.
    - s3: 289 and 9,233.
  - **The rule:** the inspection under the tie rule differs from H at every seed, in 6 or 7 counts (s1: 364 top-1 hits, 8,684 nearest). So the gate can see which rule ran.
- **`I-Data`:**
  - the 12 checkpoint files and DEV's two files passed their pins;
  - the three sealed inspections matched their pins and claude-146's seal;
  - all nine `--dev` steps printed `dev: 428 windows, 54 sampled, 5795 positions, 4823 past t = 17`.
- **`I-GPU-Tests` at `1a5a11afc`:** 44 passed, 0 failed. The six named tests each ran and passed.
- **`I-Snapshot`:**
  - on the pod, each DEV snapshot equals its fixed snapshot byte for byte;
  - on the laptop, each writer copy is the snapshot less 23 bytes, each artifact does not read nearest, and every comparison names its artifact.
- **`I-Default`:**
  - at every seed and on both sequence sets, the snapshot form's D11 readout is the `d11-tie` trace form's, byte for byte;
  - `d11` reached the sort: its kept sets and readouts differ from `d11-tie`'s.
- **`I-Shape`, `I-Faults`, `I-Finite`:**
  - no fault was logged, on the pod (no `instrument-faults.txt`) or on the laptop;
  - every trace comparison lists 0 faults;
  - the job ended with EXIT 0, and `read.sh` exited 0.
- **`I-Copy`, on all three arms at every seed and both reads:**
  - 0 inconsistent pairs;
  - every query and key value bin is empty;
  - 0 bits differ from the CPU control and from the integer session;
  - the control equals the integer session at 8,192 of 8,192 codes.
- **The laptop's tools** are built at the pod's head, `1a5a11afc`, from a clean tree:
  - `fbs-export` `8d1ed8ce…`;
  - `fbs-compare` `07d885e5…`;
  - `fbs-toks` `ca955e9f…`.

## What changed

`crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, with three new options:

- **`--dev DATA_ROOT`,** on the snapshot form and on the `--hamming-trace` form.
  - It loads DEV with its pins (`load_pinned_uort_split`) and cuts it into the trainer's windows at 192.
  - It keeps every 8th window from the first, and cuts each after its last scored position.
  - It prints the sample's line before the GPU opens.
  - The trace checks the sample's own count.
- **`--rebind-fbs-backward MODEL_DIR OUT`:** the evaluation copy described above, with its three refusals.
- **`--inspect as-trained|tie-rule DATA_ROOT RUN_DIR`:** the trainer's own `inspect_run` on a run folder. It runs with the read's tie rule off (as the checkpoint trained) or on, and first prints the address roots' separation.

**The tests:** six CPU tests in the binary. Each fails with its own rule set aside, and passes restored (`local/claude/claude-240/neg-*.out`, `neg-forms.log`):

| Test | Set aside | Result |
|---|---|---|
| `the_dev_sample_keeps_every_eighth_window_from_the_first` | the sample rule | FAILED |
| `a_dev_window_is_cut_after_its_last_scored_position` | the cut | FAILED |
| `the_inspection_as_trained_turns_the_tie_rule_off` | `as-trained` mapped to the rule | FAILED |
| `the_forward_kernel_is_the_one_before_claude_179` | the forward cut, so the whole kernel is hashed | FAILED |
| `claude_146s_fingerprints_are_this_builds_plans_with_the_kernel_before_claude_179` | the kernel hash put back | FAILED |
| restored | | 6 passed |

These tests pin the helpers' rules. The refusals inside `rebind_fbs_backward` itself have no test of their own:
- the existing-folder check;
- the check that each fingerprint appears once in the config;
- the forward-text check at the call;
- the clean-up on a failed load.

On the pod, they ran on the real checkpoints, where every copy passed them and loaded.

**The docs:**
- the prep sheet, with its FIT, its pre-run review, attempt 1, and the decision review's two rounds (§9 to §11);
- the item's step 5 records the result;
- the notebook row is in `m4-serving.md`;
- the item's two carried nits are reworded (`50a824385`).

## Reviews

All of them are fresh-context reviewers that did not write the work. Each record is on #3385.
- **FIT:** FIT (`ec06f1318`), with five minor findings applied. For attempt 2's delta: FIT (`de83faabf`).
- **Pre-run:** PASS (`5ab73fc01`). For attempt 2's delta: PASS (`de83faabf`).
- **Decision review on the stop "`fbs-snapshot` refuses it":** round 1 REVISE (`e7dd3e40e`), with four findings applied; round 2 APPROVE (`de83faabf`), with three minor findings applied. Logged as `[claude] owner log:` on #3385 (comment 6556213845).
- **Exact-head code review (at `2d2bf259f`):** APPROVE, with no blocker, no major and three minor findings. The minors are under Still open.
- **Claims check (at `2d2bf259f`):** PASS. Every number in this REPORT, the notebook row and the item matches the sealed evidence. Its four minor wording findings are applied.

## Measurements and cost

- **Attempt 2's pod:** vast-58642838, one RTX 5090 (driver 580.178.04, compute capability 12.0), at `1a5a11afc`.
  - Bootstrap ready at 09:41:21Z. The job ran from 09:41:46Z to 09:50:04Z (`out/job.log`). The control's cold build took 20 s.
  - Deleted at about 09:53Z, so about 13 minutes at $0.563/h: about $0.12. With attempt 1, the unit spent about $0.17 on GPUs.
- **Stored** at `caseyallard/uor-r4-store:results/claude/integer-hamming-read-dev-2026-11-01/`:
  - `out/`: 156 sealed files plus their `SHA256SUMS`, `5fe0dd27fcd2ec347b668d942cc9e06ffd28aa49bf549b63b576e5f79eba2d32`;
  - `reading/`: 122 sealed files plus their `SHA256SUMS`, `5395af1a4558ee50815d0dff6125afcae4710a98ca4145b454cea2d777b30a70`.

  Both were listed back.

## Checks run at the head

The code last changed at `e7dd3e40e`; the commits after it are docs.
- `cargo test --locked -p uor-r4-float-train --no-fail-fast`: 221 passed, 0 failed (`local/claude/claude-240/crate-cpu-4.out`).
- `cargo test -p uor-r4-float-train --bin fbs-snapshot`: 6 passed, after the set-asides above.
- **The CUDA code type-checks with the stub nvcc** (`cargo check --features cuda --all-targets`): 0 errors (`cuda-check-5.out`).
- `uor-harness loop gates --full`: exit 0, every gate GREEN (`gates-full-4.log`).
- **On the pod:** the 44 CUDA tests, as above.
- `rustfmt --check` on `fbs-snapshot.rs`: clean.

## Skills used

None.

## Still open

- **Done-when 3, the read's long-context cost.** The sweep stops at 192, the artifact's context. Timing the read past it needs an artifact with a longer context. A synthetic one in `stack/synthetic.rs` is in this item's scope, since speed does not depend on trained weights. That is the next unit.
- **The type-head query flips** (21 to 101 sets per cell) are within the bar. They were not traced.
- **(r-drop), the checkpoint the one-model path ships,** met the bars on the fixed sequences (claude-238), but has not been run on DEV.
- **The exact-head review's three minors** (no blocker, no major). They are left for a later unit, because a code change now would move the head the GPU result rests on:
  - a failed copy or write after the copy's folder is made leaves a half-written folder. A retry is refused, and nothing wrong loads, since the step config is written last;
  - `rebind_fbs_backward`'s own refusals have no test (above);
  - `--inspect` runs `FbsArm::OneAtomActive` only. A run on another arm is refused by `check_run_arm`, not inspected wrongly.
- **Carried from claude-238 and claude-239:**
  - the guard's restore on a panic;
  - the read's unguarded backward;
  - T1's export leg, which has no negative of its own;
  - `.round()` in the tie key, which has no test that fails without it.

## Plain summary

The integer version of the look-back read now matches the float version of the same trained models on the held-out DEV text, at all three seeds.
- **Next-word choices:** they agree at 98.8 to 99.2% of positions.
- **The kept positions:** the two versions choose the same 16 earlier positions 98.7 to 100% of the time. The copy heads match exactly.

To load the old checkpoints, the evaluation made a copy of each. The copy updates only a build fingerprint that a later kernel change had moved. The copy was then held to the trainer's own recorded evaluation, and it reproduced it exactly, number for number. So did the original training build, as a control.

The read costs 13 to 20% of decode speed on this laptop at the model's full 192-token context. Its cost at much longer contexts is the next measurement.
