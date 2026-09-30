# Track B dense checkpoint parity: completed numerical failure

On September 30 the unchanged `1e-4` all-logit gate completed at source
`648ba939020054679f20e58d048dd6ae10a87758`. All 45 reference positions and
294 candidate rows were evaluated: 14,450,688 scalar comparisons across the
stock Candle loader and shared differentiable model, each on CPU and Metal.
Coverage passed; every backend/implementation failed numerical fidelity.

| Implementation | Backend | Compared rows | Maximum absolute logit error | Cells exceeding 1e-4 |
|---|---|---:|---:|---:|
| Stock Candle Llama | CPU | 53 | 0.0014320611953735352 | 138,560 |
| Stock Candle Llama | Metal | 53 | 0.008588790893554688 | 345,550 |
| Shared TrackBModel | CPU | 94 | 0.0070133209228515625 | 349,490 |
| Shared TrackBModel | Metal | 94 | 0.0067539215087890625 | 381,652 |

The first stock CPU failure occurs at zero-based window 2, position 1
(prefix `[1, 2]`, maximum error `0.00042629241943359375`). Stock Metal already
misses on `[1]` (`0.00010573863983154297`). The largest discrepancies occur in
the `[1, 2, ..., 8]` window. Small synthetic-token KL values do not override
the frozen absolute-logit gate. These are numerical fidelity results, not a
language-quality evaluation, rejection of geometry, or transformer-serving
qualification. No training ran; B2 fitting remains unqualified.

## Identity and cost

- Model: hash-verified SmolLM2-135M-Instruct; all four input identities are retained in `inputs.json`.
- Executable SHA256: `f1892eb7281cd15007f894d0867b1daadc1ebc22e8b53b8f4057ed23e6983be4`.
- Source diff: empty; source revision above was embedded at compilation.
- Reference: `uor-matmul` exact GEMM, pinned revision `b13c98449948174f590e337c4dc25dfc394a07d0`, two workers. No observation-BLAS exception.
- Optimized cold build: 372.42 seconds elapsed; maximum process RSS 3,246,276,608 bytes.
- Parity process: 387.50 seconds elapsed; maximum process RSS 1,484,341,248 bytes, reported peak memory footprint 1,679,918,880 bytes. These host measurements are not optimized serving benchmarks.
- Reference construction: 366.998780708 seconds; complete driver result: 386.8010895 seconds. Overlapping host jobs mean elapsed rates are not performance evidence.
- Report: `/Volumes/UOR-Workspace/uor-r4-models/track-b/parity-648ba939-20260930`. The driver sealed and verified its complete inventory before exit 2.
- Manifest SHA256: `a575fe03a1fa42869bb73c7a70ff95ba13382582ea019b29a571ef55a01970b3`.
- [Compact machine-readable receipt](track-b-parity-2026-09-30.json) includes all output-file SHA256 identities. Raw logits remain in the sealed root; metadata and logs are preserved on the operational branch.

## Next discriminating action

Do not repeat this full comparison unchanged or alter the tolerance. Use the
smallest failing prefixes to locate the first diverging layer/operator between
the exact reference and stock/shared implementations. Distinguish reduction,
normalization, activation, rotary/cache layout and batch-shape effects using
identical inputs and retained reference outputs. A discrepancy in stock CPU as
well as Metal means the separate harmonic Metal-contraction finding cannot
explain this result by itself. Make a causal correction before rerunning the
full gate; keep the original failed report immutable.
