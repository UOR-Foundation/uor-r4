# Pre-registration: exact reply-boundary suffix memory

Run ID: `cpu-training-pilot-copy-boundary-2026-11-05`

## Status

NOT RUN. This sheet fixes the D84 CPU development comparison before the fresh
panel is generated or read.

## Question

Can an exact reply suffix that occurred only at the end of TRAIN replies emit
decoder BOS, while an observed continuation prevents stopping, and thereby turn
the established `Longest32To1` copy body into complete replies better than
no-boundary and matched shuffled-boundary controls?

This is exact-memory recall of TRAIN reply endings. It is not fresh-wording
generalization, useful chat or GPU qualification.

## Why this is new

- `deepseek-153` established the `Longest32To1` body selector and showed it
  cannot stop because its exact-copy candidate domain excludes BOS.
- `deepseek-155` measured an oracle termination ceiling but did not construct a
  deployable stop.
- `deepseek-157` and `deepseek-159` trained stop classifiers over generated
  query states; neither transferred.
- `deepseek-110` derived exact reply-memory keys only for nonterminal
  next-token transitions.
- This unit maps observed reply endings to decoder BOS and requires the
  longest known generated suffix to decide between termination and
  continuation. It does not train another classifier.

## Source and binary

- base: `86d654170`;
- implementation source:
  `4832461ba66df263a9259b38aa97c5f73283c5e0`;
- evaluator binary:
  `local/deepseek/copy-boundary/target/release/cpu-copy-boundary-reply`,
  SHA-256
  `ecb0d8cbf8cccaab92faa6c5a99b9646eb7901a8b077783acbc4abbcbb91c869`;
- panel-generator source tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- panel-generator binary:
  `local/deepseek/copy-boundary/generator-target/debug/heldout-panel`,
  SHA-256
  `1e5e9675191f226db8c487d55299d60806a8dfe45bf39334957aae928f22f8d5`;
- host: AMD Ryzen 9 9950X, 16 cores / 32 threads.

## Base artifact

Two public dual-c131 raw FBS artifacts:

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are unchanged one-table FBS artifacts with the shared writer table. No
model parameter is updated.

## Data split

Training panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711`

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- response mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- `cell=train,train`, seed 2711, 128 conversations.

Fresh evaluation panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-boundary/2026-11-05/panels/dd-2723`

- generated only after FIT and GO;
- `cell=dev,dev`, seed 2723, protocol 2, 128 conversations;
- tokenizer SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- the panel root is absent before the run.

The boundary table is derived from the training panel only. No evaluation
reply enters it.

## Fixed construction

For each normalized TRAIN reply:

- remove one trailing corpus EOS/BOS marker;
- count terminal suffixes at depths `4..=32`;
- count every internal suffix at those depths as nonterminal.

At each generated token:

- use `Longest32To1` exact copy for body content and writer fallback;
- inspect only suffixes of the generated reply;
- examine suffix lengths from 32 down to 4;
- the first known suffix with terminal observations and no continuation emits
  decoder BOS;
- the first known suffix with any continuation observation continues;
- no known suffix continues.

The control shuffles terminal counts across suffix keys separately at each
depth, preserving the number of keys and terminal observations per depth while
leaving every nonterminal count fixed.

## Arms

For each artifact seed, on the fresh panel:

1. `longest32_no_boundary`;
2. `longest32_true_boundary`;
3. `longest32_shuffled_boundary`;
4. `reference_length_oracle`, reported only as the attainable completion
   ceiling.

The no-boundary arm must equal the existing `Longest32To1` evaluator row for
row.

## Steps

Run only after FIT and D67 GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-boundary
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-boundary/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2723
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BIN=$SHARED/local/deepseek/copy-boundary/target/release/cpu-copy-boundary-reply
GEN=$SHARED/local/deepseek/copy-boundary/generator-target/debug/heldout-panel

