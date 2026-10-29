# Prep sheet: the scoring protocol for the model to ship (one-model-end-to-end, claude-206, 29 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md), done-when 3. Order: [claude-206](../../loop/tasks/one-model-end-to-end/claude-206/ORDER.md). It builds on [the assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) §5–§6 and on step 2's result ([claude-205](../../loop/tasks/one-model-end-to-end/claude-205/REPORT.md): mixed).

TEST FITNESS: (to be filled by the fresh reviewer). Round 1 at `8aec46877` was NOT FIT, with six blockers; this is round 2. What changed is at the end.

This sheet fixes, before any training run, how the model the project ships is scored, and what each outcome decides. The run needs the instruments listed below, and its own pre-run review (D67) once they exist.

## The question

**Which trained recipe should the project ship, and does it read the prompt, or only recall TRAIN's templates?**
- **What step 1 found.** Three quarters of DEV's and Fresh V2's replies are verbatim TRAIN replies (claude-202). P-dev's prompt templates are absent from TRAIN, while its replies are not (claude-204).
- **What step 2 found** (claude-205):
  - the memory recipe and a recipe that trains without it disagree in sign across seeds, on the rows the memory cannot name;
  - (r) loops under greedy decoding;
  - neither trunk puts a rare value from the prompt into its reply.
- **What this sheet is for.** Every decision below is therefore made on rows no table names, on DEV and on a panel no decision has touched, and against recall controls.

## The arms

**The base recipe is claude-194's (r):** `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --seed S`, at width 256 on the pinned TRAIN, with the fixed-order backward (claude-179). Each trained arm changes one thing:

