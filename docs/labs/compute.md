# Shared GPU pods — one tool, one cadence

Every lab (Claude, Codex/GPT, OpenCode/DeepSeek and any lab that joins later)
uses the same tool, `scripts/pod/uor-pod`, and the same cadence to see, lease,
create, share and delete the owner's Runpod GPU pods. The goals: no pod idles,
no lab is surprised by another's pod, and any lab can take over a pod when
another lab runs out of tokens. History is on the
[Compute board #1750](https://github.com/UOR-Foundation/uor-r4/issues/1750);
the source of truth is the laptop state directory below.

## The cadence

1. **Look first.** `scripts/pod/uor-pod status` lists every pod (GPUs, $/h,
   uptime), its leases, per-GPU utilisation and memory over SSH, the reaper,
   the live RTX 5090/4090 stock in each datacenter, and flags any running pod
   with **NO LIVE LEASE**. `status --du` adds the shared volume's usage.
2. **Share before you create.** If a running pod has free GPUs of the type you
   need, lease them: `uor-pod lease POD --lab L --gpus 0,1 --purpose "…" --hours H [--card URL]`.
   `up` refuses (exit 3) and prints the free slot when one exists.
3. **Create only through `up`** when no slot is free:
   `uor-pod up --lab L --purpose "…" --hours H [--card URL]` (defaults: 2 × RTX 5090).
   It applies the GPU policy, placement order and caps below, waits for SSH,
   seeds a non-canonical volume, runs the bootstrap (toolchain, cached
   binaries + parity, reaper) and writes your lease.
4. **Renew every ≤ 30 minutes while working:** `uor-pod renew POD --lab L [--hours H]`.
   A lease is a promise that someone is actively using the GPUs; expiry is
   automatic release. Renewal is not posted to the board (the ledger has it).
5. **One job per GPU.** `uor-pod run POD --lab L --gpu K -- CMD…` starts a
   detached job under `flock /root/gpuK.lock` (a second job on the same GPU
   queues behind it), with the bootstrap environment, logging to
   `/workspace/uor-r4/jobs/<lab>/<UTC>-gpuK.log`, ending with an `# exit=N` line.
   `--gpu 0,1` holds both locks for a data-parallel run.
6. **Everything durable goes to `/workspace`** (layout below). The container
   disk (`/root`) dies with the pod.
7. **Release when done:** `uor-pod release POD --lab L`, then, if nobody else
   holds a lease, `uor-pod down POD --lab L`. Do not leave a pod for the reaper
   to find; it is the safety net, not the plan.
8. **Anyone may reap:** `uor-pod reap` deletes running pods that have no
   unexpired lease and whose GPUs have been idle ≥ 20 minutes (no compute
   process, ≤ 2 % utilisation, no held GPU lock). It never touches a pod with a
   live lease and never deletes a pod it cannot probe. Run it at the start of
   every GPU session.

`uor-pod log [-n N]` tails the ledger; `uor-pod ssh POD [CMD]` opens a shell;
`uor-pod gpus` prints the policy table and live stock.

## GPU policy

Measured for **our** code (FP32 CUDA-core trainer with TF32 matmul; the
bf16/tensor-core rewrite is in progress). Owner decision, 5 October 2026.

| Key | GPU | VRAM | `CUDA_COMPUTE_CAP` | Secure $/GPU/h | Tier | Use |
| --- | --- | --- | --- | --- | --- | --- |
| `5090` | RTX 5090 | 32 GB | 120 | 0.99 | **default** | The only automatic choice. 96M fine-tune step ≈ 0.09–0.10 s/GPU (2 × 5090: 2,000 steps, batch 16, ctx 384 in ≈ 3.5 min); parity 22/22. ≈ 1.7–2 × a 4090 and ≈ 25 % cheaper per job. |
| `4090` | RTX 4090 | 24 GB | 89 | 0.74 | explicit | Only with `--gpu 4090` (≈ 0.17–0.20 s/step). |
| `l40s`, `6000ada` | L40S / RTX 6000 Ada | 48 GB | 89 | 1.09 / 0.84 | explicit | Only when a job needs more than 32 GB. |
| `a100`, `h100`, `h200`, `b200`, … | A100/H100/H200/B200 | 80+ GB | 80/90/100 | 1.59+ | **forbidden** | Measured A100 = 0.6–0.75 × a 4090 for this trainer. Never automatic; refused without `--owner-approved`. Revisit after the bf16 Phase 2 parity gate. |

Default shape: **2 GPUs per pod** (`data_parallel=2`, or two seeds in
parallel); 1 GPU for evaluation-only or grading jobs (`--count 1`).

## Placement

`up` tries the requested type (5090 unless `--gpu` says otherwise) on secure
cloud in the network-volume datacenters, in this order:

| Order | Datacenter | Volume | Role |
| --- | --- | --- | --- |
| 1 | EUR-NO-1 | `lmd1pfah3y` `uor-shared-EUR-NO-1` (200 GB) | **canonical store** |
| 2 | EU-RO-1 | `uor-shared-EU-RO-1` (100 GB, created on first placement) | non-canonical |
| 3 | EUR-IS-1 | `uor-shared-EUR-IS-1` (100 GB, created on first placement) | non-canonical |

These are the only network-volume datacenters that ever list 5090 stock
(EU-CZ-1 and EUR-IS-2 have 5090s but no network volumes). The two
non-canonical volumes are owner-approved and created lazily, only when a pod
is first placed there (posted to the board).

If none of the three has stock, `up` **does not fall back silently**: it prints
the stock table and stops. Then:

* `--wait [--wait-hours H]` retries every 5 minutes for up to H hours
  (default 2), posting "waiting" and "gave up" to the board;
* `--gpu 4090` is the explicit alternative (same datacenter order);
* `--allow-off-volume` places the pod in any stocked datacenter with a local
  150 GB `/workspace` and no network volume (seeded like a non-canonical one).

### Non-canonical and off-volume pods

A non-canonical `/workspace` starts empty. `up` seeds it with the **hot set**
([`scripts/pod/hot-set.txt`](../../scripts/pod/hot-set.txt)) before bootstrap,
copying from a running pod that mounts the canonical volume, through the
laptop (`uor-pod seed POD`, or `UOR_POD_HOT_MIRROR=DIR` for a laptop copy of
the same paths). If no canonical pod is running, seeding is skipped with a
warning and bootstrap installs the toolchain and builds cold.

| Hot-set path (under `/workspace`) | What | Size (2026-10-05) |
| --- | --- | --- |
| `toolchain/rustup`, `toolchain/cargo` | Rust 1.97.1 + cargo registry cache | ~1.5 GB |
| `bin/<sha>-sm<cap>/` | cached release binaries for the commit being bootstrapped | ~0.1 GB |
| `uor-r4/data/` | tokenizer, fine-tune stores (`ft-*.tar`), the 100M base being fine-tuned (`geo-100m-*`), paraphrases, step5 inputs (corpora, sieve compiler/trunk, open panels), `MD5SUMS` | 2.6 GB |
| `toolchain/ollama`, `ollama/` | Ollama binary and qwen2.5:7b (only with `--with-ollama`) | ~5 GB |

Results produced off-canonical must be **archived back before `down`**:
`uor-pod push POD uor-r4/runs/… uor-r4/evals/…` (relays to the canonical
volume through the laptop) or fetched to the laptop and recorded in the task
issue. `down` refuses a non-canonical or volume-less pod until you pass
`--confirm-archived`. `uor-pod pull POD PATH…` copies extra inputs in.

## Layout of `/workspace` (canonical volume)

| Path | Contents | Owner |
| --- | --- | --- |
| `uor-r4/data/` | shared inputs (tar + `MD5SUMS`) | all labs |
| `uor-r4/runs/`, `uor-r4/evals/` | training runs and evaluations, one sealed report root each | the run's lab |
| `uor-r4/step*/` | per-step bundles (`MD5SUMS`, `.done` markers) | the step's lab |
| `uor-r4/jobs/<lab>/` | `uor-pod run` logs | the lab |
| `uor-r4/pods/reaper.log` | pod-side reaper log (all pods) | tool |
| `bin/<sha>-sm<cap>/` | release binaries, `BUILD.json`, `parity.log`, `SHA256SUMS` | tool |
| `toolchain/` | `rustup/`, `cargo/` (RUSTUP_HOME/CARGO_HOME), `ollama/` | tool |
| `ollama/` | Ollama models (`OLLAMA_MODELS`) | tool |
| `codex-uor-r4-20261004/`, `pearlfortune/` | preserved lab material | its lab |

Never delete another lab's material on the volume; it is the shared archive.

## Standard template and bootstrap

* **Template** `uor-r4-cuda128` (`h15vb984sw`): the official
  `runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404` image (CUDA 12.8.1
  devel, `nvcc` present), 60 GB container disk, port 22/tcp only, volume at
  `/workspace`. SSH keys come from the account (`PUBLIC_KEY`); Jupyter is off
  (the image starts it only when `JUPYTER_PASSWORD` is set, and no HTTP port
  is exposed; the image's internal nginx listens on a few ports that are not
  mapped publicly). It replaces `vishva123/nvidia-cuda-13.2-devel-runpod`, which
  ignored `PUBLIC_KEY` and ran an unauthenticated Jupyter.
* **Bootstrap** (`scripts/pod/uor-pod-bootstrap.sh`, run by `up` or
  `uor-pod bootstrap POD`): checks `nvcc` 12.8 (apt install only as a
  fallback), maps the GPU to `CUDA_COMPUTE_CAP` (5090 → 120, 4090/L40S/6000
  Ada → 89, A100 → 80, H100/H200 → 90), installs Rust 1.97.1 once per volume
  into `/workspace/toolchain`, and builds only when
  `/workspace/bin/<sha>-sm<cap>/BUILD.json` is missing:

  ```
  CUDA_COMPUTE_CAP=<cap> cargo build --release -p uor-r4-training --features cuda \
    --example geometric-stack --example m-world --example mqar-bench \
    --bin chat-grade --bin dialogue-recall-corpus --bin mix-chat-corpus
  UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training --features cuda \
    --test cuda_stack_ops_parity -- --test-threads=1
  ```

  The source is a shallow fetch of the public repository at the exact commit
  (`--ref`, default `main`), built on the container disk; the binaries,
  `parity.log` and `BUILD.json` (commit, cap, rustc, nvcc, timings, parity
  result) are moved into the cache atomically. `--with-ollama` adds the
  judge. Shells and `uor-pod run` jobs source `/root/.uor-pod-env`
  (`UOR_BIN`, `CUDA_COMPUTE_CAP`, PATH, LD_LIBRARY_PATH, cargo, Ollama).
* **Pod-side reaper** (`/root/uor-reaper.sh`, started by bootstrap only on
  pods made by `up`): every minute, if no `/root/leases/*.json` is unexpired
  and the GPUs have been idle ≥ 20 minutes, it deletes its own pod through the
  Runpod API with the pod's own `RUNPOD_API_KEY` (read from `/proc/1/environ`
  at use, passed to curl on stdin, never logged). Log:
  `/workspace/uor-r4/pods/reaper.log`. Leases reach the pod as
  `/root/leases/<lab>.json` on every lease/renew; **if a renew cannot reach the
  pod the reaper will not see it** — the tool warns, retry.

