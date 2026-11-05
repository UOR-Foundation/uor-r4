# REPORT: scale-probe, claude-295 (the first run unit)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Pin: `e052abf4104fc33ba28a489cf0dfdc4e94ab70b1`. PR #3750.

**DRAFT, written while the runs train.** Every number below names its source file or comment. The readings are added when their rows are graded.

**Claim check.** P5 of Casey's 4 November plan, "the bigger train = `scale-probe`", bounded at one sheet unit and one or two run units. This is the first run unit. It trains the one model, recipe (b) trained as served, at widths 256, 512 and 1,024 on a quarter, a half and all of 28M tokens of human-written text, at two seeds, and reads whether it gets measurably better with width and with data.

## What did not work, first

- **Width 1,024 does not fit a 32 GB RTX 5090** (attempt 1, #3472).
  - It ran out of memory at batch 16 (peak 31,952 MiB, the whole card) and at batch 8 (29,456 MiB). The sheet's stop rule covers only a width that fails at batch 16, so the job stopped at EXIT 5, as written.
  - Classification: instrument (the GPU's memory). No run trained.
  - The repair kept the recipe and moved width 1,024 to a 96 GB RTX PRO 6000 (my decision review, logged on #3472).
- **The 96 GB card never came up on Vast.**
  - Pod D failed to start three times. All three hosts belonged to one operator, Vast host 120840 in Texas.
  - The cause was measured on the third start: the host was RUNNING, its SSH port open, and its SSH server refused our key (`Permission denied (publickey)`). My first guess, an image still pulling, was wrong, and is corrected on #3472.
  - About $3 went on the three starts.
  - The sheet's stop rule replaces a failed pod "once". Pod D's second replacement was a departure, logged before it ran (#3472 6669956916).
- **No route to a 96 GB card is the lab's alone.** Under `uor-pod`'s own filters (a verified datacenter host, reliability above 0.98), the only RTX PRO 6000 Server host is 120840. One RTX PRO 6000 Workstation host qualifies (Hungary, $2.468/h), but `uor-pod`'s GPU table has no row for that edition, and `scripts/pod/uor-pod` is an owner path. Runpod by hand needs `--owner-approved`. Both are asked of Casey on #2028 (6670812323 and the addendum 6672201438).
  - **So the deciding readings R1 and R2 are unread in this unit until width 1,024 runs.** Both use it.
- **`uor-pod`'s exact-driver option can never match on Vast.** Vast's API returns nothing for a `driver_version=` predicate, though the same offers report that driver unfiltered. `uor-pod` also filters by driver locally and rechecks at create, so only the server-side copy of the filter is at fault. It is a tool item for its own unit (FRICTION).
- **My D30 slip.** Three read-only offer searches used the Python `vastai` CLI on PATH, not `uor-pod`'s Rust adapter. Nothing was created or changed. Every later query used the adapter.
- **Width 512's scoring ran out of memory on the 5090.** The scorer's GPU memory grows with each split it scores in one process. w512-d1-s1 went 12.3 → 16.4 → 27.2 → 30.0 GB and failed on its sixth split (22:58Z). Width 256 peaks near 8.5 GB and scores whole.
  - Classification: instrument (scoring memory), not the trained model. The checkpoints are untouched.
  - The repair, `rescore-295.sh`, scores the same checkpoint with the same binary, flags and seed, one panel per process. Each process also scores DEV, and the four DEV scorings must agree.
  - It passed two fresh review rounds (round 1 CHANGES, eight findings, all fixed; round 2 APPROVE, low findings fixed).
  - Before it counts, it must reproduce a width-256 checkpoint's whole-process rows exactly, on all 10 row sets (pending).
- **I-2 rests on a projected 1,024 term.** The sheet projects I-2 from each smoke, and the 1,024 smoke has not run. At the measured 5090 ratios (×1.97 a width doubling in step time, ×1.87 in scoring), the 20 runs come to 19.83 GPU-h, under 20, so there is no cut. The 1,024 smoke is checked against this when it runs, and an overshoot is recorded, not cut.
- **The laptop's disk fell to 2.6 GiB** (23:21Z, 5 Nov). The largest folder is another lab's build folder (17 GiB), noted for that lab on #2028 (6672142338). The ORDER's rule pauses downloads under 5 GiB until my store-sealed local copies are cleared.

## Departures from the sheet's pod plan

- The sheet puts seed 1 on pod A, seed 2 on pod B and width 512 on pod C. Since attempt 1, each width has its own GPU type: width 256 on pod A (a 5090: 6 grid cells and the contrast cell), width 512 on pod C (a 5090: 6 grid cells), and width 1,024 on a 96 GB card (7 cells, not yet run). The cells are unchanged, and each is covered once.
- The REPORT gives the GPU beside every training rate (R4).

## Measurements

(Filled in as the rows are graded.)

- **I-0:** 377 passed, 0 failed, on pods A and B in attempt 1 (#3472). Pods A and C passed setup in attempt 2; their counts are read from the sealed setup logs.
- **I-1 and I-4, the smokes,** graded on the laptop:
  - width 256 (pod A): exit 0; 1,460,697 of 1,460,697 rows;
  - width 512 (pod C): exit 0; 1,460,697 of 1,460,697 rows;
  - the same pooled row identity, `5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`;
  - width 1,024: not run.
- **I-2's inputs:** 0.169 s a step at 256 and 0.333 at 512, both on the 5090; scoring 179 s and 335 s.

## Still open

- Width 1,024's 7 cells (the owner call on #2028).
- The rescore of width 512's six checkpoints, after the equivalence check on a width-256 checkpoint.
- The grades, the readings R3 to R9, the seal, the notebook row and gap 9.
