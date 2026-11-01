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

| Cell | G₁₀ | Positions | Counted | Cut windows | Rows | Offsets | R's gaps |
|---|---|---|---|---|---|---|---|
| DEV s1 | 30,228 | 2 | 2 | 0 | 2 | ≤ 22 | 56,280 to 64,326 |
| DEV s2 | 22,526 | 4 | 4 | 0 | 3 | ≤ 31 | 33,416 to 72,272 |
| DEV s3 | 24,022 | 5 | 5 | 0 | 5 | ≤ 37 | 25,889 to 82,405 |
| p-w s1 | 10,112 | 60 | 53 | 7 | 54 | ≤ 41 | 10,408 to 109,897 |
| p-w s2 | 8,949 | 34 | 32 | 2 | 34 | ≤ 33 | 9,245 to 35,321 |
| p-w s3 | 7,760 | 79 | 66 | 13 | 71 | ≤ 42 | 7,779 to 125,797 |
| **All** | | **184** | **162** | **22** | **169** | ≤ 42 | 7,779 to 125,797 |

- One position is an opening (offset 0, p-w s1). The rest are later positions of a reply.
- The counted positions are claude-244's 11 on DEV and 151 on p-w. The next unit's sheet registers this listing and checks it again (I-Set below).

**A disclosure.** While listing the set, the author saw which side names the reference token at each position: R at 49, F′ at 48 and neither at 87. This only splits a and b, which claude-246 already read, and it is no trace value. The rule below never compares a pick with the reference.

### What R's step decides, at the base

Every discrete decision in R's step at the base (`stack_step_fbs`, `crates/uor-r4-integer/src/stack/session.rs:3232–3349`; the Hamming read, `stack/hamming.rs:556–656`) is one of four kinds.

1. **A Hamming code.** A code packs 64 bits, each the sign of one coordinate (a coordinate at or above 0 is a 1). Its source depends on the head:
   - **from token ids and parameters:** every read's copy-head query and key codes, and its type-head key codes (`record`, `hamming.rs:568–590`);
   - **from the stream at that position:** the type-head query codes (`types.queries(point)`).
2. **A kept set:** the positions a head keeps, chosen from the codes and the token ids by the read's tie rule.
3. **A turn-memory lookup:** an exact key on token ids, and the stored entry it reads (`session.rs:3296–3318`).
4. **The nearest:** the whole table's nearest token, by −L1 at Q12, with ties going to the smaller id (`session.rs:3354–3370`).

R has no point key: claude-244's forced mode refuses a point-key artifact. Everything else in the step is arithmetic: the gathers, norms, recurrence, tables, clamps, saturation and rounding. F′'s trunk computes the same decisions in float, which is what claude-235's trace comparison relies on.

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
  - F′'s own scoring path, `score_rows`, with claude-243's arguments for that seed (its [sheet](../../../harness/prep/integer-nearest-readout-tie-parity-2026-11-01.md), line 22), on the GPU;
  - with the trace on and the top two written with their scores.
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
- at every position q ≤ p: every stream read's codes and kept sets, and the turn memory's lookups;
- at p alone: the readout read's codes and kept set, and the nearest.

It does not read the readout read's choices or the pick at any q < p. Their outputs reach only q's own pick, and the readout read's payloads are arithmetic. The trace records every decision at every position up to p. Only the decisions in p's prefix enter p's account; the rest are kept as readings.

**Near ties.** These are fixed now, in the same way the bar fixes the pick's near tie (claude-244, claude-245: a margin in its cell's lowest decile).
- **A code's family** is one read's codes of one head kind (copy or type) in one role (query or key).
- **G₁₀(φ, P)**, for family φ on side P, is the 10th-percentile |v| (nearest rank, rounding down) over every coordinate of every code of φ that P's trace records in the cell. That covers all the cell's traced rows, at every position the trace records.
- **A code difference is a near tie** when every bit that differs has |v_P| ≤ G₁₀(φ, P), where v_P is the coordinate's value on the step's reference side P.
- **A pick difference at p is a near tie** when the step's two sides each rank the two picks within G₁₀:
  - P picks u, Q picks w, and G₁₀ is the cell's sealed G₁₀;
  - s_P(u) − s_P(w) ≤ G₁₀, and s_Q(w) − s_Q(u) ≤ G₁₀;
  - each score is in R's units (Q12 of −L1). The sheet registers the float scores' conversion with its code path.
- **A kept set or a memory lookup** that differs is never a near tie. With equal codes and equal token ids, both sides must decide it the same way.

**Forced replay, for a step (P, Q) whose picks at p differ.** The forced set Φ starts empty.
1. Run Q on p's window with every decision in Φ forced to P's value. Then compare Q's decisions with P's in the order the forward makes them:
   - position by position, from the window's start to p;
   - within a position: the stream reads by depth, then the readout read, then the turn memory, then the nearest;
   - within a read: heads in head order, and within a head, the query code, the key code, then the kept set.
