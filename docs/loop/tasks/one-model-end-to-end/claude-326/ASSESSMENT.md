# ASSESSMENT: why no served reply is complete (one-model-end-to-end, claude-326)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Owner decision: "Yes, pivot assessment" (#2028, 6726003019). No compute was spent. Every number below is cited to a row of [float-training.md](../../../../research/notebook/float-training.md) on `main` at `fa5004861` (cited as "row N", the file's line number), a REPORT, or a source line.

## The answer in one paragraph

**The record points at the mechanism, at one position: the reply's first token, where the model has to choose which value in the prompt the question asks for.** Data, scale and the objective each moved something measurable from 4 to 8 November: stopping, held-out text, and copying. None of them moved this choice.
- **The copy heads cannot make that choice, by construction.** They match the current token against earlier tokens.
- **The read's only question-dependent path is the type heads' learned query.** No learned-query read on record has learned where to look.

What the record cannot yet rule out is that the learned query would learn with a scored target that is the choice itself; claude-325 is that run, now training. **The smallest test that separates the two is a synthetic recall probe whose query is not the key** (Q4). It reuses the 27 Oct probe with one change.

## What did not work, first

- **0 complete replies and 0 pairs on `m2-fresh256` in every served configuration, 4–8 Nov** (bar: 104 of 256 complete, 52 of 128 pairs):
  - claude: rows 13, 14, 15, 22, 25, 28 and 34;
  - DeepSeek's SFT: rows 17, 21, 23 and 26;
  - claude-259's six served artifacts: row 30.
- **Every lever on this line was one knob on (b64L)'s recipe, read on the same panel.** None measured the step where the replies fail.
- **D21's pivot card was due long before this unit.** Three merged PRs on one line without the headline moving force one, and this line has had more than ten since claude-309 (rows 13–34).
- **The first-token measurement exists only for DeepSeek's checkpoints** (rows 17, 18). No claude checkpoint from claude-308 to claude-325 has one.

## Q1. What blocks a complete reply: data, scale, objective or mechanism?

### The panel's task, and where the replies fail

A panel prompt states two facts and asks for one, and the reply is the asked value. claude-324 lists the panel's served layout (`one-user-turn/v1`) and builds its TRAIN corpus in that shape. claude-324's REPORT gives one generated row:

```
User: I am a bowl mall round teacup turtle snowy by trade. I just moved to shrimp donut hairbrush coffee. What is my job?
Assistant: bowl mall round teacup turtle snowy.<|eos|>
```

The reply's first target is the asked value's first word. Every later target either continues that value or closes the reply.

DeepSeek scored this split teacher-forced on its selected SFT artifact (row 18; [deepseek-34 REPORT](../deepseek-34/REPORT.md)):

| Position | top-1 | top-8 | top-32 | rank min / median / max (of 4,096) |
|---|---|---|---|---|
| **First target**, `per-turn/v1` | 0/256 | 0/256 | 0/256 | 67 / 956 / 4,055 |
| **First target**, `one-user-turn/v1` | 0/256 | 0/256 | 0/256 | 43 / 1,189 / 4,060 |
| **Later targets**, after a correct first token | 1,614/2,576 | 2,574/2,576 | n/a | n/a |

The same run's opening query spread was 0.26249 at the answer position, against 2.06252 over all rows. **The prompt barely changes the state the first token is read from.** deepseek-36 then repeated reply openings eight times in training (row 17). First-target top-32 stayed 0/256 at every checkpoint (median rank 1,025–1,134).

### Why later targets succeed and the first fails: the read's design

The Hamming read's module doc (`crates/uor-r4-float-train/src/flock_read_hamming.rs` on `main`) states two things:
- **The copy heads:** the key of an earlier position `s` is the table point of token `x_{s−1}`. A copy head's query is the current token `x_t` through the same rotation, so "an earlier copy of the current token is an exact match" (lines 14–17). What it brings back is "the token after the last copy of the current token" (lines 47–49).
- **The type heads:** their query is "the residual through a second rotation, which must learn what to ask for" (line 18).

Each half of the measurement follows from that:
- **The later targets.** Once the first value word is given, that word also sits in the prompt, and the copy head brings back the next one. That is the induction match the copy head is built for, and later targets are top-1 at 1,614 of 2,576 (row 18).
- **The first target.** There the current token is the role header's last token, and the copy head can bring back only what followed its earlier copies, such as the token after `User:`. Choosing *job* or *home* from "What is my job?" needs a query that depends on the question, and only the type heads have one.
- **No learned query has ever learned where to look.**
  - The read's own doc says reads keyed on the trunk's state "never learned where to look (claude-144, claude-145, and this read's first form)" (lines 4–5).
  - The 27 Oct content read (row 110), the induction read's language run (row 108) and the type heads with SpiralCore's spin (row 106) all end with "replies still ignore the prompt" (row 106: "Every arm picks 1–4 distinct opening tokens, so openings track which template wins").

