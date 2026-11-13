# Prep sheet: the address table's missing seeds, and the snapshots of its six models (one-model-end-to-end, claude-366, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-366](../../loop/tasks/one-model-end-to-end/claude-366/ORDER.md), PR #3956. The runs go ahead on this lab's own pre-run review (CHARTER §5.1), inside the caps, and are logged for the owner.

## Question

Served as integer artifacts on the laptop, do the models trained with the address table give more right everyday replies than the saved models of the same seed?

- **What exists.** claude-346 trained three address models at 16,384 steps: `e8-s2`, `e8-s3` and `token-s2`. In float they give more exact everyday replies than the same recipe without the table (of 520: 119 without; 142 and 177 for `e8`; 141 for `token`). Its bar on all 1,108 replies read "not called" (M +29.5 against +30), and its sheet names seed 1 of `e8` as the next run.
- **What is missing.** Each arm needs three seeds beside the three saved models, and no address model has a snapshot, so none can be exported.
- **This sheet covers two pod visits:** the three missing training jobs, and one short job that takes a snapshot of all six address checkpoints.

## Base artifact

- **Recipe:** claude-358's p arm with `--address-memory ARM`, word for word as in [claude-346's sheet](one-model-end-to-end-address-memory-2026-11-13.md) (its Base artifact and Arms).
- **Training pin:** `f941a33fd1a4eea751705d1cd6021af35452a4b7`, claude-346's pin, so the three new models are trained by the code that trained the three sealed ones.
- **Snapshot pin:** a pushed commit of this unit's branch at which the ORDER's Done-when 1 tests pass. It is written into this sheet before the snapshot pod goes up; that amendment changes no question, arm, reading or bar.
- **Sealed address models** (claude-346, float scoring pass, replies equal to the wanted reply of 1,108): `e8-s2` 218 (`run/e8-s2/try-3`), `e8-s3` 254 (`run/e8-s3/try-3`), `token-s2` 219 (`run/token-s2/try-4`), under `results/claude/one-model-end-to-end-address-memory-2026-11-13/`.
- **Two pins, and the check between them.** The saved p models were trained at claude-358's pin `f694d162d9b04d1226fd17c0a81825f00e9da007`; every address model is trained at `f941a33f…`. claude-346 ran the saved recipe with no address table at `f941a33f…` and seed 2 (`b-s2`, sealed at `run/b-s2/try-4`): 188 exact of 1,108 in float, claude-358's figure for p-s2. That check is one seed and in float. So `b-s2` gets a snapshot with the address checkpoints and is served beside p-s2 as a same-pin reading, with no bar.
- **Compared with, not rerun:** claude-358's three saved models, as exported then.

  | Model | Artifact, SHA-256 |
  |---|---|
  | p-s2 | `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680` |
  | p-s1 | `9a209883ff74cdeb6936fc11714d4dd4bb63947775609754c6a5f678e39df9bd` |
  | p-s3 | `9615060f4db98a4cc5b21220e454428b816cda1d47256cf85091963692e83d20` |

## Data split

