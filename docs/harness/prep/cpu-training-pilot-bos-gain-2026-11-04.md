# Direct final-gain training for reply termination

**Status:** READY FOR FIT REVIEW

## Question

Can one Q12 final-gain coordinate search, evaluated through the complete CPU
FBS serving path, make decoder BOS the top-1 stop at held-out terminal
decisions on two public model seeds while preserving nonterminal content and
BOS suppression, and beat matched shuffled-label and random-gain controls?

This is D84 CPU development research on the Ryzen 9 9950X. It is not GPU
qualification, complete-reply correctness or useful conversation.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `51279a81a58710df08e65f69002634f8837d1288` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public manifest says `split=train`. "Held-out" means odd-ordinal documents
and learned evaluation keys are absent from training memory; base-model
exposure is not claimed to be zero.

## Data selection

The model-free driver selected 16 eight-token keys. Even document ordinals
train and odd ordinals evaluate, up to eight examples per key.

| Partition | Samples | Decisions | Terminal | Nonterminal |
|---|---:|---:|---:|---:|
| training | 98 | 3,969 | 98 | 3,871 |
| evaluation | 103 | 3,765 | 103 | 3,662 |

The seed-1 label shuffle has 98 terminal labels, preserves class counts and
changes 196 training label positions. Both seeds select the same corpus rows.
Opening/reply memory is derived from training examples only.

## Training procedure

For each training decision:

1. build the exact teacher-forced CPU FBS trajectory with reply start marked at
   the final prompt token;
2. label the decision terminal when its actual next corpus token is EOS id 1,
   and nonterminal otherwise;
3. cache the pre-final-gain normalization, fixed memory move and content target;
4. replay every final-gain coordinate at `+1` and `-1` through the full served
   query path; and
5. accept one coordinate only when nonterminal BOS top-1 does not increase,
   nonterminal content top-1 does not fall, and the terminal objective improves
   lexicographically by BOS top-1, admission, top-8, rank sum, distance sum,
   then nonterminal content top-1.

The fixed budget is eight proposal rounds. A round with no strict improving
candidate records no update. The true and shuffled-label trainers run the same
budget independently. The random control applies exactly the true trainer's
accepted-move count with deterministic `+/-1` coordinate moves.

## Arms

1. frozen memory baseline;
2. true-label termination-gain training;
3. shuffled-label termination-gain training;
4. equal-move random final gain; and
5. free greedy generation for all four arms at a 16-token cap.

## Bar

Both seeds must pass:

- held-out terminal BOS top-1 is at least **26/103 (25%)**;
- terminal BOS top-1 beats shuffled and random controls by
  `max(2, ceil(2 * sqrt(b + c)))`, where `b` is candidate-only correct and
  `c` is control-only correct;
- nonterminal BOS selections are at most **36/3,662 (1%)** and no greater than
  frozen;
- nonterminal content top-1 is not below frozen; and
- every arm has zero query-coordinate, token and kept-cell mismatches.

Free-generation BOS, premature-BOS, cap, context, exact-reference, opening,
distinct and repeated-4-gram readings are reported, not the primary bar.

## Instrument gates

- Input, source and binary hashes match.
- Training and evaluation contain terminal and nonterminal rows.
- The true trainer accepts at least one move; random applies that many.
- Shuffled labels preserve classes and change the terminal subset.
- Every accepted step records before/after objective and exact CPU/integer
  parity.
- Every artifact reparses, has no Hamming record and changes after a move.
- A seed-1 repeat reproduces every non-timing and non-resource field.

## Resource and stop projection

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| fresh release build | 300 s |
| two seeds plus seed-1 repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **2,100 s** |

- Build: 16 jobs, 4 GiB RSS maximum, 2 GiB target maximum.
- Runs: 32 Rayon workers, 1 GiB RSS maximum, 600 s each.
- Retained pipeline: 64 MiB maximum.
- Physical reserve after the run: at least 30 GiB plus 128 MiB.
- Previous ledger: `cpu-training-pilot-point-prototypes-2026-11-04/resource-ledger.json` at `34674.680792822 s`.
- Admitted cumulative projection: **36,774.680792822 s**.

## Steps

Use a fresh source-bound release target and fresh output roots:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-bos-writer-training
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE_PIN=51279a81a58710df08e65f69002634f8837d1288
PIPELINE=$SHARED/local/deepseek/termination-gain/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/termination-gain-131
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE"
CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-termination-gain-driver
BIN=$BUILD/release/cpu-termination-gain-driver
BIN_SHA=$(sha256sum "$BIN" | cut -d' ' -f1)
printf '%s  cpu-termination-gain-driver\n' "$BIN_SHA" >"$PIPELINE/binary.sha256"
run() {
  seed=$1 artifact_pin=$2 out=$3
  test ! -e "$out"
  /usr/bin/time -v timeout 600s env RAYON_NUM_THREADS=32 \
    "$BIN" "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" \
    "$DEV/dev-manifest.json" "$out" \
    --expected-artifact-sha256 "$artifact_pin" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --steps 8 --reply-cap 16 --seed 1
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  test "$(jq -er '.training.true.accepted_moves' "$out/report.json")" -gt 0
  (cd "$out" && find . -type f ! -name SHA256SUMS -print0 |
    sort -z | xargs -0 sha256sum >SHA256SUMS &&
    sha256sum -c SHA256SUMS --quiet)
}
run 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1"
run 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$PIPELINE/seed-2"
run 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1-repeat"
diff <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1/report.json") \
  <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1-repeat/report.json")
test "$(jq -er '.bar.passes' "$PIPELINE/seed-1/report.json")" = true
test "$(jq -er '.bar.passes' "$PIPELINE/seed-2/report.json")" = true
test -z "$(git -C "$SOURCE" status --porcelain -- crates third_party Cargo.toml Cargo.lock .cargo rust-toolchain.toml)"
```

Any source, input, binary, output-root, control or deterministic mismatch is
an instrument fault requiring a new FIT before retry.

## Claim boundary

A positive supports only this final-gain direct-BOS objective on the fixed
public split and two public seeds. A negative closes only this eight-move,
one-code configuration. Neither outcome changes GPU qualification or
establishes complete replies, fresh phrasing or useful chat.

## TEST FITNESS

PENDING