## Leases, state and the board

* A lease is `{lab, pod, gpus, purpose, card, started, expires, renewed, hours}`
  in `~/.local/share/uor-r4/compute/leases/<pod>/<lab>.json` on the laptop all
  labs share. Writes take one global `flock` and replace files by rename.
  `ledger.jsonl` (append-only) records every event; `pods.json` caches what the
  API does not say (GPU type, datacenter, volume, whether `up` made it).
* `up`, `down`, `lease`, `release`, `reap`, volume creation and `up --wait`
  outcomes post one line to [#1750](https://github.com/UOR-Foundation/uor-r4/issues/1750).
* `UOR_POD_DRY_RUN=1` prints every mutation instead of doing it;
  `scripts/pod/tests/uor-pod-dryrun.sh` exercises the logic against fake
  `runpodctl`/`ssh`/`gh`.

## Caps and what needs the owner

Defaults: **≤ 2 running pods and ≤ $4/h in total**, enforced by `up`.
`--owner-approved` overrides a cap or the GPU policy and is only used when the
owner's decision is cited in the card. The owner decides: raising the caps,
anything above $4/h, GPU types outside the policy table (and any use of the
forbidden tier), new network volumes beyond `uor-shared-EU-RO-1` and
`uor-shared-EUR-IS-1`, other providers and community cloud.

## Takeover

A lease past `expires` with no renewal is **abandoned**. Before reusing or
deleting its pod, the next lab: checks `uor-pod status` (GPU utilisation,
compute processes, held GPU locks) and the job logs under
`/workspace/uor-r4/jobs/<lab>/`; leaves any live job running and leases only
free GPUs; cites the ledger line (`uor-pod log`) and posts on the owning task
issue. An abandoned pod with no live job may then be leased (`lease` reuses
expired slots) or deleted with `down`. Results already on `/workspace` stay
where they are and keep their owner.

## Security

* Never read, print or copy `~/.runpod/config.toml` or any API key. Use
  `runpodctl` and `gh`. The tool strips `env` from every pod record it shows.
* No unauthenticated Jupyter. Use the standard template; never set a weak
  `JUPYTER_PASSWORD` or expose 8888 publicly.
* The account SSH key for pods is `~/.ssh/uor_compute`
  (`UOR_POD_SSH_KEY` overrides). Pods are ephemeral, so host keys are not
  pinned (`StrictHostKeyChecking=no`); do not reuse a pod for secrets.

## Recovering pod access when SSH keys fail

1. `runpodctl ssh list-keys` must show the key you use
   (`ssh-keygen -lf ~/.ssh/uor_compute.pub` gives its fingerprint). If it is
   missing, add the **public** key with `runpodctl ssh add-key`; it reaches only
   pods created afterwards.
2. `runpodctl pod get POD` shows the SSH address and whether the pod is
   running; `runpodctl pod logs POD` shows boot errors.
3. On a standard-template pod, a key missing from the pod means it was added
   after creation: create a new pod with `up` (the volume keeps the work) and
   `down` the old one once its jobs are done.
4. On a legacy pod (image that ignores `PUBLIC_KEY`), do not open
   Jupyter to the internet to inject keys. Prefer replacing the pod; if a
   running job must be rescued, use `runpodctl send`/`receive` or the Runpod
   web terminal, then append the public key to `/root/.ssh/authorized_keys`.
