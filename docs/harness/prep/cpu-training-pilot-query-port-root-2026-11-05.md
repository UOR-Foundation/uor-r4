# Pre-registration: CPU query-port root training

Run ID: `cpu-training-pilot-query-port-root-2026-11-05`

## Status

READY FOR FIT REVIEW

## Question

Can eight safe moves of only the 16 final-gain coordinates that scale the
writer query's two 8-coordinate ports reduce held-out query-port distance to
the target's cell roots, compared with frozen, random-gain and shuffled-cell
controls, while retaining target admission, top-1 and exact CPU/serving
parity?

This is D84 CPU development research. It is not GPU qualification, a language
result or a complete-reply result.

## Why this is new

Claude-289 found that one seed's query ports collapsed to the roots' centre,
where all 16 roots tie. Claude-290 anchors the table side and schedules the
query-side reading for its next run. The closed CPU experiments trained token
rank, candidates, memory weights, termination and a general target distance.
None optimized the two query ports against the target cell's roots.

This run changes 16 final-gain codes, the exact coordinates that scale the
query's first and last eight-coordinate blocks. It uses no matrix
multiplication, softmax, squashing activation or float inference.

## Base artifacts

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are memory-free and Hamming-free raw FBS artifacts.

## Data split

Training corpus:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/m2-evidence/train-control-v3`

- tokens SHA-256:
  `a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5`;
- response mask SHA-256:
  `89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f`;
- manifest SHA-256:
  `980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672`;
- split `train`, 7,906 tokens and 2,708 response targets.

Held-out panel:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/m2-evidence/dev-panel-v3`

- tokens SHA-256:
  `add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442`;
- response mask SHA-256:
  `da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01`;
- manifest SHA-256:
  `ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949`;
- split `dev`, 1,925 tokens and 616 response targets.

Both manifests bind tokenizer
`a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`,
vocabulary 4,096 and BOS 0. The panel is not used to select targets or moves.

## Fixed construction

For every response target `t`:

- derive cell `c = t mod 256`;
- derive roots `c / 16` and `c % 16` from the 16 parsed SpiralCore roots;
- calculate exact Q12 L1 distance from query columns `0..8` to the first root
  and columns `W-8..W` to the second.

First project every token row's two 8-coordinate port blocks to its own cell
roots. This table-anchored artifact is the baseline and the starting point for
the query-side trainer, matching Claude-290's construction. Only final-gain
coordinates `0..8` and `W-8..W` are trainable. A step evaluates
every coordinate at `±1` code and reexports/read-backs the complete artifact.
It accepts a strictly lower training target-root distance only if training
target admission, top-1 and top-8 do not fall. Ties retain the lower coordinate
and positive move. The run performs eight steps.

The cached trajectory is built before movement and reuses the exact
normalization immediately before final gain. The final artifact is reparsed.
Every accepted move also runs `CpuPanelEvaluator`, requiring zero query, token
and kept-cell mismatches.

## Arms

| Arm | Construction |
|---|---|
| raw frozen | source artifact, no projection and no gain move |
| table anchor baseline | each token row's two table port blocks set to its own cell roots |
| trained | eight actual-cell query-port-root steps from the table-anchor baseline |
| shuffled cell | the same trainer from the baseline, with every target cell rotated by one |
| random gain | exactly the trained arm's accepted move count from the baseline, selected deterministically from port coordinates |

The raw-frozen arm exposes the cost of Claude-290's table-side projection. The
trained arm is judged against the table-anchor baseline, not raw-frozen.

## Source and binary

- base: `7787313b2`;
- implementation pin: `b9c4f51c8`;
- driver:
  `target/deepseek/query-port-root/release/cpu-query-port-driver`,
  SHA-256 `17189223571174c3e46fde7d6ec7b5946906b8a6b7be066d29e3d136879db847`;
- host: AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- execution: 16 Rayon workers.

## Output

`local/deepseek/query-port-root/2026-11-05/seed-1` and
`local/deepseek/query-port-root/2026-11-05/seed-2`.

Both output roots are absent at registration.

## Steps

Run only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-query-port-root
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/query-port-root/2026-11-05
BIN=$SOURCE/target/deepseek/query-port-root/release/cpu-query-port-driver
TRAIN=$SHARED/local/deepseek/m2-evidence/train-control-v3
PANEL=$SHARED/local/deepseek/m2-evidence/dev-panel-v3
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut

test ! -e "$OUT"
test "$(git -C "$SOURCE" rev-parse --short=9 HEAD)" = b9c4f51c8
sha256sum -c <<EOF
a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  $SEED1
da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  $SEED2
a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5  $TRAIN/tokens.u16
89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f  $TRAIN/response_mask.u8
980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672  $TRAIN/manifest.json
add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442  $PANEL/tokens.u16
da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01  $PANEL/response_mask.u8
ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949  $PANEL/manifest.json
EOF

mkdir -p "$OUT"
run() {
  seed=$1 artifact=$2 artifact_sha=$3
  RAYON_NUM_THREADS=16 timeout 1200s "$BIN" \
    "$artifact" "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$OUT/seed-$seed" \
    --expected-artifact-sha256 "$artifact_sha" \
    --expected-train-tokens-sha256 a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5 \
    --expected-train-mask-sha256 89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f \
    --expected-train-manifest-sha256 980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672 \
    --expected-panel-tokens-sha256 add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442 \
    --expected-panel-mask-sha256 da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01 \
    --expected-panel-manifest-sha256 ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949 \
    --seed 183 >"$OUT/seed-$seed.stdout.json" 2>"$OUT/seed-$seed.stderr.log"
}

run 1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run 2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
```

## Minimum test power

- two public artifact seeds;
- 2,708 training response targets and 616 panel response targets;
- eight candidate sweeps of 34 port-gain moves each;
- at least one accepted actual-cell move per artifact;
- non-empty shuffled-cell and random-gain controls;
- exact CPU/serving parity in every arm;
- artifact and step replay identity within the run.

## Bar

On both artifact seeds, the trained arm must:

- reduce held-out target-port distance below the table-anchored baseline;
- reduce it below the equal-move random-gain control;
- reduce it below the shuffled-cell control;
- retain held-out target admission and top-1 at or above the table-anchored
  baseline;
- retain zero CPU/serving query, token and kept-cell mismatches;
- reproduce the artifact and all step records on an in-process repeat.

## Outcomes

- **KEEP:** both artifacts clear every bar item.
- **REJECT:** either artifact completes but misses a distance or safety bar.
- **INSTRUMENT_FAULT:** inputs, artifacts, parity or repeat identity drift.
- **NO_MOVE:** an arm accepts zero port-gain moves.

## Cost and stop

- local CPU only, $0;
- two runs, 16 workers, at most 20 minutes each;
- stop immediately above 1 GiB peak RSS;
- stop rather than retune after either fresh output root is read.

## Claim boundary

A pass supports only this eight-step port-root gain configuration on two public
CPU-development artifacts. It does not establish unattended training,
unseen-key transfer, language quality, complete replies, or GPU qualification.

## Verdict

TEST FITNESS: PENDING
