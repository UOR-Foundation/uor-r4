# Hierarchical copy-key depth screen

**Status:** `TEST FITNESS: PENDING`

## Question

Does exact copy retrieval improve when the read backs off from the longest
available suffix key, depth four to one, instead of fixing one key for every
position?

Claude-267 measured latest exact matches at copy-key depths one and two on
public DEV. Its `k2>k1` backoff was right most often in that screen. This unit
tests whether extending the same backoff to depths three and four adds a
material exact-match signal. It is a data-only read screen, not a trained-model
or language result.

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `c53b06b5f` |
| public M1 DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| public M1 DEV mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| public M1 DEV manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| train-control-v3 tokens | `a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5` |
| train-control-v3 mask | `89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f` |
| train-control-v3 manifest | `980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672` |
| dev-panel-v3 tokens | `add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442` |
| dev-panel-v3 mask | `da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01` |
| dev-panel-v3 manifest | `ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Fixed construction

Use 192-token BOS-delimited scoring windows, the same boundaries as
`build_document_windows`. For eligible target position `p`:

```text
query_d(p) = tokens[p-d..p]
key_d(s)   = tokens[s-d..s]
```

A candidate `1 <= s < p-1-win_start` predicts `tokens[s]`; the latest exact
slice match wins. Missing key history at the start of the window repeats its
first token, matching Claude-267's depth-two reference.

The candidate is `longest_4_to_1`: use depth four if it has an exact match,
then three, two, one. Controls are fixed depths one through four,
`k2>k1`, and a deterministic random choice over available depths. The seed is
149.

## Instrument gate

Public DEV must reproduce Claude-267 exactly before the bar is read:

- 8,243 copyable positions;
- depth 1: 6,306 exact matches and 3,859 right;
- depth 2: 4,383 exact matches and 3,506 right;
- `k2>k1`: 4,388 right.

Any mismatch stops the run as an instrument fault.

## Bar

The candidate passes only if it beats `k2>k1` by the paired two-sigma rule on
both public DEV and `dev-panel-v3`, and beats the random-depth control by the
same rule on both. The paired threshold is
`max(2, ceil(2*sqrt(control-only)))`.

A pass makes depths three and four a supported CPU read candidate for the next
Claude copy-read build. A fail closes the fixed longest-first 1–4 exact suffix
router at this context, not deeper keys generally.

## Stop rules

Stop on a public-DEV parity mismatch, input/hash drift, nondeterminism,
cross-document or cross-window candidate use, a vacuous random control,
resource breach, or any need to change serving or Claude-owned source.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and review | 600 s |
| release build and controls | 300 s |
| three corpus screens and one repeat | 60 s |
| delivery and evidence | 240 s |
| total increment | **1,200 s** |

- Build jobs: 16.
- Run cap: 600 s and 1 GiB RSS per command.
- Evidence cap: 33,554,432 bytes.
- Previous measured cumulative: `36,068.788816343 s`; previous admitted
  cumulative: `69,727.677487699 s`.
- New admitted cumulative: **70,927.677487699 s**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT`.

## Steps

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-depth
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-depth/2026-11-05
BUILD=$SOURCE/target/deepseek/copy-key-depth-run
CONTROL=$SOURCE/target/deepseek/copy-key-depth-controls
DEV=$SHARED/local/deepseek/public-m1-parity
TRAIN=$SHARED/local/deepseek/m2-evidence/train-control-v3
PANEL=$SHARED/local/deepseek/m2-evidence/dev-panel-v3
IMPL_PIN=c53b06b5f

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  .cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party
sha256sum -c <(printf '%s\n' \
  '0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  '"$DEV"'/dev-tokens.u16' \
  'a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  '"$DEV"'/dev-response-mask.u8' \
  'ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab  '"$DEV"'/dev-manifest.json' \
  'a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5  '"$TRAIN"'/tokens.u16' \
  '89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f  '"$TRAIN"'/response_mask.u8' \
  '980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672  '"$TRAIN"'/manifest.json' \
  'add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442  '"$PANEL"'/tokens.u16' \
  'da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01  '"$PANEL"'/response_mask.u8' \
  'ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949  '"$PANEL"'/manifest.json') --quiet

mkdir -p "$OUT"
seal() {
  (cd "$OUT"; find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
    sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet)
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-copy-key-depth

BIN=$BUILD/release/cpu-copy-key-depth
run() {
  name=$1
  dir=$2
  /usr/bin/time -v -o "$OUT/$name.time.txt" \
    timeout 600s "$BIN" "$dir/tokens.u16" "$dir/response_mask.u8" "$dir/manifest.json" \
      --context 192 --seed 149 >"$OUT/$name.json"
}
run dev "$DEV"
run train-control "$TRAIN"
run dev-panel "$PANEL"
run dev-repeat "$DEV"

test "$(jq -r '.reading.copyable_positions' "$OUT/dev.json")" -eq 8243
test "$(jq -r '.reading.fixed[0].positions_with_match' "$OUT/dev.json")" -eq 6306
test "$(jq -r '.reading.fixed[0].right' "$OUT/dev.json")" -eq 3859
test "$(jq -r '.reading.fixed[1].positions_with_match' "$OUT/dev.json")" -eq 4383
test "$(jq -r '.reading.fixed[1].right' "$OUT/dev.json")" -eq 3506
test "$(jq -r '.reading.k2_backoff_k1.right' "$OUT/dev.json")" -eq 4388
test "$(jq -r '.reading.random_changed_predictions' "$OUT/dev.json")" -gt 0
cmp "$OUT/dev.json" "$OUT/dev-repeat.json"

jq 'del(.reading.random_changed_predictions)' "$OUT/dev.json" >"$OUT/dev.normalized.json"
jq 'del(.reading.random_changed_predictions)' "$OUT/dev-repeat.json" >"$OUT/dev-repeat.normalized.json"
cmp "$OUT/dev.normalized.json" "$OUT/dev-repeat.normalized.json"
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

Read `longest_vs_k2_backoff.passes` and `longest_vs_random.passes` in both
`dev.json` and `dev-panel.json`. Both must be true for a candidate pass.
`train-control.json` is supporting evidence only and does not move the bar.

## Rollback boundary

The unit adds an isolated CPU module and binary. Reverting it removes only the
screen; no model artifact, serving format, CUDA code or Claude-owned source is
changed.

## TEST FITNESS

TEST FITNESS: PENDING independent fresh-context review.

## Pre-run GO

GO: PENDING independent fresh-context review after FIT.

## Outcome

Pending the registered run.
