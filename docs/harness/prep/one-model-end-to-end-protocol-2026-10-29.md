# Prep sheet: the scoring protocol for the model to ship (one-model-end-to-end, claude-206, 29 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md), done-when 3. Order: [claude-206](../../loop/tasks/one-model-end-to-end/claude-206/ORDER.md). It builds on [the assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) §5–§6 and on step 2's result ([claude-205](../../loop/tasks/one-model-end-to-end/claude-205/REPORT.md): mixed).

TEST FITNESS: (to be filled by the fresh reviewer).

This sheet fixes, before any training run, how the model the project ships is scored, and what each outcome decides. The run itself needs the instruments listed below, and its own pre-run review (D67) when they exist.

## The question

**Which recipe should the project ship, and does it read the prompt, or only recall TRAIN's templates?**
- **What step 1 found.** Three quarters of DEV's and Fresh V2's replies are verbatim TRAIN replies (claude-202). P-dev's prompt templates are absent from TRAIN, while its replies are not (claude-204).
- **What step 2 found** (claude-205):
  - the memory recipe and a recipe that trains without it disagree in sign across seeds, on the rows the memory cannot name;
  - (r) loops under greedy decoding;
  - neither trunk puts a rare value from the prompt into its reply.
- **What this sheet is for.** A score that rewards recall alone would pick the wrong mechanism. So every decision below is made on rows the memory cannot name, and on a panel no decision has touched.

## The arms (trained)

**The base recipe is claude-194's (r):** `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --seed S`, at width 256 on the pinned TRAIN, with the fixed-order backward (claude-179). Each other arm changes one thing:

| Arm | Change | Role |
|---|---|---|
| **(r)** | none | the recipe in hand |
| **(r-drop)** | memory dropout, p = 0.5 per training window (I-B) | step 2's equal candidate |
| **(r0)** | `--turn-memory-empty 1` (both tables empty) | the no-memory control (L-010) |
| **(r-flock)** | `--flock-read 7` (claude-144's K) | reading by meaning |
| **(r-spin)** | `--type-spin 1` | the type heads' spin |
| **(r-ident)** | transport fixed to the identity (`FbsArm::IdentityTransport`, I-C) | transport control |
| **(r-frozen)** | routing queries frozen at their seed (`FbsArm::FrozenQueries`, I-C) | transport control |

- **Seeds.** Every arm runs at seeds 1, 2 and 3. Step 2 showed that two seeds can disagree in sign beyond their own noise, so the band needs a spread over more than two.
- **The memory at serving time.** Memory dropout acts in training only. Every arm is scored with its own tables, as it would be served.

## The controls (counted on the laptop, no model)

- **(T) tables only.** The memory's own tables:
  - at a row the opening table or the reply table names, the stored token;
  - at any other row, no prediction (a miss).

  It is the bar the keyed stratum must clear: a model that does no better than its tables on keyed rows adds nothing there.
- **(S) longest suffix (an ∞-gram over TRAIN; Liu et al. 2024, arXiv:2401.17377).** At each row:
  - take the longest suffix of the window's tokens, up to the row, that occurs in TRAIN;
  - predict TRAIN's commonest next token after it, ties to the smaller id.

  It is the strongest lawful recall control (CHARTER §3.10, §4.4). A model that does not beat it on unkeyed and new-template rows is recalling, not reading.

## The strata (teacher-forced, nearest top-1)

| Stratum | Rows | Count | Judged |
|---|---|---|---|
| **K: keyed** | DEV's openings the first-8 table names, and its later rows the reply table names at share 0.5 | 1,194 + 9,655 | against (T) |
| **U: unkeyed** | DEV's 3,406 unnamed later rows and its 32 unkeyed openings (claude-202's decision rows) | 3,438 | against (r0) and (S) |
| **N: new template** | Every scored row of P-dev (`dev,dev`; `tokens.u16` sha256 `790b7e12934a241f07ec5c04fc6e929c86e970583ba5dee9a8646f7d235266cd`; HF `inputs/claude/heldout-panels-2026-10-29/p-dev`) | 16,417 | against (r0) and (S) |
| **C: copied** | The rows of each stratum that copy the prompt (`copies_prompt`) | per stratum | reported |