test ! -e "$OUT"
test "$(git -C "$SOURCE" rev-parse HEAD)" = 4832461ba66df263a9259b38aa97c5f73283c5e0
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = ecb0d8cbf8cccaab92faa6c5a99b9646eb7901a8b077783acbc4abbcbb91c869
test "$(sha256sum "$GEN" | cut -d' ' -f1)" = 1e5e9675191f226db8c487d55299d60806a8dfe45bf39334957aae928f22f8d5
test "$(git -C "$SOURCE" rev-parse 4832461ba66df263a9259b38aa97c5f73283c5e0:docs/labs/one-model-end-to-end/instrument)" = 869e65db6d0bb997a5f1094ab0de4efa1ac10c25
sha256sum -c <<EOF
a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377  $TOKENIZER
140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91  $TRAIN/tokens.u16
d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad  $TRAIN/response_mask.u8
d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5  $TRAIN/manifest.json
a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  $SEED1
da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  $SEED2
EOF

mkdir -p "$OUT/panels" "$OUT/reports"
"$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2723 protocol=2 \
  >"$OUT/panel.stdout.json"

jq -e '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2723 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
' "$PANEL/manifest.json" >/dev/null

PT=$(sha256sum "$PANEL/tokens.u16" | cut -d' ' -f1)
PM=$(sha256sum "$PANEL/response_mask.u8" | cut -d' ' -f1)
PF=$(sha256sum "$PANEL/manifest.json" | cut -d' ' -f1)

run() {
  name=$1 artifact=$2 expected=$3
  timeout 1800s env RAYON_NUM_THREADS=32 "$BIN" \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" \
    --expected-train-tokens 140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 \
    --expected-train-mask d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad \
    --expected-train-manifest d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 \
    --expected-panel-tokens "$PT" --expected-panel-mask "$PM" \
    --expected-panel-manifest "$PF" --expected-artifact "$expected" \
    --cap 64 --seed 191 >"$OUT/reports/$name.json"
}

run seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run seed1-repeat "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-repeat "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
```

Before reading either result file, the harness checks that:

- all four commands exited zero;
- each report has 200 or more reachable replies;
- the no-boundary arm equals the existing evaluator;
- each report names the same fresh-panel hashes and its expected artifact;
- no output root existed before generation.

The two runs per artifact are normalized by removing wall-time and
throughput fields, sorted with `jq -S`, and compared byte-for-byte.

## Minimum test power

- two public artifact seeds;
- one 128-conversation fresh evaluation panel, expected around 450 reachable
  references at the 64-token cap;
- at least 200 reachable replies per artifact and arm;
- at least 10 oracle-complete replies before the true arm can be assessed;
- four complete arms per artifact;
- fresh-process repeats of both artifact seeds;
- the no-boundary result must equal the established `Longest32To1` rows.

## Bar

On each artifact seed:

- if `reference_length_oracle.exact_reference < 10`, return
  `INSUFFICIENT_POWER`;
- true boundary exact replies must be at least
  `max(5, ceil(oracle_exact / 2))`;
- true boundary exact replies must beat both no-boundary and shuffled-boundary
  by `max(2, ceil(2*sqrt(candidate_only + control_only)))` with a positive sum;
- true-boundary premature BOS count must be at most 4;
- true-boundary correct body positions must be at least the no-boundary arm's;
- the shuffled table must move at least one terminal suffix key;
- true and shuffled arms must have different generated stopping decisions;
- both artifact seeds and fresh-process repeats must satisfy the result bars.

## Outcomes

- **KEEP:** all bars pass on both artifact seeds.
- **REJECT:** either seed completes but misses a bar.
- **INSUFFICIENT_POWER:** the oracle completes fewer than 10 fresh replies.
- **CONTROL_FAULT:** the shuffled table or its generated stopping behavior is
  vacuous, or a seed's control is otherwise invalid.
- **INSTRUMENT_FAULT:** input, artifact, panel, output or repeat identity drifts.

A miss closes this exact suffix-boundary construction, not geometric
termination generally.

## Cost and stop

- local CPU only, $0;
- four driver processes, at most 30 minutes total;
- at most 2 GiB peak RSS;
- stop on source, input, panel, artifact, control, parity, output or repeat
  drift;
- no post-observation feature, depth, threshold or bar change.

## Claim boundary

A pass establishes exact-memory recall of TRAIN reply endings with the existing
longest-copy body. It does not establish fresh wording, semantic correctness or
useful chat. A serving claim requires a later artifact-bound sidecar and exact
integer-session parity.

## Verdict

TEST FITNESS: PENDING
