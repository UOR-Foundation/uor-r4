# Prep sheet: the keyed opening in free generation (one-model-end-to-end, claude-219, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-219](../../loop/tasks/one-model-end-to-end/claude-219/ORDER.md), written by hand on Casey's word: "screw kimi forge ahead as you were before with your plan for the single serving model". It follows claude-218 ([result](../../labs/one-model-end-to-end/memory-key-2026-10-30/README.md)).

TEST FITNESS: FIT (round 2, a fresh non-author reviewer, 30 Oct, at `42dc7ff50`). The rounds are below.

Round 2 was FIT, with no blocking points. It found row 1's repair in the sheet, the code and the test, and the rest of the sheet matching the code. Its notes are taken in:
- a keyed token of BOS is now a refusal in the grade, as the instrument checks say, not a tolerance;
- where row 4 holds while sign and hsign are thin, the REPORT names the key that decided it.

Round 1 was NOT FIT (a fresh non-author reviewer, 30 Oct, at `4ef780f87`), on one blocking point, repaired here:
- **Row 1** fired only when the forced arm failed to lift at all three seeds. So one seed's fault would have fallen to row 4 and read as a mechanism negative. Row 1 now fires when the forced arm does not lift at any one seed (`keyed_outcome`), with a unit test for one flat seed and for one too-thin seed.

Its non-blocking notes are taken in too:
- the sample rule is stated exactly (Data split);
- the probe now also decodes the keyed arms on DEV, so a fault in them costs the probe, not the main run (Steps 4);
- a key never names BOS (the instrument checks).

## The question

**Does a trunk key's opening carry the reply when it is served in free generation?**

claude-218 showed two things, teacher-forced, on the same (r-drop) checkpoints:
- **The trunk keys name missed openings better than the trunk alone.** On p-u's 1,218 openings that the memory's exact first-8 key misses: sign +46/+37/+33, point +125/+185/+114, hsign +46/+33/+26, each beyond its band. Row 3 shipped the sign key.
- **A right first token is worth a lot.** Forcing each reply's reference first token lifts exact replies from 4/0/0 to 50/49/52 of 256 on p-u.

This unit serves the key, and asks whether the gain survives the decode:
- **The rule ([`keyed_opening`]):** at a reply's opening where the served memory's exact key misses, the key's named token is the reply's first, when the key's distance is at most τ.
- **Everywhere else**, and for every later token, the decode is greedy with the memory on, as served.

**What each outcome unlocks** is set out in the decision table under Bar. In short:
- **a key carries:** the next unit trains and serves that opening in the one model and scores it end to end (#3407, step 4);
- **none carries:** the next unit trains the trunk's opening point itself;
- **too thin to tell:** a larger panel;
- **the forced arm does not lift:** an instrument repair.

## Base artifact: the model and the arms

**The model.** (r-drop)'s step-16,384 checkpoints at seeds 1–3: claude-211's training, the ones claude-217 and claude-218 scored. Each has its turn memory, the opening first-8 table and the reply table. They are in the HF store under `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/models/step-016384/`. The job checks each seed's 11 files against that seed's `SHA256SUMS`, as claude-218's job did.

**The code.** `train-in-space --score-rows CKPT --opening-keys 1 --keyed-decode sign=T,hsign=T,point=T`, at this unit's head (`crates/uor-r4-float-train/src/in_space_long.rs`). Every arm decodes 64 reply tokens with the nearest-cell objective, on the same prompts ([`sample_windows`]):

| Arm | File | What it is |
|---|---|---|
| G | `samples-p-v-greedy-memory.json` | Greedy, memory on: what is served now |
| G0 | `…-greedy-no-memory.json` | Greedy, memory off (reported) |
| F | `…-forced-memory.json` | The reference's first token, then greedy: the ceiling and the instrument row |
| K-sign, K-hsign, K-point | `…-keyed-{key}-memory.json` | The keyed opening: each forced token is kept as the sample's `keyed` |
| Flock-end | `…-flock-end-memory.json` | Decoded as in every run (reported) |

DEV's files carry the same names, with DEV's greedy at `samples-memory.json`.

**τ is never chosen on p-v.** It is `KEYED_TAUS`: claude-218's levels (sign 2, hsign 1, point 10), chosen on its development panel p-s at each seed and committed in `grade-p-s-levels.json`:
- sign: 0.0078125, 0.01953125 and 0.0078125 at seeds 1–3 (2 and 5 of 256 sign bits);
- hsign: 0;
- point: +∞, so it fires on every opening the exact key misses.

**Whether the exact key fires** is the served memory's own first-8 lookup on the decode's first window (`TurnMemory::stored`). The grade recomputes it on the laptop from TRAIN's first-8 table. The turn anchor makes the two the same (claude-218's anchor check, run by the grade at every opening), and the grade refuses any keyed token that differs.

**The key names and distances** are claude-218's instrument, unchanged: `opening_points` with the memory off, then `l1_named`. They are now taken before the decodes and written after them, as before.

