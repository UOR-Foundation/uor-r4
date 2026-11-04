# Target point medoids on unseen opening keys

**Status:** PENDING

## Question

Can a deterministic target-level geometric medoid learned from training-only
point rows improve unseen-key opening transfer over point-off and a matched
shuffled-target medoid on both public model seeds, while preserving exact CPU
and integer-serving decisions through the complete Hamming, turn-memory and
point path?

This is D84 CPU development research on the Ryzen 9 9950X. It is not GPU
qualification or language capability.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API or
external model execution.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `5b3b8f361abf27da25c2764379445a5f14f22a3f` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Fixed split

Replay deepseek-104/106 exactly: even document ordinals train, odd document
ordinals evaluate, 8 training keys and 8 disjoint evaluation keys, up to 8
examples per key, with 60 training and 60 evaluation examples. The selected key
sets and zero exact-key overlap are checked before training.

The opening key is the first eight prompt tokens. Opening/reply memory is
derived only from training examples at key window 192 and reply turn 8. Every
evaluation opening key must miss that exact memory table.

## Training procedure

Build the existing production raw point rows from training prompts:

- step each training prompt through the exact CPU FBS path;
- store the final pre-memory Q12 query and first reply token;
- coalesce exact duplicate points by commonest target, ties to the smaller id;
- sort by point bytes.

For each distinct target in those rows, choose an observed member minimizing the
exact sum of L1 distances to all members of that target group. Break equal
totals by lexicographic point bytes and then target id. Store one medoid per
target in the existing point-key artifact.

The treatment carries the same memory and Hamming records as the controls.
Point-off removes only the point table. Raw-row nearest and raw-row vote at
`k = 8` are descriptive controls. The matched shuffled-target medoid preserves
the point multiset and target counts while permuting target assignments
deterministically. If the first permutation is unchanged, rotate once; refuse
an unchanged or all-equal control.

## Arms

Evaluate the same 60 held-out examples through:

1. point-off;
2. raw-row nearest;
3. raw-row vote at `k = 8`;
4. target medoid;
5. shuffled-target medoid.

Every arm runs opening and complete teacher-forced evaluation plus the complete
served-forward audit.

## Bar

For each seed, pair the medoid and each primary control by evaluation row:

- medoid opening top-1 must exceed point-off;
- medoid opening top-1 must exceed shuffled-target medoid;
- each paired gain must be at least
  `max(2, ceil(2 * sqrt(b + c)))`, where `b` is medoid-only correct and `c` is
  control-only correct;
- medoid complete teacher-forced top-1 must not be lower than either primary
  control.

Both seeds must pass. All arms must preserve exact opening distance for the
point-fallback invariant.

## Instrument gates

- Input hashes, fixed key selection, zero key overlap and every held-out
  exact-memory miss check.
- At least two medoid targets.
- Medoid rows are observed training points, deterministic across two builds.
- Shuffled medoids preserve the point multiset and target counts and change the
  served image.
- Every arm reports non-zero gated/point-fallback exposure where applicable.
- Zero query-coordinate, token and kept-cell mismatches against
  `IntegerStackModel` for every arm and seed.
- Seed-1 repeat reproduces every non-timing field.

## Steps

The following runs only after FIT. Build once in a fresh release target, bind
the binary hash, then execute seed 1, seed 2 and a seed-1 repeat into fresh
output roots:

```sh
cargo build --release --locked -p uor-r4-cpu-train \
  --bin cpu-point-transfer-driver
RAYON_NUM_THREADS=32 target/release/cpu-point-transfer-driver \
  ARTIFACT TOKENS MASK MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_PIN \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --train-keys 8 --evaluation-keys 8 --per-key 8 \
  --key-tokens 8 --reply-turn 8 --random-seed 1
```

Exact artifact pins and output roots are fixed in the implementation before the
run. The seed-1 repeat differs only in output root.

## Minimum test power

Two model seeds over 60 fixed held-out openings; both must pass the paired bar.
Complete teacher-forced top-1 over 2,050 decisions per seed guards against an
opening-only gain that damages continuation. This is bounded exploratory
evidence, not a capability bar.

## Cost and stop

Local workstation only; no GPU, pod or paid API. Expected runtime is under two
minutes for all three runs at 32 workers and peak RSS below 1 GiB. Stop on input
drift, nondeterminism, a malformed or unchanged control, memory-key overlap,
CPU/serving divergence, or evaluation-time tuning.

## Claim boundary

A positive supports only target-level medoid point transfer on the fixed
unseen-key panel and two public seeds. A negative closes only this deterministic
medoid construction. Neither changes GPU qualification or establishes complete
replies, fresh phrasing or useful chat.

## TEST FITNESS

TEST FITNESS: PENDING
