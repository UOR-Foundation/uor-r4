# Pre-registration: decoupled output-table readout refit

Run ID: `cpu-training-pilot-readout-refit-2026-11-06`

## Status

DRAFT pending exact-head FIT and pre-run GO. No fresh DEV panel has been
generated or read.

## Question

Can the nearest-token output table be refit independently of the fixed input
embedding and Hamming-read path?

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
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

Fresh evaluation is `cell=dev,dev`, seed 2789, protocol 2, 128
conversations. `panels/dd-2789` and `reports/dd-2789` are absent at
registration and are generated only after FIT and GO. Only the TRAIN-only
preflight files exist under the run root.

## Base artifacts

- seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are the public 4,096-token, width-256, context-192, raw FBS artifacts.

## TRAIN-only preflight

On both artifacts, all 5,114 masked TRAIN targets reached the artifact context
and replay. The fixed eight-observation floor retains 146 token rows and 4,142
observations per artifact. The matched shuffle changes 4,127 TRAIN
assignments. The two captures complete in 5.41 seconds and 5.35 seconds with
16 Rayon threads; peak RSS is 66,612 KiB and 66,076 KiB.

Thresholds 12, 16, 32 and 64 retain fewer than 100 rows. The eight-observation
floor is therefore the strongest tested floor that clears the row-power bar.

## Instrument gate

Inputs and hashes match. Every corpus prefix is at most the artifact context.
Decoupled-identical must reproduce every original decision hash. Both corpus
captures expand to exactly one observation per masked target. The shuffle
must change targets while preserving target counts and the global query
multiset. Each arm shares the panel, artifact and target rows. Run repeats must
match after removing timing.

The fresh panel must report `cell=dev,dev`, seed 2789, protocol 2, 128
conversations and the pinned tokenizer. `panels/dd-2789` is absent immediately
before generation, and `reports/dd-2789` is absent at GO.

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

Decoupled-identical must equal original decisions exactly. On both artifacts,
the median output refit must:

- strictly beat original and shuffled-control top-1 under
  `candidate_sum > control_sum` and
  `candidate_only >= max(2, ceil(2*sqrt(candidate_only + control_only)))`;
- not reduce top-8;
- repeat byte-identically after timing fields are removed.

Mean centroid and shared-table refit are reported controls and do not decide
the primary bar.

## Outcomes

- **KEEP** if every primary bar passes on both artifacts.
- **REJECT** if the instrument and power gates pass and a primary comparison
  fails.
- **INSUFFICIENT_POWER** below a fixed floor.
- **CONTROL_FAULT** on identity failure, vacuous shuffle, input drift or
  nondeterminism.

## Stop rule

Do not retune the eight-observation floor, rounding, shuffle or comparison rule
after seed 2789 opens. Stop on drift, unsupported artifact shape, float or
dense arithmetic, less than 100 refitted rows, less than 1,000 refit
observations, less than 1,000 panel targets, a vacuous control or
nondeterminism.

## Cost

Local CPU only. Maximum 30 minutes, 2 GiB peak RSS and 32 MiB retained
evidence.

## Steps

1. Release build:

```sh
CARGO_BUILD_JOBS=32 CARGO_TARGET_DIR=target/deepseek/readout-refit \
  cargo build --release -p uor-r4-cpu-train --bin cpu-readout-refit
```

2. Fresh panel after FIT and GO:

```sh
cargo build --locked \
  --manifest-path docs/labs/one-model-end-to-end/instrument/Cargo.toml \
  --features panel-writer --bin heldout-panel

heldout-panel out=PANEL tokenizer=TOKENIZER \
  cell=dev,dev conversations=128 seed=2789 protocol=2
```

3. Evaluate each artifact:

```sh
RAYON_NUM_THREADS=16 cpu-readout-refit run \
  TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
  PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT \
  --seed 2789 --min-observations 8 \
  --expected-train-tokens 140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 \
  --expected-train-mask d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad \
  --expected-train-manifest d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 \
  --expected-panel-tokens PANEL_TOKENS_SHA256 \
  --expected-panel-mask PANEL_MASK_SHA256 \
  --expected-panel-manifest PANEL_MANIFEST_SHA256 \
  --expected-artifact ARTIFACT_SHA256
```

## Source pins before FIT

- implementation commit: `af68e98f4`;
- module:
  `21abcad62049f20e5c8ec6d0b4d2393642bc560655386c5c9219578a5a704eae`;
- driver:
  `92b6b8a5732683b946997e6d21f488648750423d3d0e43a2e44801c26bf5cfea`;
- library:
  `ac1423f5d48e73a3425737395b5772326e990b857818c6a268f00ed9f716ddd2`;
- corpus:
  `02d67bdfb792cfea93545fbb42ccf30e0ac01dec16cfaefb5581246a23b191b5`;
- train patch:
  `85840169921e2611b1482ca2c2a47fb41c54f8513b37ce57d9224e1fe2c63b2d`;
- release binary:
  `e537b82b3b2f5855dec99b83dd26d9529c5c1bef3770eae88750f411a9976ece`.

4. Repeat each artifact in a fresh process, remove timing fields, and compare
   the reports byte-for-byte.
5. Seal the panel, reports, timing, source and binary pins, then deliver
   through exact-head review and the protected merge queue.

## TEST FITNESS

TEST FITNESS: PENDING exact-head non-author review.

## Pre-run GO

PENDING exact-head non-author review after FIT.
