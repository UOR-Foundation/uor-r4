# Content readout serving sidecar

**Status:** FIT

## Question

Can the positive modulo-256 content readout be attached to the exact integer
FBS serving path without changing the `UORLUT01` artifact, and reproduce every
CPU-generated content decision on both public seeds?

This is D84 CPU development research. It is not a language result, GPU
qualification or a claim that the generated replies are useful.

## Compute class

Local CPU development on the Ryzen 9 9950X; no GPU, pod, paid API, upload or
external model execution.

Attempt 1 at pin `a463a1c8d` is retained as superseded integration evidence.
Final audit added canonical readout identity to serialized sessions. Attempt 2
at pin `fe253c5ca` passed both public seeds but was superseded after
exact-head review found that loopback JSON omitted the sidecar path and file
SHA-256. Attempt 3 ran under pin
`8194823789244096e2db317bb45954d5a7eeaf8f` and passed every parity and
control assertion, but its test builds grew the chat target to 2,246,424,310
bytes, above the registered 2 GiB ceiling. It is retained as instrument
evidence. Attempt 4 reruns the same source pin with a separate 3 GiB control
target and post-control target-size checks.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `8194823789244096e2db317bb45954d5a7eeaf8f` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use the deepseek-137 public split unchanged: 16 conversation groups split into
98 training and 103 evaluation samples for each artifact. The training side
contains 1,120 content decisions and the evaluation side 1,085 decisions.
Seed 1, seed 2 and the seed-1 repeat use runner seed `1`.

## Minimum test power

The unit compares two exact implementations of the same deterministic
integer function over complete generated trajectories. A single differing
token, fallback, signature, distance or fired decision fails the arm; there is
no sampling noise to estimate. Two trained artifacts plus a byte-identical
repeat test transfer across model weights and host repeatability. The result
does not generalize beyond these artifacts and remains D84 development
evidence.

## Fixed construction

The CPU content driver derives the existing generated-state code readout,
adds the training-derived turn memory, writes `serving.uorlut`, and writes the
same readout as schema `uor-r4.stack-content-readout/1`. The sidecar binds the
serving artifact SHA-256.

The integer loader validates every row's 2,048-bit width, target vocabulary,
nonterminal-only targets and learned Hamming radius. It attaches the readout
to `IntegerStackModel`. `IntegerStackSession::serve_step` first obtains the
native writer or point-key fallback, then applies the nearest-example override.
`step` and `step_native` remain unchanged.

Session snapshots bind the canonical attached-readout hash. Same-readout
restore must reproduce state; a destination with no readout or a different
readout must refuse before mutation. `UR4SESS2` carries the identity, while
legacy `UR4SESS1` stays byte-compatible when no readout is attached.
This restore control runs on the synthetic no-turn-memory FBS model: the
public artifact carries turn memory, whose pre-existing restore refusal is
unchanged and is not represented as a successful public-artifact restore.

The content driver runs the same 103 evaluation samples through:

1. its CPU `CpuFbsForward` code readout; and
2. `IntegerStackModel::serve_step` carrying the emitted sidecar.

Every decision compares chosen token, native/point fallback, packed 2,048-bit
signature, nearest distance, readout-fired state and complete reply tokens.

The loopback service exposes `content_readout` and
`content_readout_sha256` on health, model, capability, generation and final
stream JSON. Real CLI one-shot, REPL, prompt-ID and loopback launches must load
the sidecar; every JSON path must report those exact values.

## Arms and bar

Run public seed 1, seed 2 and a seed-1 repeat.

Both seeds must pass:

- the sidecar binds the exact `serving.uorlut`, carries at least 640 rows and
  16 targets, and has `max_distance <= 2048`;
- CPU and integer serving agree on every evaluated prompt and decision, with
  zero token or decision mismatches and equal decision digests;
- each seed checks 103 prompts and at least one content-readout override;
- malformed, foreign, wrong-width, terminal and out-of-vocabulary sidecars are
  refused before a step;
