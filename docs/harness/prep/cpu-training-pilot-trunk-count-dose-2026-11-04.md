# 4,096-row public 19-table count training

**Status:** PENDING

## Question

On the fixed 512-row document-disjoint evaluation panel, does an exact additive
dose of 4,096 training rows improve the public 19-table trunk's top-1 over no
update and deterministic shuffled targets, without losing admitted or top-8
coverage?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `1ca202ef217918d4c8444499ebaabc3a91e1878a` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.

## Fixed split and dose

Replay deepseek-124 exactly:

- 200 training documents / 7,080 eligible training targets;
- 193 evaluation documents / 512 evaluation targets at stride 2;
- the first 4,096 deterministically ordered training rows.

The dose is the merged driver's existing maximum and was chosen from that
operational bound, not from observing the 128-row outcome.

## Arms

1. no update;
2. exact additive counts from 4,096 training rows;
3. deterministic, non-identity shuffled targets over the same prefixes.

Every arm evaluates the same 512 held-out rows.

## Bar

Pair exact and each control row by row:

- exact top-1 must exceed no update;
- exact top-1 must exceed shuffled targets;
- each gain must be at least `max(2, ceil(2 * sqrt(b + c)))`, where `b` and `c`
  are the paired discordant top-1 rows;
- exact admitted and top-8 counts must not be lower than either control.

All instrument gates must pass. A positive is bounded evidence for this public
panel and dose. A negative closes only this 4,096-row additive configuration.

## Instrument gates

- source pin, artifact, corpus and manifest hashes;
- fresh output/build roots and no source drift;
- exact and shuffled artifacts differ;
- all 13 count tables increase by exactly 4,096;
- independent full decoded table contents match each arm;
- codebook, routes, gates, recurrence and normalization sections unchanged;
- zero query-coordinate, admitted-cell, top-8 and token mismatches against
  integer serving;
- deterministic artifact/report repeat;
- peak RSS at most 2,097,152 KiB and wall at most 600 seconds per run.

## Steps

The final FIT revision must include the exact executable build/run recipe,
binary hash receipt, output checksum inventory and repeat canonicalization.

## Minimum test power

The fixed panel has 512 held-out rows, above D22's 200-row floor. One public
artifact is sufficient for this bounded host comparison because the mechanism
and parity were established on that artifact in deepseek-124.

## Cost and stop

Local workstation only. Final storage/time bounds are fixed before FIT. Stop on
cap failure, source drift, a required code change, count/parity mismatch,
nondeterminism or an evaluation-tuned dose.

## Claim boundary

A positive supports only this additive count dose on the fixed public DEV split.
It does not establish fresh phrasing, complete replies, useful chat or GPU
qualification.

## TEST FITNESS

TEST FITNESS: PENDING
