# Prep sheet: the point key served in the one model (one-model-end-to-end, claude-220, 31 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-220](../../loop/tasks/one-model-end-to-end/claude-220/ORDER.md), written by hand on Casey's word: "screw kimi forge ahead as you were before with your plan for the single serving model". Decision: [DECISION.md](../../loop/tasks/one-model-end-to-end/claude-220/DECISION.md), APPROVE on five conditions. It follows claude-219 ([result](../../labs/one-model-end-to-end/keyed-opening-2026-10-30/README.md)).

TEST FITNESS: PENDING (a fresh non-author reviewer reads this sheet and its sources before any pod).

## The question

**Served inside the model, does the point key still carry the reply, on a second fresh panel? And what does that model's free generation look like?**

- **claude-219 read the key from a keys file** computed before the decode. Its keyed-point arm carried the reply at all three seeds on p-v: templates (16, 3), (9, 1) and (14, 2) against greedy.
- **This unit serves the key from inside the decode** (`served_opening`). The reply's first forward gives the opening row's own point. Where the memory's exact key misses, that point is named by the commonest target among the served table's L1-nearest TRAIN openings, with ties to the smaller id. The table is TRAIN's 212,968 opening points under the trunk, built once, saved, read back to the same bytes, and served from the copy read back.

**What each outcome unlocks** is set out in the decision table under Bar:
- **the served key carries:** the one float model serves it; next comes its integer export, under its own item;
- **it does not carry:** claude-219's reading stands alone, and the next unit trains the trunk's opening point;
- **too thin:** a larger panel;
- **the forced arm does not lift, or parity fails:** an instrument repair.

**These are float research evaluations** (CHARTER §3.5, §3.7), on the GPU. Their tok/s is the GPU float decode's, not the served speed (the decision's condition (a)).

## Base artifact: the model and the arms

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3, with their turn memories: the ones claude-217 to claude-219 scored. They are under `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/models/step-016384/`, each checked against its seed's `SHA256SUMS` (6 files).
- **The code:** `train-in-space --score-rows CKPT --opening-keys 1 --served-key 1` at this unit's head. Every arm decodes 64 reply tokens with the nearest-cell objective on the same prompts:

| Arm | File | What it is |
|---|---|---|
| G | `samples-{split}-greedy-memory.json` (DEV: `samples-memory.json`) | Greedy, memory on |
| G0 | `…-greedy-no-memory.json` | Greedy, memory off (reported) |
| F | `…-forced-memory.json` | The reference's first token, then greedy: the ceiling and the instrument row |
| S | `…-served-memory.json` | The served point key: each given token kept as the sample's `keyed`, with the search's count and seconds |
| Flock-end | `…-flock-end-memory.json` | As in every run (reported) |

**The served key has no τ:** it is the point key at +∞, as claude-219's levels chose (level 10). It fires wherever the exact key misses.

## Data split

