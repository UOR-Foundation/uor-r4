# Prep sheet: the capacity sweep, trained as served (one-model-end-to-end, claude-259, 2 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [claude-259](../../loop/tasks/one-model-end-to-end/claude-259/ORDER.md). Path step 2, "a model that replies".

## Question

Does a larger trained-as-served model reply better on prompts phrased in ways TRAIN never used, and at what cost in served speed? The sealed (r-drop) run's DEV top-1 stops rising by step 2,048 (11,770 → 11,841 of 14,287 by step 16,384) while its train loss keeps falling. So more steps at width 256 buy nothing. The model's size has not been varied since claude-153 (28 Oct, before the turn memory).

## Arms

Every arm is claude-211's (r-drop) recipe, trained as served (claude-258), with a size change and a shorter run:

`train-in-space --data-root DATA --out OUT --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 8192 --eval-every 1024 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --seed S --width W --hamming-read H`

| Arm | `--width` | `--hamming-read` (heads) | Head width |
|---|---|---|---|
| (b) the base | 256 | 4 | 64 |
| (h) more heads | 256 | 8 | 32 |
| (w) wider | 512 | 8 | 64 |
| (W) widest the writer serves | 1,024 | 16 | 64 |

Seeds 1, 2 and 3; one pod per seed, the four arms as four processes on its GPU. The run is 8,192 steps, 4× the step where (r-drop)'s DEV reading stopped rising, read every 1,024 steps.

After training, each arm's step-8,192 checkpoint is scored with `train-in-space --data-root DATA --score-rows CKPT --out SCORED --samples 1536 --reply-tokens 64 --retain-rates 1 --panel p-w=PW --seed S`. That is greedy decoding with the memory on and with it out, on DEV's prompts and p-w's 1,202. Then `fbs-snapshot CKPT SNAPSHOT.json` is taken for the export.

## Readings

- **R1 (decides): exact replies on p-w, memory on.** A reply is exact when its first tokens equal the reference including its end mark: `jq '[.[] | select(.reply[0:(.reference|length)] == .reference)]'` on `samples-p-w-greedy-memory.json`. Each arm is paired with (b) at the same seed on the same 1,202 prompts: b = exact under the arm only, c = exact under (b) only.
- **R2 (decides ties): the opening.** `.reply[0] == .reference[0]` on p-w, paired the same way.
- **R3 (reported): DEV strata.** Openings, copies and the rest from `progress.jsonl`'s `dev_nearest`, at every eval. The step where each arm stops rising is reported.
- **R4 (reported): served.** On the laptop, each arm's seed-1 checkpoint is exported (`fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT OUT.uorlut`) and served by `uor-chat` on conv-a, conv-b and conv-c, the conversations of `m4-export-uorlut01-cli/claude-259`, with transcripts. Decode tok/s uses that unit's protocol: a 177-token run minus a 1-token run, the median of 5. The arm that ships (row 1 or 2) is served at all three seeds.

## Decision table

An arm **carries** at a seed when R1's b − c > 2·√(b + c) and b + c ≥ 10. It **carries** when it carries at every seed.

| Row | Outcome | What it changes |
|---|---|---|
| 1 | (w) or (W) carries | Size is a lever. The smallest wider arm that carries becomes the base. The next unit trains it to its plateau and ships it, with its tok/s cost stated. `scale-probe` gets this curve. |
| 2 | Only (h) carries | Read heads are the lever. 256 / 8 becomes the base at width 256's speed. |
| 3 | No arm carries, and every arm's R1 at every seed lies within (b)'s floor | Size is not the ceiling at this data. Width stays 256. The next lever is the reply itself (path step 4: copies and the reply's continuation, fixed with geometry), as claude-155 read it. This closes these four configurations only (CHARTER §4.1). |
| 4 | Signs disagree across seeds, or an arm carries at some seeds only | Keep and iterate: the arm with the larger mean b − c is the candidate. The next unit runs it at 3 more seeds before anything ships. |

R2 breaks a tie between two arms that both carry. Among those, the one with the smaller tok/s cost ships.

## Instrument checks

- **I-1 smoke (probe a):** 64 steps of every arm at the run's head on one pod: `run_long` runs, `config.json` holds `"train_as_served": true`, the checkpoint scores, and `fbs-snapshot` writes. On the laptop, `fbs-export` of each smoke snapshot loads in `uor-chat`. An arm that fails is dropped and recorded (the ORDER's stop rule).
- **I-2 throughput (probe b):** the smoke's `train_context_tokens_per_second`, with four processes on one GPU, projects each arm's wall time. An arm projected past 75 minutes has its steps cut to fit, and the cut is recorded.
- **I-3 parity:** for each arm's seed-1 export, `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64` on DEV's prompts, taken with `jq` from the arm's own `samples-memory.json`. At most 9 of 310 first tokens may disagree with the float file, claude-243's bar. Past that, the arm's R4 is an instrument fault, not a reading.
- **I-4 seeds pair:** every arm at a seed decodes the same p-w prompts in the same order (`position` and `prompt` equal index by index). Otherwise R1 is not computed.

## Data

- TRAIN, DEV and the tokenizer: `w2-inputs/2026-10-25/driver-data/`, pinned by the code (claude-211's sheet).
- p-w: `inputs/claude/heldout-panel-p-w1200-2026-10-31` (seed 2121, 1,202 prompts a seed), checked against its `SHA256SUMS`. claude-220 and claude-243 read it; no size was ever chosen on it.
- The binary: built on each pod from this unit's head, `--locked`, sha256 recorded before the first arm.

## Cost and stop rule

- **Cost:** 3 × RTX 5090 on Vast, at about $0.35–0.70/h each, for at most 3 h. That is at most $6.30, inside 4 pods and $8/h with what other labs hold at launch (`uor-pod status`).
- **Stop rule:**
  - A diverged loss (non-finite) or an export refusal mid-run is an instrument fault: that arm stops, and the fault is repaired in its own unit, never counted.
  - Outputs go to `results/claude/one-model-end-to-end-capacity-2026-11-02/` with `SHA256SUMS`, read back, before any pod goes down.
  - A watcher is armed for every terminal state, and pods go down after their save.
