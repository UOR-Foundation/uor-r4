# Prep sheet: the E8 address memory trained in the model, and the model trained on people's conversations (one-model-end-to-end, claude-346, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-346](../../loop/tasks/one-model-end-to-end/claude-346/ORDER.md), which built the part (PR #3913). The run goes ahead on the lab's own pre-run review inside the GPU caps (CHARTER §5.1), logged for Casey, who can reverse it.

## Question

1. **Does the E8 address memory, trained inside today's model on the model's own corpus, change what the model says?** The reading is the count of replies equal to the wanted reply on the 1,108-prompt scoring panel, against the same recipe without it.
2. **Does today's model learn anything from people's own conversations (OASST2), and does the address memory change that on held-out conversations?**

Why now. Today's model has about 1.1 million learned values, 1,048,576 of them the token table. The address memory adds 11,796,480, read by the state's own E8 cells. It was built on 9 Nov, and its tests passed on an RTX 5090 on 10 Nov (483 passed, 0 failed; record at `results/claude/one-model-end-to-end-address-2026-11-10/gpu-tests/try-1`). It has never been trained in the model. Every training run so far read one template generator's text. OASST2 has been in the store since 5 Nov and no run has read it.

## Base artifact

- **Code:** `38e209ee2ed733ae1a54732d06b614ab4d6da7cd`, PR #3913's head: the address memory of `cb9d32f99e1c81034aba128fc16f9fb732fefa9b` with main merged in, which brings claude-357's copy rule.
- **Recipe:** claude-358's p arm, unchanged: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1`. On the model's own corpus it adds `--hamming-place KIT --hamming-place-pull 1 --hamming-copy-longest 714,28`. On OASST2 it adds neither: that text has no marker.
- **Measured with this recipe** at `f694d162d9b04d1226fd17c0a81825f00e9da007` (claude-358, float scoring pass): replies equal to the wanted reply, of 1,108: **188** at seed 2, **188** at seed 1, **225** at seed 3. Last `train_loss_mean` and `dev_loss`: 0.5447 and 2.5630 at seed 2; 0.4981 and 2.5471 at seed 3.

## Data split

| Use | Store root | Tokens | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
|---|---|---:|---|---|
| TRAIN m (the model's own corpus, no memory line) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/corpus-noline/train` | | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| Scoring panel m (generator seed 14, 1,108 prompts) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/train` | | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| TRAIN o (OASST2, train) | `results/claude/real-text-corpus-2026-11-05/oasst2/train` | 3,773,419 | `8bdcabfb41284a54575ea2f730f532a896b29dc69250098957b46c6bcae40b90` | `a2b0b2a3ce8cc7491b8ced86eda487353ba4e70da0f9aa5a63161ca6802de7a1` |
| Scoring panel o (OASST2, held out) | `results/claude/real-text-corpus-2026-11-05/oasst2/heldout` | 214,758 | `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e` | `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |

The place kit is `results/claude/one-model-end-to-end-place-read-2026-11-11/kit/train-places.json` (`8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`). The tokenizer is the w2 driver data's (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`), with its `ft/dev` as the trainer's DEV. On the OASST2 jobs the trainer's DEV lines are still that DEV and are not a reading. OASST2 is conversations people wrote; no model-written text is in this run (§3.10).

## Arms

| Job | Pod | TRAIN | Address memory | Steps | Seed |
|---|---|---|---|---:|---:|
| `e8-s2` | a | m | `e8` | 16,384 | 2 |
| `e8-s3` | a | m | `e8` | 16,384 | 3 |
| `o-b` | b | o | none | 4,096 | 2 |
| `o-e8` | b | o | `e8` | 4,096 | 2 |
| `b-s2` | b | m | none | 16,384 | 2 |
| `token-s2` | b | m | `token` | 16,384 | 2 |

