# Pre-registration: decoupled output-table readout refit

Run ID: `cpu-training-pilot-readout-refit-2026-11-06`

## Status

DRAFT pending exact-head FIT and pre-run GO. No fresh DEV panel has been
generated or read.

## Question

Can the nearest-token output table be refit independently of the fixed input
embedding and Hamming-read path?

## Whole-project chain and decision

The public raw FBS artifacts still share one input/output table. The prior CPU
writer and branch screens exercised admission and complete-reply selection but
did not isolate whether the output rows are a measurable readout bottleneck.
This unit freezes the exact input, Hamming and admission path, captures the
query immediately before every TRAIN reply target, and changes only the output
rows. It therefore answers whether the next useful work is a two-table
artifact/interface design or a different readout objective.

- `FINAL_KEEP`: authorize a follow-up two-table artifact/export design. It does
  not change serving in this unit.
- `FINAL_REJECT`: close coordinate-wise median and mean-centroid output refit
  at eight observations; do not infer failure of the geometric readout.
- `FINAL_INSUFFICIENT_POWER`: make no mechanism decision and do not reuse seed
  2789 for a retuned design.
- `FINAL_CONTROL_FAULT`: repair or repeat the instrument; make no mechanism
  decision.

## Fixed construction

Teacher-force each public FBS artifact over every masked TRAIN `tt-2711`
target. For each target, capture the exact Q12 query after the complete causal
prefix and before the target decision. Keep the artifact's input table,
Hamming reads, trunk and admission path unchanged.

Refit only output rows with at least eight TRAIN observations. Each output
coordinate is:

- the primary median: the lower midpoint of the two central sorted query
  coordinates, an exact integer point in the L1-minimizing interval;
- the comparison mean centroid: nearest integer, ties away from zero.

Under-observed rows stay byte-identical.

The matched control cyclically permutes target labels only among observations
whose original targets meet the floor. It chooses the deterministic offset
that changes the most assignments. Per-target observation counts and the
global query multiset stay fixed.

The shared-table diagnostic patches the same median table into the complete
`UORLUT01` artifact. It must recapture queries because this arm changes the
input, Hamming-read and output paths. It is diagnostic only and is not the
primary decoupled path.

## Arms

1. original shared table;
2. decoupled identical table;
3. output median refit;
4. output mean-centroid refit;
5. shuffled-target matched control;
6. shared-table median refit diagnostic.

## Data split

TRAIN `tt-2711`:

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- BOS: `0`.

Fresh evaluation is `cell=dev,dev`, seed 2789, protocol 2, 128
conversations. `panels/dd-2789` and `reports/dd-2789` are absent at
registration and are generated only after FIT and GO. Only the TRAIN-only
preflight files exist under the run root. The code enforces seed 2789, eight
observations, the tokenizer digest, BOS and the panel shape; these are not
runtime-tunable flags.

## Base artifacts

- seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are the public 4,096-token, width-256, context-192, raw FBS artifacts.

## Hardware

- AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- Linux `7.0.0-42-generic` x86_64;
- Rust `1.97.1`;
- 59 GiB installed memory;
- 269 GiB free storage at preparation.

The local resource record is
[`resource-ledger.json`](../../labs/cpu-training-pilot-readout-refit-2026-11-06/resource-ledger.json).

## TRAIN-only preflight

On both artifacts, all 5,114 masked TRAIN targets expanded to 5,114 causal
observations and reached the artifact context. Independent writer scoring
reproduced every captured artifact decision with zero changed decisions.

The fixed eight-observation floor retains 146 token rows and 4,142
observations per artifact. The matched shuffle changes 4,127 TRAIN
assignments. The two captures complete in 5.37 seconds and 5.35 seconds with
16 Rayon threads; peak RSS is 66,680 KiB and 66,564 KiB.

Thresholds 12, 16, 32 and 64 retain fewer than 100 rows. The eight-observation
floor is therefore the strongest tested floor that clears the row-power bar.

