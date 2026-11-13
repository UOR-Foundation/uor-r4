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
| The bar's panel | the same writer at `seed=23`, written on the laptop after the ORDER's commit (13 Nov 2026, 14:12:07Z) | recorded in the REPORT when written |

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

The snapshot job (one pod, after the served form is built): for each of the six checkpoints, `fbs-snapshot` at the snapshot pin writes the trunk snapshot with the rows beside it, and the served float readout with the rows set on the trunk. Nothing trains.

- **Rules check (§3).** As in claude-346's sheet: the `e8` address is the E8 root system the project's cells use; the read is 32 row lookups and adds a layer and token; no squashing function; no state is multiplied with a table. `token` is a learned table addressed by a fixed hash of the input token: it was the `e8` arm's control, and whether it may be served in a product is the owner's reading of §3.1.
- Not run: a no-address job (claude-358's three are the pairs); the `sign` arm; a fourth seed.

## Readings

**On the laptop, as integer artifacts,** exported as the saved models were (`fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`) and served with the counted table as merged by claude-365 (`raw-id-replies --mark-reply 1 --max-tokens 48 --whole-prompt`, `asks-floors.json`, `--place-speak 1 --place-serve-again 1`).

- **G:** everyday replies of the seed-23 panel (not a repeat row, not a recall or never-told question) that are the reference or a reply TRAIN gives the same user turn word for word, and end.
- **D** = G of an address model − G of the saved model of the same seed. **M** = the mean of D over an arm's three seeds.
- **The row check:** the share of the snapshot's scored rows on which the artifact picks the token its own float model picks, beside the saved models' share (lowest 0.9860).
- **Reported, no bar:** told values stated; never-told questions answered "not told"; replies that never end; the nine-turn conversation word for word; tokens a second with and without the rows.

**On the pod, in float (reported, no bar):** each new job's exact-reply count of 1,108 from `--score-rows`, its last `train_loss_mean` and `dev_loss`, and its `address.jsonl` census.

## Bars

The ORDER's, fixed before any served reply of an address model is read.

| Outcome, for an arm | Word |
|---|---|
| M ≥ +10, told values stated not down by more than 3 a seed, never-told "not told" not down by more than 2 a seed | adds |
| M < +3 | does not add at this configuration |
| anything else | not called |

- **Row 0.** An address artifact whose row check is more than 0.010 under 0.9860 is an export or reader fault, not a reading. A new job that fails I-n or I-a is that job's row 0.
- If both arms add, the line's models become the arm with the higher M; within 3 of each other, the arm with fewer replies that never end.

## What each outcome unlocks

| Outcome | Next |
|---|---|
| `e8` adds | The `e8` models become the line's models. Next: the state-keyed read on top of them. |
| only `token` adds | Nothing is served before the owner's §3.1 reading. The next arm addresses after a mix of the state, so the state's address carries what the token hash does. |
| an arm is not called | Its replies are read by kind to find which everyday turns moved; no fourth seed before that. |
| neither adds | The float gain on exact replies does not reach served replies. An instrument question first (§4.2): the row check and the cells' agreement with float. A negative on this configuration only (§4.1). |
| Row 0 | The export or the reader is repaired first; no reply of that artifact is read. |

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

## Minimum test power

On the seed-22 panel the three saved models' G is 137, 134 and 136 of 236: a spread of 3. The address models' spread is not known in served form; in float the two `e8` seeds differ by 35 exact everyday replies of 520. If serving keeps that spread, M over three seeds has a standard deviation near 10 on a panel of this size, and M ≥ +10 comes up by chance with no true gain about one time in six. That is enough to choose which models the line serves next, and not enough for a milestone claim.

## Steps

1. Launch pod a (`try-1`), watch it reach its chain, then pods b (`try-2`) and c (`try-3`). Each pod's watcher is armed before anything ships. `uor-pod status` is read first.
2. Each pod runs pre, then its one job; each stage is sealed to `results/claude/one-model-end-to-end-address-seeds-2026-11-13/run/STAGE/TRY` and read back. A pod goes down when its chain ends with every job 0 and every seal 0 and the four checks pass.
3. When the served form's tests pass, the snapshot pin is written here and one pod takes the six snapshots, sealed to `…/snap/MODEL/TRY`.
4. On the laptop: export, the row check, the seed-23 panel, the readings, the REPORT and the notebook row.

## Cost

Three RTX 5090 pods for about 1.3 hours each, each refused above $1.60/h, then one for under an hour: $3 to $8 at the rates claude-346 paid ($2.97 and $3.29 for longer lanes). Inside the caps of 4 pods and $8/h.

## Compute and stop rule

Vast through `scripts/pod/uor-pod`, sessions `claude-366-a`, `claude-366-b`, `claude-366-c`. A stall, spend or time notice is never a stop: a healthy run is extended and the notice is recorded. A failed pre stops its pod's job. A failed scoring pass is said and does not fail its job. No run writes into an existing root (L-002).

## FIT

Not yet read.

TEST FITNESS: PENDING - research, #3407 (claude-366), awaiting a fresh non-author read