2. Take the first decision in p's causal prefix where Q differs from P:
   - **a code:** add it to Φ with P's value, record whether the difference is a near tie, and go back to 1;
   - **a kept set or a memory lookup:** the step is not accounted for, and the replay stops;
   - **no difference before p's nearest:** go to 3.
3. **At p's nearest,** the step is accounted for when every code added to Φ was a near tie, and either:
   - Q's pick equals P's;
   - or the two picks still differ, and the pick difference is a near tie.

The replay ends, because each pass adds a code not yet in Φ, and the prefix holds finitely many. Effects run only forward, so the first difference in the forward's order is never caused by another difference. Forcing it removes every difference it caused.

**Accounted for, and Q5.** A position is accounted for when each of its steps whose two picks differ is accounted for. **Q5 holds when Row 0 holds and every one of the set's 184 positions is accounted for.**

**Row 0.** These are the instrument checks. They run on every cell before anything is read. If any fails, nothing is read and the instrument is repaired.
- **I-F:** F reproduces F′'s sealed `nearest` and `targets` at every position of every row it scores.
- **I-R:** R's traced run reproduces claude-244's sealed `picks`, `score_q12`, `runner` and `runner_score_q12` at every position of every traced row. A test shows the trace leaves the served picks unchanged.
- **I-Twin:** T's evaluation copy, run through `fbs-snapshot` and `fbs-export --raw-gates --raw-transition`, gives R's sealed artifact byte for byte (sha256) at each seed. If it does not, T is not R's twin, and it is repaired (for example, by loading T's parameters from R's artifact) before anything is read.
- **I-Rerun:** a second run of F, T and R with Φ empty writes byte-identical traces.
- **I-Force:**
  - in every forced run, each decision in Φ takes P's value;
  - forcing a side to its own codes leaves its trace unchanged.
- **I-Align:**
  - the three sides' decision lists match one for one: the same reads, heads, positions and kinds, on every traced row;
  - the tokens each side reads are the window's.
- **I-Complete:** a code-path check, registered in the sheet with its file and lines at the unit's head and checked by its FIT.
  - The four kinds above are every discrete decision that R's step and F's path make.
  - No point key or served key changes F's teacher-forced nearest.
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
    - its first decision not accounted for: a code with a clear bit (its family, coordinate, value and G₁₀), a kept set or memory lookup that differs, or a clear pick difference;
    - its row, position, read and head.
  - **The step places the remedy:** the export for the structure step, the engine's arithmetic for the arithmetic step.
  - The next unit locates the cause and fixes it with a test, or adopts a finer format. Q1 to Q5 are then read again on the new artifact. The item stays `ready`.
- **VOID:** Row 0 fails. Nothing is read, and the instrument is repaired and run again.

**Readings, which are not part of the bar:**
- for each position: f, t and r; F's own margin at p; and each step's Φ, with the largest |v_P| / G₁₀(φ, P) among its bits;
- the set's split by step: structure only, arithmetic only, or both;
- every code difference outside the prefixes, counted by family.

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
  - Rounding flips a bit only where |v| lies below the rounding error at that coordinate, and a fault's error is not confined to near-zero coordinates.
  - The decile is a generous band. It would pass a fault whose flips all fall below it.
  - The largest |v_P| / G₁₀ in each Φ is reported as a reading, so the margin each pass has is visible.
- **The writer:** the claude lab, in the item's next unit, under its own sheet with a FIT and a pre-run review (CHARTER §4.5, D67).
- **The scope change.** The instrument needs three files outside the item's scope, which the next unit's ORDER adds to the item's scope list:
  - `crates/uor-r4-integer/src/stack/hamming.rs`: the trace records each code's 64 coordinates, and a forced code replaces the computed one;
  - `crates/uor-r4-float-train/src/flock_read_hamming.rs`: the same for the float read, with a force switch beside its `TIE` and `TRACE`;
  - `crates/uor-r4-float-train/src/in_space_long.rs`: `score_rows` writes each position's top two with their scores, and runs on chosen rows with the trace and a forced set.

  Already in scope:
  - `fbs-snapshot.rs` and `fbs_export.rs` write T's rounded copy, as a new form beside `--rebind-fbs-backward`;
  - `fbs-toks.rs` runs `--forced` with the trace and a forced set.

  `in_space_long.rs` and the Hamming read files are also in other items' scopes, including `one-model-end-to-end`, so the claim's overlap check orders the units.
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

Pending.

## What a reversal costs

- **The rule is text until the next unit reads it.** Reverting it, or choosing B or D instead, costs this card.
- **After it is read,** the traces are kept and sealed. A revised rule can be re-read on them on the laptop, unless it needs a value the trace does not record.
- **Q1 to Q4 are unaffected.** They stand on claude-244's and claude-246's sealed files.
