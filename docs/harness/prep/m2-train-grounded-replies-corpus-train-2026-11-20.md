# Prep sheet: first native integer training run on D87 at 4,096 rows (m2-train-grounded-replies, claude-443, 20 Nov 2026)

Lab: claude. Machine: wick (Casey's own RTX 4090), bare metal, no rental.

## The question

Does the integer block-coordinate law that trained the M1 model (BVG-K with rows-K scoring, no softmax, no matrix multiply) keep learning when it is fed D87 TRAIN at 4,096 rows a batch through the new `corpus-train` command, and does the trained model reach the M1 trunk's ft/dev top-1 count?

Milestone moved: M2, "complete correct replies on one accepted saved model (dev)", 0 of 512 today. This run produces the saved native model that the D11 export and the reply client then score on `m2-reply512` dev. The run itself is graded on the open P1(d) floor below; it claims no reply count.

## Steps

1. Build `discrete-stack` (release, `cuda` feature) on wick from the PR head; record `build-identity`.
2. Inputs, all on wick under `~/work/claude/c443/data/`:
   - TRAIN: the leading whole-record slice of claude-420's D87 TRAIN stream cut by `corpus-slice` at a 268,435,456-token limit: 268,432,999 tokens, 67,341 records, 235,129,441 response targets; tokens sha256 `7561395e…5ee4f5`, mask sha256 `12efa306…bc503d`.
   - DEV: M1's ft/dev (42,856 tokens, 14,287 targets; tokens `0919e90f…a6555b`, mask `a838bb1a…fb078`).
   - Address table: `address-cells-codebook … structured` built from the TRAIN slice only (file sha256 `8b186f32…5009ab`).
3. Config `train-r4096-k16.json`: the M1 Stage T relation config (vocabulary 4,096, 4 blocks, context 192, kept cells 8, relation read kept 8, `interleaved_read_every8_v1`, `nearest_address_margin_v1`, `rows_v1`, `coordinate_block` 16, seed 1) with `batch_rows` 4,096 and the inputs above.
4. Run `corpus-train <config> runs/main --batches 16404 --checkpoint-every 256 --eval-every 512` in tmux with a completion watcher.
5. Record every DEV line (top-1, top-8 of 14,287) and the progress file's loss and targets per second.

Already checked before this sheet (finite, not the run): 8 batches straight and 4 + 4 resumed give the same model sha256 `3c4ca645…2bab67` and the same cursor; ft/dev top-1 went from 56 to 581 in those 8 batches.

## Bar (pre-registered)

- PASS: some registered DEV evaluation has ft/dev top-1 at or above 7,156 of 14,287: the M1 trunk's count of 7,002 (the P1(d) floor) plus all 154 DEV positions whose 13-token window also occurs in the TRAIN slice, counted as if every one were a scored target answered from memory. (Raised from 7,002 after the run started, on the round 2 review; the change only tightens the bar.)
- Otherwise the result is the best top-1 reached and the curve. A miss closes this configuration only (rows 4,096, K 16, this slice, 16,404 batches), never the law.
- Instrument check: if DEV top-1 at batch 512 is below this run's own batch-0 value, the run is stopped and classed as an instrument failure, not a model result. Any fix after that inspection runs as a new configuration on a new sheet.

## Base artifact

A fresh model from `new_model(0)` with the pinned address table. No trunk, donor or count table is loaded. The M1 trunk is read by nothing here.

## Data split

TRAIN is D87 TRAIN only. Overlap, measured with `corpus-overlap` on wick: of ft/dev's 42,833 windows of 24 tokens, 0 occur in the TRAIN slice (0 of 42,809 at 48 tokens; 154 of 42,844 positions at 13 tokens, 30 distinct windows). D87 DEV-tune and DEV-test are not opened. ft/dev is the M1 development set, used for evaluation only; evaluation applies no update. No M2 panel is read.

## Minimum test power

14,287 DEV targets; the PASS line is a count, and a 1-point change is 143 rows. 16,404 batches at 4,096 rows is 67.2M TRAIN targets, 28.6% of the slice, in one pass with no repeats.

## Cost

Measured 1.56 s a batch (2,600 targets/s) at 22 GB host memory free and an idle GPU: about 7.1 hours plus 34 DEV evaluations (batch 0, each multiple of 512, and the end; under 2 s each in the smoke run). No money is spent. The GPU is shared: if another lab's job needs it, the run is resumed from its last checkpoint.

## Stop rule

Stop at 16,404 batches. Stop early only on the instrument check above, an error, or a GPU conflict with another lab; a run stopped by an error or a GPU conflict resumes with `--resume` on the same `build-identity`; a run stopped by the instrument check is never resumed under this sheet. Results go to a new folder; nothing is written into an earlier run's folder.

TEST FITNESS: NOT_FIT - round 2 (non-author) asked for the overlap positions to be counted in the bar and for resume to exclude instrument stops; both are now written above, after the run had started, and no third read was made.
