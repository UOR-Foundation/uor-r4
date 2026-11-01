# Decision card: Q5's rule, the exact trace of every clear-margin disagreement (integer-nearest-readout, claude-247, 1 Nov 2026)

Lab: claude. ORDER: [claude-247](ORDER.md). This card registers the rule for Q5 of done-when 3's bar, which is a change to a bar (CHARTER §5.1, §5.2). claude-245's [card](../claude-245/DECISION.md) required Q5 and left its rule to a card of its own. claude-246's [REPORT](../claude-246/REPORT.md) read Q1 to Q3 HELD, with Q4 carried, so done-when 3 now waits on Q5 alone.

## The question

**For each position where R, the raw integer export, and F′, the trained model's float decode, pick different tokens while R's own top two are clearly apart, what must a trace show for the difference to count as accounted for? And what is a fault?**

claude-245 set four requirements for the answer:
1. the rule, and any bound it uses, is fixed before either side's values are read;
2. it traces each position's whole causal prefix, not the position alone;
3. it has its own Row 0: the float side reproduces F′'s nearest, the integer side reproduces R's pick, and a rerun matches. A position that cannot be traced is an instrument repair, not a fault;
4. it states its writer, any scope change, and its cost.

## What is measured

### The set Q5 covers

**The rule:** a position is in the set when:
- R's teacher-forced pick (claude-244's sealed forced decode, `seal/R/forced/forced-{split}-s{seed}.json`) differs from F′'s sealed nearest at the same position (claude-243's `rows-{split}-memory.json`, read at `position + i`, the join claude-244's grader uses);
- and R's own top-two gap there (`score_q12 − runner_score_q12`) is greater than its cell's G₁₀ (claude-244's sealed `readings.g10`).

**The rows:** claude-244's counted rows, those whose prompt starts at `<|bos|>`, and the cut windows, whose prompt does not. p-w's cut windows join because claude-246's I-Align held there, with 0 violations and none of 405 rows unchecked. DEV's cut windows hold no position of the set, so whether they join changes nothing; the card includes them.

**The listing** used only the sealed files above and claude-232's prompts. It reproduces claude-244's (d, d_low) exactly in all six cells:
- DEV: (80, 78), (46, 42), (69, 64);
- p-w: (409, 356), (383, 351), (454, 388).

| Cell | G₁₀ | Positions | Counted | Cut windows | Windows | Offsets | R's gaps |
|---|---|---|---|---|---|---|---|
| DEV s1 | 30,228 | 2 | 2 | 0 | 2 | ≤ 22 | 56,280 to 64,326 |
| DEV s2 | 22,526 | 4 | 4 | 0 | 3 | ≤ 31 | 33,416 to 72,272 |
| DEV s3 | 24,022 | 5 | 5 | 0 | 5 | ≤ 37 | 25,889 to 82,405 |
| p-w s1 | 10,112 | 60 | 53 | 7 | 54 | ≤ 41 | 10,408 to 109,897 |
| p-w s2 | 8,949 | 34 | 32 | 2 | 34 | ≤ 33 | 9,245 to 35,321 |
| p-w s3 | 7,760 | 79 | 66 | 13 | 71 | ≤ 42 | 7,779 to 125,797 |
| **All** | | **184** | **162** | **22** | **169** | ≤ 42 | 7,779 to 125,797 |

- A window is one row in one cell. Row ids repeat across seeds, so the 169 windows are counted per cell (cell × row).
- One position is an opening (offset 0, p-w s1). The rest are later positions of a reply.
- The counted positions are claude-244's 11 on DEV and 151 on p-w. The next unit's sheet registers this listing and checks it again (I-Set below).

**A disclosure.** While listing the set, the author saw which side names the reference token at each position: R at 49, F′ at 48 and neither at 87. This only splits a and b, which claude-246 already read, and it is no trace value. The rule below never compares a pick with the reference.

### What R's step decides, at the base

**R's path.** R's recurrence is the geometric retention reading, because F′'s command passes `--retain-rates 1` (claude-226). Its gates are the raw Hamilton gate, and nothing in it is snapped. So R's step runs these branches:
- `stack_lanes`' geometric branch (`session.rs:3768–3777`);
- the geometric output (`session.rs:3706–3724`);
- `stack_swiglu`'s Hamilton branch (`session.rs:3109–3127`);
- `stack_rotation` for the gate (`session.rs:3826–3832`).

It never runs the legacy branches: the exp chain with its keep floor, the two legacy activation tables, and the snap's selection. I-Complete (Row 0) confirms this from R's artifact.

**The decision kinds.** On that path, every discrete decision in R's step at the base (`stack_step_fbs`, `crates/uor-r4-integer/src/stack/session.rs:3232–3349`; the Hamming read, `stack/hamming.rs:556–656`) is one of five kinds.

1. **A Hamming code.** A code packs 64 bits, each the sign of one coordinate (a coordinate at or above 0 is a 1). Its source depends on the head:
   - **from token ids and parameters:** every read's copy-head query and key codes, and its type-head key codes (`record`, `hamming.rs:568–590`);
   - **from the stream at that position:** the type-head query codes (`types.queries(point)`).
2. **A kept set:** the positions a head keeps, chosen by the read's tie rule from the head's query code and the window's token ids alone (`choose(h, tokens)`, `choose(j, query, tokens)`; `hamming.rs:577–586`).
3. **A turn-memory lookup:** an exact key on token ids, and the stored entry it reads (`session.rs:3296–3318`).
4. **The nearest:** the whole table's nearest token, by −L1 at Q12, with ties going to the smaller id (`session.rs:3354–3370`).
5. **A rate bucket**, one per layer, lane and position. The lane's rate score s, clamped to [0, 1], selects entry ⌊16 s⌋ (at most 15) of a fixed 16-entry table of λ and keep. One step of the entry roughly halves or doubles the decay, so the bucket is a decision, not arithmetic.
   - On R: `s_q16 = clamp(decay + offset, 0, 2^16)` and `idx = min(s_q16 >> 12, 15)` (`session.rs:3768–3777`).
   - On the float side: `raw_score.clamp(0, 1)`, then `rate_select`, which truncates `score × 16` and caps it at 15 (`geometric_stack_native.rs:1439–1449`; `cuda_stack.cu:254–257`).
   - The float path has two other forms of the score, a fold and a reflecting floor, chosen by the run's `fold_scores` and `reflect_floor` (`geometric_stack_native.rs:39–69`). R clamps, so F's run must clamp too. I-Complete checks this from the run's configuration.
   - The boundaries, k/16 for k = 1 to 15, are the same on both sides.
   - On both sides the score is used only to choose the entry. λ and keep come from the entry, so forcing the entry forces the decision.

R has no point key: claude-244's forced mode refuses a point-key artifact. Everything else in the step is arithmetic:
- the gathers, whose codes are fixed parameters;
- the norms, the convolution and the Hamilton products;
- the rotation's normalization;
- each block's choice of exponent in `stack_quantize16`, which moves only the rounding grid;
- the clamps, the saturation and the rounding.

F′'s trunk computes the same five kinds of decision in float. claude-235's trace comparison relies on this for the reads.

### What the instruments can and cannot do, at the base

- **R:**
  - `fbs-toks --hamming-trace` writes each read's codes and kept sets as `uor-r4.hamming-trace/1`. It writes no coordinates, and it cannot force a decision.
  - `fbs-toks --forced` (claude-244) writes the picks, scores and runner-up at every offset, with no trace.
- **F′:**
  - Its rows come from `score_rows` (`crates/uor-r4-float-train/src/in_space_long.rs:5254`), claude-220's arm-G command at claude-243's arguments. They hold positions, targets, nearest and the writer's pick. They hold no scores and no trace.
  - The float read's trace (`flock_read_hamming.rs`, the `TRACE` static) records codes and kept sets, and nothing forces it.
- **fbs-snapshot:**
  - `fbs-snapshot --hamming-trace` runs on fixed sequences or a DEV sample, with no turn memory and no nearest readout.
  - Its rounding functions (`round_tables_as_exported`, `round_hamming_as_exported`) round a loaded trunk's tables and reads as exported.
- **So the trace this rule needs does not yet exist,** and it has to be built. Its scope is stated under Constraints.

### What is not measured

No trace value of any position in the set has been read: no code, coordinate, kept set or score of F′, T or R. Nor has F′'s own margin anywhere, which claude-244 could not read.

## The options

- **A. Trace R against F′ alone.** Find the first point in the prefix where they diverge.
  - The two sides differ both in parameters, through the export's rounding, and in arithmetic. So no single forced decision makes them comparable.
  - Deciding a divergence would need a bound on how far rounding can move each value through every layer. No such bound has been measured.
  - Set aside.
- **B. Attribution only, the leading proposal as claude-245 wrote it.** Read R's float twin T beside both sides:
  - T = F′ ≠ R places the cause in R's arithmetic;
  - T = R ≠ F′ places it in the exported structure.

  That says where a difference arises, not whether it is rounding. Arithmetic can be wrong, and an export can be too coarse, so attribution alone accounts for nothing. Kept as the first stage of C.
- **C. Three-way, with forced replay (recommended).** B's two steps, each accounted for by forcing its discrete decisions in the forward's own order. A flip counts as rounding only if it is a near tie, judged by the decile rule the bar already uses for the pick. Defined exactly below.
- **D. Three-way, with a rounding bound per operation,** an ε for each layer derived from the format's resolution.
  - It needs an error analysis of every operation through every layer, with no measured basis.
  - A bound tight enough to catch a fault may fail honest rounding.
  - Set aside. C needs no ε.
- **E. No rule: revise claude-245's bar instead.** The ORDER's stop rule takes this path only if no rule meets the four requirements on an instrument the item can build. C does, so it is not taken.

## Recommendation: C

**The three sides.** Each runs one cell (split × seed), under teacher forcing, on the set's rows. Each row runs from its window's start to its last position in the set.
- **F** reruns F′:
  - F′'s own scoring path, `score_rows`, with claude-243's arguments for that seed (its [sheet](../../../../harness/prep/integer-nearest-readout-tie-parity-2026-11-01.md), line 22), on the GPU;
  - with the trace on, and its query at each traced position written, so any token's score can be computed (below).
- **T** is R's float twin:
  - F's path, with F's build and arguments, on an evaluation copy of the seed's checkpoint;
  - in that copy, every parameter the artifact holds (the trunk's tables, the Hamming reads and the turn memory) is rounded as `fbs-export --raw-gates --raw-transition` rounds it. So T's parameters are R's, and only its arithmetic is float.
- **R** reruns claude-244's forced decode:
  - `fbs-toks --forced` on R's sealed artifact for that seed, on the laptop;
  - with the trace on.

**The two steps.** Row 0 (below) makes F's pick f equal F′'s nearest y, and R's pick r equal R's sealed pick x. T's pick is t.
- **The structure step (F, T)** differs only in parameters, the export's rounding. Its reference side is F.
- **The arithmetic step (T, R)** differs only in arithmetic, float against fixed point. Its reference side is T.
- Since f ≠ r, at least one step's two picks differ. A step whose two sides pick the same token at the position needs no account.

**The causal prefix of a position p.** Under teacher forcing every side reads the window's reference tokens. So the forward at p reads exactly these decisions:
- at every position q ≤ p: every stream read's codes and kept sets, every layer's rate buckets, and the turn memory's lookups;
- at p alone: the readout read's codes and kept set, and the nearest.

It does not read the readout read's choices or the pick at any q < p. Their outputs reach only q's own pick. The readout read's payload at q is written from the stream before the read's sum, so it does not depend on q's codes or kept set (`hamming.rs:556–563`, `637–656`). The trace records every decision at every position up to p. Only the decisions in p's prefix enter p's account; the rest are kept as readings.

**Near ties.** These are fixed now, in the same way the bar fixes the pick's near tie (claude-244, claude-245: a margin in its cell's lowest decile).
- **A family** is a set of values that make one kind of decision:
  - for a code, one read's codes of one head kind (copy or type) in one role (query or key), and the value is a coordinate v;
  - for a rate bucket, one layer's buckets, and the value is the score's distance to its nearest interior boundary, d = min over k = 1 to 15 of |s − k/16|.
- **G₁₀(φ, P)**, for family φ on side P, is the k-th smallest value of φ, with k = max(1, ⌊N/10⌋), where N counts every value of φ that P's trace records in the cell. Coordinates count as |v|, and zeros count.
  - It covers all the cell's traced rows, at every position the trace records.
  - It is computed from P's unforced run only, in P's own values as the decision reads them. P, a step's reference side, is always F or T.
  - "As the decision reads them" matters when the GPU build stores activations in 16 bits (`act_t`, `cuda_stack.cu:62`). There, `rate_index_of` and the code's sign read the rounded value. The trace records that value, and the sheet names the build.
- **A code difference is a near tie** when every bit that differs has |v_P| ≤ G₁₀(φ, P), where v_P is the coordinate's value on the step's reference side P.
- **A rate-bucket difference is a near tie** when the two sides' entries are adjacent and P's score lies within G₁₀(φ, P) of the boundary between them: |s_P − b| ≤ G₁₀(φ, P), where b is that boundary. Entries two or more apart are never a near tie.
- **A pick difference at p is a near tie** when the step's two sides each rank the two picks within G₁₀:
  - P picks u, Q picks w, and G₁₀ is the cell's sealed G₁₀;
  - s_P(u) − s_P(w) ≤ G₁₀, and s_Q(w) − s_Q(u) ≤ G₁₀.
- **Each score is in R's units,** the Q12 of −L1:
  - The rule needs each side's score for the other side's pick, which may lie below its own top two. So every side's trace writes its query at p, and each score is computed from that query and the side's table.
  - R's score is s_R(u) = −Σ_j |Q_j − T_{u,j}|, from R's Q12 query and table, by the nearest's own rule (kind 4). For R's pick and runner-up it equals the sealed score, which I-R checks.
  - A float side's score is s(u) = −2^12 · Σ_j |q_j − e_{u,j}| over the 256 coordinates, computed in f64 from the side's f32 query at p and its own table. That table is F's float table for F, and for T the rounded table, which is R's table divided by 4096 exactly.
  - This is 2^20 times `nearest_scores`' negative mean L1 (`in_space_long.rs:672`).
  - The cell's sealed G₁₀ is R's. Applying it to F's and T's margins is a convention, stated here and fixed.
- **A kept set or a memory lookup** that differs is never a near tie. With equal codes and equal token ids, both sides must decide it the same way.

**Forced replay, for a step (P, Q) whose picks at p differ.** The forced set Φ starts empty.
1. Run Q on p's window with every decision in Φ forced to P's value. Then compare Q's decisions with P's in the order the forward makes them:
   - position by position, from the window's start to p;
   - within a position: layer by layer, each layer's rate buckets in lane order and each read at its own depth, as `stack_step_fbs` runs them; then the readout read, then the turn memory, then the nearest;
   - within a read: heads in head order, and within a head, the query code, the key code, then the kept set.

   The sheet registers this order with its file and lines at the unit's head, and I-Align checks it. That includes where each stream read runs (`stack_step_fbs` calls `stream(l + 1, …)` in its layer loop), and a citation either way on whether any read sits at depth 0.
2. Take the first decision in p's causal prefix where Q differs from P:
   - **a code or a rate bucket:** add it to Φ with P's value, record whether the difference is a near tie, and go back to 1;
   - **a kept set or a memory lookup:** the step is not accounted for, and the replay stops;
   - **no difference before p's nearest:** go to 3.
3. **At p's nearest,** the step is accounted for when every decision added to Φ was a near tie, and either:
   - Q's pick equals P's;
   - or the two picks still differ, and the pick difference is a near tie.

**Why the replay is sound, and that it ends.**
- Each pass adds a decision not yet in Φ, and the prefix holds finitely many, so the replay ends.
- Effects run only forward, so the first difference in the forward's order is never caused by a later one.
- Forcing a decision removes its jump: every later value then sees P's choice there. It does not remove the continuous drift, the small differences in values that the two sides' parameters or arithmetic leave everywhere. A later decision that the drift flips is found in its turn, forced, and judged by the same near-tie rule.
- So a FAIL caused by drift alone is a real FAIL: the drift carried a decision past its decile, or left a clear pick difference. Its remedy is a finer format or a fix, not a looser rule.

**What the replay costs.**
- A prefix holds at most about 192 × (384 rate buckets + the reads' codes and kept sets) decisions: 6 layers × 64 lanes of buckets per position, over a window of at most 192 positions.
- A pass may resume from the saved state at the position of the decision just forced, since nothing before it changes. Each pass is then at most 192 steps.
  - On R: at claude-243's 1,204 tok/s, about 0.16 s a pass on the laptop.
  - On T: a single-window forward on the GPU, milliseconds.
- A run is never cut short by a self-set budget. If it runs long, the lease is renewed and the extension noted.

**Accounted for, and Q5.** A position is accounted for when each of its steps whose two picks differ is accounted for. **Q5 holds when Row 0 holds and every one of the set's 184 positions is accounted for.**

**Row 0.** These are the instrument checks. They run on every cell before anything is read. If any fails, nothing is read and the instrument is repaired.
- **I-F:** F reproduces F′'s sealed `nearest` and `targets` at every position of every row it scores.
- **I-R:** R's traced run reproduces claude-244's sealed `picks`, `score_q12`, `runner` and `runner_score_q12` at every position of every traced row. A test shows the trace leaves the served picks unchanged.
- **I-Twin:** T's evaluation copy, run through `fbs-snapshot` and `fbs-export --raw-gates --raw-transition`, gives R's sealed artifact byte for byte (sha256) at each seed. If it does not, T is not R's twin, and it is repaired (for example, by loading T's parameters from R's artifact) before anything is read.
- **I-Rerun:** a second run of F, T and R with Φ empty writes byte-identical traces.
- **I-Force:**
  - in every forced run, each decision in Φ takes P's value: a code its bits, a rate bucket its entry;
  - forcing a side to its own decisions leaves its trace unchanged;
  - a pass that resumes from saved state writes the same trace as a pass run from the window's start.
- **I-Align:**
  - the three sides' decision lists match one for one: the same reads, heads, positions and kinds, on every traced row;
  - the tokens each side reads are the window's.
- **I-Complete:** a code-path check, registered in the sheet with its file and lines at the unit's head and checked by its FIT.
  - **R's path is the one this card names.** R's artifact header gives `rec_geometric`, `rec_retention`, `gate_hamilton`, `raw_gates` and `raw_transition`, and it is not snapped. So the exp chain with its keep floor (`session.rs:3778–3790`), the two legacy activation tables and the snap are not run, and the sheet lists each with its lines.
  - **F's path is the same.** F's run configuration has `retain_rates` on, `fold_scores` and `reflect_floor` off, and the transition unsnapped.
  - **The five kinds above are every discrete decision** that R's step and F's path make. Every other data-dependent branch on either path is listed with the reason it is arithmetic: a clamp, a saturation, a rounding, or the choice of exponent in `stack_quantize16`.
  - **No point key or served key** changes F's teacher-forced nearest.
  - **A decision kind this card does not list** voids the card, not only the run. The rule then needs a new card before anything is read.
- **I-Set:** the set's listing reproduces claude-244's (d, d_low) in every cell, and the counts above.
- **I-Inputs:** every input reads back from the store with its SHA256SUMS.

A position that cannot be traced is an instrument repair, never a fault: its row fails Row 0, or its replay cannot apply a forcing or does not end.

**The outcomes:**
- **HOLDS:** Row 0 holds and every position is accounted for.
  - With Q1 to Q4 held, done-when 3 is met for R.
  - The item names R's recipe (`fbs-export --raw-gates --raw-transition`) as the served artifact's. It closes once done-whens 4 and 5 are checked at that head.
- **FAILS:** Row 0 holds and some position is not accounted for. Done-when 3 is not met for R, and no later card re-bars Q5 once it is read.
  - **Each such position is recorded with:**
    - its step;
    - its first decision not accounted for: a code with a clear bit (its family, coordinate, value and G₁₀), a clear rate bucket (its layer, lane, both scores and G₁₀), a kept set or memory lookup that differs, or a clear pick difference;
    - its row, position, read and head.
  - **The step places the remedy:** the export for the structure step, the engine's arithmetic for the arithmetic step.
  - The next unit locates the cause and fixes it with a test, or adopts a finer format. Q1 to Q5 are then read again on the new artifact. The item stays `ready`.
- **VOID:** Row 0 fails. Nothing is read, and the instrument is repaired and run again.

**Readings, which are not part of the bar:**
- for each position: f, t and r; F's own margin at p; and each step's Φ, with the largest ratio of value to G₁₀(φ, P) among its decisions;
- the set's split by step: structure only, arithmetic only, or both;
- every decision difference outside the prefixes, counted by family;
- each rate family's G₁₀ against 1/32, the largest distance an interior score can have from its nearest boundary. A layer whose scores crowd at 0 or 1 can have a loose decile, and this shows it.

**Q5 is read again for any other artifact or recipe,** as claude-245 set.

## Constraints

- **Fixed before it is read.**
  - Every bound is a rule fixed here. G₁₀ for the pick is each cell's sealed number. G₁₀(φ, P) is computed by the rule above from the reference side's trace, with nothing left to choose.
  - The next unit's sheet registers this rule verbatim, with a fresh FIT and a pre-run review. Changing it needs a new card.
  - This unit reads no trace value. Nobody knows whether Q5 holds: it can fail.
- **Strict by design.**
  - A clear flip anywhere in p's prefix fails p, even if forcing it was not needed to reconcile p's pick. A clear flip is a divergence beyond the decile on the path to a clear-margin disagreement, between two sides that should differ only by rounding.
  - A clear pick difference left after every code is forced fails p, even where a clamp, a table or the export's rounding caused it. In that case the artifact does not serve the trained model at a clear margin, and the remedy is a fix or a finer format, as claude-245 sets for a LOSS.
- **Why the decile separates rounding from a fault.**
  - Rounding moves a decision only where its deciding value lies within the rounding error of the threshold: a bit where |v| is that small, a bucket where the score is that near a boundary. A fault's error is not confined to near-threshold values.
  - The decile is a generous band. It would pass a fault whose flips all fall below it.
  - The largest ratio of value to G₁₀ in each Φ is reported as a reading, so the margin each pass has is visible.
- **The writer:** the claude lab, in the item's next unit, under its own sheet with a FIT and a pre-run review (CHARTER §4.5, D67).
- **The scope change.** The instrument needs five files outside the item's scope, which the next unit's ORDER adds to the item's scope list:
  - `crates/uor-r4-integer/src/stack/hamming.rs`: the trace records each code's 64 coordinates, and a forced code replaces the computed one;
  - `crates/uor-r4-float-train/src/flock_read_hamming.rs`: the same for the float read, with a force switch beside its `TIE` and `TRACE`;
  - `crates/uor-r4-float-train/src/geometric_stack_native.rs` and `crates/uor-r4-float-train/src/cuda_stack.cu`: the recurrence records each lane's rate score and entry, and a forced entry replaces `rate_select`'s;
  - `crates/uor-r4-float-train/src/in_space_long.rs`: `score_rows` writes each traced position's query and its top two with their scores, and runs on chosen rows with the trace and a forced set.

  Already in scope:
  - `stack/session.rs`: R's recurrence records each lane's `s_q16` and entry, and a forced entry replaces the computed one;
  - `fbs-snapshot.rs` and `fbs_export.rs` write T's rounded copy, as a new form beside `--rebind-fbs-backward`;
  - `fbs-toks.rs` runs `--forced` with the trace and a forced set.

  `in_space_long.rs`, the recurrence and the Hamming read files are also in other items' scopes, including `one-model-end-to-end`, so the claim's overlap check orders the units.
- **The cost:**
  - **GPU:** F and T on 169 windows of at most 192 tokens, plus the forced replays of single windows. That is minutes of compute: with the nvcc build, under an hour on one RTX 5090, about $1 to $2 (CHARTER §3.5).
  - **The laptop:** R's traced and forced runs are integer decodes, product serving under the owner's 31 Oct allowance for this chain (CHARTER §3.6), and the grade is a count on stored outputs.
- **The residual risks:**
  - **A fault confined to near ties** passes. This is the bar's own blind spot (claude-245), now at every decision rather than at the pick alone.
  - **Forced replay shows that the forced set suffices,** not that each member matters. A near tie that did not matter is harmless. A clear flip that did not matter still fails the position (see "Strict by design").
  - **T's arithmetic is f32 on the GPU.** It is the arithmetic step's reference by convention, not exact.
  - **Teacher forcing,** as in Q1 and Q2: a fault in the serving loop after offset 0 does not show here. claude-244's I-Forced-Greedy covers it in part.
- **Nothing in CHARTER §3 or the caps is touched.** The trace adds records and a forcing switch to existing paths. It adds no operation to the model.

## The decision review

**Round 1 (fresh context, claude-opus-5-5, 1 Nov): REVISE.** The reviewer read claude-243 to claude-246, the item, this card at `c441c1223` and the instruments at the base. It found that the set, the counts and the overall approach hold up, and raised one blocker, two majors and five minors:
- **B1, a decision kind was missing:** the rate bucket, where the recurrence's clamped score chooses one of 16 entries. It is now kind 5, with its near tie, its place in the order, its forcing, its files in the scope change, and I-Complete's check that both sides clamp. I-Complete now also checks that R's and F's paths are the ones named here, and a kind it finds that the card does not list voids the card.
- **M1, the score conversion was left to the sheet.** It is now fixed here: s(u) = −2^12 · Σ |q − e| in f64, from each side's own table. Applying R's cell G₁₀ to the float margins is stated as a convention.
- **M2, the prefix's exclusion of the readout read's earlier choices was not cited.** It now cites `hamming.rs:556–563` and `637–656`.
- **Minors:**
  - "Forcing it removes every difference it caused" is replaced: forcing removes a decision's jump, not the drift, and a FAIL from drift alone is a real FAIL.
  - G₁₀(φ, P) is defined by rank, with zeros counted, from P's unforced run in P's f32 values.
  - The replay's cost is bounded, and a run is never cut short by a self-set budget.
  - Kept sets cite `choose`'s arguments.
  - "Windows" is defined as cell × row.

**Round 2 (fresh context, claude-opus-5-5, 1 Nov): APPROVE.** The reviewer read the revision at `230cef2cb` against the code. Its findings:
- **Every round-1 finding is resolved, and the four requirements hold.**
- **It confirmed the following against the code:**
  - the bucket's boundaries agree exactly: k/16 in f32 and k · 4096 at Q16;
  - on both sides, the score only chooses the entry in the forward;
  - the conversion's factor, 2^12 · 256 = 2^20;
  - the readout payload's exclusion;
  - the replay order in `stack_step_fbs`.
- **It raised three minors, applied here as requirements on the sheet:**
  - the values are judged as the decision reads them under a 16-bit `act_t` build;
  - each rate family's G₁₀ is reported against 1/32;
  - the sheet cites where each stream read runs, including whether any sits at depth 0.

**Approved,** by the claude lab's top session after these two rounds (CHARTER §5.1, §5.2). It is logged on #3462 as `[claude] owner log:`, and Casey can reverse it.

## What a reversal costs

- **The rule is text until the next unit reads it.** Reverting it, or choosing B or D instead, costs this card.
- **After it is read,** the traces are kept and sealed. A revised rule can be re-read on them on the laptop, unless it needs a value the trace does not record.
- **Q1 to Q4 are unaffected.** They stand on claude-244's and claude-246's sealed files.
