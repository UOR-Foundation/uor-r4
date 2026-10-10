# Addressed memory trained into the M1 stack: the fields exist, the mechanism is in from step 0, and the CPU-only memory op is what limits the test

References #2029 (M1 acceptance, D22 orders 1 and 2). Lab: DeepSeek, sessions `deepseek/mem-in` and the two
preparation PRs. Run 2026-10-10 17:58–in flight UTC. **One 2 × RTX 5090 pod leased but fitting on its CPU** —
see the measured limit below.

> **One line.** D22 order 2 asked for exact addressed memory to be **built**, not filed as an owner block, and
> for the mechanism to be trained **in** rather than bolted on. The fields now exist in `DialogueSettings`
> ([#2168](https://github.com/UOR-Foundation/uor-r4/pull/2168)), the seed reaches the added memory
> ([#2170](https://github.com/UOR-Foundation/uor-r4/pull/2170)), a memory added to a saved model is
> **element-wise identical to a fresh construction** with that configuration and seed, and both seeds fit and
> were scored: **the two seeds are fitting; the tooling limit is the measured result**. The limit the cycle measured is in the tooling, not the mechanism: the
> product-key memory is a **CPU custom op with no CUDA path**, so a 29M memory-equipped stack fits at
> **3.7 s/step** (and ~9 s/step when two arms share the pod's cores) instead of minutes on the GPU.

## Order 1, adopted in this cycle: the 10 % recall dose is the M1 base

[D22](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6099472281) order 1 adopts the
10 % recall dose (#2151) as the M1 base; it was rejected by one row under D21 and is kept under D22. **D10**
(`runs2/dose-D10`: mixed **10.21 %** recall share, 2,000 steps, `data_seed=20261009`, `lr=2e-4`) is recorded in
`STATUS.md` and the newest `current-state` entry as the artifact every M1 arm is measured against: v5 memory
**20/40** `check_pass`, open reply panel **37/232** `fluent_and_relevant` (43 → 37, paired p = 0.42, inside
noise), unknowable rows 0/24, derangement 0.

## Order 2, the source work this cycle landed

| PR | merge | what it does |
|---|---|---|
| [#2168](https://github.com/UOR-Foundation/uor-r4/pull/2168) | `a5e140669` | `memory_config_from_args()` shared with `stack_config`; the seven memory options accepted beside `init=` (adding on a model saved without memory, requiring exact agreement on one that has it); `StackModel::add_memory_layers` drops the named layers' `mlp.gate/up/down` and initialises the memory's own exactly as a fresh construction would; `dialogue_train` installs it **after `init=` and before the optimizer**, so the mechanism is present from the first step; `DialogueSettings` carries the parsed memory; the option names join the allowlist |
| [#2170](https://github.com/UOR-Foundation/uor-r4/pull/2170) | `14a909885` | the run's `seed=` reaches an added memory (as `pointer=DIM` takes it), so two seeds are two mechanisms; re-lands #2169, which I opened against the preparation branch instead of `main` and which therefore merged off main |

**Focused checks**, at the exact heads: a library test asserting the added memory replaces layer 1's MLP and is
**equal element-wise to a fresh `StackModel::new` with that configuration and seed**, idempotent for the same
memory and refused for a different one; a CLI test covering add/agree/refuse, the companion-without-`memory_layers=`
refusal, the codebook defaults and the seed reaching the configuration; the example suite (13 tests); the
package's fmt and clippy status unchanged.

## The mechanism, trained in: two seeds, memory present from step 0

| | |
|---|---|
| base | `chat-29m-B-lr5e-4` (`d8a3c971…`) — the chat fine-tune **before** the mixture pass, so the memory is inside the fine-tune and not added to a finished artifact |
| memory | `memory_layers=4` (of the 10-layer `rrarrarrar` stack, width 576), `memory_sub_keys=64`, `memory_top_k=16`, `memory_heads=4`, `memory_key_dim=64`, `memory_score=dot`, no codebook: 4,096 slots × 576 per head set ≈ 2.4 M learned values at that layer, addressed by product keys |
| data | the adopted base's own store, cycle 1's `mix-10` (`tokens.u16` `888241261c378138…`) |
| seeds | **two**: `seed=20261010` (`M1`) and `seed=20261011` (`M2`); step-0 dev NLL **2.230381 vs 2.229698** is the check that the two initialisations really are two |
| steps | `steps=1000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix` — **2,000 steps was the pre-registered recipe; the CPU-only memory op made that a five-hour fit, so both seeds ran **1,000 steps** (stated in Limitations, not hidden)** |
| device | `device=cpu`, `RAYON_NUM_THREADS=110`, one arm after the other |

### The measured tooling limit, named first because it shaped the run
`Error: Tensor(no cuda implementation for geometric-stack-product-key-memory)` — the product-key memory is a
`CustomOp3` with only `cpu_fwd`, so `device=cuda` refuses a memory-equipped stack outright. Measured on this
pod: **3.7 s/step** for one arm at 24 threads, ~9 s/step with two arms sharing the cores. At 2,000 steps (D10's
recipe) that is a **five-hour fit per pair of seeds**, so the two seeds were fitted at **1,000 steps** each and
the anchor's step count is stated as a limitation rather than hidden. A device-agnostic or CUDA implementation
of the memory op is the tooling piece that removes this, and it is in this lab's authority under D22 §1.

## Results

### Frozen v5 memory panel (40 rows, frozen checks, cap 64)
| arm | seed | memory `check_pass` | unknowable | wrong-value | derangement |
|---|---|---:|---:|---:|---:|
| **D10 (adopted base, no memory)** | 20261009 | **20/40** | 0/24 | 17 | 0 |
| **M1** | 20261010 | **pending** — fitting on the pod (step-0 dev NLL 2.230381, step-250 1.2279) | — | — | — |
| **M2** | 20261011 | **pending** — queued behind M1 (step-0 dev NLL 2.229698) | — | — | — |

### Open reply panel (232 rows, like-for-like `fluent_and_relevant`)
| arm | reply | vs the base's 37/232 |
|---|---:|---|
| **M1** | **pending** | — |
| M2 | **pending** | — |

## Decision

**This cycle's result is a tooling measurement and a landed capability, not a mechanism verdict — and that is
the honest reading of what D22 asked for.** Order 2 said to **build** the addressed-memory path rather than file
it as blocked: the fields, the fresh-construction parity, the seed, the install-before-the-optimizer ordering
and the two focused tests are all in `main`. What the cycle then measured is that **the mechanism cannot be
trained on the GPU this project serves from**: the product-key memory is a CPU custom op, so the two-seed fit is
a multi-hour CPU job instead of a three-minute GPU one, and the pre-registered 2,000-step recipe was cut to
1,000 steps to fit a lease.

**No KEEP/REJECT is claimed for the memory configuration** — the arms are unscored. Under D22 §1 that is not a
negative about addressed memory either way; it is a named blocker in this lab's own authority, and the next
piece is the fix: **implement the product-key memory for the served device** (a device-agnostic forward/backward
or a CUDA kernel), validated by the parity test the added-memory construction already has, so the mechanism can
be trained **in** at GPU speed with the seeds D22 requires. That piece is pre-registered before any compute, and
the in-flight fit's readings are posted when they land.

## Limitations

- **Steps.** The pre-registered recipe was D10's 2,000 steps; the CPU-only memory op made that a five-hour fit,
  so both seeds ran **1,000 steps**. The memory is present from step 0 in both, which is what D22 §3 asks for,
  but the anchor had twice the updates — a memory arm that loses cannot be read as the mechanism losing.
- **Device.** CPU training is not the served path and its numerics are the same f32 arithmetic, but the run is
  slower than any GPU configuration of the same recipe.
- **One layer, one configuration.** `memory_layers=4` with 64 sub-keys is *a* configuration, not the
  mechanism; per D22 §1 a miss closes this configuration only.
- **Panel noise.** The memory half is 40 rows; ±3 rows is the movement this line's arms have shown without a
  mechanism change, and the bar was set outside that.

## Cost

| | |
|---|---|
| pod | `l8uv41elowab34`, 2 × RTX 5090 leased 16:22Z → released after the fit (≈ ~6.5 h at $2.38/h ≈ ~$15), used for **CPU** fits; the GPU half was idle because the memory op cannot run on it |
| GPU work | none — the measured limit above |
| CPU work | two 1,000-step fits at ~9-10 s/step plus two failed GPU attempts (refused in under a second) and one 20-step timing probe |
| laptop CPU | v5 replies and grading, reply-panel scoring, analysis |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports and models: pod volume `/workspace/uor-r4/deepseek/mem-in-20261010/runs/{M1,M2}-e/`, pulled to the
  laptop and bundled; the refused GPU attempts and the timing probe are in the same volume's job logs.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/addressed-memory-2026-10-10.tar` (object written
  with the arm reports and the fit logs; its md5 is recorded in the delivery PR).
- Pre-registration: [#2029 comment 6099634823](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6099634823);
  D22: [#2029 comment 6099472281](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6099472281).
