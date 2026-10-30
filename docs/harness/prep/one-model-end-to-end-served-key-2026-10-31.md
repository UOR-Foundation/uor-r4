# Prep sheet: the point key served in the one model (one-model-end-to-end, claude-220, 31 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-220](../../loop/tasks/one-model-end-to-end/claude-220/ORDER.md), written by hand on Casey's word: "screw kimi forge ahead as you were before with your plan for the single serving model". Decision: [DECISION.md](../../loop/tasks/one-model-end-to-end/claude-220/DECISION.md), APPROVE on five conditions. It follows claude-219 ([result](../../labs/one-model-end-to-end/keyed-opening-2026-10-30/README.md)).

TEST FITNESS: PENDING (round 3; a fresh non-author reviewer reads this sheet and its sources before any pod).

Round 2 was NOT FIT (a fresh non-author reviewer, 31 Oct, at `cad0ae00f`), on one blocking point, repaired here before any model read p-w:
- **The pooled guard could label a harm as "underpowered."** Any row 4 turned to row 3 when the pooled templates gained, even where a seed failed on loops, exact replies or DEV, which a larger panel cannot fix.
- **Repaired:** the guard applies only when every seed is harmless on bars (b)–(d) (`KeyedReading::harmless`), so the failing seeds miss on (a) alone. A seed with a harm keeps row 4. A unit case pins it.

Its notes are taken in:
- the probe's served-file byte check strips `served_distance` first, so DEV's parity reads before the main run;
- parity with no served opening fails;
- the job's header names its flags.

Round 2 found round 1's repairs holding: power, rule (a), the 1% threshold and row 1, and the Steps against the job.

Round 1 was NOT FIT (a fresh non-author reviewer, 31 Oct, at `17c5cef9f`), on two blocking points, each repaired here before any model read p-w:
1. **Power.** With S required to carry at all three seeds at claude-219's b + c (19, 10, 16), the chance of row 2 under a real effect was about 0.25–0.4. Row 4 could have come from an underpowered panel.
   - **Repaired:** p-w is redrawn at 1,200 conversations (seed 2121), for about three times the discordant pairs a seed.
   - Row 4 now fires only where the seeds pooled show no gain beyond their band; otherwise the reading is row 3.
2. **No rule fixed in advance for a served name that differs from the keys file's,** though the two come from different forwards. A refusal on p-w would have forced a choice after reading it.
   - **Repaired by rule (a):** S is read by its own served tokens.
   - Each served sample records its search's least distance. The grade counts, at every split and seed, the served tokens that differ from the keys file's name, and measures the distance drift where they agree (`served_parity`).
   - Parity fails only past 1% disagreeing.
   - The byte checks of the served files against claude-219's are reported, not stops.

Its non-blocking notes are taken in:
- the row-1 text names p-w;
- the table's read-back is "the same values", and its file's sha256 is recorded;
- the job's exact checks are named in Steps.

## The question

**Served inside the model, does the point key still carry the reply, on a second fresh panel? And what does that model's free generation look like?**

- **claude-219 read the key from a keys file** computed before the decode. Its keyed-point arm carried the reply at all three seeds on p-v: templates (16, 3), (9, 1) and (14, 2) against greedy.
- **This unit serves the key from inside the decode** (`served_opening`). The reply's first forward gives the opening row's own point. Where the memory's exact key misses, that point is named by the commonest target among the served table's L1-nearest TRAIN openings, with ties to the smaller id. The table is TRAIN's 212,968 opening points under the trunk, built once, saved, read back to the same values, and served from the copy read back.

**What each outcome unlocks** is set out in the decision table under Bar:
- **the served key carries:** the one float model serves it; next comes its integer export, under its own item;
- **it does not carry, even pooled:** claude-219's reading stands alone, and the next unit trains the trunk's opening point;
- **too thin, or below the bar at a seed while the pooled seeds gain:** a larger panel;
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
| S | `…-served-memory.json` | The served point key: each given token kept as the sample's `keyed`, its least distance as `served_distance`, with the search's count and seconds |
| Flock-end | `…-flock-end-memory.json` | As in every run (reported) |

**The served key has no τ:** it is the point key at +∞, as claude-219's levels chose (level 10). It fires wherever the exact key misses.

## Data split