| Use | Where | `tokens.u16` SHA-256 |
|---|---|---|
| TRAIN (the model's own corpus, no memory line) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/corpus-noline/train` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| The pod's scoring panel (generator seed 14, 1,108 prompts) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/train` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| The bar's panel | the same writer at `seed=23`, written on the laptop after the ORDER's commit (13 Nov 2026, 14:12:07Z) | committed to this unit's task folder (`panel-23.sha256`) and pushed before any served reply of any model is read, so "read once" can be checked |

- The place kit and the tokenizer are claude-346's (`8d28571a…`, `d36d3e87…`). The pre stage also reads OASST2 for two of its smokes; no reading here uses it.
- **No model has read the seed-23 panel,** and nothing is fitted on it. The counted table that every model is served with (claude-365) was counted on TRAIN.
- The panel shares its world and templates with TRAIN: it tests unseen conversations of the same kind, not open text. 229 of 236 everyday user turns of the seed-22 panel stand in TRAIN word for word.

## Arms

Training, one job a pod, each after its pod's pre stage:

| Job | Pod | Address | Steps | Seed |
|---|---|---|---:|---:|
| `e8-s1` | a | `e8` | 16,384 | 1 |
| `token-s1` | b | `token` | 16,384 | 1 |
| `token-s3` | c | `token` | 16,384 | 3 |

The snapshot job (one pod, after the served form is built): for each of the six address checkpoints, `fbs-snapshot` at the snapshot pin writes the trunk snapshot with the rows beside it, and the served float readout with the rows set on the trunk; `b-s2`, which has no rows, gets the same snapshot without them. Nothing trains.

- **Rules check (§3).** As in claude-346's sheet: the `e8` address is the E8 root system the project's cells use; the read is 32 row lookups and adds a layer and token; no squashing function; no state is multiplied with a table. `token` is a learned table addressed by a fixed hash of the input token: it was the `e8` arm's control, and whether it may be served in a product is the owner's reading of §3.1.
- Not run: a no-address job (claude-358's three are the pairs); the `sign` arm; a fourth seed.

## Readings

**On the laptop, as integer artifacts,** exported as the saved models were (`fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`) and served with the counted table as merged by claude-365 (`raw-id-replies --mark-reply 1 --max-tokens 48 --whole-prompt`, `asks-floors.json`, `--place-speak 1 --place-serve-again 1`).

- **G:** everyday replies of the seed-23 panel (not a repeat row, not a recall or never-told question) that are the reference or a reply TRAIN gives the same user turn word for word, and end.
- **D** = G of an address model − G of the saved model of the same seed. **M** = the mean of D over an arm's three seeds.
- **The row check:** the share of the snapshot's scored rows on which the artifact picks the token its own float model picks, beside the saved models' share (lowest 0.9860).
- **Reported, no bar:** told values stated; never-told questions answered "not told"; replies that never end; the nine-turn conversation word for word; tokens a second with and without the rows; `b-s2`'s G beside p-s2's (the same recipe at the two pins).
- **Where this runs, and the rule (CHARTER §3.6).** §3.6 allows serving the artifact on a client device and says "panel scoring, qualification and milestone evaluation of an exported artifact remain GPU research compute". These readings are the product served on the laptop with its replies read and counted by kind, as claude-356 to claude-365 did in merged units. They choose which models the line builds on next. They qualify nothing: no milestone claim, no qualification bar and no charter bar rests on them. Whether a count by kind of served replies is "panel scoring" under §3.6 is the owner's reading; it is named in the REPORT and put to the owner with a proposed wording. If the owner reads it so, the counts are redone on GPU compute before anything is built on them.

**On the pod, in float (reported, no bar):** each new job's exact-reply count of 1,108 from `--score-rows`, its last `train_loss_mean` and `dev_loss`, and its `address.jsonl` census.

## Bars

The ORDER's, fixed before any served reply of an address model is read.

| Outcome, for an arm | Word |
|---|---|
| M ≥ +10, told values stated not down by more than 3 a seed, never-told "not told" not down by more than 2 a seed | adds |
| M < +3 | does not add at this configuration |
| anything else | not called |

- **Row 0.** An address artifact whose row check is more than 0.010 under 0.9860 is an export or reader fault, not a reading. A new job that fails I-n or I-a is that job's row 0.
- **M needs all three seeds.** A failed job, a job at its row 0 and an artifact at Row 0 have no D. The job is rerun under a new try root, or the export repaired, before that arm's M is read; until then the arm has no word.
- If both arms add, the line's models become the arm with the higher M; within 3 of each other, the arm with fewer replies that never end.

## What each outcome unlocks

Keyed on the two arms' words together; one row applies.

| `e8` | `token` | Next |
|---|---|---|
| adds | does not add, or not called | The `e8` models become the line's models. Next: the state-keyed read on top of them. |
| adds | adds | The ORDER's rule names the arm. If `e8`: as the row above. If `token`: the `token` models are the line's research models, and nothing built on them is served as a product before the owner's §3.1 reading; the `e8` models, which also add, stay the set that may be served meanwhile. Both results go to the owner with that question. |
| does not add, or not called | adds | Nothing is served before the owner's §3.1 reading. The next arm addresses after a mix of the state, so the state's address carries what the token hash does. |
| not called | does not add, or not called | Each not-called arm's replies are read by kind to find which everyday turns moved; no fourth seed before that. |
| does not add | not called | As the row above, for `token`. |
| does not add | does not add | The float gain on exact replies does not reach served replies. An instrument question first (§4.2): the row check and the cells' agreement with float. A negative on this configuration only (§4.1). |

An arm with a Row 0 artifact has no word: the export or the reader is repaired first, and no reply of that artifact is read.

**Prediction, written before the runs** (a calibration record): `e8-s1` 200 to 260 exact of 1,108 in float; `token-s1` and `token-s3` 200 to 240; M for `e8` between +5 and +25; M for `token` between 0 and +15; `e8` models keep more replies that never end than the saved models.

## Instrument checks

- **Training pods:** claude-346's pre stage, unchanged (I-p, I-g, I-3, I-s, I-n, I-z, I-a, I-r and the scoring pass's arguments). Each long job repeats I-n and I-a on its own log and files.
- **Snapshot pod.** I-p: a clean checkout at the snapshot pin. I-g: the `e8_address` tests pass on the pod's GPU, with the test that the snapshot's rows are the trainer's own rounding. I-c: each checkpoint's files have the sums of its seal. I-d: a checkpoint's served readout taken with the rows set differs from one taken with the rows left out (the rows are read).
- **Laptop.** The integer cell of a block equals the float trainer's on shared vectors, walls and ties included; the read adds the row the address names and no other; an artifact with no rows reads byte for byte as today. Each fails with its rule set aside.

## Risks the build leaves open

- The float trunk works a block's cell out in 32-bit floats and the reader in integers. On a near tie they can name different cells; the row check measures what that costs.
- GPU training is not bit-exact between runs. The new seeds are trained at claude-346's pin for that reason.
- The seed-23 panel shares its templates with TRAIN, so G says the model answers turns of the kind it trained on, not that it answers open questions.
- The `token` arm fills its rows differently from `e8` and has no band, as claude-346's sheet says.
- G counts a reply right only when TRAIN gives it for that turn word for word; a right reply in other words counts as wrong for every model alike.
- **The counted table was set on the saved models.** claude-365 chose `asks-floors.json`'s floors while reading the saved models on the seed-22 panel. On told values and never-told questions, the two guard readings, that favours the saved models; an address model can lose a guard for that reason and not for its own.
- **The pins differ** (Base artifact). The same-pin check is one seed in float; `b-s2` beside p-s2 is the served check.

## Minimum test power

On the seed-22 panel the three saved models' G is 137, 134 and 136 of 236: a spread of 3. The address models' spread is not known in served form. In float the two `e8` seeds differ by 35 exact everyday replies of 520, a standard deviation near 25 from two values (one degree of freedom, so itself a rough figure). Scaled to 236 turns by the square root of the count, that is about 17 a seed, and about 10 for M over three seeds. Two errors follow:

- With no true gain, M ≥ +10 comes up about one time in six.
- With a true gain of +10, M < +3 comes up about one time in four, so "does not add" is wrong that often.

That is enough to choose which models the line builds on next, and not enough for a milestone claim.

## Steps

1. Launch pod a (`try-1`), watch it reach its chain, then pods b (`try-2`) and c (`try-3`). Each pod's watcher is armed before anything ships. `uor-pod status` is read first.
2. Each pod runs pre, then its one job; each stage is sealed to `results/claude/one-model-end-to-end-address-seeds-2026-11-13/run/STAGE/TRY` and read back. A pod goes down when its chain ends with every job 0 and every seal 0 and the four checks pass.
3. When the served form's tests pass, the snapshot pin and the snapshot job's kit are written here and take their own fresh read; then one pod takes the seven snapshots, sealed to `…/snap/MODEL/TRY`.
4. On the laptop: export and the row check; the seed-23 panel is written and its SHA-256 committed and pushed; then the readings, the REPORT and the notebook row.

## Cost

Three RTX 5090 pods, each refused above $1.60/h. A pod runs pre, about 57 minutes of training at the measured 0.21 s a step, and a scoring pass of up to 40 minutes: 1.7 to 2 hours. Then one pod for under an hour. $4 to $11 in all; claude-346 paid $2.97 and $3.29 for its two lanes. Three pods at once are at most $4.80/h, inside the caps of 4 pods and $8/h.

## Compute and stop rule

Vast through `scripts/pod/uor-pod`, sessions `claude-366-a`, `claude-366-b`, `claude-366-c`. A stall, spend or time notice is never a stop: a healthy run is extended and the notice is recorded. A failed pre stops its pod's job. A failed scoring pass is said and does not fail its job. No run writes into an existing root (L-002).

## FIT

A fresh reviewer (claude, fresh context, 13 Nov 2026) read this sheet at `ed38134a6da31b243826f376d67c28087eeecdf0` beside the ORDER, the kit's job and chain scripts and CHARTER §3. It ran nothing. Verdict: **fit with changes.** It checked the arms against the kit and the bars against the ORDER line by line, and both match: the three lanes run `pre` then their one job, the job refuses any pin but `f941a33f…`, and the seeds, steps and address arms are the table's.

**This FIT covers the three training lanes.** The snapshot job's pin and kit do not exist yet; they take their own fresh read before that pod goes up (Steps, 3).

Its nine findings, all taken in this revision:

1. *Medium.* The outcome table's rows overlapped and left a gap. It is now keyed on the two arms' words together, "neither adds" is both "does not add", and the case where both add and `token` is higher is written.
2. *Medium.* The saved models and the address models were trained at different pins, unsaid. Base artifact now gives both pins and claude-346's check (188 at seed 2), and `b-s2` is served beside p-s2.
3. *Medium.* §3.6 names panel scoring of an exported artifact as GPU research compute, and the sheet cited nothing for reading served replies on the laptop. Readings now gives the rule's words, what these counts are used for, the merged precedent, and that the reading of §3.6 is the owner's.
4. *Medium.* No rule for an arm with fewer than three readable seeds. Bars: M needs all three.
5. *Medium.* The snapshot pod's kit was not in what was read. This FIT is scoped to training.
6. *Low.* The power paragraph did not say how the float spread was scaled, and gave one error of two. Both are written.
7. *Low.* The seed-23 panel's SHA-256 was left for the REPORT. It is committed and pushed before any served reply is read.
8. *Low.* The counted table's floors were set while reading the saved models. Named in Risks.
9. *Low.* 1.3 hours a pod was low. Cost now counts pre and the scoring pass; the kit's stale header is corrected.

TEST FITNESS: FIT - research, #3407 (claude-366), a fresh non-author read at ed38134a of the three training lanes, every finding taken; the snapshot job takes its own read before its pod goes up
