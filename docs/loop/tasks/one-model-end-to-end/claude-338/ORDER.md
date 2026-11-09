# ORDER: one-model-end-to-end, claude-338

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f75725d30`.

## Goal

**Train a language model with the ask heads, the question carry and the found-copy pull, serve it in integers, and score it on the reply panel against a no-carry control.**

- [claude-335](../claude-335/REPORT.md): the carry answers B2 at both seeds. [claude-336](../claude-336/REPORT.md): the integer session serves it exactly. [claude-337](../claude-337/REPORT.md): `run_long` trains the pull at its scored rows.
- No served model yet puts the asked value in a reply: the panel's bar is 0 at every seed so far ([claude-322](../claude-322/REPORT.md)).
- **The risk, named first:** the language pull sits at the scored rows only. That is claude-335's B2-aC placement, which failed at seed 1 (`R_ask` 0.24, still rising at 16,384) and passed at seed 2. The sheet decides how the run meets it.

**The work:** one prep sheet, then a FIT, a D67 pre-run review, a probe ladder, and the two-seed run.
- Arms, on claude-322's (b64L) recipe and TRAIN `d3mwcap6`:
  - L-aC: ask heads, carry 4, pull λ;
  - L-a: the same without the carry (L-010's control);
  - the `−C` cut at evaluation.
- Served through `fbs-snapshot` → `fbs-export` → `served-reply-panel`, not `cpu-fbs-export`, which ignores the ask table.

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end; the recall gap.
- **(b) What it changes:** the first language model that trains the ask heads under the signal they need, served with them.

## Cancel probe (run first)

1. No open PR touches the item's files.
2. claude-337 is merged and verified.
3. No pod of mine is running; the caps leave room.
4. Disk: at least 20 GiB free.

**Result (instrument, at claim):** pass.
- PRs #3889 (DeepSeek) and #3890 (Kimi) touch none of the files.
- claude-337 merged as `f75725d3071abb9488e058ac158a04d8d093014d` and was verified on fresh `main` (lib 340/0, bin 11/0).
- 0 pods running. 29 GiB free.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is written by hand under Casey's harness exception, as claude-331 to claude-337 were.

## Files in scope

- `docs/harness/prep/one-model-end-to-end-language-carry-2026-11-09.md`
- `docs/loop/tasks/one-model-end-to-end/**`, `docs/labs/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`
- Code only if the probe ladder finds an instrument fault: `crates/uor-r4-float-train/src/{in_space_long.rs,bin/train-in-space.rs,flock_read_hamming.rs}`

## Done when

1. The sheet has a FIT and a pre-run review, posted on #3407.
2. Both seeds of every arm are trained, served and scored, and sealed to the store. Or the sheet's stop rule fired, and the REPORT says why.
3. A REPORT reads the outcome map. Every result is one notebook row.
4. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). If the throughput check projects more than about 3 hours per arm, cut the arms, not the steps, and record the cut. An instrument fault is repaired, never counted as a negative (L-007).