- **TRAIN, DEV and the tokenizer:** the pinned `w2-inputs/2026-10-25/driver-data`.
- **p-w decides.** It is a fresh held-out-phrasing panel at seed 2121, which no unit had used. It was first drawn at 400 conversations and redrawn at 1,200 for power (FIT round 1) before any model read either:

  `heldout-panel out=p-w cell=dev,dev conversations=1200 seed=2121 protocol=2`

  - **The files:** `tokens.u16` sha256 `038b0085e6888d988d4b0b6ceb7846015f21f1c40f26ec3286ddcc049c17d217`, `response_mask.u8` `797fc23474231595ae42b9cfbf7930ae2021b13d23d8b8b3d4fb3c7dda06ec16`. A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-w1200-2026-10-31/` with `SHA256SUMS`, and read back OK. The 400-conversation draw stays at `inputs/claude/heldout-panel-p-w-2026-10-31/`, unread by any model.
  - **The census** ([census-p-w-2026-10-31.json](../../labs/one-model-end-to-end/census-p-w-2026-10-31.json)) counts 4,340 openings:
    - 0.000 of its prompts are in TRAIN, exactly or as templates at K = 1,024 (0.047 at K = 256);
    - 0.647 of its replies are verbatim TRAIN replies;
    - the exact key fires on 721 openings.
  - **No model has read it.**
- **p-v**, claude-219's deciding panel, carries the parity check against claude-219's own files. It is reported, deciding nothing.
- **DEV** carries the not-worse condition (d).
- **The prompts:** `--samples 1536`. `sample_windows` keeps the windows whose first scored target opens a reply after at least 16 prompt tokens, and takes every `max(1, eligible / 1536)`-th.
  - Each conversation fits one 192-token window, so p-w has about 1,200 eligible prompts, and DEV and p-v about 310 and 393.
  - Each is read whole (the stride is 1). DEV's and p-v's are the same prompts claude-219 read at 512.

## Steps

1. **On the laptop, done:**
   - the build and its unit tests: `the_opening_table_reads_back_as_saved_and_is_never_replaced` and `the_served_parity_counts_disagreements_and_the_pooled_seeds_guard_row_4`, with negative gates;
   - the CUDA test `the_served_key_names_a_missed_opening_from_the_decodes_own_point`, type-checked with the stub nvcc;
   - the grade (`served_grade`), written before the pod;
   - p-w and its census.
2. **The FIT and the pre-run review:** each by a fresh reviewer, posted on #3407.
3. **The pod:** one RTX 5090 on Vast (`uor-pod up --count 1`). The source is pinned to the unit's head. The crate's CUDA tests must pass, including the new one and claude-218's and claude-219's named ones. The binary is kept with its sha256, and every input is checked against its `SHA256SUMS` and this sheet's sha256s.
4. **The reproduction probe:** seed 1, DEV only, with `--samples 1536 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1 --served-key 1`. Against claude-219's sealed run (`run-da4a6212/out/r-drop-s1/`, checked against its `SHA256SUMS`), it must reproduce:
   - **six DEV files, `cmp -s` byte for byte:** `rows-dev-memory.json`, `rows-dev-no-memory.json`, `samples-memory.json`, `samples-no-memory.json`, `samples-dev-flock-end-memory.json` and `samples-dev-forced-memory.json`;
   - **`keys-dev.json`,** compared after `sed -E 's/"(point|sign|hsign)_ms":[0-9]+//g'` on both sides.

   It also reports, without stopping, whether `samples-dev-served-memory.json`, with its `served_distance` fields stripped (`sed -E 's/,"served_distance":[^,}]*//g'`), equals claude-219's `samples-dev-keyed-point-memory.json` byte for byte.

   Any other difference is an instrument fault. The probe's time projects the main run, at most 18 times it (about 6 times the replies, on three seeds sharing the GPU): past 5 hours, stop.
5. **The main run:** seeds 1–3 side by side, with the probe's flags plus `--panel p-w=DIR --panel p-v=DIR`. Afterwards:
   - at every seed, the same checks must hold on p-v and DEV against claude-219's main run;
   - the served-file byte equality is reported, on p-v and DEV.
6. **Sealing.** The job checks that every expected file is written, with `opening-table.safetensors` at every seed and its sha256 recorded. The run is sealed off the pod to `results/claude/one-model-end-to-end-served-key-2026-10-31/run-<sha8>/` with `SHA256SUMS`, and read back. Then `down`.
7. **The grade, on the laptop:** `train-in-space --served-grade 1 --runs OUT --panel p-w=DIR --panel p-v=DIR` writes `grade-p-w.json`; then the REPORT.

## Bar

**The served arm S is read against greedy G** (memory on) by its own served tokens (rule (a)), paired on the same prompts, at each seed, on p-w. In each pair, b counts S right and G wrong, c the reverse, and the band is 2·√(b + c). S **carries** when all four of these hold at all three seeds (`KeyedReading::carries`, as in claude-219):
- **(a) Templates:** value-masked template matches, b − c > band, with b + c ≥ 10.
- **(b) Exact replies:** b − c ≥ −band.
- **(c) Loops:** b − c ≤ band, with b counting S looping where G does not.
- **(d) DEV:** S's templates, b − c ≥ −band.

**The decision table**, read in order on p-w (`served_outcome`):
1. **Parity fails at some split and seed** (more than 1% of the served openings name otherwise than the keys file), **or F does not lift on p-w at some seed** (its exact pair not beyond its band with b + c ≥ 10): an instrument check. Repair.
2. **S carries:** the one float model serves the point key. The next unit is its integer export, under its own roadmap item (the decision's condition (d)), with a bar measured on the integer trunk's own points (condition (b)); then the integer model end to end.
3. **S is thin** (template b + c under 10) at some seed, **or it would be row 4 while every seed is harmless on bars (b)–(d) and the seeds pooled show a template gain beyond their band:** underpowered at some seed. Next: a larger panel, or the seeds pooled under a fresh sheet. A seed that fails (b), (c) or (d) is a harm, and keeps row 4.
4. **Otherwise:** the served key does not carry on a second fresh panel, even pooled, and claude-219's p-v reading stands alone. Next: train the trunk's opening point to gather paraphrases.

**Instrument checks.** The grade or the job refuses on any of these. Each is an instrument fault, repaired, never a negative (CHARTER §4.1):
- parity past 1% (row 1);
- a served reply its own token does not fix (`keyed_reading`);
- a missed opening the served memory's exact key claims to fire on;
- a served token without its distance;
- the reproduction checks of steps 4–5;
- the table's read-back;
- F's replies not opening with their reference's first token, or an arm not reading G's prompts.

**Reported, deciding nothing:**
- **tok/s first:** every arm's decode, and S's with its search's count and seconds inside it, a single query per reply (condition (c));
- **parity:** each split's and seed's agreeing count, and the largest distance drift where the names agree. That measures whether the decode's point and the batched forward's are equal, not assumes it;
- **the protocol's free-generation measures** for G and S on every split (`generation`): complete replies, verbatim TRAIN replies, the share of new n-grams for n = 4–8, distinct-1 and distinct-2, and the repeated 4-gram rate;
- **residency** (condition (e)): 218 MB of table resident a checkpoint, 654 MB in the store for three seeds, and 218 MB scanned a keyed opening;
- G0, Flock-end, p-v's readings, and the served-file byte equality.

## Minimum test power

- **On p-v at 393 prompts, claude-219's keyed-point arm** read templates (16, 3), (9, 1) and (14, 2): b + c of 19, 10 and 16, with b about 0.87 of the discordant pairs.
- **p-w has about 1,200 prompts,** so b + c should be about three times that: roughly 30–55 a seed.
- **At b + c = 30 and b about 0.87 of it,** net 22 against a band of 11, and no seed is near the floor of 10. So each seed carries with high probability if the effect is claude-219's size.
- **claude-219's estimate passed a filter,** so the true effect may be smaller. Hence row 3's pooled guard: a seed below the bar while the pooled seeds gain is read as underpowered, never as row 4.
- **The parity count has no power question:** it counts every served opening.

## Stop rule

- **An instrument fault** (a reproduction difference, parity past 1%, the table's read-back, or a refusal in the grade): stop and repair, then rerun on a new pod. It is never counted.
- **The probe projects past 5 hours:** stop, and fix the code.
- **A seed's scoring fails:** stop, and classify it as instrument or mechanism first.
- **The caps:** one pod; at most 4 pods and $8/h across the labs. The lease is renewed while the run is healthy, and a healthy run is never cancelled for a self-set budget.

## Cost

One RTX 5090 on Vast, about $0.56/h:
- the setup, about 15 minutes;
- the probe, about 15 minutes;
- the main run, about 3–3.5 hours, with three seeds sharing the GPU. claude-219's two-split run took about 90 minutes for about 4,900 replies a seed; this one decodes about 9,500.

About $2.5 in all. The sealed run carries the three seeds' tables, 654 MB. The grade takes minutes on the laptop.
