# Track B conversion pipeline: execution-limited first smoke

Date: 2026-09-29. Board [#1515](https://github.com/UOR-Foundation/uor-r4/issues/1515),
epic [#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509).
Status: **Candle parity NOT_RUN; B2 NOT_RUN. No numerical or model-quality negative.**

The pinned Candle 0.9.2 Llama loader and Metal-enabled parity executable compile.
Three focused loader checks and the all-vocabulary gate check pass. The exact
reference ran, but neither Candle backend was evaluated before the shared
storage reserve was crossed. The conversion gate remains unqualified.

## Implementation and fixed comparison

The [wrapper](../../crates/uor-r4-training/src/track_b/conversion.rs) preserves
SmolLM2-135M-Instruct's full Q/K/V widths, grouped-query attention, tied
embeddings, split-half RoPE, SiLU, and original tokenizer. It safely widens
checkpoint BF16 tensors to F32 and validates every required tensor before
entering upstream block construction. Each call starts an empty cache.
It is currently an inference teacher, not the differentiable student.

The [driver](../../crates/uor-r4-training/examples/track-b-parity.rs) compares
every logit at every singleton-decoding position, plus full-prefill and
prefix-then-singleton final logits. Upstream Candle's public forward returns
only the final position and does not support multi-token cached tails with
its square mask. The wrapper does not expose that unsupported mode.

The [preregistration](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5897931883)
fixes four windows of lengths 1, 4, 8 and 32; all 49,152 logits; CPU and Metal;
F32 computation; a maximum absolute difference of 1e-4; and a 600-second
smoke. Completing all modes would compare 53 rows / 2,605,056 cells per
backend. The exact oracle must report uor-matmul exact GEMM; the observation
BLAS exception is not used. The environment pins native transcendental
arithmetic with TLESS_CANONICAL_DETERMINISTIC=0. Synthetic-token NLL/KL
would be numerical diagnostics, not language-quality scores.

## Preserved attempts

All paths below are beneath
`/Volumes/UOR-Workspace/uor-r4-lab/codex-track-b-conversion-20260929`.

| Attempt | Observed outcome | Exact positions | Candle backends | Wall seconds |
|---|---|---:|---:|---:|
| parity-1 launch | The nohup child did not survive tool-command completion; no report claim or model load | 0 | 0 | NOT_MEASURED |
| parity-2 | STOPPED_PROJECTED_WALL_BOUND; default one-worker reference projected beyond 600 seconds | 5/45 | 0 | 92.47 |
| parity-3 | STOPPED_SHARED_STORAGE_RESERVE; two exact workers; shared free space crossed the registered reserve | 25/45 | 0 | 260.77 |

The first reference attempt took roughly 15–22 seconds per token. The
[prospective correction](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5898295077)
explicitly selected two dedicated exact row-partition workers within the
already registered thread cap. No token window, threshold, arithmetic owner,
or time limit changed. The first five rows from both attempts have identical
SHA-256 `2362ba67b4e0faad1ac3b2de83e34a8528814b987ad476a141cf5042ff35dae1`.

At the storage trigger, free SSD space was 30,882,200 KiB (29.45 GiB), below
the registered 30 GiB reserve. After stopping it was 30,751,516 KiB. The
principal stopped only its own process and released its model-slot claim.
The [board outcome](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5898407253)
requests reserve restoration and coordinated execution from Lab 1/steward.
No artifact, worktree or cache was removed.

The two loaded attempts were claimed before model work. After their processes
ended, closed logs, launch scripts and explicit interrupted-result records
were added, then the complete roots were sealed and verified through
`uor_r4_core::report_output`. Neither root is reused. Manifest SHA-256:

- parity-2: `4bb227e053c5a8c79fc56cbd852a05dc620df4b00df48bf70bba9c847ad9238e`.
- parity-3: `6b49ebedac0c6541acb6f261a29f8f7ae755febedfd5b08f632098fd3a1987a2`.

## Provenance, checks and cost

The two-worker executable is SHA-256
`32b60c0e8ed79ad7b44d4d36e44f2162401c1e9a0937df1233f29fa433579f20`.
Each root's inputs.json binds its own executable. The source revision of parity-3 is
`76ef1d1c6b6f3b3968ccd81d59b3fb210dbc6294`, with an empty source diff.
The [compact evidence](../evidence/track-b-conversion-result-2026-09-29.json)
binds the actual per-attempt executable, source and model hashes.

Executed locally on the shared 8-core/16-GiB M1, macOS 27.0, rustc 1.97.1:

- `cargo test --offline --release -p uor-r4-training --features metal --lib track_b::conversion::tests -- --test-threads=1`: 3 passed; 0.01 s tests; 9 min 39 s initial compile.
- `cargo build --offline --release -p uor-r4-training --features metal --example track-b-parity`: passed on the merged plan/main snapshot; 1 min 32 s. Worker-count-only rebuild: 5.55 s.
- `cargo test --offline --release -p uor-r4-training --features metal --example track-b-parity -- --test-threads=1`: 1 passed, including last-coordinate mismatch and nonfinite rejection.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Both interrupted report roots: seal and complete-file-set verify passed.
- Independent non-author source/mathematical review approved the implementation
  and subsequent two-worker delta; it made no checkpoint-parity claim.

Model-process wall time is 353.24 s across the two loaded attempts, both exact
reference only. Training steps, Candle CPU evaluations and Metal evaluations
are all zero. Maximum process RSS is 817,725,440 bytes (one worker) and
817,004,544 bytes (two workers). These are interrupted execution receipts, not
serving throughput or energy measurements. Compiler peak RSS and gross new
physical build allocation were not measured.

The [prospective resource card](../evidence/track-b-conversion-projection-2026-09-29.json)
uses the existing Codex SSD build cache, with no new cache or allowance extension.
The first full-cycle charge covers 1,694,337 ms from 20:11:42 UTC through the
recorded checkpoint, raising the live ledger to 784,232,518 / 1,130,000,000 ms.
It includes preparation, investigation, engineering, build, model work and review
once; overlapping component times are not added again. Delivery-tail charges
remain separately recorded in the shared ledger.

## Next action and B2 boundary

Restore the shared storage reserve, then run the committed two-worker binary
on the same windows in a new root through the runner once available. A fresh
parity decision is required before B2 fitting or the 360M ladder. Reuse this
build; do not spend another full compile without a source change.

Independent B2 implementation work has a
[reviewed mathematical proposal](track-b-harmonic-design-2026-09-29.md):
positive shifted-power kernels expressed in compact trace-free harmonic bands,
correct sparse overlap subtraction, held-out operator MSE and explicit GQA/state
cost. The harmonic map and differentiable dense model are being drafted
separately. Their tests and model behavior are NOT_RUN; the drafts are not
part of this pipeline's compiled module. The current wrapper provides no
pluggable attention hook yet. Flock integration remains coordinated with DeepSeek.