| Arm | Change | Role |
|---|---|---|
| **(r)** | none | a candidate to ship |
| **(r-drop)** | memory dropout, p = 0.5 per training window (I-B) | a candidate to ship (step 2's mixed cell) |
| **(r0)** | `--turn-memory-empty 1` (both tables empty) | the no-memory control (L-010) |
| **(r-flock)** | `--flock-read 7` (claude-144's K) | add-on: reading by meaning |
| **(r-spin)** | `--type-spin 1` | add-on: the type heads' spin |
| **(r-ident)** | transport fixed to the identity (`FbsArm::IdentityTransport`, I-C) | transport control |
| **(r-frozen)** | routing queries frozen at their seed (`FbsArm::FrozenQueries`, I-C) | transport control |

- **Reported arms, scored, not trained.**
  - **(r-off)** is (r) scored with its memory taken out (`score_rows`' second pass), and **(r-drop-off)** likewise for (r-drop).
  - On U and N-u they equal their arm by construction (claude-205 I-3: the memory names none of those rows). On K they show how much the trunk leans on the tables.
- **Seeds.** Every trained arm runs at seeds 1, 2 and 3. Step 2 showed that two seeds can disagree in sign beyond their own noise.
- **Scoring with tables.** Memory dropout acts in training only. Every trained arm is scored with its own tables, as it would be served.

## The controls (counted on the laptop, no model)

- **(T) tables only.** At a row the opening table or the reply table names, the stored token; at any other row, a miss. It is K's bar.
- **(S) longest suffix (an ∞-gram over TRAIN; Liu et al. 2024, arXiv:2401.17377).** At each row:
  - take the longest suffix of the window's tokens, up to the row, that occurs in TRAIN;
  - predict TRAIN's commonest next token after it, ties to the smaller id.

  It is the strongest lawful recall control (CHARTER §3.10, §4.4).

## The strata (teacher-forced, nearest top-1)

| Stratum | Rows | Count | Judged |
|---|---|---|---|
| **K: keyed (DEV)** | DEV's openings the first-8 table names, and its later rows the reply table names at share 0.5 | 1,194 + 9,655 = 10,849 | against (T) and (r0) |
| **U: unkeyed (DEV)** | DEV's 3,406 unnamed later rows and its 32 unkeyed openings (claude-202's decision rows) | 3,438 | against (r0) and (S) |
| **N-u: new template, unkeyed** | P-dev's rows that TRAIN's tables do not name, by U's rule | set by I-G before the run | against (r0) and (S) |
| **N-k: new template, keyed** | P-dev's other rows | 16,417 − N-u | reported |
| **C: copied** | The rows of each stratum that copy the prompt (`copies_prompt`) | per stratum | reported |

- **P-dev itself:** `dev,dev` cell, `tokens.u16` sha256 `790b7e12934a241f07ec5c04fc6e929c86e970583ba5dee9a8646f7d235266cd`, at HF `inputs/claude/heldout-panels-2026-10-29/p-dev`.
- **P-dev has touched no decision.** claude-204 built it, and no unit has scored a model on it. This protocol opens it once, for the run it registers. Any later decision, such as a combination of add-ons, needs a new panel: `heldout-panel` at a new seed, with its own census.
- **Fresh V2 is never a stratum.** It is reported per arm, descriptive only.

## The band and the cell

For arm X against arm or control Y, on one stratum, at each seed `s` in {1, 2, 3}:
- **The counts.** `b(s)` is the rows only X gets, `c(s)` the rows only Y gets, and `d(s) = b(s) − c(s)`.
- **The band.** `band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d(s) − min_s d(s))`, the paired floor or the spread of `d` over the three seeds. A control has no seed: against (T) or (S), `d(s)` varies only with X's seed.
- **The cell.**
  - X is **ahead** of Y if `d(s) > band(s)` at all three seeds;
  - X is **behind** if `d(s) < −band(s)` at all three seeds;
  - otherwise the cell is **inside**.
- **What "inside" means.** It means the comparison is underpowered at this scale. **It is never read as a negative, nor as "no difference"** (L-007). This is claude-205's rule (`seed_cell`, `overturn_verdict`), with the spread over three seeds.

## The decisions, fixed in advance, in this order

1. **The base recipe to ship: (r) or (r-drop).** Compare (r-drop) with (r):
   - if (r-drop) is ahead on N-u, it ships; if (r) is ahead on N-u, (r) ships;
   - else, if either is ahead on U, it ships;
   - else (r) ships, the recipe already served in part.

   The shipped recipe is always a trained arm.
2. **The add-ons.** (r-flock) and (r-spin) are each compared with (r) on U and on N-u. Each gets one of three readings:
   - **a candidate** if it is ahead on N-u and not behind on U;
   - **a negative for this configuration** (D22) if it is behind on N-u;
   - **no decision** (underpowered) otherwise.

   A candidate does not join the shipped recipe here. Joining it, alone or combined, is a new recipe that needs its own run, sheet and new panel. The type spin cannot be served until the integer engine serves `step_rotation`, which `fbs_export.rs:382-384` refuses today.
3. **The transport.** (r-ident) and (r-frozen) are each compared with (r) on U and on N-u.
   - **Learned transport earns its place** if both controls are behind (r) on N-u.
   - **A negative for learned transport in this configuration** if either control is ahead of (r) on N-u.
   - **No decision** otherwise.

   No arm is cut on this; a negative revisits the map's transport stage.
4. **The decode rule, on the shipped recipe only: greedy (G) or Flock rank (F).** At each seed, its free generation under each rule is 512 replies: 256 DEV prompts and 256 P-dev prompts. Let `g` and `f` be the counts of complete replies under G and F, and `p = g / 512`.
   - F ships if at all three seeds both hold:
     - F's repeated-4-gram rate is below G's;
     - `f ≥ g − ⌈2·√(512 · p(1 − p))⌉`, a count against a count.
   - Else G ships.

   Teacher-forced top-1 does not depend on the decode rule.
5. **The verdict, on the shipped recipe with the shipped decode rule.**
   - **The free-generation tests** use its 256 P-dev replies at each seed, against P-dev's own references for the same prompts. Let `v` be its verbatim-TRAIN share of complete replies, `v_ref` the references' share, and `n_c` its complete count.
   - **The tolerance** is `tol = max(0.10, 3·√(v_ref(1 − v_ref) / n_c))`, so that it widens when few replies complete.
   - **The verdicts:**
     - **"generalizes":** it is ahead of both (r0) and (S) on U and on N-u; and at every seed, `n_c ≥ 64`, `|v − v_ref| ≤ tol`, and its repeated-4-gram rate is at most the references' + 0.10;
     - **"memorized"** (the spec's AND): it is ahead of (r0) on K, and ahead of neither (r0) nor (S) on U or N-u, and `v − v_ref > tol` at every seed;
     - **"mixed":** otherwise. The REPORT names which condition failed.
   - **The 0.10 floor** is about 3 binomial standard errors at 256 replies with p = 0.64 (claude-204: P-dev's replies are 0.640 verbatim). It can fail on real data: claude-205's (r) at seed 1 had 35 of 44 complete replies verbatim (0.80) against the references' 0.64.

## The decode rules

- **(G) greedy:** the whole table's nearest token, as `greedy_reply` decodes today.
- **(F) Flock rank.**
  - **The candidates:** the K = 16 table tokens nearest the readout point, by the nearest objective's score, ranked 0 to 15, ties to the smaller id. K is 16, the Hamming read's kept count and one of the Flock selector's top-k values.
  - **The weights:** rank `i` has weight `W_i = RAW_RANK_WEIGHTS_Q16[i]` (`1/(i+1)` in Q16, `crates/uor-r4-integer/src/stack/flock.rs:382`). Their sum is `H = Σ_{i<16} W_i`.
  - **The draw:**
    - one xorshift64* step, seeded as `epoch_order` seeds (`state = seed·0x9E3779B97F4A7C15 + index·0xD1B54A32D192ED03`, then `| 1`, so it is never zero), with `index` the sample's index times 64 plus its step;
    - `r = output mod H`;
    - the rank is the first `i` with `W_0 + … + W_i > r`.

    There is no exponential and no normalized exponential, the weight is the harmonic table (the assessment §2.3), and the run is deterministic for a seed.
  - **Where a reply stops:** at its first `<|eos|>` or BOS, or at 64 tokens.

## The scores

- **Teacher-forced:**
  - nearest top-1 per stratum, for every trained and reported arm;
  - the writer's pick beside it, because the served session reads it (claude-205: it is right on 77–1,280 rows where nearest is right on 10,079–11,513);
  - Fresh V2, descriptive only.
- **Free generation,** for every trained arm at each seed: 256 DEV prompts and 256 P-dev prompts (`sample_windows` over each panel), 64 tokens each, under G and under F. For each set:
  - the complete share;
  - the verbatim-TRAIN share of complete replies;
  - new n-grams for n = 4–8, read beside the repeat rate (claude-205: loops inflate novelty);
  - distinct-1 and distinct-2;
  - the repeated-4-gram rate;
  - the table hit rate: under generation, the share of steps whose reply key the table names, against teacher forcing on the same replies (I-F).

  The references are scored the same way.
- **On the integer artifact,** for the shipped recipe only:
  - kept-set agreement and first greedy divergence against its float forward;
  - decode tok/s against context on this laptop (`fbs-toks`).

  These run once the serving units have merged: `integer-hamming-read` step 5, `turn-memory-integer` units 2–3, and the nearest readout. Until then, "end to end" is not claimed.
- **Lead with tokens per second** in every REPORT that has one (done-when 4).

## The instruments the run needs

Each exists, with its check passing, before the run's pre-run review.

| Id | Instrument | Check |
|---|---|---|
| I-A | `--score-rows --panel NAME=DIR` on an unpinned panel, with its sha recorded, and `sample_windows` over that panel | P-dev scores 16,417 rows, its census count. Its 256 sample prompts each open a reply in P-dev |
| I-B | Memory dropout, `--turn-memory-dropout P`: a dropped training window names no memory row | At P = 0, training is byte-identical to (r): the same `progress.jsonl` at step 256. At P = 1, the reply weight stays 0.0 |
| I-C | The transport arm as a flag, `--fbs-arm one-atom\|identity\|frozen`, for training and scoring | `one-atom` is byte-identical to today's default at step 256. `frozen` leaves every routing query equal to its seed after 256 steps. `identity`'s forward equals the tree selection, `y_o = x_σ(o)` (the arm kernels' CUDA tests) |
| I-D | The controls (T) and (S), per stratum, on the laptop | (T) names exactly 1,194 openings and 9,655 later rows on DEV, and gets the census's 8,822 of those later rows (claude-202 M1). (S) is reported beside the T2 bigram's 6,300 on DEV (`PLAN.md`, M1 bar); a value below it is explained before (S) is used |
| I-E | The Flock rank decode | At K = 1 it equals greedy exactly. Over 10^6 seeded draws, each rank's frequency is within 0.002 (absolute) of `W_i / H`; rank 15's standard error is about 0.00013 |
| I-F | The table hit rate | Under teacher forcing on DEV it counts exactly the 10,849 named rows |
| I-G | The grader, extended to the strata (N-u and N-k by U's rule, counted and recorded), the comparisons and the five decisions | A unit test per rule, each with a negative gate, as claude-205's pairing test |
| I-H | Reproduction | (r) and (r0) at seeds 1 and 2, trained by the run's binary, reproduce claude-194's sealed readings at step 16,384 exactly (11,513 / 11,361 and 10,079 / 11,171). A miss is an instrument finding: training is not reproducible across builds (the assessment §2.1). It does not void this run's comparisons, which all use one binary, but it is reported first, and fixed before any claim across runs |

## Data and pins

- **Pinned by the code:** TRAIN `mw-balp/train`, DEV `ft/dev` and the tokenizer.
- **P-dev:** pinned by its sha above, checked against its `SHA256SUMS` in the store.
- **Never read:** Fresh V2's rows enter no decision, and the K16 fresh slice is never opened.

## Cost and stop rule

- **Training.** There are 7 trained arms × 3 seeds = 21 runs. claude-194 trained three arms in about 1.7 h on one RTX 5090 (about 34 min each, at $1.11/h), so 21 runs take about 12 GPU-hours, about $8–14 at $0.6–1.1/h. The Flock arm may run slower; the throughput check prices it.
- **The pods.** Up to 3 at once, if `uor-pod status` shows room inside the caps (4 pods, $8/h across labs) at lease time.
- **Scoring.** About 2 minutes per arm per seed for the rows (claude-205: 72–77 s), plus free generation under two rules.
- **Stops:**
  - an arm whose first 256 steps project past 1 hour stops alone; the others go on, and the REPORT names it;
  - the whole run stops at $20, or if any instrument check fails on a pod;
  - a small miss on a bar is "keep and iterate", not a stop.
- **Before any pod:** the run needs its own pre-run review (D67), with each instrument's check result in hand.

## What each outcome unlocks

- **The shipped recipe and decode rule** set the model that `turn-memory-integer`, `integer-hamming-read` and the nearest readout export and serve.
- **The add-on and transport readings** set the next recipe run: candidates combined, on a new panel.
- **The verdict says what the end-to-end score means:**
  - **"generalizes":** the model reads the prompt at this scale;
  - **"memorized":** the next mechanism work is reading values from the prompt (the copied stratum C) before more scale;
  - **"mixed":** the REPORT names which condition failed, and the next unit targets it.

## What changed in round 2

The round-1 FIT's blockers:
1. **The memory-off arm.** The reported (r-off) and (r-drop-off) are added.
2. **N's keyed rows.** Decisions use N-u only, and N-k is reported.
3. **The shipped recipe.** It is always a trained arm. Add-ons are candidates for a new run on a new panel, and the type spin's serving gap is named.
4. **"Inside".** It is underpowered, never a negative. Transport and add-on negatives need a control or arm to be ahead or behind at all three seeds.
5. **The verdict.** It is on the shipped recipe and decode rule, per seed, against named controls, with "memorized" as the spec's AND, and with a complete-count floor and a tolerance that widens with fewer complete replies.
6. **The decode rule's tolerance.** It is a count against a count, judged on the shipped recipe at every seed.

**Its notes:**
- I-E's bound is absolute, and the draw is specified (seeding, never zero, mapping to a rank).
- I-C checks `identity`, and I-H checks reproduction.
- The table's line is 382.
- The cost uses 34 minutes and $1.11/h, an arm's stop is per arm, and the room for three pods is checked at lease time.
