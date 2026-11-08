# Prep sheet: recall where the query is not the key (one-model-end-to-end, claude-327, 8 Nov 2026)

TEST FITNESS: pending round 2. Round 1 (a fresh non-author FIT) read NOT FIT, with 2 blockers, 4 majors and 4 minors; every finding is applied below. A D67 pre-run review comes before any pod.

## The question

[claude-326's assessment](../../loop/tasks/one-model-end-to-end/claude-326/ASSESSMENT.md) (Q4, Part B) asks whether the trained Hamming read can **choose a value by a query that is not the key**, the way `m2-fresh256`'s "What is my job?" chooses the value after "I am a … by trade".
- **What the 27 Oct probe showed:** it recalled 0.984–0.994 ([sheet](in-space-trunk-to-text-hamming-recall-2026-10-27.md), notebook row 109).
- **Why it does not answer this:** its query writes the key again (`in_space_long.rs:11803`), so the copy head's exact match on the current token does all the work.
- **Why the panel cannot answer it:** claude-327's Part A shows that on claude-322's and claude-323's served artifacts the reply's first token is one of 2–3 frame words across all 256 rows, whichever relation is asked. So the panel's first-target rank reads format, not choice.

**Two routes to the value exist in this read, and the sheet separates them:**
- **(i) Table alignment.** The input table is trained (`fix_input_table: false`), so a copy head can pull `Q(K)`'s sign code onto `K`'s and match `K` exactly. This works only when the query token is the current token.
- **(ii) A learned query.** The type heads' query is built from the residual, so it can carry the query forward and pick `K`'s fact from any position.

In the panel, the current token at the answer position is the header's last token (`:`), the same for every question. So **only route (ii) answers the panel**. B2's header recreates that.

## Steps: the arms

**Common to every arm**, as row 109's sheet:
- the FBS sheet trunk, `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`;
- context 192;
- seeds 1 and 2, with seed 3 if the two seeds split, as row 109;
- held-out queries decoded as the whole table's nearest token.

| Arm | Task | Read |
|---|---|---|
| **B0-h** | `--recall 4`, unchanged: the query writes the key again | `--hamming-read 4` |
| **B0-n** | the same | none |
| **B1-h** | two-class, query at the scored position | `--hamming-read 4` |
| **B1-n** | the same | none |
| **B2-h** | two-class with a 4-token header | `--hamming-read 4` |
| **B2-n** | the same | none |

**The two-class task** (new, `--recall-query 1`):
- **Episodes:** each 192-token window holds **4 episodes**, each over 2 relation tokens. That is 8 distinct relations per window, drawn from a 16-token relation range.
- **Facts:** each relation is **written once**, as `K v`, with a fresh value.
- **The query:** each episode asks one of its two relations, `j`, drawn uniformly and **independently of which fact is nearer the query**. The query writes `Q(K_j)`, where `Q` is a fixed bijection from the 16 relation tokens to a disjoint 16-token query range, the same in every window and seed. Each `Q` token occurs at most once in a window.
- **Targets:** the scored target is `K_j`'s value. These 4 are the window's only scored targets.
- **Signal:** 4 targets × 16 windows × 4,096 steps is 262,144 scored targets, row 109's count at its saturation step, 1,024 × 16 × 16.
- **Distances:** the distance from the asked fact to its query falls in the far buckets, 33–96 and 97–180, two episodes per bucket per window. The other fact's distance is drawn in the same buckets independently.
- **Ranges:** values, relations, queries, header and filler are disjoint.
- **Recording:** each held-out query records whether nearest names the asked value (`R_ask`), the paired relation's value (`R_oth`), another of the window's values, or none.

**The header** (`--recall-header 4`): four fixed tokens `H1 … H4`, from their own range, follow each `Q(K_j)`, and the target is scored after `H4`.
- The current token at every scored position is then `H4`, whichever relation is asked, so route (i) cannot tell the relations apart.
- The header occurs 4 times a window, so a copy head that matches `H4` finds earlier headers. Their next tokens are the other episodes' targets, which are scored "another value", not `R_ask`.

**Held-out:** 256 windows, 1,024 queries, so 512 per far bucket per arm. Each query's bucket is its asked fact's distance.

**Eval-time readings at every checkpoint, for B1-h and B2-h:**
- **The trace:** for each copy and type head at the query's scored position, whether the asked fact's value position, the paired one, or neither is among its 16 kept positions, and the head's weight.
- **The copy ablation:** `R_ask` recomputed with the copy heads' combine weight set to 0. This changes the readings only, not training.

**The code this needs** (the next unit's build, in `crates/uor-r4-float-train`, added to the item's scope in its ORDER):
- `recall_window` gains the two flags, default off;
- a held-out window count setting, since `RECALL_EVAL_WINDOWS` is 128;
- the outcome counts;
- the trace;
- the eval-only copy ablation.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

`R_ask`, `R_oth`, `R_any` (another value) and `R_ask^{−c}` (with the copy heads ablated) are counted over 512 held-out queries per bucket.

| Arm | Passes when |
|---|---|
| **B0 (instrument)** | `R(B0-h) ≥ 0.95` and `R(B0-n) ≤ 0.02`. Otherwise the run is an instrument fault and nothing else is read |
| **B1 (choice)** | `R_ask(B1-h) ≥ 0.50`, `R_ask(B1-h) ≥ R_oth(B1-h) + 0.25`, and `R_ask(B1-h) ≥ R_ask(B1-n) + 0.10` |
| **B2 (carry)** | the B1 rule for B2-h against B2-n |

**What each rule rejects:**
- A copier that picks one of the episode's two relations at random scores about 0.5 on `R_ask` and on `R_oth`, so the bar asks for selectivity.
- A copier over the window's 8 values scores about 1/8.

## Outcome map (read in order)

0. **B0 fails:** an instrument fault. Repair it and rerun; nothing else is read.
1. **Undertrained.** If `R_ask` of an arm still rises by more than 0.05 between its last two checkpoints and misses its bar, the arm is undertrained: extend it to 16,384 steps under this sheet. That is not a mechanism result.
2. **B2 passes:** a learned query carries the query four tokens forward and makes the choice. Check the trace and the copy ablation:
   - **If `R_ask^{−c}` still passes B2's rule,** it is the type heads (route ii). Next: claude-325's bare-answer recipe, larger or with a curriculum from B2's form to the panel's.
   - **If `R_ask^{−c}` fails,** the copy heads carry it by an unforeseen match, which the trace names. It is read as an instrument finding, and the next sheet removes that match.
3. **B2 fails, B1 passes:**
   - **The copy ablation and the trace show B1 carried by the copy heads** (route i, table alignment). The type heads never learned a query; read with the trace, row 4.
   - **Otherwise,** the type heads choose at the query but do not carry it four tokens. Next: the carry, a read at the query token whose result is written forward. Tested on B2 first.
4. **B2 fails, and B1 fails or is carried only by the copy heads,** split by the trace at B2-h's scored position:
   - **The asked value's position is not kept by any type head:** a query failure. This read's configuration (type-head rotation query, read 4, width 256) does not learn the choice on this task; that is mechanism-side for the configuration (§4.1, §4.2). Next: lever (b) of the assessment, a question-keyed read, under its own sheet. It must pass B2 here before any language run.
   - **Kept, but `R_ask` fails:** a payload or combine failure. The type heads' payload, the residual `v_s` added on the readout, is the form that failed the 27 Oct probe (module doc, steps 5–6). Next: a type-head payload that brings back the kept position's table point, as the copy heads' payload does. Tested on B2 first.

**Seeds:** two seeds that split run a third; at least 2 of 3 must agree. Every cell and margin is reported either way.

## Instrument checks

- **I-1:** with neither new flag, `recall_window` writes windows whose SHA-256 equals a fixture hashed from `main`'s output at the pin. The fixture covers seeds 1 and 2, training indices 0–127, and the held-out seeds `seed ^ RECALL_EVAL_SEED`. The test fails if the default path moves.
- **I-2 (two-class windows):**
  - the query token is never a relation token;
  - `Q` is the same in every window, and each `Q` token occurs at most once a window;
  - each relation is written once;
  - the target is the asked relation's value;
  - exactly 4 targets are scored;
  - the ranges are disjoint;
  - over 10,000 windows, `j` is balanced against which fact is nearer, within 2%.

  Each is a unit test with a mutation that makes it fail.
- **I-3 (header):** each target sits 4 tokens after its `Q(K_j)`, and the header tokens are fixed. Tested the same way.
- **I-4:** every read arm's step-0 held-out reading equals its no-read twin's, since the read's weights start at 0. The Hamming combine weights move off 0 by step 512. The read's existing negative gates fail as they did on the 27 Oct sheet.
- **I-5:** the trace and the copy ablation match a hand-built two-episode window in a unit test. Ablating a head that found the value drops `R_ask` on it.
- **I-6 (finite CUDA):** a 32-step smoke of every arm on the pod, at finite losses, before the timed run.

## Probe ladder

B0 first on the pod. If it misses its bar, stop and repair; B1 and B2 do not run on a failed instrument.

## Data split (generated, none from the store)

Every window is generated from `(seed, index)`. Held-out windows use `seed ^ RECALL_EVAL_SEED`, as the 27 Oct sheet, so they never equal a training window at these run lengths.

## Base artifact

None. Every arm trains from step 0 at the pin the next unit names.

## Minimum test power

512 held-out queries per bucket per arm.
- **Near 0.5:** one arm's standard deviation is about 0.022, and a difference of two arms about 0.031. The 0.25 margin is about 8 standard deviations, and the 0.10 margin about 3.2.
- **Near 0.985:** B0's standard deviation is about 0.0054, so its 0.95 bar sits about 6.5 standard deviations under row 109's seed-1 reading.

## Cost

Row 109 ran at about 12,187 context tokens a second with the read, so one arm of 12.6M positions takes about 17 minutes with the read and about 12 without. Six arms at two seeds is about 3 hours of one RTX 5090, about $1.5–2.5, inside the caps. The undertrained extension (outcome 1) would add about 4× for the arms it covers.

## Results and stop rule

- **Results:** written to `results/claude/one-model-end-to-end-query-recall-2026-11-08/` in the HF store with a `SHA256SUMS`, and read back before any pod is brought down.
- **Stop** at B0's failure, or at any non-finite loss.
- **A pod that does not answer SSH** is retried once on another offer.
- **A watcher is armed on every pod** (L-004).