- **P-dev has touched no decision.** No unit has scored a model on it (claude-204 built it; claude-205 left it alone). This protocol opens it once, for the run it registers.
- **Fresh V2 is never a stratum here.** It is reported per arm, descriptive only.
- **N's keyed and unkeyed parts are reported apart.** That split uses TRAIN's tables, the same rule as K and U (claude-204: 245 of P-dev's 1,443 openings are keyed).

## The band and the comparisons

For arm X against arm or control Y, at seed `s`, on one stratum:
- **The counts.** `b(s)` is the rows only X gets, `c(s)` the rows only Y gets, and `d(s) = b(s) − c(s)`.
- **The band.** `band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d − min_s d)`, the paired floor or the spread over the three seeds.
- **The cell.** X is **ahead** if `d(s) > band(s)` at every seed, **behind** if `d(s) < −band(s)` at every seed, and **inside** otherwise.

This is claude-205's rule (`seed_cell`, `overturn_verdict`), with the spread taken over three seeds. A control has no seed, so for X against (T) or (S), `d` varies only with X's seed.

## The decisions, fixed in advance

1. **Which recipe ships: (r) or (r-drop).**
   - If one is ahead of the other on N, it ships.
   - Else, if one is ahead on U, it ships.
   - Else (r) ships, the recipe already served in part.
2. **Which add-ons join it.** (r-flock), (r-spin), (r-ident) and (r-frozen) are each compared with (r) on U and on N:
   - an add-on **joins** if it is ahead on N, and not behind on U;
   - a transport control that is **not behind** (r) on both U and N means the learned transport earns nothing at this scale. That is reported as a negative for this configuration (D22), and the map's transport stage is revisited. It is not a cut.
3. **The verdict on the shipped recipe**, against (r0) and (S):
   - **"generalizes":** it is ahead of both (r0) and (S) on U and on N; and on P-dev's free generation (below), its share of complete replies that are verbatim TRAIN replies is within 0.10 of P-dev's own references, and its repeated-4-gram rate is at most the references' + 0.10;
   - **"memorized":** it is ahead of (T), (r0) or (S) on K but on neither U nor N; or its verbatim share exceeds the references' by more than 0.10;
   - **"mixed":** otherwise.

   The 0.10 tolerance is about 3 binomial standard errors at 256 replies (at p = 0.64, √(p(1−p)/256) ≈ 0.03).
4. **Which decode rule ships: greedy (G) or Flock rank (F).** On each arm's free generation (DEV and P-dev pooled, 512 replies per seed):
   - F ships if, at every seed, its repeated-4-gram rate is lower than G's;
   - and its complete share is not lower than G's by more than `2·√(n·p(1−p))` (n = 512, p = G's complete share).
   - Else G stays.

   Teacher-forced top-1 does not depend on the decode rule.

## The decode rules

- **(G) greedy:** the whole table's nearest token, as `greedy_reply` decodes today.
- **(F) Flock rank:**
  - **The candidates:** the K = 16 table tokens nearest the readout point, by the nearest objective's score, ranked 0 to 15. K is 16, the Hamming read's kept count and one of the Flock selector's top-k values.
  - **The draw:** rank `i` is drawn with weight `RAW_RANK_WEIGHTS_Q16[i]` (`1/(i+1)` in Q16, `stack/flock.rs:379`), by a seeded integer draw. It uses the same xorshift as `epoch_order`, seeded by the run seed and the sample index.
  - **The rule's guarantees:** there is no exponential and no normalized exponential, the weight is the harmonic table (the assessment §2.3), and the run is deterministic for a seed.
  - **Where a reply stops:** a reply ends at its first `<|eos|>` or BOS.

## The scores