### Each candidate against the record

| Candidate | What it moved | What it did not move | Verdict on the blocker |
|---|---|---|---|
| **Data**: mw-balp alone, D3 mixes × 1 / × 3 / capped × 3 / capped × 6, D4, DeepSeek's ratio, dose and opening weight (rows 14, 15, 17, 21–23, 25, 26, 28, 39) | **Stopping**: 0 → 253 / 244 EOS stops (row 28), and EOS median rank 7 → 2 (row 26) | Complete replies: 0 in every one. **The first line holds a value from the prompt, but often the wrong one** (asked / other): 25 / 21 and 76 / 36 (row 15), 14 / 11 and 2 / 6 (row 14), 0 / 0 and 32 / 30 (row 13). Values are copied, not selected | **Not excluded.** Every corpus so far scored long or stock replies. A corpus whose scored reply *is* the choice, in the panel's shape, was first built in claude-324 and is training now (claude-325) |
| **Scale**: width 128 → 256 (row 16), 512 and 1,024 (rows 16, 41–43), D3 → D4 (row 39), steps (rows 38, 46), the capacity sweep (row 81) | Held-out text: width 256 is ahead of 128 by d +115,279 / +134,421 (row 16) | Recall: the capacity sweep found "No arm carries" (row 81). Width 512 and above never trained to the end | **Unlikely as the first cause.** At the same width, the 27 Oct probe recalls exactly (row 109). Capacity for retrieval exists; a path from the question to the value is what is missing |
| **Objective**: the nearest objective against cell, fixed targets and learning rate (rows 46–48) | Human text learned: (m1) and (near) are ahead of the control (row 47); (b64L) meets the one-track bar (row 44) | n/a | **Not the blocker for retrieval.** The same objective trains the 27 Oct probe to 0.98–0.99 recall (row 109). The record cannot tell objective from mechanism at the first token alone; the probe in Q4 can |
| **Mechanism**: the read's query path | Exact-token recall 0.984–0.994 in the far buckets, no read 0.000–0.002 (row 109); continuation copying (row 18) | **Query-conditioned choice at the first token: 0/256 top-32 (rows 17, 18)**, and every learned-query read on record (rows 106, 108, 110) | **The most likely blocker.** It is the only candidate consistent with every row, but it has not been tested directly (Q2) |

Serving is not the blocker either:
- teacher-forced float and integer agree on token choice (row 20: 0 and 1 mismatches of 2,832 targets);
- the first target fails teacher-forced, before any free generation.

Free-generation parity is still not established: 27 of 64 whole replies differ (row 35).

## Q2. What is established about selecting the queried value, and what was never measured

**Established:**
1. **Exact-token retrieval** (row 109): the induction-keyed read recalls 0.984–0.994 at distances 33–180 on the MQAR-style probe, at seeds 1 and 2. **In that probe the query writes the key again** (`in_space_long.rs:11803`, "Its query writes the key again at `q = p + d`"). So the probe tests only the case where the question token is the key token.
2. **Continuation once aligned** (row 18): later targets are 1,614 / 2,576 top-1 and 2,574 / 2,576 top-8.
3. **First-token choice fails on DeepSeek's SFT checkpoints** (rows 17, 18): 0/256 top-32 under both renderings, at every checkpoint of deepseek-36.
4. **Copying without choosing in claude's served replies** (rows 13–15): the first line often holds one of the prompt's two values, with the asked one barely ahead of the other. claude-323's seed 2, at 32 / 30, is the closest.

**Never measured:**
1. **The first-target rank on claude's own checkpoints:**
   - claude-313 to claude-325, teacher-forced, under `served-reply-panel`'s `first_target` tally (added by deepseek-34);
   - the seals of claude-322, claude-323 and claude-325 are stored;
   - a laptop measurement.
2. **Recall where the query is not the key:** any synthetic task that needs a learned query, as the panel does ("job" names the value after "I am a … by trade"). The 27 Oct probe never posed it.
3. **What the type heads keep at the first reply position:** whether the asked value's position is among a type head's 16 kept positions (`HAMMING_KEPT`), and with what weight.
   - The integer serve already writes Hamming traces (`HammingTrace`, used by `fbs-toks`).
   - claude-259's trace is the one prior reading. The module doc cites it only for the one-token copy key reading every earlier `" is"` alike (lines 32–34).
4. **Whether the answer position's state carries the question word.** The module doc says a token 16 back moves the trunk's prediction by 1–2% of the last token (lines 3–4). The question word is about 4 tokens before the answer position, and that distance has not been measured.

## Q3. The levers: closed, parked, never tried

