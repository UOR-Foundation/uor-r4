# Prep sheet: step off a reply's own cycle (one-model-end-to-end, claude-217, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-217](../../loop/tasks/one-model-end-to-end/claude-217/ORDER.md). It follows claude-216 ([result](../../labs/one-model-end-to-end/cut-train-2026-10-30/README.md)), by Casey's direction of 30 Oct (verbatim on #3407 6523338624): "Yes: after claude-216's grade, register a decode unit that steps off a reply's own cycle (the nearest token outside a repeated 4-gram), on a fresh panel, against greedy."

TEST FITNESS: PENDING, round 2 (a fresh non-author FIT is run on this sheet before any pod). Round 1 was NOT FIT on one blocking point, repaired here: the matched content could not see what the cycle step wrote where it stepped, so salad there could read "improves" and ship. The cycle step's content is now also read on the prompts it changed ("Changed content", in the Bar).

## The question

Does stepping off a reply's own cycle, with no draw and no retraining, remove greedy's loops without the content loss every earlier loop fix paid? And, as a second registered decision, does claude-216's decode-only cut keep its reported "improves" on prompts no decision has seen?

**Why this, now:**
- **claude-213:** 96–99.6% of loop steps admit a single candidate. A rule that spreads choice by margin or rank collapses to greedy exactly where greedy loops.
- **claude-215 and claude-216:** the cut reduced loops, but cost content when trained in. Decoded alone, on matched prompts, it read "improves" on one panel.
- **The cycle step acts only where a loop would close.** Everywhere else it is greedy.

## Base artifact: the model and the arms

- **The model:** the sealed (r-drop) at step 16,384, seeds 1–3, at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/`, each checked against its seed's `SHA256SUMS`. It is the recipe claude-216 kept. There is no training.
- **The cycle step, `DecodeRule::CycleStep { k: 16 }`** (`in_space_long.rs`, `cycle_step_pick`):
  - **At each step,** of the 16 nearest table tokens in rank order (ties to the smaller id, as the Flock rules rank them), take the first whose append would not close a 4-gram the reply has already produced.
  - **If every one would,** take the nearest. The step is marked exhausted.
  - **It looks at the reply's own tokens only,** never the prompt's.
  - **It stops after the end of turn,** as the Flock rules do. Greedy's loop count is already cut there.
  - **It is exact and deterministic:** no draw, no seed, no squashing function. Every step records a mark: nearest, stepped off, or exhausted.
- **The decode arms,** with the memory on, 256 replies per split and seed, every arm on the same prompts:

  | Arm | Rule | Role |
  |---|---|---|
  | **base** | greedy (`greedy`) | the base |
  | **cycle step** | `CycleStep` (`cycle-step`) | decision 1 |
  | **decode-only cut** | greedy with claude-216's mask cut (`greedy-read-all`) | decision 2 |
  | both | `CycleStep` with the cut (`cycle-step-read-all`) | reported, and decides what ships if both pass |

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code.
- **p-s decides.** It is a new held-out-phrasing panel, drawn on 30 Oct by claude-204's `heldout-panel` at seed 2091, a seed no unit had used:

  `heldout-panel out=p-s cell=dev,dev conversations=400 seed=2091 protocol=2`

  - **The panel:** 49,332 tokens and 15,967 reply targets.
    - `tokens.u16` sha256 `62099a4800f6ae34a467762a42fcc79535836f222f64a1613d57aea9176610f1`;
    - `response_mask.u8` sha256 `9053e117eab299b48623f00f293aa7dd5275cae3804a6ce55f01b1303f05f11a`.
    - A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-s-2026-10-30/`, with a `SHA256SUMS` and the census, read back OK.
  - **The census** ([census-p-s-2026-10-30.json](../../labs/one-model-end-to-end/census-p-s-2026-10-30.json)):
    - 0.000 of its 1,400 prompts are in TRAIN exactly;
    - 0.000 are in TRAIN as value-masked templates at K = 1,024 (0.049 at K = 256);
    - 0.624 of its replies are verbatim TRAIN replies.
- **DEV is reported.**
- **p-t, p-r, p-c, P-dev and p-f are not used;** each has been opened by an earlier decision.

## Steps

One pod with one RTX 5090 runs `local/claude/claude-217/job.sh <head>` in this unit's worktree, with the three seeds side by side. Its sha256 is in the pre-run review.

