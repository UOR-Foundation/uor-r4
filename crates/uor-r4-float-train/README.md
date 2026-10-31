# UOR-R4 Geometric Language Model: offline float training

> **Status on 30 Oct 2026:** pre-alpha, 1 of 8 milestones complete; no model holds a useful conversation yet. This crate trains the separate float FBS trunk for the current complete-reply research; it is not the saved discrete M1 model or the serving path.

## What this crate is

`uor-r4-float-train` is the D37 scanned module for offline float training. It trains the SpiralCore/FBS geometric trunk on the GPU in the model's own space, then hands the result to the integer crate for export. It is a separate crate with no dependency edge on `uor-r4-training` (CHARTER §3.4, §3.7). Float arithmetic lives here only. Serving is D11 integer, in [`uor-r4-integer`](../uor-r4-integer/README.md).

Training uses no squashing of any kind and no dense matmul. Research compute is GPU-only.

## Main modules

- `in_space_long`: long training of the 6-layer trunk, with text written at every checkpoint.
- `flock_read_hamming`: the induction-keyed Hamming read, copy heads and type heads (claude-146).
- `flock_read_memory`: the exact turn memory keyed by a reply's first tokens (route-plus-memory, claude-161).
- `spiralcore_fbs`: the native eight-dimensional SpiralCore/FBS projection.
- `fbs_export`: the D11 converter from a trained float trunk to the integer crate's snapshot (claude-123).
- `geometric_stack`, `cuda_stack_kernels`: the geometric stack extract and its CUDA kernels.

## Binary

`train-in-space` trains the trunk (`--data-root`, `--out`, `--objective`, `--steps`, `--seed`), and also runs the recall task, the opening-bound probe and the step profile. Its module docs list every flag.

## Where this stands

The 27–30 Oct line trained the trunk in its own space, added the Hamming read and the turn memory, and exported it to integer. The turn memory names the first word of a reply on familiar DEV prompts (598.7 and 598.3 of 1,226 openings at two seeds, claude-161), but its exact-token key finds little on new phrasing. Claude-217's p-s reply openings are only 17.4%, 13.7% and 18.6% at three seeds, against about 80% on later tokens. Claude-219's point-key opening improves free generation at three seeds on fresh p-v, but reaches only 12–18% of the forced-opening lift; the sign key is inconclusive at this panel size. Claude-220 then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. Claude-222 has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../../docs/loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../../docs/loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../../docs/loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../../docs/loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../../docs/loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; end-to-end greedy parity is next before the p-x rerun. Useful chat remains unproven. Details are in [the pipeline map](../../docs/model/pipeline-map.md), [the integer point-key report](../../docs/loop/tasks/point-key-integer/claude-222/REPORT.md) and [PLAN.md](../../PLAN.md).

## Build and test

Host tests, no GPU needed:

```sh
~/.cargo/bin/cargo test -p uor-r4-float-train --lib
```

The CUDA tests need the `cuda` feature (nvcc, off by default) and run on a GPU pod through `scripts/pod/uor-pod`. The crate also has a `metal` feature.
