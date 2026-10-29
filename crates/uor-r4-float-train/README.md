# UOR-R4 Geometric Language Model: offline float training

> **Status on 29 Oct 2026:** pre-alpha. No milestone is complete and no model holds a useful conversation yet. This crate trains the float FBS trunk that the replies run on; it is not the serving path.

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

The 27–29 Oct line trained the trunk in its own space, added the Hamming read and the turn memory, and exported it to integer. The turn memory names the first word of a reply (598.7 and 598.3 of DEV's 1,226 openings at two seeds, claude-161). After that word, replies are still templates. Carrying the memory past it is the next step. Details are in [the pipeline map](../../docs/model/pipeline-map.md) and [PLAN.md](../../PLAN.md).

## Build and test

Host tests, no GPU needed:

```sh
~/.cargo/bin/cargo test -p uor-r4-float-train --lib
```

The CUDA tests need the `cuda` feature (nvcc, off by default) and run on a GPU pod through `scripts/pod/uor-pod`. The crate also has a `metal` feature.
