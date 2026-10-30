# Prep sheet: a memory key a reworded question can reach (one-model-end-to-end, claude-218, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-218](../../loop/tasks/one-model-end-to-end/claude-218/ORDER.md). It follows claude-217 ([result](../../labs/one-model-end-to-end/cycle-step-2026-10-30/README.md)) and step 0 ([counts](../../labs/one-model-end-to-end/memory-key-2026-10-30/README.md)). The redirect from the rollout hinge was made by the lab's decision review (D68), logged on #3407 (6526603691).

TEST FITNESS: round 1 NOT FIT (a fresh non-author reviewer, 30 Oct, at `bb9047baa`), on four blocking points, each repaired here:
1. **τ across seeds.** One distance cannot carry across seeds' trunk scales, so τ is now a decile level, applied at each seed's own quantile.
2. **Ties.** The device's reduction picks among tied rows by thread. Each key now names the commonest target among all tied rows, and records the tie count.
3. **Row 1.** It was two-sided, counted verbatim rather than correct replies, and argued its power on DEV. It is now one-sided, on exact and template matches, with an underpowered floor.
4. **The exact key.** Nothing showed it is the memory's served key on the panels. Three instrument checks now do.

Round 2 is pending.

## The question

When a question is reworded, can the trunk's own opening point find TRAIN's version of that question, where the memory's exact key cannot? And does a right opening alone lift the reply?

