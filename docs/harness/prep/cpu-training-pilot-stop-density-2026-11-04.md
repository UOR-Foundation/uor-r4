# Generated-state offset-conditioned density stop gate

**Status:** PENDING FIT

## Question

Can all generated terminal rows, searched by an offset-conditioned
class-balanced k-nearest density rule, identify held-out reply endpoints and
complete exact replies where the fixed eight-prototype gate did not?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `8eb4a1a7a01c2cac3f6afb4d00f5fb87c730179a` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use deepseek-137's 16-key split: 98 training and 103 evaluation samples for
each artifact, with runner seed 1. Seed 1, seed 2 and the seed-1 repeat use
the same prompt data but separate fitted artifacts.

## Minimum test power

The comparison is exact and deterministic over complete generated
trajectories. A token or gate mismatch fails the mechanism. Two trained
artifacts and a separate one-worker repeat prevent a one-seed or worker-order
claim. The result remains local to these public development artifacts.

## Fixed construction

Use the deepseek-137 memory and 2,048-bit modulo-256 content readout. Generate
training replies with the exact content arm and derive every generated state:

- terminal when its offset equals normalized reference length;
- nonterminal otherwise;
- store the complete signature and generated offset.

Fit separate terminal and nonterminal row sets. For each query:

- choose k from 1, 2, 4 or 8;
- use only same-class rows whose offsets differ by at most two;
- add the maximum signature distance for missing neighbours;
- score
  `sum(k nearest nonterminal distances) - sum(k nearest terminal distances)`;
- choose k and margin by leave-one-row-out training;
- require nonterminal false stops at or below 1%;
- maximize terminal recall, then lower false stops, larger margin and smaller
  k in that order.

Emit BOS at or above the fitted margin. The content choice before the gate and
every ungated token remain the positive code arm's decisions. The
class-preserving control rotates terminal labels through the same row order
without changing offsets or class counts.

## Preflight

Build the release driver and run `--select-only` for both seeds using
`--stop-density-neighbors 8 --stop-density-offset-radius 2`.

Proceed only when each seed has:

- at least 84 terminal and 1,484 nonterminal generated rows;
- training terminal recall at least 26/84;
- nonterminal false stops at or below its 1% limit;
- a selected k in `{1,2,4,8}`;
- different true and shuffled gate hashes.

The preflight output is retained even when it stops the run.

## Arms and bar

Run no-gate code, true density, shuffled density and the historical
eight-prototype gate over complete 16-token replies.

Both seeds must pass:

- true training recall at least **26/84**, with nonterminal false stops at or
  below **1%**;
- at least **9/103** BOS completions and at least **8** at or before the
  normalized reference length;
- at least **8/103** exact reference replies that stop at BOS;
- true BOS completion exceeds no-gate and shuffled by
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching depth are not below no-gate;
- every ungated token equals the code arm and every fired decision is BOS;
- true terminal-row coverage exceeds shuffled by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- gate hashes differ and row counts match the generated training rows; and
- a seed-1 repeat at one worker matches every non-timing report field and
  generated JSON file from the 32-worker seed-1 run.

## Instrument gates

- Source and five public inputs match.
- Generated terminal labels and offsets are exact.
- Density fitting and label shuffle are deterministic.
- Missing-neighbour penalties prevent sparse offsets from creating confidence.
- Fired decisions are BOS and ungated token mismatches are zero.
- Workers 1 and 32 produce identical generated evidence.
- Full release tests, scoped Clippy, rustfmt and manifests pass.

## Stop rules

Stop on source/input/split drift, missing classes, preflight failure, vacuous
shuffle, ungated token mismatch, worker nondeterminism, resource breach or any
need to change serving, integer-session, CUDA, float-training or Claude-owned
source.

## Decision consequence

A pass makes offset-conditioned generated-state density a supported CPU
termination candidate and justifies a later artifact-bound serving unit. A
fail closes this full-row density representation and requires a different
continuation objective or trajectory representation. Neither result alone
establishes useful chat.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build and two-seed preflight | 600 s |
| two seeds plus repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **2,400 s** |

- Build: 16 jobs, 4 GiB RSS, 2 GiB target maximum.
- Runs: 32 workers for seed 1/2, one worker for the repeat, 1 GiB RSS and
  600 s per run.
- Retained evidence: 192 MiB maximum.
- Previous ledger:
  `cpu-training-pilot-content-serving-2026-11-04/resource-ledger.json` at
  `35,683.988816343 s` measured and `52,027.677487699 s` admitted cumulative.
- Admitted cumulative projection: **54,427.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-stop-density
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/stop-density/2026-11-04
BUILD=$SOURCE/target/deepseek/stop-density-142
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=8eb4a1a7a01c2cac3f6afb4d00f5fb87c730179a
MIN_FREE_KIB=31588352
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train
test ! -e "$PIPELINE"
test ! -e "$BUILD"
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
BIN=$BUILD/release/cpu-content-readout-driver
sha256sum "$BIN" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
select() {
  seed=$1 artifact=$2
  RAYON_NUM_THREADS=32 "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$PIPELINE/select-only-unused-$seed" \
    --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --stop-prototypes 8 --stop-density-neighbors 8 \
    --stop-density-offset-radius 2 --seed 1 --select-only \
    >"$PIPELINE/seed-$seed.preflight.json"
  jq -e '.modes.gate_density.training as $g |
    $g.terminal_rows >= 84 and
    $g.nonterminal_rows > 0 and
    $g.terminal_stops >= 26 and
    $g.nonterminal_stops <= $g.nonterminal_stop_limit and
    ([1,2,4,8] | index($g.leave_one_out_neighbors)) != null and
    $g.gate.terminal_rows != [] and
    $g.gate.nonterminal_rows != []' \
    "$PIPELINE/seed-$seed.preflight.json" >/dev/null
}
select 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
select 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run() {
  seed=$1 label=$2 artifact=$3 workers=$4
  out=$PIPELINE/$label
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" timeout 600s \
    env RAYON_NUM_THREADS="$workers" "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$out" --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --stop-prototypes 8 --stop-density-neighbors 8 \
    --stop-density-offset-radius 2 --seed 1 \
    >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and
    (.bar.gate_density != null)' "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 32
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 32
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 1
jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json"
while IFS= read -r -d '' file; do
  cmp "$file" "$PIPELINE/seed-1-repeat/${file#"$PIPELINE/seed-1/"}"
done < <(find "$PIPELINE/seed-1" -maxdepth 1 -type f -name '*.json' \
  ! -name report.json -print0 | sort -z)
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le 201326592
```

Any source, input, split, gate, token identity, worker-order, repeat or
resource mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: PENDING - M2 #2030 offset-conditioned density stop gate:
docs/harness/prep/cpu-training-pilot-stop-density-2026-11-04.md
head: 8eb4a1a7a01c2cac3f6afb4d00f5fb87c730179a
implementation pin: 8eb4a1a7a01c2cac3f6afb4d00f5fb87c730179a
reviewer: pending
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
