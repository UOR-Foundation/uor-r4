# Pre-registration: integer whole-reply neighbor ranker

Run ID: `cpu-training-pilot-prompt-reply-ranker-2026-11-06`

## Status

REGISTERED. Implementation and TEST FITNESS precede compute.

This is D84 CPU research. It learns a small integer ranking function over
existing TRAIN prompts and complete replies. It does not synthesize new text,
change `UORLUT01`, or alter the serving default.

## Question

Can TRAIN-only leave-one-out supervision choose a better complete stored reply
from the raw Jaccard top-16 than raw unigram Jaccard alone on a fresh panel?

## Fixed construction

The ranker receives a query prompt and at most 16 TRAIN prompt neighbors ranked
by exact unigram-set Jaccard. The top-1 raw Jaccard candidate is the primary
control. Candidate order is the raw rational rank with lower ordinal on ties.

The eight features are:

1. `bias`;
2. `unigram_jaccard_q16`;
3. `query_coverage_q16`;
4. `candidate_coverage_q16`;
5. `reply_prompt_overlap_q16`;
6. `writer_distance_advantage`;
7. `reply_frequency`;
8. `reply_length_delta`.

Q16 values are floor-rounded and capped to 65,536. Writer advantage is the
closest candidate query distance minus the candidate distance, clipped to
`[-65,535, 0]`. Reply frequency counts identical normalized TRAIN replies.
Length delta is the integer mean reply length minus the candidate length.

For every TRAIN row, exclude that row from the candidate set and retain a group
only when its normalized reference reply occurs in the top 16. Positives are
candidates whose normalized reply equals the row reference. A deterministic
pairwise perceptron visits groups in ordinal order and updates
`positive_features - negative_features` whenever a positive does not strictly
outrank the best negative. It runs eight epochs at unit learning rate with
weights bounded to +/-65,536.

The shuffled-target control preserves every group, candidate and positive
count, but selects each group's positive candidates by SplitMix64 seed
`213` over its candidate ordinals. It must change at least one label set and
produce weights distinct from the true fit.

## Arms

1. `learned_ranker`;
2. `raw_unigram_jaccard`;
3. `shuffled_ranker`;
4. `query_geometry`;
5. `fixed_ordinal`;
6. `reference_membership_oracle`.

## Data split

TRAIN `tt-2711` supplies all ranker fitting. A fresh `cell=dev,dev`, seed 2779,
protocol-2 panel is generated only after FIT and GO. No evaluation reply enters
feature fitting, candidate construction or control training.

All panel prompts must be absent from TRAIN by exact token sequence; at least
200 replies must be reference-reachable; both public dual-c131 artifacts run.

## Instrument gate

- source, artifacts, tokenizer, TRAIN and panel pins match;
- the seed-2779 panel and output root are absent at registration;
- TRAIN yields at least 96 supervised groups and 24 baseline-wrong groups;
- the label shuffle changes at least one group and yields distinct weights;
- each raw top-16 contains at most 16 unique entry ordinals;
- all feature values and weights fit their declared integer bounds;
- three alternating fresh processes per artifact and arm finish;
- raw and learned selectors emit every reachable row with zero premature BOS;
- decisions and weights repeat after timing removal;
- sidecars remain byte-identical.

Any failure is an instrument or power fault, not a ranker negative.

## Bar

For both artifacts:

- learned exact replies exceed raw Jaccard by the paired two-sigma rule;
- learned correct body positions do not fall below raw;
- learned exact replies exceed shuffled-target ranker by the paired two-sigma
  rule;
- learned emits every reachable row with zero premature BOS;
- fresh top-16 candidate-membership oracle reaches at least 20 exact replies;
- all runs repeat after timing removal.

## Outcomes

- **KEEP** if every bar and instrument condition passes on both artifacts.
- **REJECT** if supervision is valid and the learned ranker misses either
  primary paired exact comparison or loses body positions.
- **INSUFFICIENT_POWER** if TRAIN has fewer than 96 supervised groups, fewer
  than 24 baseline-wrong groups, the fresh panel has fewer than 200 reachable
  rows, or the candidate oracle has fewer than 20 exact replies.
- **CONTROL_FAULT** on a vacuous shuffle, input drift, nondeterminism, emission
  failure or unexpected premature BOS.

## Stop rule

Do not retune TOP_K, features, epochs, learning rate or the pair rule after
observing seed 2779. Stop on any instrument/power failure or any prohibited
float, dense matrix operation, serving-kernel change or Claude-owned source
change.

## Cost

Local CPU only. Cap: 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `dc2a05d72`;
- TRAIN tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- TRAIN mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- TRAIN manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- sidecar seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- sidecar seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

## Panel and output

Sidecars remain unchanged. Candidate prompts/replies are loaded from the bound
TRAIN sidecars. The fresh panel is generated at
`local/deepseek/prompt-reply-ranker/2026-11-06/panels/dd-2779`; this root and
the final output root are absent at registration.

## Steps

Run only after FIT and GO. The implementation binary writes no artifact except
sealed JSON evidence; all output roots must be new.

```sh
set -euo pipefail
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE=$SHARED/.worktrees/deepseek-prompt-reply-ranker
OUT=$SHARED/local/deepseek/prompt-reply-ranker/2026-11-06
PANEL=$OUT/panels/dd-2779
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
test ! -e "$OUT"
mkdir -p "$OUT/panels" "$OUT/reports"
```

The implementation binary is `cpu-prompt-reply-ranker`; its exact command and
source/binary pins are added after implementation review and before GO.

## Minimum test power

- both public artifacts;
- at least 96 TRAIN supervised groups;
- at least 24 baseline-wrong TRAIN groups;
- at least 200 fresh reference-reachable rows;
- at least 20 exact rows in the fresh top-16 candidate oracle;
- 16 candidates per query and three alternating process runs;
- paired learned-vs-raw and learned-vs-shuffled exact comparisons;
- body non-fall, full emission and zero premature BOS.

## TEST FITNESS

TEST FITNESS: PENDING - implementation and exact-head test review pending.

## Pre-run GO

Pending after FIT.
