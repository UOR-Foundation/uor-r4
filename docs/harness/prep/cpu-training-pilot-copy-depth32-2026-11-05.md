# Pre-registration: longest-32 exact copy depth

Run ID: `cpu-training-pilot-copy-depth32-2026-11-05`

## Decision

Do exact suffix matches deeper than four improve generated reply body
selection? Deepseek-149 selected longest-4 from a depth-one-through-four
search; this tests the untried 8-32 range while preserving longest-4 as a
nested control.

This is a D84 CPU development screen. It is not GPU qualification, complete
reply, language or serving evidence.

## Prior evidence and change

Deepseek-153's longest-4 complete-reply arm preserved 409/4,661 train and
556/4,614 dev correct body positions, beat its shallower controls, and
terminated zero replies because copy can never emit BOS. No prior unit
searched exact copy depths above four in complete replies.

The candidate arm searches depths 1 through 32, chooses the deepest available
exact suffix, and falls back through shorter matches. The existing decoder,
writer fallback, memory, stopping contract and artifact are unchanged.

## Base artifact

- seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- implementation pin `421fc8f65ab2`, source hashes
  `ba02bd01a90f5bc19fc851d90e6c4016314cd5dd2233c812188870b1416ab024`
  (`copy_key_reply.rs`) and
  `54ed98f43fa4843850b40f58674ae18bc76099651a575b908a4c1f33109d2227`
  (`cpu-copy-key-reply.rs`);
- panel generator tree `869e65db6d0bb997a5f1094ab0de4efa1ac10c25` at base
  `7fe55fdbf`;
- tokenizer SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Data split

- Train diagnostic: existing panel 2711 and sealed train reports.
- Fresh decision: generate one new `cell=dev,dev` panel, 128 conversations,
  protocol 2, seed 2752, after FIT and GO.
- Output root: `local/deepseek/copy-depth32/2026-11-05`; it is absent now.
- The new panel manifest is validated for seed, split, protocol, conversation
  count, world and tokenizer before generation.

## Steps

Use a fresh output root after GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-depth32
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-depth32/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2
PANEL=$OUT/panels/dd-2752
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-depth32
GEN_TARGET=$SOURCE/target/deepseek/copy-depth32-generator
IMPL=421fc8f65ab2
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=7fe55fdbf

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse "$BASE":docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  37715a5d621c5b8c2269940505491d13d84c37b414fa2eb9646a2de54000e6ee "$TRAIN/seed1-tt.json.gz" \
  4a6a274822f287a806a52704417c3c2bd75bee604484ce610852cfd40e809372 "$TRAIN/seed2-tt.json.gz" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/panels/tt-2711/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/panels/tt-2711/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/panels/tt-2711/manifest.json" |
  sha256sum -c -

mkdir -p "$OUT/panels" "$OUT/reports"
seal() {
  (
    cd "$OUT"
    find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
      sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

cd "$SOURCE"
/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-key-reply
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2752 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2752 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

gzip -dc "$TRAIN/seed1-tt.json.gz" >"$OUT/reports/seed1-tt-source.json"
gzip -dc "$TRAIN/seed2-tt.json.gz" >"$OUT/reports/seed2-tt-source.json"
BIN=$BUILD/release/cpu-copy-key-reply
run() {
  name=$1 panel=$2 artifact=$3 expected=$4
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 900s env RAYON_NUM_THREADS=32 "$BIN" \
    "$panel/tokens.u16" "$panel/response_mask.u8" "$panel/manifest.json" \
    "$artifact" --cap 64 --seed 163 >"$OUT/reports/$name.json"
  jq -e '.examples.reachable >= 200' "$OUT/reports/$name.json" >/dev/null
  jq -e '.random_nonvacuity_from_longest_32.non_vacuous == true' \
    "$OUT/reports/$name.json" >/dev/null
  jq -e --arg expected "$expected" \
    '.inputs.artifact_sha256 == $expected' "$OUT/reports/$name.json" >/dev/null
}
run seed1-tt "$TRAIN/panels/tt-2711" "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-tt "$TRAIN/panels/tt-2711" "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run seed1-dd "$PANEL" "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-dd "$PANEL" "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run seed1-dd-repeat "$PANEL" "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-dd-repeat "$PANEL" "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1-dd.json") \
  <(normalize "$OUT/reports/seed1-dd-repeat.json")
cmp <(normalize "$OUT/reports/seed2-dd.json") \
  <(normalize "$OUT/reports/seed2-dd-repeat.json")
for seed in seed1 seed2; do
  jq -e '[.arms.longest_32_to_1.reading.selected_depth_counts[4:][]]
    | add > 0' "$OUT/reports/$seed-dd.json" >/dev/null
done
gzip -9 "$OUT"/reports/seed*.json
test "$(du -sb "$OUT" | cut -f1)" -le 67108864
```

## Minimum test power

- two artifact seeds;
- at least 200 reference-reachable replies per seed and panel;
- a non-vacuous random-depth control for each of the four primary reports;
- at least one selected depth greater than four on each fresh-dev seed;
- fresh-process repeats of both fresh-dev seeds.

## Bar

For both artifact seeds:

- train diagnostic: longest-32 correct body positions are not below
  longest-4;
- fresh dev: longest-32 beats longest-4, `k2>k1`, fixed depth 1, random depth
  and writer on correct reference body positions by the paired two-sigma rule;
- fresh-dev reachable replies are at least 200;
- the random control changes at least one candidate decision;
- both candidate runs have zero premature BOS stops and no ungated token
  mismatch;
- normalized fresh-process repeats are byte-identical.

Exact and BOS-complete reply counts are reported but are expected to remain
zero because this arm changes only content selection.

## Stop rule

Stop on source, artifact, tokenizer, panel or report drift; any cell below 200
reachable replies; a vacuous random control; no selected depth above four; an
ungated token mismatch; nondeterminism; or a need to change serving, CUDA,
float training or Claude-owned source. Once the fresh panel is read, do not
change the depth, cap, arms or bar.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- six complete-reply processes at up to 32 workers and 900 s each;
- one panel generation, one release build and one generator build;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip; target reuses the worktree target.

## Outcomes

- **KEEP** if the deeper arm clears every fresh-dev comparison on both seeds;
  the next unit tests the selected depth as a trained/exported read.
- **REJECT** if any required comparison or control fails; close this
  exact-depth extension only.
- **POWER-LIMITED** if fewer than 200 replies or no depth-above-four match is
  available.

## TEST FITNESS

TEST FITNESS: PENDING
