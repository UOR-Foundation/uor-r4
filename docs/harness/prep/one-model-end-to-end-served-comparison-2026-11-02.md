# Served-surrogate training comparison

**Status:** DRAFT until a fresh-context FIT review.

## Question

At a fixed short budget on RTX 5090, does training through the full served
surrogate remain finite and operationally equivalent to ordinary float training,
while producing a checkpoint that export rounding cannot change? The comparison
is two modes at two seeds:

- `plain`: `--train-as-served 0`;
- `served`: `--train-as-served 1`.

This is a four-run engineering comparison. It does not establish language
capability, a milestone bar, or a preference for CPU over GPU.

## Inputs and pins

- Data root: `/workspace/w2-inputs/2026-10-25/driver-data`.
- TRAIN tokens SHA-256:
  `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
- TRAIN mask SHA-256:
  `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV and tokenizer are read through the trainer's existing pinned loader.
- Code: the exact PR head that passes the pre-run review; the pod records its SHA.
- Hardware: one RTX 5090, no more than one pod and $1.50 total.

## Recipe

The same command for both arms except the training-mode switch:

```text
train-in-space --data-root DATA --out RUN --objective cell --hamming-read 4 \
  --retain-rates 1 --decay-mult 1 --batch 16 --steps 256 --eval-every 64 \
  --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 \
  --turn-memory-dropout 500 --train-as-served MODE --seed SEED
```

Seeds are `1` and `2`. A run that writes any non-finite loss stops as an
instrument fault.

## Measurements

For every run:

- train-loss trace at steps `0, 64, 128, 192, 256`;
- final checkpoint SHA-256;
- step-256 `--inspect` with `--served-forward 1`, including nearest/writer top-1,
  top-8, admitted rows and query spread;
- `--decode-speed` with `--served-forward 1`, reported as decode tokens/s;
- `fbs-snapshot --round-as-exported`: the set of saved files whose bytes change.

The served-forward override is for evaluation only and must not change the
training objective.

## Doors

1. **Instrument door:** all four runs are finite and complete; all four saved
   checkpoints load under the served evaluator; the served arm's saved files are
   unchanged by export rounding while at least one plain-arm file changes.
2. **Operational door:** served training time is at most `2x` plain training
   time at the same seed and step count.
3. **Quality smoke:** on each seed, compare served-evaluator top-1 of the served
   checkpoint against the plain checkpoint. Report the delta and the two-seed
   spread. Call served "materially below" only if it is below plain by at least
   `max(24, spread)` on both seeds; likewise for plain. Otherwise the quality
   reading is inconclusive.

The instrument and operational doors decide whether the wiring is ready for a
longer run. The quality smoke never changes a mechanism status.

## Stop rules

- Stop on any non-finite loss, failed checkpoint load, missing data pin, or
  output hash mismatch.
- Stop if the job would exceed one GPU-hour or $1.50.
- Do not open a fresh panel; DEV is diagnostic here.
- If the served arm fails the instrument door, preserve the run and classify the
  failure as instrument or mechanism before any retry.