- **Teacher-forced:** nearest top-1 per stratum, and the writer's pick beside it. The writer is reported because the served session reads it (claude-205: it is right on 77–1,280 rows where nearest is right on 10,079–11,513).
- **Free generation:** 256 prompts from DEV and 256 from P-dev (`sample_windows`), 64 tokens each, under G and under F. For each set:
  - the complete share;
  - the verbatim-TRAIN share of complete replies;
  - new n-grams for n = 4–8, read beside the repeat rate (claude-205: loops inflate novelty);
  - distinct-1 and distinct-2;
  - the repeated-4-gram rate;
  - the table hit rate under generation: the share of steps whose reply key the table names, against teacher forcing on the same replies (I-F).

  The references (each prompt's true reply) are scored the same way.
- **On the integer artifact,** for the shipped recipe only:
  - kept-set agreement and first greedy divergence against its float forward;
  - decode tok/s against context, on this laptop (`fbs-toks`).

  These run once the serving units have merged: `integer-hamming-read` step 5, `turn-memory-integer` units 2–3, and the nearest readout. Until then, "end to end" is not claimed.
- **Lead with tokens per second** in every REPORT that has one (done-when 4).

## The instruments the run needs

Each exists, with its check passing, before the run's pre-run review.

| Id | Instrument | Check |
|---|---|---|
| I-A | `--score-rows` on an unpinned panel (`--panel NAME=DIR`, its sha recorded) | P-dev scores 16,417 rows, its census count |
| I-B | Memory dropout, `--turn-memory-dropout P`: in a dropped training window the memory names no row | At P = 0, training is byte-identical to (r) (the same `progress.jsonl` at step 256). At P = 1, the reply weight stays 0.0 |
| I-C | The transport arm as a flag (`--fbs-arm one-atom\|identity\|frozen`), for training and scoring | `frozen` leaves every routing query equal to its seed after 256 steps; `one-atom` is byte-identical to today's default |
| I-D | The controls (T) and (S) on the laptop, per stratum | (T) names exactly 1,194 openings and 9,655 later rows on DEV, and gets the census's 8,822 of those later rows (claude-202 M1). (S) is reported beside the T2 bigram's 6,300 on DEV (`PLAN.md`, M1 bar); a value below it is explained before (S) is used |
| I-E | The Flock rank decode | At K = 1 it equals greedy exactly. Over 10^5 seeded draws, each rank's frequency is within 1% of `(1/(i+1)) / H_16` |
| I-F | The table hit rate under generation | Under teacher forcing on DEV it counts exactly the 10,849 named rows |
| I-G | The grader extended to the strata, the comparisons and the decisions above | Unit tests for each rule, with a negative gate for each, as claude-205's pairing test |

The integer measures need the serving units named above. They are not instruments of the training run.

## Data and pins

- **Pinned by the code:** TRAIN `mw-balp/train`, DEV `ft/dev` and the tokenizer.
- **P-dev:** pinned by its sha above, checked against its `SHA256SUMS` in the store.
- **Never read:** Fresh V2's rows enter no decision, and the K16 fresh slice is never opened.

## Cost and stop rule

- **Training.** There are 7 arms × 3 seeds = 21 runs. claude-194 trained three arms with probes and readings in about 1.7 h on one RTX 5090, so a run takes about 30 minutes, and 21 runs about 10.5 GPU-hours.
- **The pods.** Up to 3 pods at once, at about $0.6–1.0/h, cost about $7–11 in all, over about 3.5 hours. That is inside the caps (4 pods, $8/h across labs).
- **Scoring.** About 2 minutes per arm (claude-205: 72–77 s per arm, before F's samples).
- **Stop** if:
  - any arm projects past 1 hour (the throughput check on its first 256 steps);
  - the run passes $15;
  - any instrument check fails on the pod.

  A small miss on a bar is "keep and iterate", not a stop.
- **Before any pod:** the run needs its own pre-run review (D67), with each instrument's check result in hand.

## What each outcome unlocks

- **The ship decisions** (recipe, add-ons, decode rule) set the model that `turn-memory-integer`, `integer-hamming-read` and the nearest readout export and serve.
- **The verdict says what the end-to-end score means:**
  - **"generalizes":** the model reads the prompt at this scale;
  - **"memorized":** the next mechanism work is reading values from the prompt (the copied stratum C) before more scale;
  - **"mixed":** the REPORT names which stratum failed, and the next unit targets it.
