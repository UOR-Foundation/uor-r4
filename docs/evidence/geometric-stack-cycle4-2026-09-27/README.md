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
| `ablation/<arm>/report.json`, `attempt.json`, `manifest.json`, `model/config.json` | Each seed-1 ablation run (§7): `dot`, `norot`, `readsonly`. The full-stack reference is `pilot/geometric_lr0.004` |
| `ablation/launch.log` | Source commit, executable identity, learning rate and start/end times of the ablation stage |
| `sources/` | The launchers: pilot, post-pilot chain (with the selection rule), ablations and main comparison |
| `packet.json` | Per run: group, source commit, settings, threads, final metrics, cost and identities. Also the SHA-256 and size of every file in this directory |

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
- **Executables.** Release builds with `-C target-cpu=native`; any rebuild on another host differs in bytes.
  - The pilot used commit `dca1b790` (SHA-256 `fcf710ad…`).
  - The ablations used commit `b87acd63` (SHA-256 `05889874…`). Relative to `dca1b790` it changes only the example's sample decoding, its evaluate mode and early-stop checkpoint retention, not the model, optimizer or training loop.
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
- The ablations, claim to seal: `dot` 56.6 min and `norot` 57.5 min together, two threads each; then `readsonly` 46.5 min alone, four threads. Training alone took 2,739–3,386 s per run. Thread count changes speed and floating-point summation order, not the model or the update rule.
