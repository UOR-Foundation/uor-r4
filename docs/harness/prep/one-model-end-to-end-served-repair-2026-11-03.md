# Served-training repair: CUDA gate, profile and the matched rerun

**Status:** REGISTERED (claude-262, [ORDER](../../loop/tasks/one-model-end-to-end/claude-262/ORDER.md), PR #3587).

TEST FITNESS: pending

## Question

claude-262 found that the float stand-in for the integer step's activation requantizer
(`activation_quantize16`) left out the step's exponent (`PRODUCT_EXP = −32`). So a served training
forward rounded every activation below 2^15 to a whole number, and `mlp.down` and `rec.out` got
almost no gradient. The fix scales by `2^−ACTIVATION_EXP` first and finds the shift in six exact
passes instead of 49. With it, on RTX 5090:

1. does a served step give every layer's `mlp.down.weight` and `rec.out.weight` a gradient within
   4× of plain, never 0, and does the tensor quantizer equal its scalar reference value for value?
2. where does a served step's time go, stage by stage, against a plain step's?
3. on [deepseek-5's sheet](one-model-end-to-end-served-comparison-2026-11-02.md), unchanged, does
   served training pass the instrument and operational doors, and where does it read against plain
   in the quality smoke?

This is an engineering repair and a matched short comparison. It does not establish language
capability, a milestone bar, or a training law.

## Base artifact

The exact PR #3587 head that passes the pre-run review. If step P (profile) leads to a repair inside
the unit's scope, that repair is a new commit: step I-0 reruns at the new head before step R, and
step R runs only at a head whose I-0 passed. The pod records the source SHA, the release
`train-in-space` and `fbs-snapshot` SHA-256 and every data file's SHA-256 before the first run.

## Data split

deepseek-5's pins, unchanged:

- Data root: `/workspace/w2-inputs/2026-10-25/driver-data`.
- TRAIN tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
- TRAIN mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

A data hash that does not match stops the run.

## Steps

**I-0. CUDA gate.** Build with `--features cuda` and run
`cargo test -p uor-r4-float-train --features cuda --lib`. Every test passes, and these three by name:

- `served_activation_quantizer_is_its_integer_reference_on_cuda`: rows at every power-of-two edge
  from 2^−20 to 2^31 and a saturated row; quantized values, straight-through gradient and the
  following weight's gradient equal the scalar reference bit for bit; on a realistic row
  (0.3, −0.12, 0.004, −1.5·10^−5) the following weight's gradient is within 2^−17 of the activation.
- `served_surrogate_gradients_are_exact_on_cuda` (existing, its literals updated).
- `a_step_as_served_reaches_every_projection_after_a_requantized_activation`: one served forward
  and backward against one plain, on the same model and batch; every layer's `mlp.down.weight` and
  `rec.out.weight` gradient norm is above 0 and served/plain is within [0.25, 4].

**Negative gate:** set aside only the bodies of `activation_scale_scalar` and
`activation_quantize16` (the patch is `git diff` of those two bodies against the reviewed head,
streamed from the laptop), rebuild the test binary and read its build line: the first and third tests
above must fail. Restore with `git checkout` of the file, rebuild, and rerun all three green.

**P. Profile.** One plain and one served run on seed 1 with the R recipe below, `--steps 40
--profile 16 --eval-every 40`. Report each run's `profile.json`: stage medians (GPU and host),
groups, `top3`, `median_unprofiled_ms`, and served/plain median step time. A stage whose median
alone adds more than one plain step to the served step is named with its cause. If it is in the
ORDER's files, it is repaired in this unit and I-0 reruns; if not, it is named in the REPORT and R
runs at the current head.

**R. The matched rerun.** deepseek-5's four jobs, in order plain/s1, served/s1, plain/s2,
served/s2:

```text
train-in-space --data-root DATA --out RUN --objective cell --hamming-read 4 \
  --retain-rates 1 --decay-mult 1 --batch 16 --steps 256 --eval-every 64 \
  --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 \
  --turn-memory-dropout 500 --train-as-served MODE --seed SEED
```

After each job, stop on a non-finite loss or a missing checkpoint. Then, with deepseek-10's corrected
evaluation step, `EVAL` absent before each command and `fbs-snapshot` creating it:

```text
fbs-snapshot --round-as-exported RUN/models/step-000256 EVAL/models/step-000256
train-in-space --inspect EVAL --data-root DATA --served-forward 1
train-in-space --decode-speed EVAL --data-root DATA --served-forward 1
```

A wrapper checks every command's exit status and writes `DONE` only after all of them succeed.

**S. Seal.** Logs, progress, configs, `profile.json` files, source, binary, data and copy hashes,
reports and `SHA256SUMS`, in one tar under `results/claude/one-model-end-to-end-served-repair-2026-11-03/`;
upload, read back and check against `SHA256SUMS` before `down`.

## Bar

- **I-0:** the three named tests pass at the head; with the two bodies set aside, the first and third
  fail; restored, they pass. A failure here stops the run before P.
- **R, deepseek-5's bar unchanged:**
  1. *Instrument door:* all four runs finish with finite losses and load through the served
     evaluator; both served checkpoints are byte-identical after `--round-as-exported`; at least one
     plain file changes.
  2. *Operational door:* served training time is at most 2× plain at the same seed.
  3. *Quality smoke:* served-evaluator top-1, served minus plain, per seed. Served is "materially
     below" only if it is below plain by at least `max(24, two-seed spread)` on both seeds; likewise
     for plain. Otherwise the reading is inconclusive.
- **Unit reading:** the repair holds when I-0 passes, both R doors pass, and served is not materially
  below plain. DEV nearest hits are reported beside top-1, against deepseek-5's 23/20 served and
  225/229 plain.

Decode tokens/s (`--decode-speed --served-forward 1`, contexts as the command reports) are reported
first in the REPORT.

## Minimum test power

Two seeds, 256 steps: enough for finite operation, export identity, a time ratio and a smoke-level
quality reading, not for a training-law claim. The gradient check is one step on one batch; it shows
the gradient reaches the projections, not that training converges.

## Stop rule

- Stop on a non-finite loss, a failed checkpoint load, a data-pin mismatch or an output hash mismatch.
- Stop if the work would exceed one pod, two GPU-hours or $1.50; more is a new ORDER.
- Do not open a fresh panel; DEV is diagnostic.
- If I-0 fails, or the served arm fails the instrument door, keep every output and classify instrument
  against mechanism before any retry.
- If served is still materially below plain, R's outputs stand as the negative. The next cause is
  found by turning the D11 snaps, the Q16 residual and the Q12 query off one at a time in a finite
  CUDA gradient check (ORDER, "Stop and ask if").

## Cost

One RTX 5090 on Vast, about $0.56/h, at most two GPU-hours and $1.50, inside the project-wide
4-pod/$8 per hour caps (two pods, $1.12/h, were running at registration). Projection: build and tests
25 minutes, negative gate 10, profile 5, four runs 10, evaluation 20.

```text
scripts/pod/uor-pod up --lab claude --session claude-262 --count 1 --gpu 5090 --hours 2 \
  --purpose "claude-262 served-training repair: CUDA gate, profile, matched rerun" \
  --card https://github.com/UOR-Foundation/uor-r4/pull/3587
```

A completion watcher is armed on every detached job. The lease is renewed at least every 30 minutes
and released, and the pod destroyed, only after the sealed tar is uploaded, read back and checked.
claude-261's pods (`vast-58896165`, `vast-58912154`) are not touched.

## Verdict

(pending a fresh non-author review)
