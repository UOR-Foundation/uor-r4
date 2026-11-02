# ORDER: CPU panel admission and exact-export gate, deepseek-3

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `ee534711f` from fresh `origin/main`.

## Goal

Close the remaining #3505 checkpoint-control gap by adding one deterministic panel evaluator that reports rank-margin loss beside target admission, top-1, top-8, and the exact difference between the CPU forward and the serving integer session on the same artifact and rows. Add a matched admission gate that refuses a checkpoint when panel admission decreases.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-3/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`

No serving-source edit, corpus run, GPU run, discrete-path revival, language claim or Claude-owned path.

## Cancel probe

If the supported CPU FBS forward and `IntegerStackSession::step_native` do not expose the same complete query, token and kept-cell vectors on finite artifacts, stop this unit and preserve the first mismatching row. No approximation may stand in for parity.

## Done when

- A panel evaluator runs the same `RankMarginSample` rows through the CPU rank-margin forward and the serving integer session, with one fresh stream per row.
- It records exact query-coordinate, top-1/token and kept-cell mismatch counts, artifact digest, sample count, target admission, top-1, top-8 and summed rank margin.
- A checkpoint evaluated from artifact bytes and its resumed trainer produce byte-identical panel readings.
- A matched comparison reports admitted/top-1/top-8/margin/export-gap deltas and a gate refuses any candidate whose panel target-admission count falls below the baseline.
- The no-change control accepts with all deltas zero; at least one trained checkpoint and one constructed admission-loss candidate exercise accept and reject.
- Tests cover empty panels, malformed samples, unsupported serving artifacts, deterministic evaluation and exact serving parity.
- A release benchmark records panel rows/s and peak RSS on the Ryzen 9 9950X.

## Evidence

The unit remains component evidence on finite fixtures. GPU remains the qualification authority, and no complete-reply, fresh-phrasing or corpus-training claim follows.
