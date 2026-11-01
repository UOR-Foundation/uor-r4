# REPORT: matched CUDA performance and replication, kimi-3

Lab: kimi. Item: #3475. Order: [ORDER.md](ORDER.md). Measured source: `719a9a7861deb595dfc5269ffec8a5e7e53cb39b`.

## Negatives and rejected evidence first

- FIT reviews rejected the first design for incorrect parallelization, cold-context timing, mismatched build work, missing oracle/index checks, and incomplete identity evidence. No timing was credited to those designs.
- Attempts at `5bc7c547e` and `3fd8408f2` stopped on an ambiguous call and normal-EOF handling before valid measurement. Both were archived and their pods deleted. `7badaf6cc` fixed those issues, but the next review found a full-scan publication race, missing complete-caller warmups, and a final-only kernel oracle check. Those first two-card results and the independent third-driver result are **REJECTED**, not valid for claims.
- A later timing-boundary review found complete-caller time included oracle comparison and bookkeeping. Those runs at `ae4a6951` are also rejected. The final source stops that clock immediately after the public scan and before oracle comparison or measurement construction.
- The clean-consumer host check remains valid at the final source: exact file hashes match and all 13 admission plus host parity/fault checks pass.
- This is a fixed grid, not general exact-kNN, portability, energy, cross-library, or release evidence. The >4096 serial fallback, other V/K, other architectures/drivers, concurrent kernels, and any additional independent replication remain outside accepted scope.

## Final corrected measurements

At source `719a9a786`, both cards passed all 504 correctness rows, every GPU/fresh-CPU index comparison, and every launch's oracle check. Each produced 240 raw JSONL records with warmup 3 plus 10 retained repetitions across all three scopes, fixed query seed 7, and a manifest binding source, module, hardware logs, correctness rows, and raw-file hashes. Complete-caller warmup executes the full path; retained complete-caller timing excludes oracle comparison and bookkeeping.

| Card / driver | Operation | Kernel median (ms) | Direct API median (ms) | Complete-caller median (ms) |
|---|---|---:|---:|---:|
| RTX 5090, 580.178.04, CUDA 12.8.93 | full_scan | 0.109856 | 115.493 | 115.407 |
| RTX 5090 | probe | 0.160144 | 115.678 | 115.620 |
| RTX 4090, 580.95.05, CUDA 12.8.93 | full_scan | 0.114944 | 188.424 | 188.272 |
| RTX 4090 | probe | 0.190448 | 188.447 | 188.839 |

Probe/full kernel-median ratios by V are approximately:

- RTX 5090: **1.09×, 1.69×, 1.76×, 1.81×** at V = 64, 256, 1024, 4096.
- RTX 4090: **1.13×, 1.73×, 1.98×, 1.90×** at the same sizes.

These are per-card, same-source, same-configuration kernel-only ratios, not a cross-card comparison or end-to-end speedup. Direct and complete-caller scopes are dominated by wrapper overhead. The optimized full scan uses parallel distance evaluation with deterministic serial top-8 ordering, and only thread 0 publishes prediction/status.

Raw evidence and checksum read-backs:

- RTX 5090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/acc77e6f8b7f290deb4bab28a549be4083f3c9c4
- RTX 4090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/89e10d8b248383b152f2db81cd421e000b7bdf10

Machine-readable summaries are in `docs/labs/uor-cuda-inverted-file/gpu-performance-results.json`. All pods were deleted after verified archival. Rates for the final runs were $0.563/h and $0.789/h; no invoice total is inferred.

## Next decision

A non-author corrected RTX 5090 replication is in progress. Public release and upstream outreach remain on hold until that replication and the final exact-head review pass. No NVIDIA issue, performance announcement, or upstream acceptance is claimed.