## Instrument gate

Inputs and hashes match. TRAIN and panel must use tokenizer
`a9fb13f7…`, BOS 0, and exactly one causal sample per manifest response
target. The scorer must reproduce every captured TRAIN and panel artifact
decision before it participates in a comparison. The shuffle must change
targets while preserving target counts and the global query multiset. Each arm
shares the panel, artifact and target rows.

The aggregator requires four reports: two artifacts with two fresh-process
reports each. The normalized semantic report, which removes all timing and
resource fields, must match within each artifact. Only the aggregator can emit
the final KEEP/REJECT/INSUFFICIENT_POWER/CONTROL_FAULT result.

The fresh panel must report `cell=dev,dev`, seed 2789, protocol 2, 128
conversations, BOS 0 and the pinned tokenizer. `panels/dd-2789` is absent
immediately before generation, and `reports/dd-2789` is absent at GO.

## Power

Per artifact:

- at least 100 output rows at eight observations;
- at least 1,000 refit observations;
- at least 1,000 fresh teacher-forced targets;
- two artifacts and two fresh-process repeats.

## Metrics

For every arm: admitted targets, top-1/top-5/top-8, target L1 sum, admitted
target-rank sum, changed decisions, decision hash and table hash. Fit output
also records changed coordinates and L1 row displacement. Unigram and T2
successor top-1 counts are reported on the same panel.

## Bar

Decoupled-identical must equal original captured decisions exactly. On both
artifacts, the median output refit must:

- strictly beat original and shuffled-control top-1 under
  `candidate_sum > control_sum` and
  `candidate_only >= max(2, ceil(2*sqrt(candidate_only + control_only)))`;
- not reduce top-8;
- produce matching normalized semantic reports in two fresh processes.

Mean centroid and shared-table refit are reported controls and do not decide
the primary bar.

## Outcomes

- **FINAL_KEEP** if every primary bar passes on both artifacts and both
  repeats.
- **FINAL_REJECT** if the instrument and power gates pass and a primary
  comparison fails.
- **FINAL_INSUFFICIENT_POWER** below a fixed floor.
- **FINAL_CONTROL_FAULT** on identity failure, vacuous shuffle, input drift or
  nondeterminism.

## Stop rule

Do not retune the eight-observation floor, rounding, shuffle or comparison rule
after seed 2789 opens. Stop on drift, unsupported artifact shape, float or
dense arithmetic, less than 100 refitted rows, less than 1,000 refit
observations, less than 1,000 panel targets, a vacuous control or
nondeterminism.

## Cost

Local CPU only. Maximum one 30-minute wall envelope, 2 GiB peak RSS and 32 MiB
retained evidence. The prospective ledger charges 1,140 seconds: 180 build,
120 preparation, 30 measured preflight, 360 for four fresh evaluations, 120
controls, 150 evidence/delivery and 180 retry allowance. No GPU or paid API is
used.

## Steps

1. Verify source and preflight pins, then release-build:

```sh
BIN="$PWD/target/deepseek/readout-refit/release/cpu-readout-refit"
CARGO_BUILD_JOBS=32 CARGO_TARGET_DIR=target/deepseek/readout-refit \
  cargo build --locked --release -p uor-r4-cpu-train \
  --bin cpu-readout-refit
sha256sum "$BIN"
```

2. Verify the empty fresh paths and pinned generator:

```sh
MAIN=/home/callard/projects/codex-playground/uor-r4
OUT="$MAIN/.worktrees/deepseek-readout-refit/local/deepseek/readout-refit/2026-11-06"
PANEL="$OUT/panels/dd-2789"
REPORTS="$OUT/reports/dd-2789"
GEN="$MAIN/local/deepseek/copy-boundary/generator-target/debug/heldout-panel"
TOKENIZER="$MAIN/local/deepseek/m2-evidence/tokenizer.json"
test ! -e "$PANEL"
test ! -e "$REPORTS"
sha256sum "$GEN" "$TOKENIZER"
test "$(sha256sum "$GEN" | cut -d' ' -f1)" = \
  1e5e9675191f226db8c487d55299d60806a8dfe45bf39334957aae928f22f8d5
test "$(sha256sum "$TOKENIZER" | cut -d' ' -f1)" = \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377
```

