# Geometric scale ladder runbook (20M / 29M / ~96M)

Written 3 October 2026 (Eastern Time) by the support lab (Antigravity) from the archived launch scripts (`~/uor-r4-local/ladder/`), merged training and evaluation tools, and [PR #1660](https://github.com/UOR-Foundation/uor-r4/pull/1660) (`b88ba166`). The Claude lab administers the GPU pod and executed the ladder experiments; outcome numbers are recorded on [issue #820](https://github.com/UOR-Foundation/uor-r4/issues/820) and [issue #1552](https://github.com/UOR-Foundation/uor-r4/issues/1552).

## 1. Scope and non-claims

- **What it is:** an operational, step-by-step reproduction guide for data preparation, base geometric training, chat fine-tuning, grounded conversation sessions, and offline panel grading for the 20M, 29M, and ~96M rungs of the scale ladder.
- **Hardware split:**
  - Base training and chat fine-tuning run offline on the rented Runpod 2× RTX 4090 pod (driver 595.91.07, CUDA 12.6, `CUDA_COMPUTE_CAP=89`).
  - Grounded conversation sessions (D19) and chat grading run locally on the M1 laptop under bounded CPU thread budgets, with grading queried against a local Ollama instance (`qwen2.5:7b`).
- **Pre-alpha status:** the UOR-R4 geometric state model remains pre-alpha. General conversation, broad reasoning, frontier capability, and complete-path energy savings are not established.
- **No transformer baselines:** per [owner direction of 3 October (~1:20 PM ET)](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179), transformer comparisons are discontinued. Comparisons are strictly within the geometric architecture family (e.g., quaternion vs U(1) transport lanes; flat L2 vs Lorentz read) and against preceding geometric rungs.

## 2. Data preparation

Training and fine-tuning draw from four token corpora: TinyStories (stories), TinyDialogues (conversations), chat-v0 (protocol-2 dialogue), and chat-v1 (parquet-derived dialogue).

### 2.1 TinyStories and TinyDialogues (`prepare-text-corpus`)

Prose and dialogue corpora are encoded into memory-mapped UORT token stores (`tokens.u16`) using `prepare-text-corpus`:

```sh
# Build the data preparation tool
cargo build --release -p uor-r4-training --bin prepare-text-corpus

# Encode TinyStories training split
target/release/prepare-text-corpus \
  out=$D/corpora/ts-train \
  tokenizer=$D/tokenizer.json \
  input=$RAW/TinyStoriesV2-GPT4-train.txt \
  format=tinystories

# Encode TinyStories validation split
target/release/prepare-text-corpus \
  out=$D/corpora/ts-valid \
  tokenizer=$D/tokenizer.json \
  input=$RAW/TinyStoriesV2-GPT4-valid.txt \
  format=tinystories

# Encode TinyDialogues training split
target/release/prepare-text-corpus \
  out=$D/corpora/td-train \
  tokenizer=$D/tokenizer.json \
  input=$RAW/TinyDialogues-train.txt \
  format=tinydialogues
```

- **`format=tinystories`:** splits on `<|endoftext|>`, trims each document, verifies byte-level round-trip decoding, and appends `<|eos|>`.
- **`format=tinydialogues`:** parses one conversation per line, expands literal `\n` escapes into real newlines, normalizes `**Name**:` speaker markers to `Name:`, and validates exact text round-tripping.
- Output stores contain `tokens.u16` and a cryptographic `manifest.json` binding the input text, tokenizer, and document counts.

### 2.2 chat-v0 store

- Prepared with literal dialogue protocol 2 (`System: `, `User: `, `Assistant: `) from SmolTalk and UltraChat JSONL records.
- Stored on the pod at `/root/data/chat-v0-p2/train/tokens.u16`.
- Verified at launch (`run-20m.sh`) via SHA-256 prefix: `b5d3bf977a524d9e`.

### 2.3 chat-v1 via `prepare-chat-parquet`

Prepared directly from Hugging Face Parquet exports using the `parquet-input` feature (merged in [PR #1660](https://github.com/UOR-Foundation/uor-r4/pull/1660)):

```sh
cargo run --release -p uor-r4-training --features parquet-input --bin prepare-chat-parquet -- \
  out=/root/data/chat-v1-p2 \
  tokenizer=/root/data/tokenizer.json \
  source=smoltalk:OpenRAIL-M:13900:/root/data/parquet/smoltalk_smol-magpie-ultra.train.parquet \
  source=ultrachat:MIT:10200:/root/data/parquet/ultrachat_200k.default.train_sft.parquet \
  protocol=2
```

- **Skip offsets (13900 and 10200) and held-out panel integrity:**
  - SmolTalk Magpie Ultra skip: `13900` rows.
  - UltraChat 200k skip: `10200` rows.
  - **Rationale:** chat-v0 was extracted from row 0 through row 13,900 of SmolTalk and row 0 through row 10,200 of UltraChat. Skipping these exact counts ensures the new chat-v1 training rows are strictly disjoint from both the chat-v0 training split and chat-v0 held-out documents. Because the held-out evaluation panel (`heldout-200-a.json`, `heldout-200-b.json`) was sampled from chat-v0's held-out split, skipping this prefix prevents data contamination in downstream evaluations.
- **Filtering:** rows lacking assistant responses, empty rows, oversized rows (>8,192 tokens), or rows containing literal special tokens are dropped and counted in the resulting `manifest.json`. Produces parallel `tokens.u16` and `response_mask.u8`.

## 3. Base training per rung (`geometric-stack train`)

Base training executes offline on the Runpod GPU host using `geometric-stack train` with `device=cuda`.

### 3.1 Common base configuration

- **Data mixture:** 60% TinyStories train, 15% TinyDialogues train, 25% chat-v0 train:
  `train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16`
  `train_weights=0.6,0.15,0.25`
- **Validation set:** TinyStories validation (`valid=$D/corpora/ts-valid/tokens.u16`).
- **Context & batch:** `context=384`, `batch=16` (6,144 tokens per step), `warmup=200`.
- **Geometric kernel:** `arch=geometric`, `read=l2` (flat Euclidean distance read, which superseded Lorentz in step-2 evaluations), `rotation=true` (quaternion rotation lanes).

### 3.2 20M rung (~19.9M parameters, 300M tokens)

- **Architecture:** `width=512`, `heads=8`, `layers=8` (pattern `rrarrarr`).
- **Step count:** `steps=48828` (48,828 × 6,144 = 300,000,000 tokens).
- **Execution script (`run-20m.sh`):**

```sh
CUDA_VISIBLE_DEVICES=0 RAYON_NUM_THREADS=8 nohup /root/target/release/examples/geometric-stack train \
  out=/root/runs/geo-20m \
  train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16 \
  train_weights=0.6,0.15,0.25 \
  valid=$D/corpora/ts-valid/tokens.u16 \
  tokenizer=$D/tokenizer.json \
  width=512 heads=8 layers=8 context=384 \
  steps=48828 batch=16 warmup=200 \
  eval_every=2500 eval_windows=64 final_windows=512 \
  checkpoint_every=5000 max_seconds=21600 \
  device=cuda arch=geometric read=l2 rotation=true \
  > /root/runs/geo-20m.log 2>&1 &
```

- **Transformer control termination:** an `arch=transformer` control on GPU 1 was started in `run-20m.sh` but was stopped ~25 minutes in pursuant to the 3 October owner direction; no further transformer controls are trained.
- **Learning rate checks (`run-lr.sh`, `run-lr2.sh`):**
  - Short runs (660 s limit) compared learning rates 0.002, 0.001, 0.003, and 0.0005.
  - Validation losses are recorded on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179); lower learning rates (0.001 and 0.0005) demonstrated improved loss at steps 2500 and 5000.

### 3.3 29M rung (~28.9M parameters, 433.8M tokens)

- **Architecture:** `width=576`, `heads=8`, `layers=10` (pattern `rrarrarrar`).
- **Step count:** `steps=70609` (70,609 × 6,144 = 433,821,696 tokens; 15 tokens/parameter on 28.92M parameters).
- **Execution script (`run-29m.sh NAME GPU LR SEED`):**

```sh
NAME=$1; GPU=$2; LR=$3; SEED=${4:-1}
B=/root/target-rope/release/examples/geometric-stack; D=/root/data

CUDA_VISIBLE_DEVICES=$GPU RAYON_NUM_THREADS=8 $B train \
  out=/root/runs/$NAME lr=$LR seed=$SEED \
  train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16 \
  train_weights=0.6,0.15,0.25 \
  valid=$D/corpora/ts-valid/tokens.u16 \
  tokenizer=$D/tokenizer.json \
  width=576 heads=8 layers=10 context=384 \
  steps=70609 batch=16 warmup=200 \
  eval_every=5000 eval_windows=64 final_windows=512 \
  checkpoint_every=10000 max_seconds=21600 \
  device=cuda arch=geometric read=l2 rotation=true \
  > /root/runs/$NAME.log 2>&1
```

- Two arms were evaluated: `lr=0.001` and `lr=0.0005`. TinyStories validation NLL results are linked on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### 3.4 ~96M rung (two GPUs via `data_parallel=2`)

- **Architecture:** ~96M parameters, targeting ~1.5B tokens.
- **Two-GPU data parallelism (`data_parallel=2`):**
  - Requires `device=cuda`, an even batch size (e.g., `batch=16`), and standard float training (no QAT or transport snap).
  - Each step splits the batch into equal halves across GPU 0 and GPU 1.
  - Forward and backward passes run concurrently on both devices.
  - The replica gradients on GPU 1 are accumulated and averaged into GPU 0's buffers.
  - Optimizer updates occur on GPU 0, and updated weights are copied back to GPU 1 (`StackModel::copy_variables_from`).
  - Because the arithmetic mean of two half-batch means equals the full-batch mean, the update is mathematically identical to a single-device step (up to floating-point addition order).
- **Invocation command:**
  ```sh
  RAYON_NUM_THREADS=8 target/release/examples/geometric-stack train \
    out=/root/runs/geo-96m \
    data_parallel=2 device=cuda ...
  ```
  *(Exact pod wrapper script: unverified from local repository checkout; mechanism and CLI arguments verified in `crates/uor-r4-training/examples/geometric-stack.rs` lines 687-691, 1328-1342, and 1843-1860).*

## 4. Chat fine-tune (`dialogue-train`)

Chat fine-tuning trains the base geometric model on dialogue sequences with masked loss (loss computed only on assistant turns and their `<|eos|>` tokens).

### 4.1 Invocation pattern (`run-ft.sh`)

```sh
ft() {
  name=$1; corpus=$2; gpu=$3
  CUDA_VISIBLE_DEVICES=$gpu RAYON_NUM_THREADS=8 $B dialogue-train \
    out=/root/runs/$name \
    tokenizer=$T \
    train_tokens=$corpus/tokens.u16 \
    train_mask=$corpus/response_mask.u8 \
    train_manifest=$corpus/manifest.json \
    dev_tokens=$D/tokens.u16 \
    dev_mask=$D/response_mask.u8 \
    dev_manifest=$D/manifest.json \
    init=$I \
    pointer=32 protocol=2 context=384 policy=full_prefix \
    steps=4000 batch=16 lr=0.0003 warmup=100 \
    eval_every=500 checkpoint_every=4000 \
    dev_seed=20260930 dev_per_source=32 max_seconds=3600 \
    device=cuda > /root/runs/$name.log 2>&1
}
```

### 4.2 Critical parameter constraints

- **`init=<run>/model` (MANDATORY):**
  `init` **must point directly to the model subfolder** (e.g., `/root/runs/geo-20m/model`), **never the run root**.
  *Reason:* `dialogue-train` calls `StackModel::load`, which looks for `config.json` and weight files directly inside the directory specified by `init`. Passing the run root fails immediately with a missing file error.
- **`pointer=32`:** enables the 32-dimensional pointer mixture head for addressable memory readout.
- **`protocol=2`:** enforces literal role markers (`System: `, `User: `, `Assistant: `).
- **`policy=full_prefix`:** conditions generation on the complete preceding conversational context.
- **Fine-tuning recipes:**
  - **Recipe A (M-world only):** corpus `/root/data/ft/parar` (emit-6r synthetic factual grounding).
  - **Recipe B (Mixed):** corpus `/root/data/ft/mixed` (M-world synthetic turns + chat-v0 responses).

## 5. Grounded conversation session (D19 session)

Multi-turn factual grounding and memory retention are evaluated locally using `m-world session` (`examples/m-world.rs`).

```sh
# Fetch completed run from pod
scp -q -r -i ~/.ssh/uor_compute -P 10132 root@$POD_IP:/root/runs/$NAME $L/runs/$NAME

# Run session evaluation under local thread budget
RAYON_NUM_THREADS=3 VECLIB_MAXIMUM_THREADS=3 $BIN/m-world session \
  out=$L/evals/session-$NAME \
  world=v2 \
  model_root=$L/runs/$NAME \
  tokenizer=$T \
  compiler=$SIEVE/compiler-save-op-v25-rawtable/compiler.json \
  trunk=$SIEVE/op-model-v25/model \
  op_policy=unless_query \
  log_recall=sieve \
  max_new_tokens=64 \
  arms=default \
  reload=0 \
  > $L/evals/session-$NAME.log 2>&1
```

- **`log_recall` modes:**
  - `log_recall=sieve`: queries an exact prime-atom sieve over previous user turns to supply a recall line when the compiler leaves the turn unresolved.
  - `log_recall=off`: baseline session without sieve recall, evaluating raw unaugmented state retention.
- **Turn count & scoring:** evaluates 1,075 turns across 64 conversations. Results (e.g. 29M B lr 5e-4 scoring 972/1,075 with sieve vs 865 with store off) are documented in [issue #820](https://github.com/UOR-Foundation/uor-r4/issues/820) and [issue #1552](https://github.com/UOR-Foundation/uor-r4/issues/1552).

## 6. Chat-grade panel evaluation (`chat-grade`)

Qualitative fluency and relevance are scored by a local judge (`qwen2.5:7b` hosted via Ollama) across a 232-request evaluation panel.

### 6.1 Panel composition (232 requests total)

The panel is frozen in `~/uor-r4-local/ladder/panel/` before model evaluation:
1. `everyday-32.json`: 32 requests in everyday conversational register (smalltalk, simple questions, simple instructions, two-turn follow-ups; 8 each).
2. `heldout-200-a.json` (100 requests) & `heldout-200-b.json` (100 requests): held-out single-turn requests sampled from chat-v0 held-out split (`count=200`, `max_words=24`, seed 1; split into two files to observe the 128 requests/file panel cap).

Panel extraction command (`chat-grade extract`):
```sh
cargo run --release -p uor-r4-training --bin chat-grade -- extract \
  out=panel/heldout-200.json \
  heldout=/path/to/chat-v0-p2/heldout \
  tokenizer=/path/to/tokenizer.json \
  count=200 max_words=24 seed=1
```

### 6.2 Grading execution (`chat-grade grade`)

```sh
RAYON_NUM_THREADS=2 target/release/chat-grade grade \
  out=$L/grades/$NAME-powered-7b \
  model=$L/runs/$NAME/model \
  tokenizer=$T \
  requests=$L/panel/everyday-32.json,$L/panel/heldout-200-a.json,$L/panel/heldout-200-b.json \
  max_new_tokens=64 \
  grader=qwen2.5:7b \
  ollama_url=http://127.0.0.1:11434 \
  > $L/grades/$NAME-powered-7b.log 2>&1
```

### 6.3 Grader questions, controls, and McNemar test

- **Judge questions:**
  - **Fluency:** *"Is the last Assistant reply written in clear, correct, sensible English sentences?"* (`yes` / `no`)
  - **Relevance:** *"Does the last Assistant reply respond sensibly to the user's last message?"* (`yes` / `no`)
- **Acceptability criterion:** a reply is `acceptable` if and only if both fluency and relevance are judged `yes`. Any unparsed response counts as `no`.
- **Derangement control:**
  - To verify that the judge is not rewarding generic agreeable responses, each reply $i$ is also evaluated against the *next* request's conversation: $(i + 1) \pmod n$.
  - Fluency is expected to remain consistent across the permutation, while relevance must collapse.
- **Paired McNemar test:**
  - Evaluates discordant pairs between actual replies and deranged controls:
    - $b = \text{actual\_only}$ (actual reply acceptable, deranged control unacceptable)
    - $c = \text{control\_only}$ (deranged control acceptable, actual reply unacceptable)
  - Computes the two-sided exact McNemar p-value:
    $$p = \min\left(1.0, 2 \sum_{k=0}^{\min(b,c)} \binom{b+c}{k} 0.5^{b+c}\right)$$
  - A model discriminates relevance when $b > c$ and $p < 0.05$. Scores and p-values are recorded on [issue #820](https://github.com/UOR-Foundation/uor-r4/issues/820).

## 7. Operational rules and reproduction checklist

1. **One cargo process at a time per host:** do not run concurrent cargo builds or tests on the laptop or the pod.
2. **Strict exclusive directory creation:** all report roots (`out=`) must be claimed with `uor_r4_core::report_output::claim` before execution. Never write into an existing or sealed output folder.
3. **Local laptop thread limits:**
   - Laptop sessions run with `RAYON_NUM_THREADS=3 VECLIB_MAXIMUM_THREADS=3`.
   - Panel grading runs with `RAYON_NUM_THREADS=2`.
   - All active local jobs register in `~/.local/share/uor-r4/locks/jobs/<job>.json` to enforce the shared 8-thread machine budget.
4. **Pod file transfers:** transfer files off the pod via `scp` before destroying or terminating the cloud instance; all checkpoints, manifests, and logs must be stored locally in `~/uor-r4-local/ladder/`.
5. **D11 serving compliance:** the scale ladder checkpoints (8M, 20M, 29M) represent floating-point training artifacts; exact D11 integer serving and multiplier-free kernel verification remain open (tracked in [issue #964](https://github.com/UOR-Foundation/uor-r4/issues/964)).
