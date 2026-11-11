# Prep sheet: the place read trained (one-model-end-to-end, stage 3 of arm p, written by claude-351, 11 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Written in stage 2, which runs nothing. **Stage 3 runs only when the owner says run.**

The build is [claude-351's](../../loop/tasks/one-model-end-to-end/claude-351/ORDER.md). The routing and the counts are [claude-350's](../../loop/tasks/one-model-end-to-end/claude-350/REPORT.md). The recipe is arm r's, from the [reply-line sheet](./one-model-end-to-end-reply-line-2026-11-10.md).

## Question

Does a model trained with the place read say a stated value back when no memory line is written for it?

Today the served model says a stated value only because the chat program's sieve finds the value and writes a line for the model to copy (arm r, 4 of 4 on the script). The place read is meant to replace that line: the question finds where the value starts, a register holds the place, and the reply reads on from there.

## Base artifact

- **Recipe:** arm r's command, word for word, with four changes:
  - `--context 256` in place of 192, the window stage 1 counted in.
  - TRAIN without a line (`recall=off`), in place of the line written as the reply.
  - `--hamming-ask 1`: the type heads become ask heads, which the place read needs.
  - `--hamming-place KIT --hamming-place-pull 1`: the place read, taught from the kit, with the place pull added to the loss at weight 1.
- **The pull's weight is a first value, not a tuned one.** No earlier run used this pull. The pull is a mean of bits over a half head, between 0 and 1, and starts near 0.5. The run's `place.jsonl` holds its mean at every checkpoint, so a weight that is too small or too large shows in the first 2,048 steps. It is reported only: nothing is changed during a run. A mean pull still above 0.4 at the last checkpoint is named in the REPORT as "the pull did not train" and is the first suspect on row 4's find side.
- **The pin:** this unit's merge commit on `main`, named in the launch note. The trainer and `fbs-snapshot` are built at the pin on the pod. The exporter, the reader and the chat program run on the laptop at the pin.
- **The kit:** `train-places.json`, written on the laptop by `train-in-space --turn-census … --places-out`. It holds 34,460 places and 3,032 stop tokens and names TRAIN by its token hash. SHA-256 `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`. It goes to the store beside the run's inputs.
- **Compared with, not rerun:** arm r at seeds 1 to 3, sealed, with its line.

## Data split

Written by `m-world-recall-corpus`, world v2, protocol 2, as recorded in claude-349's and claude-350's reports. The hashes are of the files on the laptop.

| Set | Writer arguments | Tokens | tokens.u16 SHA-256 |
|---|---|---|---|
| TRAIN, no line | `conversations=60000 seed=13 recall=off` | 7,370,634 | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| TRAIN's twin, the kit's source | `conversations=60000 seed=13 recall=oracle` | not restated | `ca348c12be97724e4d5bae2a19b29200938b298724fd74f9f56abfb3c0809574` |
| Panel, no line | `conversations=1000 seed=14 recall=off` | 121,894 | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| Panel's twin, for the true places | `conversations=1000 seed=14 recall=oracle` | 131,619 | `12d7e9dfc0c34778556f34ee2c811b8388e90e84bf137bf0cfffdf00aed6d61d` |

- The panel has 3,586 replies: 562 recall replies, 79 abstentions and 2,945 general replies. It is held out by the writer's seed and shares the world and its templates with TRAIN. It tests unseen conversations of the same kind, not open text.
- The labels are the oracle's: the twin knows where each value was stated. They are used in training only. Nothing at serving reads a label.

## Arms

One pod, four jobs one after another, in this order.

| Job | What | Why |
|---|---|---|
| p-s2 | the recipe, seed 2 | the first reading |
| n-s2 | the same command without `--hamming-place` and its pull, seed 2 | the register off: the no-memory control (L-010). The ask heads then read as on `main`. |
| p-s1, p-s3 | the recipe, seeds 1 and 3 | whether p-s2's row repeats |

