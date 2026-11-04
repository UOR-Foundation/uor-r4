# CPU memory-weight complete replies

**Status:** PENDING NON-AUTHOR FIT

TEST FITNESS: PENDING

## Question

Do the memory movement weights trained by deepseek-109 change complete greedy
reply content or termination, or is their measured gain confined to
teacher-forced next-token decisions?

This is a replay of the exact deepseek-109 training configuration. It is not a
new credit rule and does not extend the closed coordinate-sweep search.

## Compute class

This is CPU development research under the owner-authorized D84 program in
`uor-r4-cpu-train`. It uses the local Ryzen 9 9950X, the same deterministic
evaluator as the CPU development path, and matched controls. CPU output cannot
substitute for GPU qualification, and every result remains development
research.

## Base artifacts and inputs

| Input | Identity |
|---|---|
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| seed 1 expected trained artifact | `c856ce21cf46a4be5a0cc02e033d9c7d0cd674bc4ab96a1185a1b9ccf3b88de4` |
| seed 2 expected trained artifact | `10f91f7309c3ed36e3ce76878b2c75c07bae0102b3933571ccbd8f3b40b350e6` |
| hardware | AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads |

## Data split and training

Use deepseek-109's fixed selection without change:

- 16 repeated opening keys;
- up to 16 examples per key;
- 102 training and 114 evaluation examples;
- 32 attempted Q24 opening/reply movement-coordinate updates;
- update seed 1, 32 Rayon workers, cached exact trajectory.

The trained artifact hash must match the corresponding pin above before any
complete-reply reading is used.

The model-free selection census records 1,326 reference tokens across the 114
evaluation replies. The maximum reference is 36 tokens. **104 of 114**
references fit both the registered 16-token reply cap and the 192-token artifact
context; the remaining ten are 26 to 36 tokens and are unreachable under the
registered cap.

The training-only memory-key census reports:

- all 114 evaluation opening keys hit the table, and the stored majority target
  is correct on 50;
- 795 of 1,212 evaluation reply keys hit, and the stored majority target is
  correct on 717;
- only 44 of 724 reply keys and 133 of 1,192 training reply decisions are
  ambiguous.

These are table-content ceilings. They do not replace the served-model bar.

## Arms

For each model seed, evaluate five matched artifacts on the 114 evaluation
samples:

1. frozen corpus-derived memory with zero movement weights;
2. the trained artifact;
3. the equal-accepted-move random control;
4. the source artifact with memory off;
5. the trained artifact with memory off.

## Steps

1. Build a fresh release `cpu-memory-panel-driver`.
2. Pass the finite reply evaluator tests before reading public data.
3. Run seed 1 and seed 2 sequentially:

```text
env RAYON_NUM_THREADS=32 target/release/cpu-memory-panel-driver \
  ARTIFACT DEV_TOKENS DEV_MASK DEV_MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_PIN \
  --expected-tokens-sha256 TOKENS_PIN \
  --expected-mask-sha256 MASK_PIN \
  --expected-manifest-sha256 MANIFEST_PIN \
  --expected-trained-sha256 TRAINED_PIN \
  --steps 32 --keys 16 --per-key 16 --key-tokens 8 --reply-turn 8 \
  --reply-cap 16 --seed 1
```

4. Repeat the seed-1 complete-reply pass in a fresh process and compare every
   ordered-token digest after removing timing fields.

## Instrument bar

- The trained artifact equals its pre-registered deepseek-109 hash.
- The 16-key, 102-training and 114-evaluation selection is unchanged.
- The teacher-forced trained top-1 and distance equal deepseek-109.
- All five arms have zero query-coordinate, token and kept-cell mismatches.
- BOS, cap and context stops use the shared integer `reply_stop` contract.
- Reference matching removes one trailing BOS and no other token.
- A reference longer than the cap or remaining context is counted as
  unreachable, not as a model miss.
- Exact-reference comparisons report both the full 114-reply panel and the 104
  references reachable under the fixed cap/context rule.
- The seed-1 repeat reproduces every complete-reply counter and digest.

## Candidate bar

On each model seed, trained memory carries into the reply-content path only if:

- exact-reference complete replies are strictly greater than frozen,
  equal-accepted-move random and memory-off;
- BOS completion is not lower than any of those controls.

A BOS or stopping improvement with no exact-reference gain is reported as a
termination-only result and does not pass the content bar.

Two seeds and 114 repeated-key prompts make this a bounded readiness screen.
No three-sigma, fresh-phrasing, semantic-correctness or useful-chat claim is
made.

## Minimum test power

Two public model seeds and 114 document-disjoint evaluation examples per seed,
with five matched artifacts. This is a repeated-key readiness screen, not a
decisive capability run.

## Outcomes

- **No propagation:** movement weights remain teacher-forced diagnostics and do
  not enter the chat path.
- **Termination only:** investigate reply stopping separately; do not call it
  reply content.
- **Replicated content propagation:** use the trained artifact as the first
  candidate for a larger fresh-phrasing complete-reply panel.
- **Instrument fault:** stop and repair without recording a mechanism negative.

## Cost and stop

Local Ryzen 9 9950X only; no GPU, pod or paid API. Two exact 32-step training
replays and five-arm complete-reply evaluations are expected to use about two
CPU minutes, plus build and tests. Stop before seed 2 if seed 1 projects the
two-seed total beyond 30 CPU minutes. Stop on artifact-hash drift, selection
drift, nondeterministic replies, missing parity, counter overflow or any
post-observation change to arms, cap, keys, steps or controls.

## TEST FITNESS

Author self-check only. The six finite evaluator tests pass, including a
deterministic complete-reply run over an exported synthetic artifact. No non-author
review has been recorded, so public compute is not authorized by this sheet.

TEST FITNESS: PENDING
