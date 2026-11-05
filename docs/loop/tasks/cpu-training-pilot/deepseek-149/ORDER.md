# ORDER: cpu-training-pilot, deepseek-149 (hierarchical copy-key depth)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `4e42ffca5`.

## Goal

Test whether exact copy retrieval improves when the Hamming read backs off from
the longest available suffix key, depth four to one, instead of fixing one key
for every position. This is a data-only geometric read screen that decides
whether Claude's next copy-read unit should extend the trained key beyond depth
two.

This is D84 CPU development research. It changes no float training, integer
serving, CUDA or Claude-owned source.

## Fixed construction

For an eligible masked target at corpus position `p`, the query is the suffix
ending at `p - 1`. At depth `d`, its key is:

```text
query_d(p) = tokens[p - d .. p]
key_d(s)   = tokens[s - d .. s]
```

A candidate at `s < p` matches when the slices are equal. The latest matching
`s` predicts `tokens[s]`; the target is `tokens[p]`. Search stays inside the
same BOS-delimited document and the same 192-token scoring window.

Compare these fixed and deterministic selectors:

- fixed depths `1`, `2`, `3` and `4`;
- `k2>k1`: latest depth-2 candidate, otherwise latest depth-1 candidate;
- `longest`: latest candidate at the greatest available depth from four to one;
- `shortest`: latest depth-1 candidate (the fixed-depth control);
- a seeded random-depth selector over available depths, as the non-vacuous
  matched control.

The public DEV result is an instrument oracle. The implementation must first
reproduce Claude-267's fixed-depth counts: 8,243 copyable positions, depth-1
6,306 matches / 3,859 right, depth-2 4,383 / 3,506, with a 192-token context.
Any mismatch stops the run as an instrument fault.

## Panels

1. Public M1 DEV, for exact parity with the registered Claude-267 counts.
2. `train-control-v3`: 64 m-world conversations, train phrasing and value.
3. `dev-panel-v3`: 16 m-world conversations, dev phrasing and value.

Panels 2 and 3 are data-only development panels. They do not qualify language
or model capability.

## Bar

The candidate is `longest`.

- It must beat `k2>k1` on true target correctness on public DEV and
  `dev-panel-v3`.
- The gain must pass the project's paired two-sigma rule over eligible target
  positions: candidate sum greater than control sum and candidate-only
  disagreements at least `max(2, ceil(2*sqrt(control-only)))`.
- It must beat the seeded random-depth selector by the same rule.
- Every fixed-depth and selector arm must account for every eligible position,
  and a fresh one-worker repeat must match.

## Stop rules

Stop on a public-DEV parity mismatch, input/hash drift, nondeterminism,
out-of-window candidate leakage, a vacuous random control, or any need to
change serving or Claude-owned source.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `.gitattributes`
- `docs/harness/prep/cpu-training-pilot-copy-key-depth-2026-11-05.md`
- `docs/labs/cpu-training-pilot-copy-key-depth-2026-11-05/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-149/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`

## Done when

1. The exact copy-key oracle, boundary/window checks, accounting and controls
   have tests.
2. A fresh FIT and GO precede the registered CPU run.
3. Public DEV reproduces Claude-267's fixed-depth figures before any candidate
   reading is accepted.
4. The result is sealed, classified and delivered through protected main.

