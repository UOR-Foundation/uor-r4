# What we are probably dealing with: an assessment before training and end-to-end tests (claude, 29 Oct 2026, revision 3)

Written for Casey's direction of 29 Oct, about 20:05Z: finish the gaps, get the real model ready, train, and test end to end again, "in a smart way that considers the whole project and outside research ... instead of making quick (and wrong) decisions". Casey's notes on SpiralCore transport, the Hamming read, Flock and greedy decoding are in sections 2 and 6.

**Inputs:**
- read-only passes over `origin/main`: the results; the data, split and panels; the milestones;
- a survey of the outside literature (the list is at the end).

**Revision 3** answers a second-round decision review, which FAILED revision 2. Its fixes:
- the control arm is (r0), turn-memory-reply claude-194's empty-table control;
- Fresh V2 is descriptive only;
- the decision rule has a noise band and a mixed cell, written in advance;
- parity is pinned to the read alone;
- the census comes first;
- the missing figures are restored.

**Revision 2** answered a fresh claims check and decision review of revision 1, which FAILED it as written. What changed:
- the diagnosis is narrower;
- the existing fresh panels and the M1 fresh-transfer bar are now in it;
- a cheap check that could overturn the diagnosis comes first;
- the parameter count and the chain's statuses are fixed;
- the M1 bar is marked as Casey's.

Every number cites its source. Inferences are marked **INFERENCE**.

## 1. The answer