3. Generate the fresh panel after FIT and GO:

```sh
"$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2789 protocol=2 \
  >"$OUT/panel-2789.stdout.json"
jq -e '
  .seed == 2789 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128
  and .tokenizer.sha256 == "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
  and .tokenizer.bos_id == 0
' "$PANEL/manifest.json" >/dev/null
```

4. Pin the generated panel and create the report root:

```sh
PANEL_TOKENS_SHA=$(sha256sum "$PANEL/tokens.u16" | cut -d' ' -f1)
PANEL_MASK_SHA=$(sha256sum "$PANEL/response_mask.u8" | cut -d' ' -f1)
PANEL_MANIFEST_SHA=$(sha256sum "$PANEL/manifest.json" | cut -d' ' -f1)
mkdir "$REPORTS"
```

5. Run both artifacts and both fresh-process repeats:

```sh
TRAIN="$MAIN/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711"
SEED1="$MAIN/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut"
SEED2="$MAIN/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut"
SEED1_SHA=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
SEED2_SHA=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run() {
  name=$1 artifact=$2 artifact_sha=$3
  RAYON_NUM_THREADS=16 "$BIN" run \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" \
    --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 \
    --expected-train-tokens 140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 \
    --expected-train-mask d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad \
    --expected-train-manifest d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 \
    --expected-panel-tokens "$PANEL_TOKENS_SHA" \
    --expected-panel-mask "$PANEL_MASK_SHA" \
    --expected-panel-manifest "$PANEL_MANIFEST_SHA" \
    --expected-artifact "$artifact_sha" \
    >"$REPORTS/$name.json"
}
run seed1 "$SEED1" "$SEED1_SHA"
run seed1-repeat "$SEED1" "$SEED1_SHA"
run seed2 "$SEED2" "$SEED2_SHA"
run seed2-repeat "$SEED2" "$SEED2_SHA"
```

6. Aggregate only after all four processes finish:

```sh
"$BIN" aggregate \
  "$REPORTS/seed1.json" "$REPORTS/seed1-repeat.json" \
  "$REPORTS/seed2.json" "$REPORTS/seed2-repeat.json" \
  >"$REPORTS/result.json"
```

7. Seal panel, reports, source and binary pins:

```sh
(
  cd "$OUT"
  find . -type f ! -name SHA256SUMS -print0 | sort -z |
    xargs -0 -r sha256sum >SHA256SUMS.tmp
  mv SHA256SUMS.tmp SHA256SUMS
  sha256sum -c SHA256SUMS
)
```

## Source pins before FIT

- implementation commit: `7a12427b2`;
- module:
  `21abcad62049f20e5c8ec6d0b4d2393642bc560655386c5c9219578a5a704eae`;
- driver:
  `0d6903edf98617a7ec799a85391c4328dfddcd1fe521d2ecf8593ef62a49ec37`;
- library:
  `ac1423f5d48e73a3425737395b5772326e990b857818c6a268f00ed9f716ddd2`;
- corpus:
  `396bfc76c6548c3298779f208af539615eea5643f172458d8da074e3a71116db`;
- train patch:
  `85840169921e2611b1482ca2c2a47fb41c54f8513b37ce57d9224e1fe2c63b2d`;
- release binary:
  `5bb7b5701e24939f81c7771288ce33f744db51162594f728c133a2d8d2b9c5c9`;
- generator source commit: `e1ad88afc`;
- generator binary:
  `1e5e9675191f226db8c487d55299d60806a8dfe45bf39334957aae928f22f8d5`.

## TEST FITNESS

TEST FITNESS: PENDING exact-head non-author review.

## Pre-run GO

PENDING exact-head non-author review after FIT.
