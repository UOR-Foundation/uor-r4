# Cycle 4 run packet: the geometric stack against the transformer control

Complete run records behind [the cycle-4 note](../../integration/geometric-stack-cycle4-2026-09-27.md). **Measured**,
development runs, one seed per arm. Nothing here is a final holdout or a language qualification.

## Contents

| Path | What |
|---|---|
| `pilot/<arm>_lr<rate>/report.json` | Each learning-rate pilot run (§5), with the fields listed below |
| `pilot/<arm>_lr<rate>/attempt.json`, `manifest.json` | The report root's claim record and seal manifest |
| `pilot/<arm>_lr<rate>/model/config.json` | The model configuration; the weights stay outside (below) |
| `pilot/launch.log`, `pilot/chain.log` | Launch times, the executable identity and the learning rates the rule chose |
| `sources/` | The launchers: pilot, post-pilot chain (with the selection rule), ablations and main comparison |
| `packet.json` | Per run: settings, final metrics, cost and identities. Also the SHA-256 and size of every file in this directory |

Each `report.json` records:
- settings and the complete model configuration;
- parameter count, completed updates and target visits;
- every periodic evaluation, with training loss, development NLL, bits per byte, gradient norm, training seconds and throughput;
- the final evaluation, with 131,072 targets;
- three prompts' greedy and sampled continuations, as token ids and decoded text;
- the size and SHA-256 of every input;
- the executable's SHA-256 and the model's SHA-256.

## Kept outside the repository

- **Model weights.** Each run's `model/model.safetensors` is 28.6 MB. They stay in the lab sandbox's scratch directory, which is not durable. `packet.json` lists each file's size and SHA-256, which the reports also record.
- **Executables.** The runs used a release build of commit `dca1b790` with `-C target-cpu=native` (SHA-256 `fcf710ad…`). Any rebuild on another host differs in bytes.
- **Corpora.** The repository code split is cycle 3's (`b12707b0…` training, `3f7c50ef…` development), in the [cycle-3 packet](../native-lorentz-packet-2026-09-26/README.md). The registry corpus for the main comparison is `af93ca73…` (29,039,409 tokens), built with `geometric-stack corpus` and encoded with `geometric-stack encode`.

## Cost

- Six runs, two at a time, with two threads each, on the 4-core sandbox.
- Complete elapsed time per round, including periodic and final evaluation, checkpointing and sampling:

  | Round | Elapsed |
  |---|---:|
  | 1 | 67.5 min |
  | 2 | 59.9 min |
  | 3 | 73.8 min |
  | Total | 3 h 21 min |

- Training alone took 3,353–4,350 s per run.
- Rounds 1 and 2 shared the machine with builds and tests of this branch and of #1410.