**Why this, now:**
- **The memory's key misses new wording** (step 0, on the development panel p-s). Its key, the turn's first 8 ids, fires on 17.0% of openings and is right on 19.3% of those. On DEV it fires on 97.4% and is right on 50.3%.
- **Order-free id keys do not help.** The prompt's distinct ids fire on 0.4%.
- **The headroom is large.** The oracle, keyed by the reference reply's own template, finds 69.1% of p-s openings in TRAIN and names 76.1% right. The model gets 14–19%.
- **So most reworded questions have answers TRAIN holds.** What is missing is a key that finds them. The trunk routes the prompt (claude-155's design). This tests whether its opening point routes a reworded question beside its known one.

## Base artifact: the model and the arms

- **The model:** the sealed (r-drop) at step 16,384, seeds 1–3, at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/`, each checked against its seed's `SHA256SUMS`. It is the recipe claude-216 kept. There is no training.
- **The table:** TRAIN's 212,968 openings. Each opening is a scored row whose previous token is not a reply token. For each, the table holds the trunk's own readout point, with the turn memory off, and its target, the reply's first token. It is built per seed, under that seed's trunk (`opening_points`). It takes 212,968 × 256 × 4 bytes, 218 MB, on the device.
- **The keys.** Each names an opening by the commonest target among all TRAIN openings at its least distance, ties to the smaller id, as the exact key names its table (`l1_named`).
  - It runs on the device in blocks of 64 queries × 4,096 rows, with no dot product.
  - A first pass finds each opening's least distance, and a second counts the targets of every row at it, by the same kernels.
  - The tie count is written per opening, since identical TRAIN prompts give identical points and sign distances come in steps of 1/256.
  - **point:** L1 on the point itself, the mean `|q − t|`, the cell objective's distance.
  - **sign:** L1 on the point's 0/1 sign pattern (`sign_code`), which is the Hamming distance over the width. It is the cheaper key: 32 bytes an opening, servable as integers.
  - **the exact key** (reported): the memory's own `first-8`, counted on the laptop as in step 0. The anchor check shows it is the served key on p-s too (1,400 of 1,400).
  - **Each key's search is timed separately** (`point_ms`, `sign_ms`). TRAIN's sign codes are taken before the sign search starts, as a served table would hold them.
- **The arms,** all on the same openings:

  | Arm | What names the opening | Role |
  |---|---|---|
  | B | the trunk alone: the memory-off nearest token (`rows-{split}-no-memory.json`) | the base |
  | served | the model as served, memory on (`rows-{split}-memory.json`) | DEV's backoff base |
  | point | the point key where it fires (distance ≤ τ), else B | decides |
  | sign | the sign key where it fires, else B | decides |
  | forced | free generation, greedy, memory on, with each reply's first token given as its reference's (`samples-{split}-forced-memory.json`) | decides row 1 |
  | U | the oracle `reply-template` key (laptop, as in step 0) | the ceiling, decides row 2 |

- **Each key's threshold τ** is chosen on p-s only, before p-u is graded.
  - **The level:** one decile level per key, from q10, q20, … q100. At a level ℓ and seed s, τ_s(ℓ) is the ℓ-quantile of that key's distances on seed s's own p-s missed rows. q100 is +∞, so it always fires.
  - **The choice:** the level with the largest net gain on p-s's missed rows, summed over the three seeds. A tie keeps the lower level.
  - **Applied unchanged:** at seed s, the key fires on p-u and DEV where its distance is at most τ_s(ℓ), seed s's own p-s quantile. Each seed's trunk has its own distance scale, so a level carries across seeds where a single distance would not.

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code.
- **p-u decides.** It is a new held-out-phrasing panel, drawn on 30 Oct by `heldout-panel` at seed 2101, a seed no unit had used. It is unopened.

  `heldout-panel out=p-u cell=dev,dev conversations=400 seed=2101 protocol=2`

  - **The panel:** 51,913 tokens and 16,350 reply targets.
    - `tokens.u16` sha256 `d8e80f38950769650a0f5786a42fbfff3332c9d3cf49a3e42d2cfe428502fbd0`;
    - `response_mask.u8` sha256 `c883aa4a14c3cf79460c8706de66c3eb5d8166ee89844484c9bb523c45206865`.
    - A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-u-2026-10-30/` with `SHA256SUMS` (sha256 of the census file `90f33036…b3bf2`), read back OK.
  - **The census** ([census-p-u-2026-10-30.json](../../labs/one-model-end-to-end/census-p-u-2026-10-30.json)):
    - 0.000 of its 1,466 prompts are in TRAIN exactly;
    - 0.000 are in TRAIN as value-masked templates at K = 1,024 (0.044 at K = 256);
    - 0.630 of its replies are verbatim TRAIN replies.
- **p-s is the development panel.** It was opened by claude-217 and step 0. It chooses τ and decides nothing. It is stored at `inputs/claude/heldout-panel-p-s-2026-10-30/`.
- **DEV is reported, with backoff.**
- **p-t, p-r, p-c, P-dev and p-f are not used.**

## Steps

1. **On one RTX 5090,** at the head's full sha:
   - build `train-in-space` with `--features cuda`, and keep the binary with its sha256;
   - run the crate's CUDA tests, which must all pass, among them:
     - `a_forced_opening_continues_greedy_and_the_opening_points_are_the_rows`;
     - `the_opening_keys_name_each_opening_by_its_nearest_train_opening`.
2. **Check every input against its sha256:** TRAIN, DEV, the tokenizer, both panels, and each seed's checkpoint.
3. **Reproduce claude-211's 15 registered files byte for byte:** seed 1, the default `--score-rows`, into a probe folder, compared with the stored files. A difference stops the run as an instrument fault.
4. **Score each seed:** `--score-rows CKPT --panel p-s=DIR --panel p-u=DIR --opening-keys 1`, three seeds sharing the GPU. Each writes:
   - the rows, memory on and off;
   - greedy and the default Flock samples;
   - the forced samples;
   - `keys-{dev,p-s,p-u}.json`.
5. **Check the outputs** (on the pod: the files; in the grade, on the laptop, each an instrument fault that refuses to grade):
   - every expected file is written;
   - each keys file's positions equal the split's openings as the laptop finds them from the panel's mask, in order;
   - every forced reply opens with its reference's first token;
   - the anchor check holds at every opening of p-s and p-u (`--opening-bound 1` with both panels), so the laptop's exact key is the served memory's;
   - on every opening where the exact key misses, the memory-on and memory-off nearest tokens are equal. So the memory moves nothing there, and B is what the served model sees on the missed rows.
6. **Seal** to `results/claude/one-model-end-to-end-memory-key-2026-10-30/run-<sha8>/` with a `SHA256SUMS`, match it on the pod file for file, read it back, and take the pod down.
7. **On the laptop, before p-u is graded:**
   - write and unit-test the grade, `--key-grade 1`;
   - choose each key's level on p-s;
   - check row 1's power on p-s's own forced arm (below);
   - commit all three, with the levels and each seed's τ, to the PR.

   **Then** grade p-u.
   - **The grade reads** the sealed run's rows, samples and keys files, plus the panels' and TRAIN's token files. It needs those for the exact key, U, the verbatim hash of the known and new strata, and the value-masked template match.
   - **Nothing from p-u is read before then.** The run's own printed summary gives no key hits: the keys files leave those fields empty.

## Bar

**The rows.** Each split's openings are split into strata, read per seed:
- **Missed rows:** openings where the exact key does not fire.
- **Known and new:** among the missed rows, those whose reference reply is a verbatim TRAIN reply (the census's hash) are known; the rest are new.

**A key's reading on the missed rows, per seed s.** At its τ, a key K names an opening where it fires and leaves B's name elsewhere.
- **The paired counts:**
  - b = rows K gets right and B gets wrong;
  - c = the reverse.
- **The net gain** is Δ_K,s = b − c, and its band is 2·√(b + c).
- **K's precision when it fires** is read against B's on the same fired rows.

**The decision table, read in order on p-u** (the first row that holds decides):

| Row | Holds when | Next unit |
|---|---|---|
| 1 | At every seed, the forced arm's lift over greedy (paired, memory on, 256 replies), b − c, is at most 2·√(b + c), both on replies equal to their reference and on replies whose value-masked template (K = 1,024) equals the reference's. If b + c < 10 on both measures at a seed, row 1 is underpowered there and does not hold | The opening is not enough. The rollout hinge, or a combined unit |
| 2 | At every seed, U's hits minus the larger of the exact key's and B's hits, on all openings, is within 2·√(b + c) of zero, paired against that larger arm | No prompt key has headroom. The hinge, and the later rows |
| 3 | Some key passes all four, at every seed: Δ_K,s beyond its band; precision when it fires at least B's on those rows; Δ_K,s ≥ 0 on the new stratum; and on DEV, the served model with K backing off where the exact key misses not lower than served by more than its paired band | Train and serve that key (the sign key if both pass), with an opening bar on another fresh panel |
| 4 | No key passes row 3, but some key fires on at least 10% of the missed rows at its τ, with precision when it fires below B's at some seed | An abstention gate (a distance or a share threshold) |
| 5 | None of the above | Train the trunk's opening point to gather paraphrases, with a geometric hinge |

**Notes on the table:**
- **Row 3's precision condition** follows from its net gain: on the rows where K fires, K's hits minus B's are b − c. Row 4's "precision below B's" likewise means a net loss at some seed. Both are stated for the reader.
- **The ORDER's band** was `max(2·√(b + c), the spread across seeds)`. Here every condition must hold at every seed, each with its own paired band, which is the stricter reading of the same rule.
- **U** is near 100% precise by construction, since its key holds the reply's first token. So row 2 mostly measures how often TRAIN holds the reply's template, which is the headroom a prompt key can reach.

**Reported, deciding nothing:**
- each key's fire rate, precision and Δ at every decile on p-u;
- the known and new strata;
- DEV at every decile;
- the forced arm's loops and complete replies;
- the search cost per opening in milliseconds, and the table's bytes;
- tokens per second for every decode.

## Minimum test power

- **p-u's openings.** p-s has 1,400 prompts and 1,400 openings, so p-u's 1,466 prompts give about 1,466 openings.
- **The missed rows:** the exact key fires on 248 of p-u's 1,466 openings (the census's keyed openings), so p-u has 1,218 missed rows at every seed. The key is the same at every seed.
- **Row 3's power:** with B at 14–19% on p-s, a key naming half of the oracle's headroom right would give Δ of several hundred. The band 2·√(b + c) is at most about 70 for 1,218 rows. A key that helps cannot hide in the band; one that does nothing reads inside it. Row 3 cannot be underpowered.
- **Row 1's power,** on held-out wording, where greedy matched its reference in only 3, 1 and 4 of 256 replies (claude-217, p-s).
  - The template match is the measure with power. 0.63 of p-u's references are verbatim TRAIN replies, so a right opening that leads into the right template gives a lift of dozens.
  - Row 1's underpowered floor, b + c < 10 on both measures, keeps a thin reading from holding row 1.
  - This power is checked on p-s's own forced arm before p-u is graded (step 7).

## Stop rule

- **Instrument faults stop the run and are repaired, never counted:**
  - a CUDA test fails;
  - an input sum differs;
  - a registered file does not reproduce;
  - a keys file's openings differ from its rows file's;
  - a forced reply does not open with its reference's first token.
- **The run stops at its cost cap**, $5, or at 2 hours of pod time.
- **The grade never reads p-u** until the grade and τ are committed.

## Cost

- **The run:** one RTX 5090 at about $1.1/h, for about 50 minutes: about $1.
  - Build and tests take about 10 minutes.
  - Each seed's TRAIN pass over its opening windows takes a few minutes.
  - The decodes match claude-217's greedy and Flock arms, plus the forced arm.
- **The cap:** $5. A lease inside the caps (at most 4 pods and $8/h across labs).
