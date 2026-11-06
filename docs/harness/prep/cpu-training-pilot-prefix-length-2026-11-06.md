# Pre-registration: prefix-length completion

Run ID: `cpu-training-pilot-prefix-length-2026-11-06`

## Status

REGISTERED. Implementation and TEST FITNESS precede compute.

## Question

Can the nearest stored reply's length terminate a four-token retrieved-prefix
continuation into a useful complete response?

## Fixed construction

Retrieve raw unigram-Jaccard top-1 with lower-ordinal ties. Force four tokens
through the exact CPU FBS state, generate greedily, and stop at the smallest of
the 64-token cap, the retrieved reply length, or the existing model stop.

Arms: model-only cap, model-only at retrieved length, retrieved-prefix length,
shuffled-prefix length, full retrieval, and reference-length oracle. The
retrieved length never reads the evaluation reference.

## Data split

TRAIN `tt-2711`; fresh protocol-2 seed 2782; both public artifacts; at least 200
reachable rows; three fresh processes per artifact.

## Instrument gate

Source, artifact, tokenizer, sidecar, TRAIN and panel pins match; panel/output
roots absent at registration; all selected lengths are positive and bounded;
every arm emits all reachable rows; repeated decisions match after timing
removal.

## Bar

For both artifacts, retrieved-prefix length completion beats model-only length
completion and shuffled-prefix length completion on paired exact replies by
two sigma, retains model-only body positions, emits every reachable row, and
repeats deterministically.

## Outcomes

**KEEP** if every bar passes; **REJECT** if the mechanism is active but a
quality comparison fails; **INSUFFICIENT_POWER** below 200 reachable rows;
**CONTROL_FAULT** on drift, invalid lengths, vacuous shuffle or
nondeterminism.

## Stop rule

No retune of K or the length rule after seed 2782. Stop on any prohibited path
or failed instrument gate.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `86996114f`;
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

## Output

`local/deepseek/prefix-length/2026-11-06`; panel `.../panels/dd-2782`.

## Steps

Build `cpu-prompt-prefix-continuation` with bounded arms; generate seed 2782;
run three alternating processes per artifact; seal reports, timings, panel and
source pins; fill implementation pins before GO.

## Minimum test power

- two artifacts;
- at least 200 reachable rows;
- three process runs per artifact;
- model-only, retrieved, shuffled, full and reference controls;
- paired exact/body comparisons and deterministic completion.

## TEST FITNESS

TEST FITNESS: PENDING - implementation pending.

## Pre-run GO

Pending after FIT.
