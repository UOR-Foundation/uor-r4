# The point key served in the one model (one-model-end-to-end, claude-220, 31 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Sheet: [one-model-end-to-end-served-key-2026-10-31](../../../harness/prep/one-model-end-to-end-served-key-2026-10-31.md), FIT in round 4, pre-run GO. Report: [claude-220](../../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md).

**The registered outcome: row 2, the one float model serves the point key.** The decode names a reply's opening itself, from its own point, against TRAIN's saved opening table, wherever the memory's exact key misses. It carries the reply at all three seeds on a second fresh panel, p-w (1,200 conversations).

**These are float research evaluations** (CHARTER §3.5, §3.7), on the GPU. Their tok/s is the GPU float decode's, with three seeds sharing one GPU, not the served speed.

## The run

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3, the ones claude-217 to claude-219 scored.
- **The code:** `train-in-space --score-rows … --opening-keys 1 --served-key 1`, built at `eb762e602` (binary sha256 `e2c8057f…5acb`).
- **The compute:** one RTX 5090 (`vast-58466776`, $0.563/h), about 3 h 20 min, about $1.90. The main run took 9,840–9,995 s a seed, within the corrected projection's 2.8 hours ([DECISION-projection.md](../../../loop/tasks/one-model-end-to-end/claude-220/DECISION-projection.md)).
- **The panels:**
  - **p-w decides:** seed 2121, 1,200 conversations, tokens `038b0085…d217` ([census](../census-p-w-2026-10-31.json)), 1,202 prompts a seed;
  - **p-v** gives the parity against claude-219: 393 prompts;
  - **DEV:** 310 prompts.
- **Sealed** at `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/`: 129 files, `SHA256SUMS` sha256 `482008278ddf2ca1d5acb7f51a3e433091d83a2913b369a818f183273bd4e069`, read back OK. The three tables match the pod's sha256s.
- **The grade:** [grade-p-w.json](grade-p-w.json), by `--served-grade 1 --runs …/out --panel p-w=… --panel p-v=…`. It raised no instrument refusal.

## The instrument: the served key is the measured key

- **Parity is exact.** At every split and seed, every served token is the keys file's name: 1,172 on p-w, 385 on p-v and 19 on DEV. The distance drift is 0.0 at every one, so the decode's own point at the opening is bit-equal to the batched forward's.
- **The served files equal claude-219's keyed-point files byte for byte** on p-v and DEV at every seed, once `served_distance` is stripped.
- **claude-219's other p-v and DEV files reproduce byte for byte** (14 of 14 checks a seed).
- **The table reads back to the same values,** and is served from the copy read back.

## Tokens per second

- **Decode on p-w, three seeds sharing one RTX 5090:** greedy 54.2–55.3 tok/s a process, the served arm 52.6–54.1, about 2% slower.
- **The served search:** 1,172 a seed, one a reply, in 37.1–37.6 s with three processes sharing the GPU, so 32 ms a search. Alone, in the probe on DEV, it took 5.8 ms (19 searches in 0.11 s).
- **Residency** (the decision's condition (e)): the table is 218,931,272 bytes, resident per checkpoint. That is 657 MB in the store for three seeds, and the whole table is scanned at each keyed opening.

## The forced opening (row 1 does not hold)

A right first token, the reference's, against greedy (memory on), on p-w's 1,202 prompts:

| Seed | Exact (b, c) | Templates (b, c) |
|---|---|---|
| 1 | (224, 0) | (230, 0) |
| 2 | (214, 0) | (219, 0) |
| 3 | (236, 0) | (238, 0) |

## The served arm against greedy

b counts the served reply right and greedy's wrong, c the reverse. For loops, b counts the served reply looping where greedy's does not. "Keyed" is how many replies the served key opened, and "right" how many of those with the reference's first token.

| Split | Seed | Keyed | Right | Exact | Templates | Band | Loops | Carries |
|---|---|---|---|---|---|---|---|---|
| **p-w** | 1 | 1,172 | 371 | (48, 9) | **(50, 9)** | 15.4 | (130, 167) | **yes** |
| **p-w** | 2 | 1,172 | 325 | (44, 4) | **(44, 4)** | 13.9 | (132, 243) | **yes** |
| **p-w** | 3 | 1,172 | 333 | (47, 4) | **(47, 5)** | 14.4 | (72, 344) | **yes** |
| p-v | 1 | 385 | 106 | (12, 3) | (16, 3) | 8.7 | (35, 60) | claude-219's |
| p-v | 2 | 385 | 105 | (9, 1) | (9, 1) | 6.3 | (47, 85) | claude-219's |
| p-v | 3 | 385 | 119 | (14, 2) | (14, 2) | 8.0 | (22, 113) | claude-219's |

DEV's template pairs are (0, 0), (0, 1) and (0, 1): the served key opens only 19 of DEV's 310 replies, since the exact key fires on the rest. p-v's rows are claude-219's own, reproduced.

## What the served model's replies are

The protocol's free-generation measures (`generation`), greedy against served, on p-w:

| Seed | Arm | Complete | Verbatim TRAIN | New 4-grams | New 8-grams | Distinct-2 | Repeated 4-gram rate |
|---|---|---|---|---|---|---|---|
| 1 | greedy | 990 | 377 | 0.732 | 0.887 | 0.078 | 0.529 |
| 1 | served | 1,048 | 665 | 0.566 | 0.790 | 0.103 | 0.471 |
| 2 | greedy | 889 | 443 | 0.808 | 0.908 | 0.061 | 0.639 |
| 2 | served | 1,006 | 619 | 0.697 | 0.878 | 0.097 | 0.554 |
| 3 | greedy | 929 | 380 | 0.839 | 0.942 | 0.077 | 0.587 |
| 3 | served | 1,072 | 728 | 0.610 | 0.825 | 0.126 | 0.434 |

**Read plainly:**
- **The served key makes the model give more complete, verbatim TRAIN replies:** 1.4–1.9 times greedy's verbatim count, with fewer new n-grams.
- **The gain is recall, not new wording.** p-w's references are 0.647 verbatim TRAIN replies (its census), so recalling the right one is the measured gain.
- **Repetition falls,** and distinct-2 rises.
- **DEV's measures are unchanged.**
