# Prep sheet: centroid refit on the current D3 checkpoint (one-model-end-to-end, deepseek-16, 6 Nov 2026)

Lab: deepseek. Item: one-model-end-to-end (#3407). Orders:
[deepseek-16](../../loop/tasks/one-model-end-to-end/deepseek-16/ORDER.md) and
[deepseek-17](../../loop/tasks/one-model-end-to-end/deepseek-17/ORDER.md).

TEST FITNESS: PENDING - this execution package is complete, but it has not received a
fresh non-author review. No capture or evaluation is authorized until that review writes
`TEST FITNESS: FIT`.

## Question

On the current stable D3/8,192-step FBS checkpoint at seeds 1 and 2, does replacing
eligible output-table rows with the mean TRAIN query for each target improve the
whole-table nearest top-1, beyond the original table and a target-count-preserving
matched-label control, without lowering top-8?

This tests the output readout only. The trunk, input query path, table cells and writer
admission remain frozen.

## Why now

Claude-297 found that D3 moves the trunk at both seeds, while (n3S) remains behind its
TRAIN's T2. Its REPORT explicitly leaves centroid refit of the readout as the next unit.
DeepSeek-237 rejected median and mean-centroid rows on two old public artifacts, so the
prior is negative. This run asks whether that negative survives on the current
human-text checkpoint and the active scorer.

## Base artifact

The base is the current pinned FBS implementation plus the two Claude-297 n3S
step-008192 freeze archives listed below. The implementation pin is
`3a9c2f11d26991a70a22f88758cfb4589a1cf14d`, which contains the verified D84 cache
evaluator and the DEV-omission repair needed for its exact split inventory. The capture
binary hash is recorded on the pod; the CPU evaluator binary hash is recorded locally.

## Raw data

| Input | Store path | Identity |
|---|---|---|
| data root | `w2-inputs/2026-10-25/driver-data` | tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| D3 TRAIN | `results/claude/scale-probe-2026-11-05/inputs/d3` | tokens `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`; mask `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`; 27,416,554 scored |
| oasst2 heldout | `results/claude/real-text-corpus-2026-11-05/oasst2/heldout` | tokens `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e`; mask `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |
| wiki-00000 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00000/heldout` | tokens `82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7`; mask `34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f` |
| wiki-00020 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00020/heldout` | tokens `9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640`; mask `a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d` |
| wiki-00040 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00040/heldout` | tokens `c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d`; mask `9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625` |

The four held-out roots contain exactly 1,460,697 scored rows. Their pooled identity is
`5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.

## Data split

Training uses D3 only. Evaluation uses the four complete held-out roots. No panel row is
used to fit a centroid, and no D3 row is scored as a panel row.

### Checkpoints

The archives are:

- seed 1:
  `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s1/steps-data-s1.tar.gz`,
  SHA-256 `1c7a09781c60475067b4fc2319db92a16ef0f589dcf355db1d09f35edcc4ec69`;
- seed 2:
  `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s2/steps-data-s2.tar.gz`,
  SHA-256 `dd3cc8885cf1b35aa840da9f7d029d8a8dc6097a8321d963e4abdfcc8ce5c467`.

Before capture, the job must verify each archive hash and the selected checkpoint against
its inner `SHA256SUMS.files`.

| Seed | Checkpoint | File | SHA-256 |
|---|---|---|---|
| 1 | `oh/run/n3S-s1/models/step-008192` | run config | `30e9113c05f266a11df6e2baa302e7a569280efc0d0d7b1bfb80b06245a71631` |
| 1 | | model config | `1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2` |
| 1 | | model | `c64894f3648e2663bfe1f323836c9fc22c6989c9a6f2917e8ce1f25c6a686fb2` |
| 1 | | Hamming read | `e2347c4ef266c2166e870926a44e740edbd5c27065ed6a0ccda5ab11da5998b5` |
| 2 | `oh/run/n3S-s2/models/step-008192` | run config | `3a3f9e0d3b428625270e6393e42a2444e14f61912e81a7499c96228329c74ca2` |
| 2 | | model config | `778145167a18ff3b1006d4f609b7f608cc528b0dd753279b3ff38c8c63a88b95` |
| 2 | | model | `30bf5db7db6600211ee14a17557bbf54f24c86de8b7c16f93b795085f0bfc9e8` |
| 2 | | Hamming read | `25805079be3bcfe5917cf0b80a362e7a774aac931c5f4cdaf5351c272684a5bd` |

The run and model configs together name D3, seed 1 or 2, objective `nearest`,
`train_as_served: true`, width 256, six layers, four Hamming heads and context 192. An
archive with a different selected checkpoint or config is a stop.

## Steps

The reviewed pod script is run once per seed on a fresh Codex-owned RTX 5090. Check
`scripts/pod/uor-pod status` first. Do not lease, stop or inspect either existing busy pod.
Two fresh one-card pods keep the run parallel and at most four total:

```sh
scripts/pod/uor-pod status --du
scripts/pod/uor-pod up --lab deepseek --session centroid-s1 --gpu 5090 --hours 7 \
  --ref 3a9c2f11d26991a70a22f88758cfb4589a1cf14d \
  --purpose "n3S centroid cache seed 1"
scripts/pod/uor-pod up --lab deepseek --session centroid-s2 --gpu 5090 --hours 7 \
  --ref 3a9c2f11d26991a70a22f88758cfb4589a1cf14d \
  --purpose "n3S centroid cache seed 2"

# Copy the exact D67-reviewed script to each pod, then check its recorded SHA-256.
scripts/pod/uor-pod ssh "$POD1" 'cat > /workspace/centroid-cache-job.sh' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/job.sh
scripts/pod/uor-pod ssh "$POD2" 'cat > /workspace/centroid-cache-job.sh' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/job.sh
scripts/pod/uor-pod ssh "$POD1" "printf '%s  /workspace/centroid-cache-job.sh\n' '$JOB_SHA256' | sha256sum -c -"
scripts/pod/uor-pod ssh "$POD2" "printf '%s  /workspace/centroid-cache-job.sh\n' '$JOB_SHA256' | sha256sum -c -"

scripts/pod/uor-pod run "$POD1" --lab deepseek --session centroid-s1 --gpu 0 -- \
  bash /workspace/centroid-cache-job.sh 1
scripts/pod/uor-pod run "$POD2" --lab deepseek --session centroid-s2 --gpu 0 -- \
  bash /workspace/centroid-cache-job.sh 2
```

`JOB_SHA256` is the value recorded by the D67 review. `POD1` and `POD2` are the two new
pod ids; store them in the run record. Watch each remote
`/workspace/deepseek-centroid-cache-s{1,2}/EXIT` without touching the other pods. Once it
exists, require the uploaded `status.json` and all readbacks before releasing and taking
that pod down. A non-zero exit preserves its logs under `$OUT_STORE/failure/`.

The script invokes `train-in-space` with `--query-cache-memory 0` and
`--query-cache-dev 0`. That yields exactly TRAIN plus the four registered panels; DEV is
still scored into the ordinary output files. The scorer may perform its normal memory-on
pass for those files, but no samples are decoded and no training occurs.

## Memory control

The cache captures:

- each query immediately before a scored target;
- target and corpus position;
- frozen whole-table nearest and writer decisions;
- opening/copy flags;
- the unchanged shared table.

TRAIN fitting uses only targets with at least eight observations. The control groups eligible
targets by exact observation count and deterministically cycles each non-singleton group once;
a singleton group maps to its own target in both arms. It therefore preserves every target's
total observation count and the global query multiset. A control that changes no target or
produces the candidate's table is `CONTROL_FAULT`.

## CPU evaluation

Restore and build on the D84 host. The following seed-1 block is repeated mechanically
for seed 2, substituting `s2`/`2` and its pin:

Set `OUT_STORE_S1` and `OUT_STORE_S2` from the two jobs' `[0/6] output=` log lines. The
timestamp-plus-random run id is immutable; do not reconstruct or reuse it.

```sh
ROOT=local/deepseek/centroid-cache/2026-11-06
export CARGO_TARGET_DIR="$ROOT/target"
cargo build --release --locked -p uor-r4-cpu-train \
  --bin cpu-query-cache-evaluate --bin cpu-query-cache-aggregate
sha256sum "$CARGO_TARGET_DIR/release/cpu-query-cache-evaluate" \
  "$CARGO_TARGET_DIR/release/cpu-query-cache-aggregate" \
  > "$ROOT/evaluator-binaries.sha256"

# Fetch the pinned checkpoint archive, its published checksum and the captured cache.
CKPT_STORE=results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s1
CKPT_DOWN="$ROOT/s1/checkpoint-download"
scripts/pod/uor-store get "$CKPT_STORE/steps-data-s1.tar.gz" "$CKPT_DOWN"
ARCHIVE="$CKPT_DOWN/$CKPT_STORE/steps-data-s1.tar.gz"
(cd "$(dirname "$ARCHIVE")" &&
  printf '%s  steps-data-s1.tar.gz\n' 1c7a09781c60475067b4fc2319db92a16ef0f589dcf355db1d09f35edcc4ec69 |
    sha256sum -c -)

mkdir -p "$ROOT/s1/checkpoint"
tar -xzf "$ARCHIVE" -C "$ROOT/s1/checkpoint" \
  oh/run/SHA256SUMS.files \
  oh/run/n3S-s1/config.json \
  oh/run/n3S-s1/models/step-008192/config.json \
  oh/run/n3S-s1/models/step-008192/model.safetensors \
  oh/run/n3S-s1/models/step-008192/hamming_read.safetensors
grep -E '  \./(n3S-s1/(config\.json|models/step-008192/(config\.json|model\.safetensors|hamming_read\.safetensors)))$' \
  "$ROOT/s1/checkpoint/oh/run/SHA256SUMS.files" \
  > "$ROOT/s1/checkpoint/selected.sha256"
(cd "$ROOT/s1/checkpoint/oh/run" && sha256sum -c "$ROOT/s1/checkpoint/selected.sha256")
CHECKPOINT_S1="$ROOT/s1/checkpoint/oh/run/n3S-s1/models/step-008192"

CACHE_DOWN="$ROOT/s1/cache-download"
scripts/pod/uor-store get "$OUT_STORE_S1/cache.tar.zst" "$CACHE_DOWN"
scripts/pod/uor-store get "$OUT_STORE_S1/cache.tar.zst.sha256" "$CACHE_DOWN"
scripts/pod/uor-store get "$OUT_STORE_S1/cache.SHA256SUMS" "$CACHE_DOWN"
scripts/pod/uor-store get "$OUT_STORE_S1/status.json" "$CACHE_DOWN"
ARCHIVE_ROOT="$CACHE_DOWN/$OUT_STORE_S1"
(cd "$ARCHIVE_ROOT" && sha256sum -c cache.tar.zst.sha256)
mkdir -p "$ROOT/s1/cache-extracted"
tar --zstd -xf "$ARCHIVE_ROOT/cache.tar.zst" -C "$ROOT/s1/cache-extracted"
(cd "$ROOT/s1/cache-extracted" && sha256sum -c "$ARCHIVE_ROOT/cache.SHA256SUMS")
CACHE_S1="$ROOT/s1/cache-extracted/cache"
STATUS="$ARCHIVE_ROOT/status.json"
MANIFEST=$(jq -r .cache_manifest_sha256 "$STATUS")
```

```sh
RAYON_NUM_THREADS=32 /usr/bin/time -v timeout 7200s \
  "$CARGO_TARGET_DIR/release/cpu-query-cache-evaluate" "$CACHE_S1" \
  --seed 1 \
  --checkpoint-root "$CHECKPOINT_S1" \
  --train train \
  --panel oasst2 \
  --panel wiki-00000 \
  --panel wiki-00020 \
  --panel wiki-00040 \
  --memory 0 \
  --minimum 8 \
  --min-fitted-rows 100 \
  --min-fit-observations 10000 \
  --expect-train-tokens ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52 \
  --expect-train-mask ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875 \
  --expect-rows 1460697 \
  --expect-identity 5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb \
  --out "$ROOT/centroid-s1.json"
```

Run the same command for seed 2 with `$CACHE_S2`, `$CHECKPOINT_S2`, `--seed 2` and
`--out "$ROOT/centroid-s2.json"` on the D84 host: AMD Ryzen 9 9950X, 16 physical cores / 32
hardware threads and 59 GiB RAM. `CHECKPOINT_S1` and `CHECKPOINT_S2` are the extracted
`step-008192` directories. The TRAIN query stream is not materialized in memory.

After each report, check that it names the captured manifest and the pre-registered
checkpoint files, then aggregate:

```sh
jq -e --arg manifest "$MANIFEST" '.cache.manifest_sha256 == $manifest' "$ROOT/centroid-s1.json"
jq -e '
  .cache.checkpoint_files["config.json"] == "1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2" and
  .cache.checkpoint_files["model.safetensors"] == "c64894f3648e2663bfe1f323836c9fc22c6989c9a6f2917e8ce1f25c6a686fb2" and
  .cache.checkpoint_files["hamming_read.safetensors"] == "e2347c4ef266c2166e870926a44e740edbd5c27065ed6a0ccda5ab11da5998b5"
' "$ROOT/centroid-s1.json"
/usr/bin/time -v timeout 300s \
  "$CARGO_TARGET_DIR/release/cpu-query-cache-aggregate" \
  "$ROOT/centroid-s1.json" "$ROOT/centroid-s2.json" --out "$ROOT/final.json"
```

Repeat the same manifest and seed-2 checkpoint assertions before aggregating.

## Instrument checks

- **I-0:** the float-training library tests pass at the capture pin.
- **I-1:** every input file matches the hashes above.
- **I-2:** the archive hash and inner checkpoint `SHA256SUMS` pass.
- **I-3:** each cache manifest names the expected checkpoint, D3 hashes, four panel hashes,
  query space and table hash, and contains exactly five no-memory splits: TRAIN plus the
  four panels.
- **I-3b:** the CPU report's `cache.manifest_sha256` and every checkpoint-file hash equal
  the uploaded status and the pre-registered seed table above.
- **I-4:** the CPU original table reproduces the frozen nearest decision at every panel
  row. Any difference is `CONTROL_FAULT`.
- **I-5:** the evaluator computes each split identity and the registered pooled identity
  `5f88a5b3...`; a mismatch is `CONTROL_FAULT`.
- **I-6:** the matched shuffle changes at least one eligible target and at least one
  resulting table coordinate while preserving every target's observation count.

## Minimum test power

- at least 100 fitted token rows;
- at least 10,000 TRAIN observations across fitted rows;
- all four panels and all 1,460,697 rows;
- both seeds.

The evaluator emits `INSUFFICIENT_POWER` from these checked counts. It stops that seed and
reports the census; it is neither a positive nor a mechanism negative.

## Bar

For each seed, pool the four panels:

- original decisions must match every frozen nearest decision exactly;
- centroid top-8 must be at least original top-8;
- centroid top-1 must exceed original and shuffled-control top-1;
- centroid-only wins against original must satisfy
  `candidate_only >= max(2, ceil(2 * sqrt(candidate_only + control_only)))`;
- the same paired rule must hold against the matched shuffle.

All conditions must hold at both seeds for a positive development result. Otherwise this
closes mean-centroid refit for the tested n3S configuration. Because this panel selected
the n3S checkpoint, even a positive can only nominate the candidate for a fresh transfer
panel; it cannot alone put the mechanism on the serving path.

## Cost and stop

- Two parallel one-card captures, each reading 27.42M TRAIN rows and 1.46M panel rows.
- Estimate: 2-4 hours on each pod. The hard job limit is six hours per seed inside a
  seven-hour lease that includes bootstrap; at a $1.15/h ceiling the two leases cost at
  most $16.10. The system cap remains four pods and $8/h globally.
- The raw cache is 28.02 GiB. The job requires 128 GiB free initially, 80 GiB after
  capture and 40 GiB before archive readback; this covers the raw cache, compressed
  archive, readback copy and reserve.
- The entire job, including downloads, CUDA tests/build, capture, sealing and upload, is
  inside one six-hour `timeout`. `zstd` has a functional round-trip preflight.
- CPU evaluation: 32 Rayon workers, at most 32 GiB projected RSS and 2 hours. The point
  estimator streams TRAIN; only one panel's queries are resident.
- Each no-memory cache is about 29 GiB; the compressed archive is uploaded and read back.
- Stop on any input/checkpoint mismatch, `CONTROL_FAULT`, insufficient power, archive
  ambiguity, resource breach or missing credential.

## Outcome use

- **Positive at both seeds:** nominate centroid for a fresh transfer panel and the next
  active served-readout comparison. The current panel cannot retain a mechanism it helped
  select.
- **Negative:** close mean-centroid refit on n3S and move to a rank-aware or two-table
  output objective. Do not retune the threshold or shuffle after opening the panels.
- **Insufficient power:** report the observed fitted-row/observation census; do not call
  it a negative.
- **Control fault:** repair the capture/readout identity before interpreting any centroid
  number.