## Data split

- **TRAIN, DEV and the tokenizer:** the pinned `w2-inputs/2026-10-25/driver-data`, with their sha256s in the code.
- **p-v decides.** It is a fresh held-out-phrasing panel, drawn on 30 Oct at seed 2111, which no unit had used:

  `heldout-panel out=p-v cell=dev,dev conversations=400 seed=2111 protocol=2`

  - **The files:** `tokens.u16` sha256 `3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0`, `response_mask.u8` `c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023`. A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-v-2026-10-30/` with `SHA256SUMS`, and read back OK.
  - **The census** ([census-p-v-2026-10-30.json](../../labs/one-model-end-to-end/census-p-v-2026-10-30.json)) counts 1,402 openings:
    - 0.000 of its prompts are in TRAIN, exactly or as value-masked templates at K = 1,024 (0.053 at K = 256);
    - 0.650 of its replies are verbatim TRAIN replies;
    - the exact key fires on 209 openings, so 1,193 are missed.
  - **No model has read it.** The grade checks both sha256s.
- **DEV** carries the not-worse condition (d).
- **p-u**, claude-218's deciding panel, already read, gives only the power check below.
- **The prompts:** `--samples 512`. `sample_windows` keeps the windows whose first scored target opens a reply after at least 16 prompt tokens, then takes every `max(1, eligible / 512)`-th of them from the front, up to 512.
  - A conversation of p-v (125 tokens on average) or DEV (107) fits in one 192-token window, so each split has about 400 eligible windows: one per conversation, plus the few whose second window starts on a reply.
  - So the stride is 1, and every eligible prompt is read: each conversation's first opening. The grade records the count.
  - Every arm reads the same prompts. claude-218's 256 were the first 256 of the same list.

## Steps

1. **On the laptop, done.**
   - The build, and the unit tests: `a_keyed_opening_fires_only_where_the_exact_key_misses_within_tau`, `the_keyed_grade_predicts_each_keyed_reply_and_reads_the_table_in_order` and `keyed_decode_names_each_trunk_key_once_with_a_tau_of_0_or_more`.
   - Two negative gates, each failing as it should: without the miss rule, and without the memory's lookup.
   - The CUDA type-check with the stub nvcc.
   - p-v and its census.
   - The power check, run on claude-218's files ([power-p-u.json](../../labs/one-model-end-to-end/keyed-opening-2026-10-30/power-p-u.json)).
2. **The FIT and the pre-run review:** each by a fresh reviewer, posted on #3407.
3. **The pod.** One RTX 5090 on Vast (`uor-pod up --count 1`).
   - The source is pinned to this unit's head.
   - The crate's CUDA tests run, and claude-218's named GPU tests must pass.
   - The binary is built and kept with its sha256.
   - Every input is fetched and checked against its `SHA256SUMS` and this sheet's sha256s.
4. **The reproduction and throughput probe:** seeds 1–3 side by side, DEV only, at claude-218's flags, `--samples 256 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1`, plus the keyed arms at each seed's τ (`--keyed-decode`, as extra files).
   - **Six files a seed** must equal claude-218's registered ones (`run-4cc00df2/out/r-drop-s{S}/`, checked against that run's `SHA256SUMS`) byte for byte. At seed 1 they are:
     - `rows-dev-memory.json` (`c4b26474…83e25`) and `rows-dev-no-memory.json` (`f7d09b98…6be67`);
     - `samples-memory.json` (`2ca7b299…f289f`) and `samples-no-memory.json` (`17501f36…6c991`);
     - `samples-dev-flock-end-memory.json` (`8dc0c897…3b530`) and `samples-dev-forced-memory.json` (`f19fb37f…1a408`).
   - **A seventh,** `keys-dev.json`, must equal the follow-up pass's (`followup-d5b6f49f/out/r-drop-s{S}/`; at seed 1 `d9c1aba4…cc8ee`), which carries the Hadamard key. So 21 files in all.
   - **Why:** this shows that moving the keys pass and sharing the forced arm's decode changed no byte. Any difference is an instrument fault.
   - **The keyed arms run here first,** so a fault in them costs the probe, not the main run.
   - **The probe's time** projects the main run, which decodes at most 4 times its replies (two splits of 7 arms at up to 512, against one split of 7 at 256). Past 3 hours, the job stops.
5. **The main run:** seeds 1–3 side by side on the one GPU, each with:

   `--samples 512 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1 --panel p-v=DIR --keyed-decode sign=τ_S,hsign=0,point=inf --seed S`

   Here τ_S is the seed's sign τ above.
6. **Sealing.** The job checks that every expected file is written. The run is sealed off the pod to `results/claude/one-model-end-to-end-keyed-opening-2026-10-30/run-<sha8>/` with `SHA256SUMS`, and read back. Then `down`.
7. **The grade, on the laptop:** `train-in-space --keyed-grade 1 --runs OUT --panel p-v=DIR` (`keyed_grade`) writes `grade-p-v.json`; then the REPORT.

## Bar

**Each keyed arm K is read against G** (memory on), paired on the same prompts, at each seed, on p-v. In each pair, b counts K right and G wrong, c the reverse, and the band is 2·√(b + c). A key **carries** when all four of these hold at all three seeds (`KeyedReading::carries`):
- **(a) Templates:** value-masked template matches (the census's mask, at K = 1,024): b − c > band, with b + c ≥ 10.
- **(b) Exact replies** (cut at the end of turn): b − c ≥ −band.
- **(c) Loops** (a reply repeating a 4-gram of its own): b − c ≤ band, with b counting K looping where G does not.
- **(d) DEV:** the same key's templates, b − c ≥ −band.

**The decision table**, read in order on p-v (`keyed_outcome`):
1. **F does not lift at some seed:** its exact pair is not beyond its band with b + c ≥ 10 there. That is an instrument check against claude-218, whose forced arm lifted 49–59 of 256 at every seed on p-s and p-u. Repair, and rerun.
2. **A key carries:** the first of sign, hsign and point, the cheapest first, ships. The next unit trains and serves that opening in the one model, and scores it end to end.
   - If point ships, that unit's decision review weighs its table: TRAIN's 212,968 opening points, 218 MB as f32, about 1 KB an opening, against the sign key's 32 bytes.
3. **Every key thin** (template b + c under 10) at some seed: underpowered. The next unit draws a larger panel, or pools the seeds under a fresh sheet.
4. **Otherwise:** the key's opening does not carry in free generation. The next unit trains the trunk's opening point to gather paraphrases, since the teacher-forced gain shows the trunk's geometry already orders them.

**Instrument checks.** The grade refuses on any of these. Each is an instrument fault, repaired, never a negative (CHARTER §4.1):
- every keyed token equals `keyed_opening`'s, at the registered τ, with the laptop's exact key;
- **every keyed reply is the one its opening fixes** (`keyed_reply`):
  - no keyed token: G's reply;
  - the reference's first token: F's reply;
  - G's own first token: G's reply;
  - any other token: the reply opens with it. A key's name is a TRAIN reply's first token, never BOS.

  So the observed exact pairs must equal the predicted ones;
- F's replies open with their reference's first token, and every arm reads G's prompts;
- `seed_picks`' checks hold: the keys name the split's openings with their targets, and the memory moves no missed opening;
- step 4's reproduction.

**Reported, deciding nothing:**
- G0 and Flock-end;
- each arm's keyed and right counts, and its predicted exact pairs;
- every arm's decode tok/s, first;
- DEV's every reading.

## Minimum test power

**The prediction from claude-218's files.** By the identity above, a keyed arm's exact replies follow from G and F alone. The grade's power check (`--keyed-grade 1 --panel p-u=…` on claude-218's run, [power-p-u.json](../../labs/one-model-end-to-end/keyed-opening-2026-10-30/power-p-u.json)) predicts, at 256 replies a seed on p-u:
- **K-point:** exact pairs (11, 1), (13, 0) and (12, 0). That is b + c of 12–13, with a net of 10–13 against bands of 6.9–7.2. Its forced openings are right on 83–90 of the 249 it gives.
- **K-sign:** (2, 0), (2, 0) and (1, 0).
- **K-hsign:** (2, 2), (3, 0) and (4, 0).

**What that means here:**
- **Templates track exact replies.** F's template pairs (52, 0), (49, 0) and (54, 0) sit beside its exact ones, (50, 0), (49, 0) and (52, 0). So (a) is powered for the point key at 256, if p-v is like p-u.
- **More prompts.** p-v's roughly 400 eligible prompts, about 85% of them missed, should give about 1.5 times p-u's b + c.
- **sign and hsign** are expected to be thin at this panel's size. A row 3 or 4 for them is a power limit, and the REPORT says so.
- **What the run adds** beyond the prediction: the replies after a wrong keyed opening (K-point gave about 165 of 256 on p-u), whose templates and loops only the decode can show; DEV under the keys; and the speed.

## Stop rule

- **An instrument fault** (a step-4 difference, or a refusal in the grade): stop and repair, then rerun on a new pod. It is never counted.
- **The probe projects past 3 hours:** stop, and fix the code.
- **A seed's scoring fails:** stop, and classify it as instrument or mechanism before anything else.
- **The caps:** one pod; at most 4 pods and $8/h across the labs. The lease is renewed while the run is healthy, and a healthy run is never cancelled for a self-set budget.

## Cost

One RTX 5090 on Vast, about $0.4–0.7/h:
- the setup, about 15 minutes;
- the probe, about 10–15 minutes;
- the main run, about 1.5–2 hours, with three seeds sharing the GPU. claude-218's three-split run took about 50 minutes a seed at 256 replies, and this one decodes about 1.8 times the replies.

About $1–2 in all. The grade takes minutes on the laptop.