- `e8`: each layer adds, for each of the state's 32 blocks of 8 coordinates, the row of that block's nearest E8 root (240 cells a block; band 0.141; rows start at zero and step at `lr/32`).
- `token`: the same table, addressed by a seeded hash of the position's own input token. It keeps the table's size and takes the state out of the address.
- `b-s2` is claude-358's seed 2 job again at this pin. The no-address job at seed 3 is claude-358's own (225).
- **Rules check (§3).** The address is the E8 root system the project's cells already use. No squashing function: the band's `max(0, ·)` is a clamp at zero (§3.2). The read is 32 row lookups and adds a layer and token; no state is multiplied with a table (§3.3). §3.6 excludes "a dense model disguised as lookup tables": a row here is chosen by one block's nearest root and added whole, as the token table's row is chosen by a token.

## Readings

All from the float scoring pass on each job's last checkpoint (`--score-rows`, greedy, turn memory on, context 192, 48 reply tokens), and from the job's own logs. Counts are made by kit scripts that write logs.

- **R1, exact replies (m jobs):** of the 1,108 panel prompts, replies equal to the wanted reply up to the end token.
- **R2, loss (all jobs):** the last `train_loss_mean` and `dev_loss` of `progress.jsonl`.
- **R3, the address census (address jobs):** `address.jsonl` at each checkpoint: cells in use a block, the share of reads whose cell moved, the share inside the band.
- **R4, held-out next token (o jobs):** over the held-out panel's reply rows, the share whose nearest token is the true one (`rows-p-memory.json`).
- **R5, replies word for word (o jobs):** the first 20 held-out prompts with the wanted reply and the model's, read by a person; the share of replies that end inside 48 tokens.
- **R6, cost of the part:** training seconds for 64 steps, each arm.

## Bars

Question 1. `D2 = e8-s2 − b-s2` and `D3 = e8-s3 − 225` on R1, and `M` their mean.

| Outcome | Word |
|---|---|
| `M ≥ +30` and both `D > 0` | adds at this size (seeds 2 and 3) |
| `M ≤ −30` and both `D < 0` | hurts in this configuration |
| `\|M\| < 15`, or the two `D` differ in sign | no move at this size |
| anything else | not called: keep and iterate |

- **Control.** `T = token-s2 − b-s2`. If the word is "adds" and `D2 − T < 15`, the gain is the table's size and not the state's address.
- **Pin check.** If `b-s2` is outside 188 ± 20, the two pins do not train alike: `D3` is then not paired, the word is taken from `D2` alone at twice the thresholds (±60, ±30), and the REPORT says so.

Question 2 has no bar. It is a first reading, and nothing is claimed from it beyond its numbers and its replies.

## What each outcome unlocks

| Outcome | Next |
|---|---|
| adds, and the control holds | The address memory joins the base recipe. Its integer form and export (the exporter refuses it today), then seed 1, then the state-keyed read on top of it. |
| adds, and the control fails | A per-token table does the same: the state's address is not yet used. Next arm: the read after a mix of the state (lexlean-r4's v9 mixes with a Hadamard transform before it codes a block). |
| no move, and e8's train loss is below b's by 0.05 or more | The rows fit TRAIN and do not carry to the panel. Next: the `sign` arm and the band; a negative on this configuration only (§4.1). |
| no move, and e8's train loss is within 0.05 of b's | The rows are not learning. An instrument question first (§4.2): the `lr/32` step and R3's moved share. |
| hurts | This configuration is closed. Next arm: a smaller row step. |
| o-e8's R4 is above o-b's by 1 point or more | Real text joins TRAIN with the address memory, and the next run mixes OASST2 with Wikipedia. |
| R4 is within 1 point, and o-e8's train loss is far below o-b's | The rows memorise 3.8 million tokens. More text and fewer passes come first. |
| neither o job writes word-ordered text (R5) | Size and context are the limit: the scale ladder comes before more data. |

