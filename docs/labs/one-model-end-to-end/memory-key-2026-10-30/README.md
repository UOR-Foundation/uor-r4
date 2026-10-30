# A memory key a reworded question can reach (claude-218, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-218](../../../loop/tasks/one-model-end-to-end/claude-218/ORDER.md). The redirect from the rollout hinge was made by the lab's decision review, logged on #3407 (6526603691).

## Step 0: what each exact key could name, on the laptop

- **The method:** `train-in-space --opening-bound 1 --panel p-s=DIR`, at `8ea674541`; re-run with the panel's anchor check at the head after FIT round 1, its bounds byte-identical. It counts over the corpus: for each key, TRAIN's commonest reply opening, scored on the openings of DEV and of the panel p-s. There is no model compute.
  - p-s is the held-out-phrasing panel of claude-217 (seed 2091; tokens sha256 `62099a48…10f1`, mask `9053e117…f11a`).
  - It is now a development panel: it chooses keys and decides nothing.
- **The output:** [opening-bound.jsonl](opening-bound.jsonl), sha256 `f22316c7db876e8eba565d064de997b40367f100828395611177f26eefa77d05`.
- **Fires** means TRAIN holds the key. **Precision** is the share right among those.

| Key | DEV fires | DEV precision | DEV right | p-s fires | p-s precision | p-s right |
|---|---|---|---|---|---|---|
| none (TRAIN's commonest opening) | 100% | 13.1% | 13.1% | 100% | 15.3% | 15.3% |
| `first-4` | 100% | 18.5% | 18.5% | 99.6% | 15.1% | 15.2% |
| `first-6` | 100% | 37.5% | 37.5% | 62.4% | 20.6% | 17.9% |
| **`first-8`, the memory's key** | **97.4%** | **50.3%** | **49.5%** | **17.0%** | **19.3%** | **15.3%** |
| `set`, the prompt's distinct ids | 63.7% | 61.8% | 47.1% | 0.4% | 80% (4 of 5) | 15.6% |
| `set-first-8` | 97.4% | 50.3% | 49.5% | 17.8% | 20.9% | 15.2% |
| **`reply-template`, the oracle U** | 76.6% | 100% | 80.8% | **69.1%** | 100% | **76.1%** |

- **Totals:** DEV has 1,226 openings and p-s 1,400.
- **The memory's key:** the turn anchor keeps `first-8`'s tokens at every opening (1,226 of 1,226 on DEV, 1,400 of 1,400 on p-s, 212,968 of 212,968 on TRAIN), so the laptop's exact key is the memory's served one.
- **The oracle U** is keyed by the reference reply itself, with its values masked (K = 1,024). So its precision is 100% by construction. It is the ceiling for a key that knew which question was asked, not a key that can be built.

## What step 0 shows

- **The memory's key rarely fires on new wording, and is mostly wrong when it does.**
  - On DEV, `first-8` fires on 97.4% of openings and names 49.5% right. That matches the model's own teacher-forced openings with the memory on (49.1–49.3%, claude-217).
  - On p-s it fires on 17.0% and names only 19.3% of those right.
  - The commonest TRAIN keys are the question's opening words ("User: Note these down:", "User: Remember these:", "User: Tell me what").
- **Exact word keys cannot reach a reworded question.** The whole prompt as an order-free key fires on 0.4% of p-s, so the reworded questions use different words, not just a different order. The first 8 as an order-free key does no better than `first-8`.
- **The headroom is large.** 69.1% of p-s openings belong to a reply template TRAIN holds, and the oracle would name 76.1% right. The best prompt key names about 15–18%, and the model about 14–19%. So most new phrasings ask questions TRAIN knows the answer to, and no key over the words finds them.
- **By the ORDER's decision table:** there is headroom ("no headroom" does not hold), and no id key reaches it. So the GPU key probe (step 2) runs. It tests whether the trunk's own opening point puts a reworded question beside its known one: by its Hamming sign cell, or by the nearest TRAIN opening point under L1. A forced-opening arm sits beside it.

## The GPU key probe (step 2)

- **The run:** (r-drop) sealed at seeds 1–3, on one RTX 5090 (`vast-58426412`), built at `4cc00df2a` (binary sha256 `20d37cbe…7fc4f`). The panels are DEV, the development panel p-s, and the deciding panel p-u (seed 2101; tokens `d8e80f38…2fbd0`, mask `c883aa4a…06865`).
  - It took about 70 minutes of pod time, about $0.66.
  - **Sealed** at `results/claude/one-model-end-to-end-memory-key-2026-10-30/run-4cc00df2/`, 115 files, `SHA256SUMS` sha256 `d9efe9d31d4f79f91818cfe3ee06173f0840ad949419e362e774755be4bf25bd`. Read back OK.
- **The follow-up keys pass** (Casey's amendment, the Hadamard sign key) ran on the same pod, built at `d5b6f49f7` (binary `4121465f…d814c`).
  - **Sealed** at `run-4cc00df2/followup-d5b6f49f/`, 83 files, `SHA256SUMS` `587b0b235de768cb1b3a0cdd080fcc3bb5b2fd6a04b04c975a302b484b6cc05b`. Read back OK.
  - **It reproduces the first binary:** its 18 rows files equal the main run's byte for byte, and its point and sign keys equal the main run's in every field (checked by the grade, `--keys-runs`).
- **Instrument checks, all passed:**
  - the CUDA tests;
  - two negative gates on the GPU, each failing on its assertion after a real recompile: ties to the larger id, and the fixed signs set aside;
  - every input sum;
  - claude-211's 15 registered files reproduced byte for byte;
  - in the grade: the anchor check on every split, the panels' token and mask sha256s, the forced arm opening on its reference's token, and the memory moving no missed row.
- **The levels**, chosen on p-s and committed before p-u was read ([grade-p-s-levels.json](grade-p-s-levels.json), at `dfa703c36`): sign at level 2, hsign at level 1, point at level 10 (always fires).
  - hsign's net gain on p-s is tied at 105 across levels 1–4, so the registered tie rule (the lower level) chose level 1.
- **The decision on p-u:** [grade-p-u.json](grade-p-u.json), by `--key-grade 1 … --key-levels sign=2,hsign=1,point=10`.

### Tokens per second

- **Decode:** 52–60 tok/s per process on p-u, with three sharing the GPU (greedy 59.9–60.0, forced 58.8–59.9, memory off 52.0–60.4).
- **Each key's search** takes about 10.2 s for p-u's 1,466 openings: about 7 ms an opening against TRAIN's 212,968 opening points (218 MB as f32). That is the two-pass instrument's time, an upper bound on a served key's.

### The registered outcome: row 3, the sign key ships

The sign key passes all four conditions at every seed, and the cheapest key that passes ships. So the next unit trains and serves the sign key, with an opening bar on another fresh panel.

On p-u's 1,218 missed rows (the exact key fires on 248 of 1,466 openings), against the trunk alone, B:

| Key (level) | Seed | Fires | b / c | Net gain | Band 2·√(b+c) | Hits where fired, key / B | New stratum (448 rows), net | DEV backoff, net |
|---|---|---|---|---|---|---|---|---|
| sign (2) | 1 | 235 | 57 / 11 | **+46** | 16.5 | 88 / 42 | +39 | +8 |
| | 2 | 243 | 60 / 23 | **+37** | 18.2 | 80 / 43 | +30 | +4 |
| | 3 | 261 | 50 / 17 | **+33** | 16.4 | 88 / 55 | +28 | +6 |
| hsign (1) | 1 | 867 | 138 / 92 | +46 | 30.3 | 205 / 159 | +38 | +5 |
| | 2 | 574 | 97 / 64 | +33 | 25.4 | 127 / 94 | +31 | 0 |
| | 3 | 562 | 88 / 62 | +26 | 24.5 | 127 / 101 | 0 | +5 |
| point (10) | 1 | 1,218 | 241 / 116 | +125 | 37.8 | 341 / 216 | +38 | +5 |
| | 2 | 1,218 | 282 / 97 | +185 | 38.9 | 342 / 157 | +55 | +1 |
| | 3 | 1,218 | 229 / 115 | +114 | 37.1 | 333 / 219 | +49 | +1 |

- **All three keys pass row 3.** The sign key, the cheapest, ships.
- **Row 1 does not hold.** A forced right first token lifts exact replies from 4, 0 and 0 of 256 to 54, 49 and 52 (paired b/c 50/0, 49/0, 52/0). It lifts template matches by 52, 49 and 54.
- **Row 2 does not hold.** U against the exact key reads 889/5 at every seed.

### What it shows (reported, deciding nothing)

- **The trunk's own opening point finds TRAIN's version of a reworded question.** The exact key cannot: it fires on 248 of 1,466 openings.
- **The point key, always firing,** lifts missed-row openings from 216, 157 and 219 right (12.9–18.0%) to 341, 342 and 333 (27.3–28.1%). That is about 3.7 times the sign key's gain (424 against 116 over the three seeds), and the sign key ships only because the sheet registers the cheapest passing key.
- **The Hadamard rotation (Casey's trick) improves the sign code.** Read with every row firing (q100), the Hadamard sign key gains +50, +72 and +30, against the plain sign key's +10, +45 and +2. At their chosen levels the two gain about the same.
  - Its level-1 τ is 0: it fires only on an exact match of the rotated sign pattern, which happens on 562–867 rows, against the plain sign key's 235–261 at level 2.
- **The opening is the bottleneck.** With only the first token right, greedy writes the exact reference in about a fifth of replies, against about 1 in 200 now.
- **The known stratum** (770 rows whose reference is a verbatim TRAIN reply), nets per seed:
  - sign: +7, +7 and +5;
  - hsign: +8, +2 and +26;
  - point: +87, +130 and +65.

  The sign key's gain is mostly on new references, the point key's on both strata.