1. **The source,** pinned to this unit's pushed head, and clean.
2. **The CUDA tests** (`--release --locked`). The job stops unless each of these passes:
   - `the_cycle_step_follows_greedy_until_it_steps_off_a_cycle_on_a_model`: at K = 1 the rule is greedy; at K = 16 it follows greedy until its first step off, where greedy's token would close a repeated 4-gram; and its reply repeats no 4-gram unless a step was exhausted;
   - claude-215's and claude-216's cut tests, and claude-211's and claude-213's decode tests.
3. **The build,** `train-in-space`, kept with its sha256.
4. **The data,** every file checked:
   - the panel, against its `SHA256SUMS` and its pinned `tokens.u16` sha;
   - each seed's 11 sealed files: the run config, 5 checkpoint files and 5 registered files.
5. **A throughput probe:** seed 1 at 8 samples, projected to 256. Past 2 hours per seed, the job stops.
6. **The scorings,** the three seeds side by side, each:

   `--score-rows CKPT --out DIR --panel p-s=PS --samples 256 --reply-tokens 64 --retain-rates 1 --flock-end 1 --reply-read all --cycle-step 1 --seed S`

   This writes, on DEV and p-s, greedy (with the memory on and off), `FLOCK_END`, greedy with the cut, the cycle step, and the cycle step with the cut.
7. **Reproduction:** claude-211's 15 registered files must reproduce byte for byte (EXIT 6 otherwise).
8. **Expected files:** every one is written, and every cycle-step reply carries its marks, 256 of 256 per file (EXIT 8 otherwise).
9. **The seal:** the pod's `/workspace/cycle-step` (without its inputs) goes into the store at `results/claude/one-model-end-to-end-cycle-step-2026-10-30/run-<sha8>/`, with a `SHA256SUMS`. It is read back before the pod is released and taken down.
10. **The grade, on the laptop:**

    `train-in-space --data-root DATA --cycle-grade 1 --runs RUNS --panel p-s=PS`

    That is `cycle_grade`, unit-tested. `RUNS` holds `r-drop-s{1,2,3}`.

## Bar

Each reading is one arm's 256 replies on one split at one seed, against that split's own references after the same prompts. The bars are claude-216's, unchanged:

- **Loops:** replies that repeat a 4-gram of their own before their end of turn.
- **Fewer loops:** the arm's loops, plus decision 4's band at greedy's count, are fewer than greedy's.
- **Holds:** decision 4 against greedy. The arm's repeated-4-gram rate is below greedy's, and it completes at least as many replies as greedy, less the band.
- **Close:** "generalizes"' condition: at least 64 complete, `v` within `tol` of the references' `v_ref`, and a repeat rate within 0.10 of the references'.
- **Matched content:** the verbatim TRAIN share on the prompts the base and the decision's arm both complete (`matched_counts`, `MATCHED_BY`).
  - **Kept:** the arm's share is below the base's by at most `max(0.10, 3·√(p(1−p)/m))`.
  - **The content floor:** at least 64 matched prompts at every seed.