**Prediction, written before the run** (a calibration record, not a bar): `M` between −10 and +50; e8's train loss below b's by 0.1 or more; `T` within ±20; o-e8's R4 above o-b's by 1 to 6 points; neither o job writes sentences a person would call a reply.

## Instrument checks

The pre stage on each pod runs these and ends the pod's jobs if one fails.

- **I-p.** A clean checkout at the pin.
- **I-g.** Every test of `e8_address.rs` (the file's own count, 17 or more) and the trunk's cache test pass on the pod's GPU, with the copy rule's tests.
- **I-3.** Every input's SHA-256 is this sheet's.
- **I-s.** Five 64-step smokes train: no address, `e8` and `token` on m; no address and `e8` on o.
- **I-n.** The no-address smoke on m prints claude-358's eight `[memory]` lines and its step 0 line.
- **I-z.** Each address smoke's step 0 line equals its no-address twin's: the rows start at zero.
- **I-a.** An address job's last checkpoint holds `address_memory.safetensors`, and `address.jsonl` has a line at each checkpoint for six layers. A job with no arm writes neither.
- **I-r.** The `e8` smokes project at most 9,000 s for 16,384 steps and 3,000 s for 4,096. Past that no long job starts.
- **The scoring pass** runs at full size on both `e8` smokes' checkpoints before any long job depends on it.

Each long job repeats I-n (m jobs) and I-a on its own log and files; a failure is that job's row 0 and is said.

## Risks the build leaves open

- The rows train in float. They are not rounded to the served grid in this run, and the exporter refuses them, so the address models cannot be served on the laptop yet. `b-s2` and `o-b` can.
- Scoring reads at context 192 and training at 256, as in claude-358.
- The o jobs make about 4.4 passes over 3.8 million tokens, and the address memory has 11.8 million values: it can memorise. R4 and R5 are read on held-out conversations for that reason.
- GPU training is not bit-exact between runs or pins. The pin check covers the comparison with claude-358's seed 3.
- R5 is read by one person.

## Minimum test power

The three same-recipe seeds of claude-358 gave 188, 188 and 225: a standard deviation of about 21. Whether pairing by seed removes that swing is not known. If it does not, `M` has a standard deviation of about 21, so `M ≥ 30` with both differences positive comes up by chance in under one run in ten. That is enough to choose the next unit and not enough for a milestone claim: a result called "adds" is repeated at seed 1 before any claim rests on it.

## Steps

1. Launch pod a (`try-1`), then pod b (`try-2`) once pod a's chain holds its GPU. Each pod's watcher is armed before anything ships.
2. Each pod runs its pre stage, then its jobs in the Arms table's order. A failed long job does not stop the jobs after it.
3. Each stage is sealed to `results/claude/one-model-end-to-end-address-memory-2026-11-13/run/STAGE/TRY` and read back. The address rows of checkpoints before the last stay on the pod.
4. A pod goes down when its chain ends with every job 0 and every seal 0 and the four checks pass.
5. On the laptop: fetch each stage, count by kit script, read R5, write the REPORT and the notebook row in PR #3913.

## Cost

Two RTX 5090 pods, each refused above $1.60/h. Pod a: about 1.6 to 5.2 hours. Pod b: about 2 to 4.5 hours. At $0.60 to $0.90/h that is $2 to $9. Two pods are inside the caps of 4 pods and $8/h; `uor-pod status` is read at launch.

## Compute and stop rule

Vast through `scripts/pod/uor-pod`, sessions `claude-346-a` and `claude-346-b`. A stall, spend or time notice is never a stop: a healthy run is extended and the notice is recorded. A failed pre stops its pod's jobs. A failed scoring pass is said and does not fail its job. No run writes into an existing root (L-002).

## FIT

Pending: a fresh reviewer who did not write this sheet reads it before any pod goes up.

TEST FITNESS: PENDING - research, #3407, awaiting a fresh non-author read (claude-346)
