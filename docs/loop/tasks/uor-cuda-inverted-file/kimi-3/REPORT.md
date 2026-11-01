# REPORT: matched CUDA performance and replication, kimi-3

Lab: kimi. Item: #3475. Order: [ORDER.md](ORDER.md). Measured source: `7badaf6ccd18fd4ec8768a197b84347c2f138795`.

## Negatives and limits first

- FIT reviews rejected the first design for incorrect parallelization, cold-context timing, mismatched build work, missing oracle/index checks, and incomplete identity evidence. No timing was credited to those designs.
- Attempt 1 at `5bc7c547e` stopped on an ambiguous CPU/CUDA `probe` call before correctness. Attempt 2 at `3fd8408f2` passed 504 correctness rows but stopped when the hash helper treated normal EOF as an error. Both failures were archived and the pods deleted; no timing was credited. `7badaf6cc` fixed those issues and passed the final FIT.
- This is a fixed grid, not a general exact-kNN, portability, energy, cross-library, or release result. The >4096 serial fallback, other V/K, other architectures/drivers, concurrent kernels, and external replication beyond the currently running independent check remain unclaimed.
- Kernel-only medians are not end-to-end speedups. Direct API and complete-caller scopes are dominated by context/module/transfer overhead, so a kernel advantage is mostly invisible at those scopes.

## Measured evidence

At the same source, both cards passed all 504 correctness rows, every GPU/fresh-CPU index comparison, and every configuration's oracle check. Each produced 240 raw JSONL records with warmup 3 plus 10 retained repetitions, three timing scopes, fixed query seed 7, and a manifest binding source, module, hardware logs, correctness rows, and raw-file hashes.

| Card / driver | Operation | Kernel median (ms) | Direct API median (ms) | Complete-caller median (ms) |
|---|---|---:|---:|---:|
| RTX 5090, 580.178.04, CUDA 12.8.93 | full_scan | 0.108528 | 115.454 | 115.531 |
| RTX 5090 | probe | 0.161184 | 115.337 | 115.505 |
| RTX 4090, 615.78.08, CUDA 12.8.93 | full_scan | 0.109568 | 155.017 | 155.072 |
| RTX 4090 | probe | 0.181616 | 155.131 | 155.403 |

Probe/full kernel-median ratios by V are approximately:

- RTX 5090: **1.15×, 1.72×, 1.78×, 1.83×** at V = 64, 256, 1024, 4096.
- RTX 4090: **1.13×, 1.74×, 1.97×, 1.90×** at the same sizes.

These are per-card, same-source, same-configuration kernel-only ratios, not a cross-card comparison or an end-to-end claim. Full scan is the optimized control, with parallel distance evaluation and deterministic serial top-8 ordering. Direct and complete-caller results use matched CPU build work for both arms.

Raw evidence and checksum read-backs:

- RTX 5090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/c2cec27d7a9b3b9da3835fdf01403b0a0ef3a66b
- RTX 4090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/38a15d3c3c46324ab485ec5d82bf05950ada6c33

Machine-readable summaries are in `docs/labs/uor-cuda-inverted-file/gpu-performance-results.json`. All pods were deleted after verified archival. Rates were $0.975, $0.563, $0.563, and $1.256 per hour for the four performance attempts; no invoice total is inferred.

## Next decision

A non-author clean-consumer and independent RTX 5090 replication is in progress. Public release and upstream outreach remain on hold until that replication and the final review pass. No NVIDIA issue, performance announcement, or upstream acceptance is claimed.