- **TRAIN, DEV and the tokenizer:** the pinned `w2-inputs/2026-10-25/driver-data`.
- **p-w decides.** It is a fresh held-out-phrasing panel, drawn on 31 Oct at seed 2121, which no unit had used:

  `heldout-panel out=p-w cell=dev,dev conversations=400 seed=2121 protocol=2`

  - **The files:** `tokens.u16` sha256 `1292e2460c8b7b777ea1f536972537436e5c2af168c6a40c54d640c654a8050f`, `response_mask.u8` `32a29da0513f3df9404552a2fe313a763426c818fea316fed0edc630dce360b6`. A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-w-2026-10-31/` with `SHA256SUMS`, and read back OK.
  - **The census** ([census-p-w-2026-10-31.json](../../labs/one-model-end-to-end/census-p-w-2026-10-31.json)) counts 1,472 openings:
    - 0.000 of its prompts are in TRAIN, exactly or as templates at K = 1,024 (0.039 at K = 256);
    - 0.632 of its replies are verbatim TRAIN replies;
    - the exact key fires on 264 openings.
  - **No model has read it.**
- **p-v**, claude-219's deciding panel, carries the parity instrument. It is reported, deciding nothing.
- **DEV** carries the not-worse condition (d).
- **The prompts:** `--samples 512`, so every eligible window (about 400 a split, the stride 1), as in claude-219.

## Steps

1. **On the laptop, done:**
   - the build and its unit tests: `the_opening_table_reads_back_as_saved_and_is_never_replaced`, with a negative gate on its never-replaced check;
   - the CUDA test `the_served_key_names_a_missed_opening_from_the_decodes_own_point`, type-checked with the stub nvcc;
   - the grade (`served_grade`), written before the pod;
   - p-w and its census.
2. **The FIT and the pre-run review:** each by a fresh reviewer, posted on #3407.
3. **The pod:** one RTX 5090 on Vast (`uor-pod up --count 1`). The source is pinned to the unit's head. The crate's CUDA tests must pass, including the new one and claude-218's and claude-219's named ones. The binary is kept with its sha256, and every input is checked against its `SHA256SUMS` and this sheet's sha256s.
4. **The reproduction and parity probe:** seed 1, DEV only, at claude-219's main flags (`--samples 512 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1`) plus `--served-key 1`. Against claude-219's sealed run (`run-da4a6212/out/r-drop-s1/`, checked against its `SHA256SUMS`), it must reproduce:
   - **six DEV files byte for byte:** `rows-dev-memory.json`, `rows-dev-no-memory.json`, `samples-memory.json`, `samples-no-memory.json`, `samples-dev-flock-end-memory.json` and `samples-dev-forced-memory.json`;
   - **`keys-dev.json`** without its three wall-clock fields;
   - **the parity instrument:** `samples-dev-served-memory.json` equals claude-219's `samples-dev-keyed-point-memory.json` byte for byte. They are equal only if every served token is the keys file's name.

   Any difference is an instrument fault. The probe's time projects the main run, which decodes about 3 times its replies on three seeds sharing the GPU (at most 9 times its time): past 4 hours, stop.
5. **The main run:** seeds 1–3 side by side, with the probe's flags plus `--panel p-w=DIR --panel p-v=DIR`. Afterwards:
   - at every seed, p-v's and DEV's rows, greedy, memory-off, flock-end and forced files must equal claude-219's main run's byte for byte, and its keys files without their wall-clock fields;
   - the served files must equal claude-219's keyed-point files byte for byte, on p-v and DEV.
6. **Sealing.** The job checks that every expected file is written, with `opening-table.safetensors` at every seed and its sha256 recorded. The run is sealed off the pod to `results/claude/one-model-end-to-end-served-key-2026-10-31/run-<sha8>/` with `SHA256SUMS`, and read back. Then `down`.
7. **The grade, on the laptop:** `train-in-space --served-grade 1 --runs OUT --panel p-w=DIR --panel p-v=DIR` writes `grade-p-w.json`; then the REPORT.

## Bar

**The served arm S is read against greedy G** (memory on), paired on the same prompts, at each seed, on p-w. In each pair, b counts S right and G wrong, c the reverse, and the band is 2·√(b + c). S **carries** when all four of these hold at all three seeds (`KeyedReading::carries`, as in claude-219):
- **(a) Templates:** value-masked template matches, b − c > band, with b + c ≥ 10.
- **(b) Exact replies:** b − c ≥ −band.
- **(c) Loops:** b − c ≤ band, with b counting S looping where G does not.
- **(d) DEV:** S's templates, b − c ≥ −band.

**The decision table**, read in order on p-w (`keyed_outcome`'s rows):
1. **F does not lift at some seed** (its exact pair not beyond its band with b + c ≥ 10): an instrument check. Repair.
2. **S carries:** the one float model serves the point key. The next unit is its integer export, under its own roadmap item (the decision's condition (d)), with a bar measured on the integer trunk's own points (condition (b)); then the integer model end to end.
3. **S is thin** (template b + c under 10) at some seed: underpowered. Next: a larger panel, or the seeds pooled under a fresh sheet.
4. **Otherwise:** the served key does not carry on a second fresh panel, and claude-219's p-v reading stands alone. Next: train the trunk's opening point to gather paraphrases.

**Instrument checks.** The grade or the job refuses on any of these. Each is an instrument fault, repaired, never a negative (CHARTER §4.1):
- every served token equals the run's keys-file name where the exact key misses, by `keyed_reading` at +∞, on p-w, p-v and DEV;
- every served reply is the one its opening fixes;
- the parity and reproduction byte checks of steps 4–5;
- the table reads back to the same bytes;
- F's replies open with their reference's first token, and every arm reads G's prompts.

**Reported, deciding nothing:**
- **tok/s first:** every arm's decode, and S's with its search's count and seconds inside it, a single query per reply (condition (c));
- **the protocol's free-generation measures** for G and S on every split (`generation`): complete replies, verbatim TRAIN replies, the share of new n-grams for n = 4–8, distinct-1 and distinct-2, and the repeated 4-gram rate;
- **residency** (condition (e)): 218 MB of table resident a checkpoint, 654 MB in the store for three seeds, and 218 MB scanned a keyed opening;
- G0, Flock-end, and p-v's readings.

## Minimum test power

- **On p-v at 393 prompts, claude-219's keyed-point arm** read templates (16, 3), (9, 1) and (14, 2): b + c of 19, 10 and 16. Seed 2 was at the floor.
- **p-w is a draw from the same cell,** with a similar count of prompts (about 400, every conversation's first opening). So its b + c should be similar, 10–20 a seed.
- **So the replication is powered for a gain the size of claude-219's,** but a seed near the floor can read thin (row 3), and the REPORT would say so. It is a power limit, not a negative.
- **The parity instrument has no power question:** it is a byte check.

## Stop rule

- **An instrument fault** (a reproduction or parity difference, the table's read-back, or a refusal in the grade): stop and repair, then rerun on a new pod. It is never counted.
- **The probe projects past 4 hours:** stop, and fix the code.
- **A seed's scoring fails:** stop, and classify it as instrument or mechanism first.
- **The caps:** one pod; at most 4 pods and $8/h across the labs. The lease is renewed while the run is healthy, and a healthy run is never cancelled for a self-set budget.

## Cost

One RTX 5090 on Vast, about $0.56/h:
- the setup, about 15 minutes;
- the probe, about 20 minutes;
- the main run, about 2–2.5 hours, with three seeds sharing the GPU (claude-219's two-split run took about 90 minutes).

About $2 in all. The sealed run carries the three seeds' tables, 654 MB. The grade takes minutes on the laptop.