1. **The replies are templated, and none of our panels holds new templates.** Three facts:
   - **DEV:** DEV and TRAIN come from the project's own generator, M-world v2 and chat-v0 (`docs/model/data-and-tokenizer.md:50`).
     - Its split hashes value strings, not documents or templates (`crates/uor-r4-training/src/milestone_world_v2.rs:12-17`).
     - The chosen reply key (turn, position, last reply token) is right on 8,822 of the 9,655 later DEV rows it names, 0.914. A reply-only control, which keys on the reply's own last tokens and never on the prompt, already gets 8,089 (`docs/research/notebook/float-training.md`, row 14). So most of that accuracy comes from the reply template itself.
   - **Fresh V2:** the sealed panel is 400 more documents from the same generator, with no exact document overlap (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:69`).
     - There the bigram table T2 scores 6,582 / 14,479, or 45% (`:111`).
     - On DEV it scores 6,300 / 14,287, or 44% (`:50`).
     - So a panel drawn fresh from the same generator looks like DEV.
   - **INFERENCE:** we can measure whether the model handles new *values* inside known templates. That is what copies, `P_succ` and bar (e) measure. We cannot yet measure whether it handles a new *phrasing*.
2. **The memory gives a real gain inside the templates, and the trunk now leans on it.**
   - **The gain:** openings are 598.7 / 598.3 of 1,226 with the table, against 266.3 / 281.0 for the empty control (claude-161). The table's ceiling is 607: its key matches 1,194 openings, and TRAIN's commonest opening for that key is right about half the time (`float-training.md`, row 14).
   - **The dependence:** without its tables, the turn-memory-reply (r) arm falls to 4,101 / 3,807 DEV hits, below the empty-table control (r0)'s 10,079 / 11,171 (`docs/loop/tasks/turn-memory-reply/claude-194/REPORT.md:11, 20`). The literature finds the same: a backbone hands its static patterns to a lookup (Engram, 2026).
   - **INFERENCE:** where no key matches, (r) is probably weaker than (r0). This is untested; section 6, step 2 tests it.
3. **Generated text has never been scored.**
   - The only greedy samples we have come from pre-memory checkpoints, and they ignore the prompt. "Remember, the code word is kandgind." gets "Got it, your friend is kl dog is."
   - After step 0, every arm picked 1 to 4 distinct opening tokens over all 1,226 openings (claude-151 REPORT:26-32).
   - In the literature, gains from exact memory shrink in free generation (Wang et al. 2023, "kNN-LM does not improve open-ended generation").
4. **The mechanisms are sound for what they are, and the literature has close relatives for each:**
   - the Hamming read is the HashAttention / Hamming-attention family, with ReLU-style weights;
   - the turn memory is kNN-LM, ∞-gram and Engram;
   - the routes are a structured sparse transition, inside PD-SSM's expressive class;
   - the objective is a large-margin LM with nearest-embedding decoding (vMF loss).

   The risk is that our panels cannot yet tell recall of a template from reading the prompt.
5. **So the next steps are two cheap measurements before the training run.** Both belong to the last item of the open direction's order: "`turn-memory-reply` first, then the integer serving of the parts, R7, and one model end to end" (`PLAN.md:86`). They decide what "end to end" must measure.

## 2. The model: token → vector → vector → integer → token, and where SpiralCore sits

| Step | What happens | SpiralCore's part | Status on `main` |
|---|---|---|---|
| token → vector | A BPE id selects a row of the shared 4,096 × 256 table, 64 quaternion lanes | — | Trained; served |
| vector → vector, the router | Six recurrence layers, each with six SpiralCore/FBS maps (`rec.in`, `rec.gate`, `rec.out` and three inner maps) | Each output block of 8 picks one input block and moves it by a signed permutation. The permutation is composed from SpiralCore's octonion generators along a path between E8 roots (`docs/mechanisms/spiralcore-fbs.md`) | Trained; served exactly under `lut-stack/5` (claude-182) |
| vector → vector, position | Word order inside the recurrence | SpiralCore's transport spin, with no RoPE (`docs/model/architecture.md`, the 29 Oct table) | Trained; served |
| vector → vector, look-back | The Hamming read: lanes turned by learned quaternion rotations, reduced to 64-bit sign codes, matched by popcount | **This is where a transported vector becomes an integer code inside the model.** The type-spin variant adds SpiralCore's path transport: frames are products of icosian steps, and a type head compares through `F_t ⊗ F_s⁻¹` | Trained in float. The integer session's hooks exist (integer-hamming-read claude-194 and 195). Since claude-200 (`d28142f9e`) the export carries it; parity is step 5. The type spin was cell D (claude-151), and the export refuses it |
| vector → vector, storage | The turn memory moves the readout point: `n + w(c − n)` | — | Trained (turn-memory-reply claude-194); exported (claude-198); not served (units 2–3 remain) |
| vector → integer | Export; the routes are exact | The transport is exact in integers. Drift can come only from the gates, the state and the sign bits | The trunk is inside the export-gap bar (claude-182) |
| integer → token | Nearest row of the same table | — | Decided (claude-150). The float best is 10,152 / 11,140 of 14,287 at width 512, one figure per seed (claude-153; `pipeline-map.md:142`). The served session still uses the native writer |

**Size:** 1,112,576 learnable values, 94% of them the table. The trunk is about 64,000 values (`docs/model/pipeline-map.md`, stage 4).

**Three open questions about the mechanisms, all from our own records:**
1. **Do learned SpiralCore routes pay?** On 26 Oct, on the rung-0 Hamilton stack at 1,024 steps:
   - learned routing lost to frozen random routing on paired float top-1 in 8 of 8 comparisons;
   - identity transport had the lowest loss;
   - the trained routes did not reproduce between byte-identical builds (`spiralcore-fbs.md`, 26 Oct row).

   The fitting hypothesis was the atomic backward, fixed-order since claude-179. That comparison has not been repeated on today's trunk with the fixed backward.
2. **Does reading by meaning (Flock) pay?** The Flock read uses the transport, Hamming distance and consensus together (`docs/mechanisms/flock-read.md`):
   - the K nearest earlier positions, ranked by frame-root L1 distance, then Hamming distance;
   - each payload carried into the query's frame by SpiralCore's transport;
   - a snapped consensus;
   - a trit-gated add.

   Its trunk run (claude-144) was cell D: openings 235 against a bar of 236 at seed 1, and 264 against 307 at seed 2. Its weight moved (0 → 0.78 / 0.94). On a panel where the winning template decides the opening (claude-151), a read by meaning could not show its effect. **INFERENCE:** it, and the type spin, deserve a retry on a panel that holds new templates.
3. **Greedy decoding** (pipeline-map gap 10: "no lawful sampling rule").
   - Our replies are fluent but templated (stage 8). Greedy decoding degenerates into repetition (Holtzman et al.).
   - The integer Flock selector's rank weights are harmonic, `w_i ∝ 1/(i+1)`, with no exponential (`crates/uor-r4-integer/src/stack/flock.rs:379-420`).
   - So a Flock rank decode rule is a lawful candidate, if it names the harmonic table as its weight. The module comment at `flock.rs:19` leaves "softmax/weight application" to the consumer, and a consumer could bring in what CHARTER §3.2 bans.
   - Choosing a rule is its own decision unit, scored against greedy.

## 3. The data and the panels

| Set | What it is | Size | Overlap evidence |
|---|---|---|---|
| TRAIN `mw-balp/train` | Generated by the project: M-world v2 + chat-v0 | **Conflicting:** 7,512,797 tokens / 2,425,925 rows (`hamilton_sheet.rs:81-82`) against 26,214,400 tokens / 10,975,281 targets for the same `tokens.u16` sha (`m1-fresh-panel-transfer-2026-10-26.md:62-65`). The census must settle which | — |
| DEV `ft/dev` | Same generator; how it was drawn is not found; its manifest says `split: train` | 14,287 rows, 1,226 openings | 1,194 openings keyed in TRAIN; table ceiling 607. Reply-only control 8,089 of later rows |
| Fresh V2 (sealed, opened once for bar (e)) | 400 `milestone_world_v2` documents | 14,479 targets, 3,594 `P_succ` | 0 exact full-document overlap with TRAIN or DEV. T2 6,582, U 1,279. The discrete model scored 7,215 / 7,228 (bar (e) PASS) |
| `m2-fresh256` | A different generator: 256 job/home recall rows, 34 fresh literals | 256 | Overlap checked against 74 historical literals, not TRAIN |
| K16 `Split::Fresh` | An unopened 2,000-row slice | 2,000 | Not examined (the reviewer's pointer) |

**Already built:** the `m1-fresh-panel-transfer` instrument (`fresh_rows_from_corpus`, `build_train_count_tables`). It is reused, not rebuilt.

## 4. What the literature predicts for this shape

- **Exact memory.**
  - Gains sit on text that overlaps the training data, and on rare tokens (RETRO; Wang et al. 2023).
  - The memory makes most other tokens worse, and degrades on model-generated queries.
  - A "most common next token" table pushes replies toward generic ones (Li et al. 2016).
- **Fixed-state recurrence.**
  - Recall past the state's capacity needs a read (Zoology/MQAR; Repeat After Me; "RNNs are not Transformers (Yet)").
  - Ours holds about 6 tokens (stage 4). So all longer recall rests on the Hamming read: its top 16 positions inside the 192-token window.
- **Hamming selection** works for picking tokens (HashAttention). Published systems still weight the picks with a softmax, or distil from a teacher.
  - **INFERENCE:** the 65 distance levels make ties at the 16th place common. Sign bits near zero flip between float and integer, so drift shows first in the kept sets.
- **Margin objective with nearest decoding.**
  - It gives a ranking, not probabilities.
  - It suffers from hubness: a few points become everyone's nearest neighbour.
  - Greedy decoding repeats.
- **Scale.**
  - At 1.1M values, about 20 tokens per value (Chinchilla) is 22M tokens.
  - Turn-memory-reply (r) already saw about 50M token positions: 16,384 steps × 16 × 192.
  - **INFERENCE:** the number of tokens is not the limit. Diversity is. The larger `balp` (99.3M tokens) is mostly the same generator, so it adds volume more than new templates.

## 5. What will probably break, and the check that shows it

| Probable failure | Why | Deterministic check |
|---|---|---|
| (r) is below (r0) where no key matches | Co-adaptation (claude-194; Engram) | Score (r), (r0), and (r) with its memory off on DEV's 3,406 later rows the reply table does not name, and the 32 unkeyed openings |
| Unseen phrasings fail | Every panel shares the generator's templates | A panel with held-out templates (section 6, step 1, or the unit after it) |
| Replies copy TRAIN verbatim, or repeat | Templated data, exact tables, greedy | Free-generation scores: the share of replies equal to a TRAIN reply; the share of new n-grams, n = 4–8 (RAVEN); distinct-1 and distinct-2; repeated-n-gram rate |
| The reply table stops hitting in free generation | Its key includes the model's own last token | Table hit rate under teacher forcing against free generation |
| Integer drifts in the read | Sign bits flip near zero | Kept-set agreement; the first divergence in greedy decoding (step 5 parity) |

The rank code (the Elias-γ length of the true token's rank) is a **diagnostic only**. CHARTER §4.8 brings bits per byte back only "once a lawful likelihood exists", so it is neither called BPB nor used as a bar.

## 6. The order of work

All of it belongs to "one model end to end" (`PLAN.md:86`). Model scoring runs on the GPU (CHARTER §3.5). Only counting the data runs on the laptop.

1. **The data census (laptop, counts only; first, because step 2's measures need it).**
   - Resolve the TRAIN size conflict. Build the TRAIN reply index (exact replies and reply n-grams) that step 2's free-generation measures read.
   - For DEV, Fresh V2 and `m2-fresh256`, measure:
     - the share of reply n-grams (n = 8 and 13) found in TRAIN replies;
     - the share of prompt templates seen in TRAIN, with values masked;
     - the 8-token key share.
   - Leave the K16 fresh slice unopened. The census reads only its manifest, to say what it holds.
   - Report which panel, if any, holds new templates. If none does, the following unit builds one from held-out phrasings, with its own overlap census.
2. **The check that could overturn the diagnosis (GPU, small; prep sheet, FIT, pre-run review).**
   - **Arms:** turn-memory-reply claude-194's checkpoints, (r) and (r0), and (r) with its memory off. The prep sheet pins:
     - their store paths and `SHA256SUMS` (claude-194 REPORT:97-104);
     - the binary's sha;
     - a lease and cost inside the caps.
   - **The decision rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged **pooled** (3,438 rows). The 32 openings alone are too few to decide anything, so they are reported beside the pool, not judged apart. The prep sheet names each arm's checkpoint step.
   - **The decision rule, fixed in advance.** It compares (r) against (r0) at each seed, with a band of `max(2·√(b+c), the two-seed spread)`, where b and c are the paired discordant rows (as the Fresh V2 sheet does). The cells:
     - (r) ahead at both seeds: the co-adaptation fear is not borne out on DEV's unnamed rows, and the memory recipe stays the training default;
     - (r) behind at both seeds: the training run carries a memory-dropout arm beside (r) and (r0);
     - mixed (the seeds disagree, or the result is inside the band): no change to the recipe, and the training run carries both arms.
   - **Descriptive only, with no threshold:**
     - Fresh V2. It was opened once, for bar (e), and is never used for a decision here, so it stays a held-out description. Even if the census finds new templates in it, it is not a decision stratum in step 3 or step 5. Those take a panel no decision has touched;
     - 64 greedy samples per arm, scored by the free-generation measures from step 1's index.
   - **The same pod also runs step 5 of `integer-hamming-read`,** the parity of the exported read against the float forward.
     - It is pinned to the read alone: kept-set agreement and the first greedy divergence, with the memory off on both sides. The memory is exported but not yet served.
     - Any miss is classified as instrument or mechanism before it is called anything (L-007).
3. **The scoring protocol, written before any training run.**
   - **Arms:** (r), (r) memory-off, (r0), tables only, and a longest-suffix (∞-gram) table over TRAIN. Count tables are lawful controls (§4.4, §3.10).
   - **Strata:** keyed, unkeyed and new-template.
   - **Scores:**
     - teacher-forced top-1;
     - free generation;
     - decode rules: greedy against a Flock rank rule;
     - on the integer artifact: kept-set agreement, first divergence, and tok/s against context.
   - **The verdict rule, written in advance:**
     - "memorized": the gains appear only on keyed rows, and generated replies are mostly TRAIN replies;
     - "generalizes": the model beats (r0) and the ∞-gram on unkeyed and new-template rows, with novelty near DEV's own replies.
4. **The serving path, in parallel:**
   - `turn-memory-integer` units 2 and 3;
   - the readout, served nearest;
   - R7, the text path.
5. **The training run for the real model,** after steps 1–3. The prep sheet takes its bars from step 3, and its data and any new-template panel from step 1 or the unit after it.
   - **Arms:**
     - (r) and (r0), at two seeds, with a memory-dropout arm if step 2 calls for one;
     - the transport controls with the fixed backward: routes frozen at their random start, and identity transport;
     - the read-by-meaning arms, scored on the new-template stratum: the Flock read, and the type spin;
     - decode rules on every arm.

## 7. Calls that are Casey's, and what this does not settle

- **The M1 bar is the owner's** (CHARTER §4.8, D39). Its fresh-transfer bar (e) is met on Fresh V2, a panel from the same generator.
  - Whether M1 should also require new templates is Casey's call. One `[claude, for Casey]` note goes on #2028.
  - Nothing here waits on it.
- **Not settled:**
  - how DEV was drawn;
  - the TRAIN size conflict;
  - the contents of the K16 fresh slice;
  - whether the generator can produce held-out phrasings at all.

## References (verified in the survey)

- **Hamming and hashing attention:** Reformer arXiv:2001.04451; HashAttention arXiv:2412.14468; Hamming Attention Distillation arXiv:2502.01770; BiBERT arXiv:2203.06390; ReLU attention arXiv:2104.07012.
- **Recall and fixed-state models:** Zoology arXiv:2312.04927; Based arXiv:2402.18668; Repeat After Me arXiv:2402.01032; RNNs are not Transformers (Yet) arXiv:2402.18510; induction heads arXiv:2209.11895; PD-SSM arXiv:2509.22284.
- **Memory:** kNN-LM arXiv:1911.00172; kNN-LM in generation arXiv:2305.14625; Infini-gram arXiv:2401.17377; RETRO arXiv:2112.04426; Engram arXiv:2601.07372.
- **Multiplication-free and integer:** MatMul-free LM arXiv:2406.02528; BitNet b1.58 arXiv:2402.17764; Q-S5 arXiv:2406.09477; Jacob et al. arXiv:1712.05877.
- **Losses and decoding:** large-margin LM arXiv:1808.08987; vMF loss arXiv:1812.04616; hubness arXiv:1412.6568; degeneration arXiv:1904.09751; diversity objective arXiv:1510.03055.
- **Evaluation and scale:** RAVEN arXiv:2111.09509; deduplication arXiv:2107.06499; TinyStories arXiv:2305.07759; Chinchilla arXiv:2203.15556.
