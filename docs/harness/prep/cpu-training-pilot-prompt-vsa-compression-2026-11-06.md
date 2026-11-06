# Pre-registration: compressed prompt VSA

Run ID: `cpu-training-pilot-prompt-vsa-compression-2026-11-06`

## Status

REGISTERED. Implementation and TEST FITNESS precede compute.

This is D84 CPU serving research. It tests a deterministic lossy compressed
representation of the exact deepseek-209 prompt selector. It is not a new
training-capability experiment or a serving-default change.

## Question

Can a compressed binary prompt signature selected on TRAIN alone preserve or
improve exact-reply, opening and body quality while reducing selection work?

## Fixed construction

The persisted `uor-r4.stack-prompt-reply-readout/1` schema remains unchanged.
The default selector remains the exact 4,096-bit vocabulary vector. Research
construction accepts a width divisible by 64 and one deterministic mapping:

```text
modulo(t, width)    = t % width
splitmix64(t, width) = splitmix64(t) % width
```

The SplitMix64 seed is the token id itself and the standard four mixing
constants are applied. Query and entry signatures use the same mapping. The
rational Jaccard ordering and lower-ordinal tie remain unchanged. Signature
bits are packed little-endian within each `u64` word.

Compression is lossy because distinct token ids can map to the same bit and
therefore change intersection and union cardinalities. No claim of exact
parity is made for the compressed arm.

## Calibration

Calibration uses only normalized TRAIN `tt-2711` prompt/reply entries. For each
prompt, compute:

```text
BLAKE3("deepseek-211-inner-v1" || each prompt token as u32 little-endian)
```

A prompt is in the inner held-out set when the digest's first byte is less than
51; all other prompts form the inner library. This split is fixed before
execution and does not read the evaluation panel.

Compare each candidate with the exact 4,096-bit selector over the same inner
library and held-out prompts. Rank candidates by:

1. selected-ordinal agreement;
2. selected-reply agreement;
3. smaller signature width;
4. fixed order: 256-modulo, 512-modulo, 1024-modulo, 1024-SplitMix64.

The winner is frozen before the fresh panel is opened.

## Arms

1. `compressed`: selected candidate, loaded from the unchanged sidecar;
2. `exact`: exact 4,096-bit vocabulary signature;
3. `cpu_oracle`: in-memory CPU `unigram_jaccard_ordinal_tie`.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2778, protocol 2, 128 conversations, generated
  only after FIT and GO;
- both public dual-c131 raw artifacts;
- 64-token reply cap;
- at least 200 reference-reachable panel rows per artifact;
- three alternating fresh-process runs per artifact and arm;
- unchanged deepseek-207 attempt-4 sidecars.

Fresh and repeat processes remove timing fields before equality checks. The
exact arm is the quality control; the CPU arm is the decision oracle only for
the exact representation.

## Instrument gate

- source, binary, generator, tokenizer, artifacts, TRAIN and sidecars match
  their pins;
- the seed-2778 panel and output root are absent before FIT and GO;
- the output signature width and mapping equal the recorded calibration
  winner on every run;
- exact emits every reachable row and matches CPU ordinals and reply SHA on
  every row;
- compressed emits every reachable row;
- the sidecar serializes and reloads byte-identically;
- API complete-hit, cap-fallback and identity tests pass.

Any failure is an instrument, power or drift fault, not a mechanism negative.

## Bar

For both artifacts:

- exact has zero premature BOS and 471/471 CPU ordinal/reply parity;
- compressed has zero premature BOS and emits all reachable rows;
- compressed `exact_reference`, `opening_correct` and
  `correct_body_positions` are each no lower than exact;
- compressed median selection throughput is at least 1.25x exact;
- compressed median selection throughput is at least 25,000 prompts/s;
- three compressed runs are identical after timing removal;
- sidecars remain byte-identical.

## Outcomes

- **KEEP** if every quality, throughput, identity and determinism condition
  passes on both artifacts.
- **REJECT** if the selected compressed representation loses any quality
  aggregate or misses either throughput floor. This closes only the selected
  lossy signature configuration.
- **CONTROL_FAULT** if the exact arm, inputs, sidecars or deterministic mapping
  drift.

## Cost

Local CPU only on the Ryzen 9 9950X. The fresh-panel run is capped at 30
minutes, 2 GiB peak RSS and 32 MiB retained evidence. Calibration and focused
tests are capped at five minutes.

## Base and inputs

- base: `4d1e705aa`;
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

`local/deepseek/prompt-vsa-compression/2026-11-06`

Fresh panel: `.../panels/dd-2778`

Both roots are absent at registration.

## TEST FITNESS

Pending implementation pins, focused tests, source review and a check that the
fresh panel and output roots remain absent.

## Pre-run GO

Pending after FIT.