- **The true-place control is read as a split, not trained.** claude-350's ORDER named a third control, the register set to the true place, to tell a find failure from a read failure (L-007). Neither the trainer nor the serve has a switch that forces the register. The same question is read on the served model itself. This is a decision of this sheet, logged for Casey, who can reverse it.
- **The split.** For each wrong recall reply of an arm p artifact, the reader prints the register at the value-start row (the place it holds and the place it offers), and the panel's twin gives the true place. The 126 replies that claude-350's census lists as out of the find's reach are counted apart. Every other wrong reply falls in exactly one of four counts:

  | Count | The register at the value-start row | Names |
  |---|---|---|
  | a | not set | the arm count, or no find inside a margin |
  | b | set at another place | the find |
  | c | at the true place, and the row not open | the open count |
  | d | at the true place, and the row open | the readout's mix, or the step |

  The **find side** is a + b and the **read side** is c + d. The side that holds more than half of those wrong replies is the side row 4 names, and the larger count inside it is the part.
- **What the split cannot show.** It only observes. It cannot say what the readout would do on the rows where the find was wrong, so when both sides fail it shows the find's failure first. Only a forced register could answer that, and none is built.
- The split needs `raw-id-replies` to print `last_place`. This is a laptop-side change to the reader, outside the pin, checked by I-l before any reading.

Not run, and what that leaves unsaid: no forced-register control, no arm with the pull off, no other pull weight, no arm at context 192, and one control seed. A result cannot say which of the four changes did what, beyond what the control and the split give.

## Readings

**On the laptop, as the integer artifact,** exported as arm r's was (`fbs-export --raw-gates --raw-transition --run-config config.json`, then `--context 8256`) and served by `uor-chat` with the sieve's line off:

- **L, the product reading. It leads.** Arm r's fixed 12-line script: of its four direct questions, those whose reply holds the stated value as a whole word and no other stated value. Arm r with the sieve's line: 4 of 4.
- The 7-check conversation and the 18-turn script, word for word in the REPORT, with the register's place printed beside each reply. This is what the owner judges by.
- **R:** exact replies among the 562 recall replies of the no-line panel, through `raw-id-replies --mark-reply 1`. **G = (R of p-s2 − R of n-s2) / 562.**
- **The split** of p's wrong recall replies into find misses and read misses, as above, and the count of general replies and abstentions on which the register was set.
- **General replies:** exact replies among the panel's general replies, p against n, and the general questions of the 18-turn script. Reported; no bar.
- **Tokens per second:** `fbs-toks --read` on p-s2's artifact beside arm r's seed 2 artifact, prefill and decode, on the same machine in the same minute.

**On the pod, in float:** `train-in-space --score-rows` on the last checkpoint over the no-line panel at `--context 256`, greedy. It gives the float replies for the float against integer reading: rows on which the float reply and the artifact's reply differ, of the rows scored. Arm r's figure was 68, 46 and 31 of 1,141 at seeds 1, 2 and 3.

**From the run's `place.jsonl`,** one line a checkpoint: tokens armed and opening, the arm votes and open votes up and down, and the mean pull.

Not measured here: open text, reworded questions, conversations longer than the context.

## Bars

Fixed before the run. They route the next step; none stops the line of work.

- **3 of 4:** L at or above 3 with the sieve's line off.
- **0.10:** G at or above 0.10, which is 57 replies.

## What each outcome unlocks