- raw `step`/`step_native` state remains unchanged with a readout attached;
- same-readout session restore succeeds while a missing or different readout
  is refused before state mutation in the synthetic no-memory FBS control; and
- the seed-1 normalized report, sidecar and serving artifact repeat
  byte-for-byte.

## Instrument gates

- Source and five public inputs match.
- The schema, artifact SHA and vocabulary are checked before attachment.
- Minimum-distance voting and ties are deterministic.
- No UORLUT byte changes when the sidecar is attached.
- No CUDA, float-training, softmax, dense matmul or Claude-owned path changes.
- `uor-chat --content-readout` loads the sidecar in the public smoke.
- Real loopback metadata and the CLI test identify the exact path and file
  SHA-256.

## Stop rules

Stop on any CPU/serving decision mismatch, artifact or split drift, a
changed raw step, sidecar acceptance without exact artifact identity, resource
breach or a need to edit the UORLUT format or Claude's active float files.

## Decision consequence

A pass makes the artifact-bound sidecar a supported exact serving contract for
this content representation. It does not establish termination, complete
replies, fresh phrasing or useful chat.

## Cost

| Component | Projection |
|---|---:|
| preparation and FIT | 600 s |
| release driver and chat builds | 600 s |
| focused integer and loopback controls | 300 s |
| two seeds plus repeat and CLI paths | 300 s |
| delivery and evidence | 300 s |
| total increment | **2,100 s** |

- Builds: 16 jobs, 4 GiB RSS, 2 GiB target maximum each.
- Runs: 32 Rayon workers, 1 GiB RSS, 600 s each.
- Focused controls: 4 GiB RSS and a separate 3 GiB target maximum.
- Retained evidence: 224 MiB maximum for every attempt.
- Previous ledger:
  `cpu-training-pilot-content-serving-2026-11-04/attempt-3-resource-ledger.json`
  at `35,534.827487699 s` measured and `47,827.677487699 s` admitted
  cumulative.
- Admitted cumulative projection: **49,927.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-content-readout-serving
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/content-readout-serving/2026-11-04-attempt-4
BUILD=$SOURCE/target/deepseek/content-readout-serving-140-attempt-4
CHAT_BUILD=$SOURCE/target/deepseek/content-readout-serving-chat-140-attempt-4
CONTROL_BUILD=$SOURCE/target/deepseek/content-readout-serving-controls-140-attempt-4
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=8194823789244096e2db317bb45954d5a7eeaf8f
MIN_FREE_KIB=31588352
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train crates/uor-r4-integer
test ! -e "$PIPELINE"
test ! -e "$BUILD"
test ! -e "$CHAT_BUILD"
test ! -e "$CONTROL_BUILD"
mkdir -p "$PIPELINE"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-content-readout-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le 2147483648
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
CHAT_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/chat-build.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$CHAT_BUILD" cargo build --locked --release \
  -p uor-r4-integer --features serving-wasm --bin uor-chat
CHAT_SECONDS=$(( $(date -u +%s) - CHAT_STARTED ))
CHAT_BYTES=$(du -sb "$CHAT_BUILD" | cut -f1)
CHAT_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/chat-build.time.txt")
test "$CHAT_BYTES" -le 2147483648
test "$CHAT_RSS" -le 4194304
BIN=$BUILD/release/cpu-content-readout-driver
CHAT=$CHAT_BUILD/release/uor-chat
sha256sum "$BIN" "$CHAT" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
/usr/bin/time -v -o "$PIPELINE/control-integer.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked \
  -p uor-r4-integer --features 'serving-wasm fbs-synthetic' \
  --lib content_readout >"$PIPELINE/control-integer.txt"
/usr/bin/time -v -o "$PIPELINE/control-loopback.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked \
  -p uor-r4-integer --features 'serving-wasm fbs-synthetic' \
  --test openai_loopback >"$PIPELINE/control-loopback.txt"