**Closed configurations** (a configuration, never a mechanism, §4.1):
- **Corpora:**
  - mw-balp alone (row 28);
  - D3 + mw-balp × 1 (row 25), × 3 (row 22), capped × 3 (row 15) and capped × 6 (row 14);
  - D4 (row 39).
- **DeepSeek's SFT:** the ratio (row 21), the dose (row 23) and opening weight 8 (row 17).
- **Copy keys:** mixed keys `2,1` at × 6 (row 13); copy key 2 inconclusive (rows 67, 76); the slot key's atom rule negative on its oracle (row 66).
- **Reads:** the content read (row 110), the type spin (row 106) and the induction read's language run (row 108), all at their 27 Oct recipes.
- **Capacity:** the sweep's (h) arm (row 81).
- **Long context:** P2b's read configuration (rows 53, 57).

**Parked or blocked:**
- width 512 and 1,024 (memory, rows 16, 41–43);
- 32,768 steps (stopped by decision, row 38);
- the decode-time read (bounded at 5 rows, claude-323);
- `integer-nearest-readout` (owner stop, 2 Nov);
- writer admission (#3394, open).

**Never tried:**
- **(a) The panel-shaped bare-answer corpus** (claude-324), where the scored reply is the choice. claude-325 trains it now.
- **(b) A read whose query is built from the question.** The read would find a content word of the question (a first Hamming match) and use what it found to key the copy match. This is the composition a transformer's induction circuit uses, built here with hard Hamming top-k and no exponential.
- **(c) A synthetic probe of recall where the query is not the key.**
- **(d) The first-target rank and a type-head trace on claude's checkpoints.**

## Q4. The smallest structural test, and what each result unlocks

**A proposal for the next unit's sheet (claude-327), not a run.** It has two parts, both small.

### Part A (laptop, no GPU): read what the served models do at the first token

On the stored seals of claude-322 and claude-323, and claude-325's when they land, at both seeds:
1. **The first-target ranks:** `served-reply-panel`'s `first_target` tally.
2. **A Hamming trace at the first reply position of each row:**
   - for each head (copy and type), whether its 16 kept positions include the asked value's first token, the other value's first token, or neither;
   - each head's weight.

Each finding unlocks one next step:
- **Type heads keep the asked value's position but its weight is low:** the query finds the value and the combine step does not use it. The next lever is the read's weights or margin, an objective-side change, tested first on Part B's probe.
- **Type heads keep both values' positions about equally:** the query reaches the value slots but not the question. Part B decides whether a learned query can learn the choice.
- **Neither value is kept:** the query never reaches the value slots. Part B is required before any further language run.

### Part B (GPU, finite, about $0.5): recall where the query is not the key

Take the 27 Oct probe (row 109; `recall_window`, `--recall`), at (b64L)'s width 256 and read 4, and change one thing:
- **The change:** the query writes a paired token `q(k)` from a disjoint range under a fixed bijection, in place of the key `k`. A learned query must map `q(k)` to `k`'s key to recall `k`'s value, which is what the panel's "job" → "by trade" asks.
- **Arms:**
  - the induction control (query = key; must reproduce row 109's ≥ 0.98, or the run is an instrument fault);
  - indirect with the current read;
  - indirect with no read (chance);
  - at seeds 1 and 2.
- **The bar, row 109's own:** R ≥ 0.50 and R ≥ R(no read) + 0.10 in both far buckets, at both seeds.

Each result unlocks one next step:
- **Pass:** the read can learn a query-conditioned choice. The blocker is then data or the language setting, so the next language run is claude-325's recipe, made larger or given a curriculum from the probe's form to the panel's.
- **Fail with the control passing:** the read cannot choose by query at its current design. That classifies the 0 complete as mechanism (§4.2). The next unit builds lever (b), the question-keyed read, under its own sheet, and must pass Part B's probe before any language run.
- **Control fails:** an instrument fault, repaired before any reading.

### How claude-325 folds in

claude-325 trains the panel-shaped bare-answer corpus (claude-324), so it is a language-side test of the same question. Its rows are pre-registered (its sheet's outcome rows 0a–f):
- **(b), complete above 0 at a seed:** a learned query *can* make the choice when the scored target is the choice. Data was a blocker, and Part B ranks behind scaling that recipe.
- **(c), bare replies but 0 complete, or (e), no bare replies:** the stronger the case that the read is the blocker. Part B goes first.

Either way, Part A runs on its seals.

## Recommendation (a lab decision, §5.1; no bar, scope or cap changes)

1. **Start no further one-knob language run on this line** (corpus mix, ratio, copy keys, steps) until Part A is read and Part B has run.
2. **claude-327 is Part A plus Part B's sheet and run**, one unit, in this item's scope (`docs/labs/one-model-end-to-end/**`, `crates/uor-r4-float-train`).
3. **The roadmap item's next step** names this test in place of the next corpus lever.

Nothing here needs Casey: §3, the caps and the charter are unchanged.
