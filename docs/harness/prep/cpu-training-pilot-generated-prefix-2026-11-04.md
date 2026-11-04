# Generated-prefix writer training

**Status:** PENDING

## Question

Can the exact CPU writer learn from the model's own generated prefixes well
enough to delay divergence, while reference-prefix, static-prefix and
same-cell shuffled-label controls separate the effect on both public model
seeds?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not GPU qualification, complete-reply correctness,
fresh-phrasing transfer or useful conversation.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `SELF_PREFIX_IMPLEMENTATION` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public manifest says `split=train`. This is a repeated-key,
document-disjoint screen, not unseen-key, fresh-phrasing or language evidence.

## Data split

Use 16 eight-token keys, even documents for training, odd documents for
evaluation and up to eight examples per key. Derive opening/reply memory from
training examples only.

Expected prior selection is 98 training and 103 evaluation examples. The run
must report actual counts. Each reply has a 16-content-token horizon.

## Mechanism

Reset one exact `CpuFbsForward` per prompt. Call `mark_reply_start()` before the
final prompt token, then:

1. record the live Q12 writer query and selected token at reply offset `j`;
2. align it to the reference content token at offset `j`;
3. generate the next query from the model's own selected token; and
4. apply one existing K=1 target/rival BVG-K writer update when the target cell
   is admitted and margin loss is positive.

Candidate acceptance is lexicographic over aligned generated top-1, target rank
sum and target L1 distance over every fixed-horizon prompt. Early BOS therefore
cannot win by removing later decisions. Each accepted artifact is reparsed,
audited against integer serving, and used for the next dynamic update.

Reference mode feeds the reference token at the same offset. Static mode
captures frozen generated queries once. Dynamic mode recaptures after accepted
updates. Same-cell shuffle rotates targets within each target-cell group,
preserving the target multiset and admission class.

## Arms

1. frozen memory artifact;
2. reference-prefix true labels;
3. static generated-prefix true labels;
4. static generated-prefix same-cell shuffled labels;
5. dynamic generated-prefix true labels; and
6. dynamic generated-prefix same-cell shuffled labels.

Every trained arm gets exactly 80 K=1 attempts. No outcome-based retune.

## Eligibility gate

Before training, each seed must find at least 640 generated training decisions
with an admitted target and positive margin loss, across at least 16 prompts
and 8 target cells, with at least 64 same-cell shuffle pairs. Failure is an
insufficiency result and spends no training compute.

## Bar

Both seeds must pass:

- dynamic true has positive summed held-out matching-prefix depth over frozen;
- dynamic true has positive summed depth over dynamic shuffled;
- each paired depth comparison has at least
  `max(2, ceil(3 * sqrt(b + c)))` candidate-only wins; and
- dynamic true aligned generated top-1 exceeds both frozen and dynamic
  shuffled.

Dynamic true teacher-forced content top-1 must not fall below frozen. Every arm
must retain exact CPU/integer query-coordinate, token and kept-cell parity.

Report exact references, BOS/cap/context stops, opening accuracy,
first-divergence histogram, admission, admitted-only accuracy,
distinct/repeated replies, effective workers, wall and peak RSS.

## Minimum test power

The fixed panel has 103 held-out prompts. Both independent model seeds must
pass. The paired threshold
`max(2, ceil(3 * sqrt(b + c)))` requires at least two candidate-only wins
after discordance, and the depth and aligned-top-1 differences must both have
the registered direction. This is a bounded configuration screen, not a
precision or language-capability result.

## Instrument gates

- Source, inputs and binary hashes match.
- Generated and reference feedback produce different query states.
- Opening and reply memory are active in the scanned trajectory.
- Fixed-horizon accounting counts decisions missing after early BOS as wrong.
- Same-cell shuffles preserve cells and counts and change targets.
- No admitted generated row or requested shuffle is vacuous.
- Seed-1 repeat reproduces every non-timing and non-resource field and artifact.
- Integer-serving files and the serving contract remain unchanged.

## Stop rules

Stop on source, split, panel, input or binary drift; a vacuous same-cell
shuffle; no eligible generated rows; any CPU/integer parity mismatch; a
teacher-forced top-1 loss; nondeterminism; or any need to change serving,
integer-session, CUDA or Claude-owned files. An eligibility failure is
retained as an insufficiency result without training compute.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build | 300 s |
| two seeds plus repeat | 600 s |
| delivery and evidence | 300 s |
| total increment | **1,800 s** |

- Build: 16 jobs, 4 GiB RSS maximum, 2 GiB target maximum.
- Runs: 32 Rayon workers, 1 GiB RSS maximum, 600 s each.
- Retained pipeline: 64 MiB maximum.
- Post-run free-space floor: 30 GiB plus 128 MiB.
- Previous ledger:
  `cpu-training-pilot-stop-signature-2026-11-04/resource-ledger.json` at
  `34856.977487699 s`.
- Admitted cumulative projection: **36,656.977487699 s**.

## Steps

After FIT and a fresh GO, build `cpu-generated-prefix-driver` in a fresh
source-bound target. Run selection on both seeds. Proceed only if the
eligibility gate passes on both. Then run seeds 1 and 2 plus a seed-1 repeat in
fresh roots under `RAYON_NUM_THREADS=32`, with 600-second timeouts, exact input
pins, a 1-GiB RSS stop and per-root checksums. Preserve every arm artifact and
the compact report.

## TEST FITNESS

TEST FITNESS: PENDING

A FIT verdict clears compute for this configuration only (D22).