CONTROL_BYTES=$(du -sb "$CONTROL_BUILD" | cut -f1)
CONTROL_RSS=$(awk -F': ' '/Maximum resident set size/ {
  if ($2 > max) max = $2
} END { print max }' \
  "$PIPELINE/control-integer.time.txt" "$PIPELINE/control-loopback.time.txt")
test "$CONTROL_BYTES" -le 3221225472
test "$CONTROL_RSS" -le 4194304
test "$(du -sb "$CHAT_BUILD" | cut -f1)" -le 2147483648
printf '{"jobs":16,"target_bytes":%s,"peak_rss_kib":%s}\n' \
  "$CONTROL_BYTES" "$CONTROL_RSS" >"$PIPELINE/control-summary.json"
run() {
  seed=$1 label=$2 artifact=$3
  out=$PIPELINE/$label
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" timeout 600s \
    env RAYON_NUM_THREADS=32 "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$out" --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --stop-prototypes 8 --seed 1 >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and .bar.serving.passes' \
    "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json"
while IFS= read -r -d '' file; do
  cmp "$file" "$PIPELINE/seed-1-repeat/${file#"$PIPELINE/seed-1/"}"
done < <(find "$PIPELINE/seed-1" -maxdepth 1 -type f \
  \( -name '*.json' -o -name '*.uorlut' \) ! -name report.json -print0 | sort -z)
"$CHAT" --stack "$PIPELINE/seed-1/serving.uorlut" \
  --tokenizer "$DUAL/tokenizer-seed-1.json" \
  --content-readout "$PIPELINE/seed-1/content-readout-sidecar.json" \
  --prompt-ids 0,7,11 --max-tokens 4 >"$PIPELINE/seed-1.chat-smoke.json"
SIDECAR_SHA256=$(sha256sum "$PIPELINE/seed-1/content-readout-sidecar.json" |
  cut -d' ' -f1)
jq -e --arg path "$PIPELINE/seed-1/content-readout-sidecar.json" \
  --arg sha "$SIDECAR_SHA256" '.schema == "uor-r4.uor-chat-stack-parity/1" and
  .content_readout == $path and
  .content_readout_sha256 == $sha and
  (.generated_ids | length) == 4' \
  "$PIPELINE/seed-1.chat-smoke.json" >/dev/null
"$CHAT" --stack "$PIPELINE/seed-1/serving.uorlut" \
  --tokenizer "$DUAL/tokenizer-seed-1.json" \
  --content-readout "$PIPELINE/seed-1/content-readout-sidecar.json" \
  --say hello --max-tokens 1 >"$PIPELINE/seed-1.chat-one-shot.json"
jq -e --arg path "$PIPELINE/seed-1/content-readout-sidecar.json" \
  --arg sha "$SIDECAR_SHA256" '.schema == "uor-r4.uor-chat-stack-reply/1" and
  .content_readout == $path and
  .content_readout_sha256 == $sha and
  (.reply_ids | length) == 1' \
  "$PIPELINE/seed-1.chat-one-shot.json" >/dev/null
printf 'hello\n/quit\n' | "$CHAT" \
  --stack "$PIPELINE/seed-1/serving.uorlut" \
  --tokenizer "$DUAL/tokenizer-seed-1.json" \
  --content-readout "$PIPELINE/seed-1/content-readout-sidecar.json" \
  --max-tokens 1 >"$PIPELINE/seed-1.chat-repl.txt"
grep -q '1 id(s)' "$PIPELINE/seed-1.chat-repl.txt"
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le 234881024
```

Any source, input, binary, sidecar, parity, worker-order, repeat or resource
mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 CPU content-readout serving attempt 4
docs/harness/prep/cpu-training-pilot-content-serving-2026-11-04.md
reviewer: codex-exact-fit-attempt-4
note: Attempt 3 target breach preserved; attempt 4 isolates controls, gates final target sizes, checks one-shot and REPL, preserves prior charges, and explicitly limits restore evidence to the synthetic no-memory FBS control.
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