Rows are checked from the top on p-s2 and n-s2, and the first that matches owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | any instrument check below fails (I-p, I-g, I-3, I-s, I-r, I-b, I-o, I-c, I-t, I-l, I-f or the seal's read-back), or the export fails | NOT_RUN, an instrument failure. No reading is classed by rows 1 to 4. | repair it and rerun under a new root, and under a new pin if the pinned code changed. A laptop-side check (I-o, I-l, I-f) is repaired and the readings retaken from the sealed checkpoint. |
| 1 | L at or above 3 and G at or above 0.10 | the served model says a stated value with no line written for it | the sieve's line comes off for this artifact. Then, in order: the 126 recall replies the find cannot reach, reworded questions, human text. |
| 2 | L at or above 3 and G under 0.10 | the laptop shows the value and the panel does not | the panel is the suspect: 20 recall replies are read by hand, with the register, before the cell is classed |
| 3 | L under 3 and G at or above 0.10 | the panel shows the value and the script does not | the script's replies are read step by step with the register printed. The script's wording against TRAIN's is the first suspect. |
| 4 | L under 3 and G under 0.10 | this configuration does not make the served model say the value | the split decides, by its fixed cut. The find side: the question rows did not learn their places (count b) or the register was never set (count a); the record's arm votes and mean pull say when they stopped. The read side: the open count (count c) or the readout's copy weight at open rows (count d) is read next. |

A row-4 reading closes this configuration only (CHARTER §4.1). It is classed as instrument or mechanism from the split and 20 replies read by hand before it is called.

**How the three seeds are read.** Seeds 1 and 3 are each classed by the same rows, with G taken against n-s2. The control has one seed, so their G crosses seeds; that is a limit of this sheet. The result is *replicated* if seeds 1 and 3 both share p-s2's row, *mixed* if one does, and *not replicated* if neither does.

- **Row 1, replicated:** the sieve's line comes off for these artifacts and row 1's next steps follow. The line does not come off on p-s2 alone.
- **Row 1, mixed or not replicated:** the line stays. The next unit reads the three seeds' split tables side by side to find which count differs, before any new arm.
- **Row 4, replicated:** the configuration is closed as the split classes it.
- **Row 4, mixed or not replicated:** the configuration is not closed. Each seed's split is reported, and the next unit is the count that differs.
- **Rows 2 and 3** are read on p-s2 by hand as the table says; seeds 1 and 3 are reported beside it.

**Prediction, written before the run and before the fitness review** (a calibration record, not a bar): R of p-s2 between 0.25 and 0.65 of 562; R of n-s2 under 0.10; L 2 to 4 of 4; row 1 or row 3. Stage 1 bounds R from above near 0.75: the find reaches 436 of 562 and D3 sets the true place on 426. Where I expect it to fail: numbers, which are one digit a token, so the step must match every digit; a register set on a general reply; and questions in the script's wording that TRAIN does not hold. What I cannot size: whether 16,384 steps are enough, since the arm count only rises once the question rows have moved, and the open count only once the arm count has.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on the pod.
- **I-g, the build's G2b and G3:** before anything trains, `cargo test --release -p uor-r4-float-train --features cuda --lib -- place` on the pod. It runs the three GPU tests written and type-checked in claude-351 and never yet run (the forward against the program; the place pull's gradient, descent and vote; the mix's gradient against a finite difference) and the six host tests. **A failed test, or a test build that does not compile, is row 0 and stops stage 3.**
- **I-3:** TRAIN's, the panel's and the kit's files match the hashes above before training. The trainer itself refuses a kit read from another corpus.
- **I-s:** a 64-step smoke of p and of n on the same inputs. p's folder must hold a `place.jsonl` with one line for the smoke's checkpoint and a finite mean pull in it; n's folder must hold no such file. This is the first time that file is written: the code that writes it cannot run on the laptop. A smoke that does not train, or a p smoke without the line, is row 0.
- **I-r, the rate, decided before the long run starts:** steps per second of p's smoke beside n's. The place read adds a host pass over two byte tensors at every forward. From p's smoke rate, 16,384 steps are projected. If the projection is over 2.5 hours, p's long run is not started: that is row 0, and the next step is the place step's host pass under `--profile`. A slowdown inside 2.5 hours is reported.
- **I-b, the bins road:** on p's smoke checkpoint, `fbs-snapshot`, `fbs-export` and `fbs-compare` agree as they do on `main`, and the artifact carries the place record and its class section. n's artifact carries neither.
- **I-o, off is main on a real export:** arm r's sealed seed 2 snapshot, exported with the pinned `fbs-export`, has the SHA-256 of arm r's sealed artifact.
- **I-c:** each run's `config.json` equals arm r's field by field apart from the four changes, and p's names the kit.
- **I-t:** each job ends exit 0 with `models/step-016384`, a snapshot and its `config.json`.
- **I-l, the reader's register:** the changed `raw-id-replies` has a test on a hand-built artifact with known places, which expects the held and the offered place at every row. On the panel it must find a true place for 562 of 562 recall replies, the census's own count. Either failing is row 0 for the split and for the scripts' printed register.
- **I-f, float against integer on the trained checkpoint:** the float replies of `--score-rows` and the artifact's replies are compared row by row. Arm r differed on 31 to 68 of 1,141 rows (0.027 to 0.060). **More than 0.10 of the rows scored differing is row 0:** an export defect, not a reading of the mechanism.
- **Seal:** each job's `run/` is sealed with a file list and SHA-256 sums, put in the store under `results/claude/one-model-end-to-end-place-read-2026-11-11/run/<job>/<try>/` and read back. A run is never written into an existing root (L-002).

## Risks the build leaves open

- **Every tensor line of the trainer's place code is unrun.** The laptop's backend refuses tensor work. I-g is where those lines first run.
- **Train and serve look back over different spans.** The trainer's find sees user turns inside its 256-token window. The serve's sees every user turn of the session. A find further back than 256 tokens is something no training step showed the model.
- **Training windows start mid-conversation,** so a register can start with no turn before it. Stage 1's counts were made the same way.
- **The pull's weight within a split step** is by windows, not by question rows.
- **In served-mode training** the place step reads the plain embedding table where the forward may read a bridged one, as the older found-copy pull does.
- **The pull and the find measure nearness differently.** The pull picks the nearer of the two ask heads by the distance on the head's learned half alone. The find ranks a head's candidates by the whole head word, the learned half and the fixed half together. So the head the pull trains toward a place is not always the head whose find would pick it. If the mean pull falls and the find's count b stays high, this is the first suspect.
- **Votes are cast at each forward.** A step split into several forwards runs its later forwards on the counts its earlier ones moved. The split is set by memory, not by the method, so two machines with different splits can differ in their counts.
- **A window whose corpus start the trainer cannot tell teaches nothing.** Such a window has no scored row. Their share of a run's windows is not counted here; the smoke's arm and open votes show whether windows are voting at all.

## Minimum test power

- **Noise.** Two figures bound it. Arm r's exact-reply share over its 399 matched pairs was 0.749, 0.759 and 0.782 at seeds 1, 2 and 3: a range of 0.033 between seeds of one recipe. And a share read on 562 replies has a binomial standard deviation of at most 0.021 (12 replies at a share of one half), so the difference of two such shares has at most 0.030.
- **The bar against it.** G's bar of 0.10 is 57 of 562 replies: three times the seed range and more than three standard deviations of the difference. A G between 0.03 and 0.10 is inside what one more seed could move, and is read as under the bar.
- G compares one seed with one control. Seeds 1 and 3 say whether p's readings repeat; they do not make G a three-seed figure.
- L has four questions and is not a powered reading: it is the product reading, shown word for word.

## Cost

One pod at no more than $1.60/h. Arm r's jobs took about 55 minutes each at context 192; context 256 makes each step longer by an amount the smoke will show. Expected: four jobs in 5 to 7 hours, under $12. Ceiling: 10 hours, $16. One of the four pods and at most $1.60/h of the $8/h cap.

## Compute and stop rule

- One pod is launched and watched until it reaches SSH and starts its job. The watcher is started before anything ships (L-004). It renews the lease and never cancels a healthy run; time, spend and stall are notices only.
- A long run starts only if I-r projects it inside 2.5 hours. A run that started and is still healthy past that is extended and the extension noted; it is never cancelled for time.
- The jobs seal `run/` on every exit, a signal included. A scoring pass is capped at 40 minutes and never fails its job.
- The pod goes down when its own four checks pass: the chain ended with its seal, the required files are in the seal's list, the store's sums read from the laptop equal the pod's, and no GPU process is left.

## FIT

Read by someone who did not write this sheet; the verdict is recorded below word for word.

**First read, 11 Nov: NOT FIT.** The reader's points, as the author summarises them (the reader's own words were not kept), and what changed for each:

| The reader's point | What changed in this sheet |
|---|---|
| Blocker: no noise figure, so the bar could not be told from seed noise | "Minimum test power" gives arm r's seed range and the binomial spread, and the bar stands at three times that range |
| Blocker: row 0 left out instrument checks, and float against integer had no threshold | row 0 names every check; I-f sets the threshold |
| Major: the true-place split was too coarse and had no cut | four counts with a fixed cut, the 126 unreachable replies counted apart, and what the split cannot show |
| Major: the change to the reader program was unchecked | I-l |
| Major: seeds that disagree had no next step | "How the three seeds are read" |
| Minor: I-r could not fail | it is decided before the long run, with a limit |
| Minor: no action for a bad pull weight | the base artifact's note on the pull's weight |

**Second read:** recorded below when it returns.
