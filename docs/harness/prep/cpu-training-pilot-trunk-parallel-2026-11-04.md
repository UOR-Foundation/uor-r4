# Parallel public 19-table CPU count driver

**Status:** FIT

## Question

Can the public 19-table count driver derive training rows and evaluate held-out
CPU/integer decisions with bounded Rayon workers while reproducing every
serial artifact, reading and checksum byte-for-byte, and does any bounded
worker count reduce wall time by at least 1.5 times on the Ryzen 9 9950X?

## Compute class

Local CPU development research under D84. No GPU, pod or paid API.

## Pins

| Input | Identity |
|---|---|
| implementation source | `ca6e1cbac7ec5d5b5a477cd8d36d579407c9201a` |
| driver binary | `8f46c82cee87568e146d0ab2319000da3c8704af2ec322018b5b937ceb5deac6` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.
The binary was built cleanly with
`CARGO_TARGET_DIR=target/deepseek/trunk-parallel-build-126 cargo build --locked --release -p uor-r4-cpu-train --bin cpu-trunk-count-driver`.

## Fixed run

Replay the accepted deepseek-125 split and dose exactly:

- 4,096 deterministically ordered training rows;
- evaluation stride 2, at most 512 held-out rows;
- seed 1;
- no-update, exact additive-count and shuffled-target arms;
- workers 1, 8, 16 and 32, each in a fresh output root.

Only `threads`, `wall_elapsed_ms` and `peak_rss_kib` may differ between worker
reports. Exact and shuffled artifact bytes, table readings, decision readings,
checksums, changed-row lists and all other report fields must be identical.

## Bar

Primary performance bar:

- at least one of 8, 16 or 32 workers completes in at most two-thirds of the
  one-worker wall time, a speedup of at least 1.5 times;
- every worker count satisfies all equivalence and resource gates below.

This is performance/instrument evidence only. A slower result closes no model
mechanism and says nothing about language capability.

## Instrument gates

- source and binary pins hold before and after each run;
- all four input hashes match the table above;
- fresh output roots and no source drift;
- exact and shuffled artifacts differ within every run;
- all 13 count tables increase by exactly 4,096 entries;
- independent full decoded table contents match every arm;
- codebook, routes, gates, recurrence and normalization sections are unchanged;
- every arm has zero query-coordinate, admitted-cell, top-8 and token
  mismatches;
- each report records its requested worker count;
- exact and shuffled artifacts are byte-identical across all four workers;
- normalized reports are identical across all four workers;
- every run stays below 600 seconds and 2,097,152 KiB peak RSS.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-parallel
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
PIPELINE=/home/callard/projects/codex-playground/uor-r4/local/deepseek/trunk-parallel/2026-11-04-attempt-1
BIN="$SOURCE/target/deepseek/trunk-parallel-build-126/release/cpu-trunk-count-driver"
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = \
  8f46c82cee87568e146d0ab2319000da3c8704af2ec322018b5b937ceb5deac6
test ! -e "$PIPELINE"
run() {
  workers=$1
  out="$PIPELINE/workers-$workers"
  /usr/bin/time -v timeout 600s "$BIN" \
    "$DATA/t16-frozen-zero-seed1-trunk.d11" \
    "$DATA/dev-tokens.u16" "$DATA/dev-response-mask.u8" \
    "$DATA/dev-manifest.json" "$out" \
    --expected-artifact-sha256 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --train-rows 4096 --evaluation-stride 2 --max-evaluation 512 \
    --seed 1 --workers "$workers" >/dev/null
  test "$(jq -er '.threads' "$out/report.json")" -eq "$workers"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 2097152
}
run 1
run 8
run 16
run 32
```

After the four runs, compare artifact hashes and normalized reports; write the
comparison and checksum inventory under the task evidence folder.

## Cost and stop

Local workstation only. Rust 1.97.1 on Linux 7.0.0-42-generic x86_64. At
registration the project filesystem had 508 GiB free and `/tmp` had 25 GiB
free. Four output roots retain about 700 MiB total. Each run has a 600-second
timeout and 2-GiB peak-RSS cap. Stop on cap failure, source drift, a worker
count mismatch, any artifact/decision/checksum mismatch, a non-vacuous control
failure or nested Rayon pools.

## Claim boundary

A positive supports only this driver's deterministic parallelism and measured
wall-time gain on this hardware and fixed workload. It is not fresh phrasing,
capability, count-training quality or GPU qualification.

## TEST FITNESS

TEST FITNESS: pending fresh-context non-author review.
