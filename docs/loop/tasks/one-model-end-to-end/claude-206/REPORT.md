# REPORT: one-model-end-to-end, claude-206 (the scoring protocol)

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-protocol-2026-10-29](../../../../harness/prep/one-model-end-to-end-protocol-2026-10-29.md) (TEST FITNESS: FIT, round 2). References #3407. Done-when 3 is met; the item stays `ready` for the instruments, the training run and done-when 4.

**Result: the protocol is written and FIT.** It fixes, before any training run:
- how the model the project ships is scored;
- which recipe ships;
- whether it reads the prompt or only recalls TRAIN's templates.

Its decisions are made on rows no table names, on DEV and on P-dev, against the recall controls (T) and (S).

**Speed.** This unit times nothing. The protocol requires decode tok/s against context on the shipped integer artifact, once the serving units have merged.

## What did not work, first

- **The first draft was NOT FIT, with six blockers**, all fixed in round 2:
  1. it dropped the spec's memory-off arm;
  2. it let P-dev's keyed rows into decisions;
  3. it could ship a combination no arm trained;
  4. it read an underpowered "inside" as a transport negative (L-007's pattern);
  5. its verdict was not mechanical;
  6. its decode tolerance compared a count with a share.

  Each would have made the run's result misleading. The fourth is the one this project has paid for before.
- **Nothing in the protocol can run yet.** Seven instruments are missing, each with its check written (I-A to I-G):
  - memory dropout;
  - the transport arm as a flag (`in_space_long` hard-sets `OneAtomActive`);
  - `--score-rows` on P-dev;
  - the (T) and (S) controls;
  - the Flock rank decode;
  - the table hit rate under generation;
  - the grader's strata and decisions.

  They, and I-H's reproduction check, are the next units.
- **The integer half of "end to end" waits on serving.** Kept sets, the first divergence and tok/s on the artifact need `integer-hamming-read` step 5, `turn-memory-integer` units 2–3 and the nearest readout. Until they merge, the protocol does not let any REPORT claim end to end.
- **The type spin cannot ship yet,** even if its arm is a candidate. The export refuses `step_rotation` (`fbs_export.rs:382-384`).
- **P-dev can serve one decision run only.** A later combination of candidates needs a new held-out panel, drawn at a new seed with its own census.

## What changed

- **The sheet:** [`docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`](../../../../harness/prep/one-model-end-to-end-protocol-2026-10-29.md).
  - **Arms:** 7 trained arms × 3 seeds, with the reported memory-off arms.
  - **Controls:** (T) tables only, and (S) a longest-suffix table over TRAIN.
  - **Strata:** K, U, N-u, N-k and C.
  - **The rule:** the three-seed band and cell, with "inside" as underpowered.
  - **The decisions:** five, in order: the base recipe, the add-ons, the transport, the decode rule, and the verdict.
  - **The decode rules:** greedy, and the Flock rank rule, specified to the draw (the harmonic `RAW_RANK_WEIGHTS_Q16`, a seeded xorshift, no exponential).
  - **Also in it:** the scores, the instruments I-A to I-H with their checks, the data, the cost, and the stop rule.
- **This REPORT and the ORDER.** No code changed.

## Measurements

None new. The protocol builds on these counts, each checked against its source by the FIT:
- DEV's 14,287 rows split as K 10,849 (1,194 + 9,655) and U 3,438 (claude-202);
- P-dev's 16,417 rows and its 245 keyed openings of 1,443 (claude-204);
- claude-205's band and cells;
- claude-194's sealed readings, which I-H must reproduce: 11,513 / 11,361 and 10,079 / 11,171.

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`. Their results are on the PR.
- **The FIT, two rounds, fresh non-author reviewers (model claude-opus-5-5).**
  - **Round 1, at `8aec46877`: NOT FIT.**
  - **Round 2, at `0e8109f1a`: FIT,** with six notes, applied:
    - `n_c = 0` gives a mixed verdict;
    - decision 1 has decision 2's guard;
    - "complete" means ending in `<|eos|>`;
    - transport is decided on N-u;
    - the P-dev references are named as a deliberate departure from the spec;
    - the Flock seed is named.
  - Both rounds are summarized on #3407.

## What it unlocks

**The instrument units,** in the item's scope (`in_space_long.rs`, `train-in-space.rs`, `flock_read_memory.rs`). Each is built with the check the sheet names:
- I-B memory dropout, and I-C the transport flag: both change training;
- I-A P-dev scoring, I-E the Flock rank decode, and I-F the table hit rate: all three are scoring;
- I-D the controls, and I-G the grader: both run on the laptop.

**Then the training run,** under its own pre-run review: 21 runs, about 12 GPU-hours, about $8–14. Its decisions set the model that the serving units export and serve.

## Skills used

None was invoked this unit.

## Still open

- **The instruments I-A to I-H.**
- **The training run and its scoring.**
- **The serving units** the integer measures need: `integer-hamming-read` step 5, `turn-memory-integer` units 2–3, and the nearest readout.
- **The type spin's integer port,** if its arm is a candidate.

## Plain summary

Before training the model the project will ship, this sheet fixes how it will be judged, so the result can't be bent afterwards. Seven versions of the model are trained three times each. They are then compared on conversations whose answers no lookup table holds, including a set of prompts written in phrasings the training data never used. Two plain lookup methods set the bar the model must beat to show it reads the prompt instead of reciting what it memorised. The sheet also decides, by a fixed rule, whether a new way of picking each next word should replace always picking the single closest word. That new way uses the project's own harmonic weights.