- **Changed content, for a cycle-step arm only** (`ChangedContent`, `cycle_outcome`):
  - **The changed prompts:** the prompts where the rule stepped off at least once.
  - **The reading:** the arm's replies there that are complete and verbatim in TRAIN, against the references on the same prompts.
  - **Kept:** the arm's share is below the references' by at most `max(0.10, 3·√(p(1−p)/m))`, with `p` the references' share and `m` the changed prompts.
  - **The floor:** at least 40 changed prompts at every seed (the loop floor's count).
- **The loop floor:** greedy loops in at least 40 of 256 p-s replies at every seed.

**What the cycle step meets by construction, stated in advance.**
- **Loops and repeats.** Its reply can hold a repeated 4-gram only after an exhausted step. So its loops, its repeat rate, and the repeat parts of "holds" and "close" are met almost by construction.
- **The matched content, too.** Away from a cycle the rule writes greedy's reply token for token. Greedy rarely completes where it loops, so the prompts where the rule steps off mostly fall outside the matched set. The matched content therefore cannot see what the rule wrote where it stepped (FIT round 1).
- **So for the cycle step, the reading that decides rests on:**
  - completion (the rest of "holds");
  - the changed content, which is where a step into salad shows;
  - `v` against `v_ref` (the rest of "close").

The step counts and the fixed reading are reported to show where it stepped and what it wrote there.

**p-s decides both decisions,** each from its arm at all three seeds, with no rows' step, since there are no teacher-forced rows here. Decision 1 uses `cycle_outcome` and decision 2 `cut_outcome`. For a cycle-step arm, "content kept" means the matched content and the changed content are both kept, and the changed floor joins the matched one. The rows are tried in this order:

| Outcome | Condition | What it unlocks |
|---|---|---|
| **underpowered** | The base loops in fewer than 40 replies at any seed. | Nothing is decided: an instrument outcome. A new panel or reply length is next. |
| **no gain** | Not fewer loops and holds, at all three seeds. For the cycle step, that means it completes fewer replies than greedy, beyond the band. | A negative for this rule, in this configuration only. |
| **underpowered (content)** | Fewer than 64 matched prompts at some seed (and, for the cycle step, fewer than 40 changed). | The loops are decided; the content is not. A longer reply or a larger sample is next. |
| **cures** | Also close, with the matched content kept, at all three seeds. | The arm becomes the one model's served decode. The next unit serves it in integers, with its tok/s. |
| **improves** | The matched content kept at all three seeds, but not close at all three. | The arm becomes the served decode, as for cures. The content gap to the references is the next work. |
| **reduces, content lost** | Otherwise. | No decode change. For the cycle step, the next candidate restricts its step to candidates inside the nearest token's cell (claude-213's margin), so a step stays near what the model meant. |

**What ships, from the two decisions together:**

| Decision 1 (cycle step) | Decision 2 (cut) | The served decode |
|---|---|---|
| cures or improves | anything else | the cycle step |
| anything else | cures or improves | the decode-only cut |
| cures or improves | cures or improves | both, if "both" reads cures or improves by `cycle_outcome` on decision 1's prompts; else the cycle step alone |
| anything else | anything else | greedy stays |

**Reported beside it, deciding nothing:**
- the "both" arm's reading;
- the DEV readings;
- each arm's decode tok/s (with three scorings sharing the GPU, a floor) and mean reply length;
- the paired loop counts against the base;
- the cycle step's step counts: steps off, exhausted steps, and replies with a step off;
- a fixed reading of the first 20 p-s replies of seed 1 under base, cycle step and decode-only cut.

## Minimum test power

- **The sample:** 256 replies per arm, split and seed; the same prompts under every arm.
- **The loop band:** at greedy's 60–76 looping replies (claude-216's p-t), the band is 14–15, and the floor of 40 gives a band of 12.
- **The changed prompts:** a looping greedy reply makes the rule step off, unless every candidate is refused at that step. So at about 60–76 loops the cycle step changes about that many prompts. At 70 changed prompts and references' `p` ≈ 0.6, the changed tolerance is about 0.18; salad, near 0.0–0.1, is far outside it. A panel drawn the same way should loop about as often; the floor decides if not.
- **The matched prompts:** a looping greedy reply does not complete, so at p-t's loop counts the base completes about 200–212 of 256. The floor of 64 is registered for that reason. The tolerance is 0.10 down to about 225 matched prompts at `p` ≈ 0.5, and widens below that (0.19 at 64).
- **The seeds:** every condition needs all three.

## Stop rule

- **The job stops** on any failed CUDA test, data check, projection, reproduction (EXIT 6) or expected-file check (EXIT 8), each with its own EXIT code.
- **EXIT 6 and EXIT 8 are instrument faults,** repaired and re-run, never read as results.
- **The budget is $4.** The laptop watcher (`local/claude/claude-217/watch.sh`) prices the pod over its time up.
  - **A healthy run is not cancelled** for a budget it set itself.
  - **An unhealthy run** is sealed and taken down. Unhealthy means a process exited non-zero, or the pod's line was unchanged for 30 minutes.
- **The pod is sealed and taken down** as soon as its job has an EXIT.
- **A small miss on a bar** is "keep and iterate", not a stop.

## Cost

- **Decoding:** claude-215 decoded the three seeds side by side on one 5090, with 6 files per seed, in about 80 minutes up. This unit has about 10 decode files per seed, and claude-216's pods decoded at 70–93 tok/s per process.
- **In all:** about 1–1.5 hours on one RTX 5090 at $0.56–1.0/h, so about $1–1.5.
- **The caps:** one pod is inside them. `uor-pod status` showed 0 pods running across the labs, $0/h of $8/h, after claude-216's pods came down.
