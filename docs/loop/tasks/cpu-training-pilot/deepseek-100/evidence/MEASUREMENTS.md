# Public corpus worker-budget readings

Host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.
Driver SHA-256: `18a99d0602bdddda4c3cf97e7eb96112ac73c7d733b52c9e5f41e9fb5a9e55ba`.
Configuration: public dual-c131 seed 1, rank-margin seed 1, 16 update attempts,
evaluation stride 2, panel cap 1,024 and checkpoint interval 16.

## One run at each worker count

The forward and reverse columns are independent process runs with the order of
worker counts reversed for the second invocation.

| Workers | Forward wall ms | Reverse wall ms | Speedup vs one (forward) | Speedup vs one (reverse) | Average CPU |
|---:|---:|---:|---:|---:|---:|
| 1 | 62,108.924 | 62,226.693 | 1.000x | 1.000x | 99% |
| 8 | 8,491.539 | 8,506.975 | 7.314x | 7.315x | 759-761% |
| 16 | 4,635.815 | 4,616.408 | 13.398x | 13.479x | 1,445-1,447% |
| 32 | 3,550.101 | 3,561.695 | 17.495x | 17.471x | 2,719-2,735% |

Peak RSS was 52,712-52,984 KiB at one worker, 67,576-67,872 KiB at eight,
86,596-87,872 KiB at 16 and 124,524-127,456 KiB at 32.

## Forward-order stages

| Workers | Setup | Baseline panel | Online updates | Final panel | Checkpoint + reload | Final audit |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 5.289 | 30,851.558 | 283.910 | 30,893.506 | 13.058 | 47.907 |
| 8 | 4.150 | 4,053.015 | 283.622 | 4,076.217 | 12.623 | 47.982 |
| 16 | 4.098 | 2,134.839 | 282.350 | 2,139.736 | 12.611 | 47.873 |
| 32 | 4.700 | 1,592.146 | 285.695 | 1,591.774 | 12.561 | 49.058 |

All values are milliseconds. The two full-panel comparisons dominate wall time;
the online update chain and the recurrent 64-position served-forward audit are
dependency-ordered and remain effectively constant across worker counts.

## State equality

All eight single-artifact runs produced:

- final artifact SHA-256
  `6b35fed04be8a98168b7a7a0a22030300de5383b7576c0c06a812f28e7bb38c8`;
- final checkpoint SHA-256
  `b3a9c501c42d94f88091f4360a9b0a67dc8d294ebf5bdbb0d58b98de20e6ce24`;
- panel `278 / 384 / 396` top-1/top-8/admitted;
- served-forward decision SHA-256
  `2ac6b0971afe222860453846f286385428bb870e802fccf66a6097965ea1b897`;
- zero query-coordinate, token or kept-cell mismatches over 262,144 Q12
  coordinates and 8,192 kept cells.

Removing worker count, stage/wall timing, path and timing-derived throughput
fields leaves one identical semantic report checksum across 1, 8, 16 and 32
workers: `cbfa12224b69d507bbf7b8328cb486ff0c86ddd93671994624c1e7cb60d6b1f8`.

## Independent two-artifact scheduling

Two public artifact seeds were compared with the same 16-update work:

- two sequential 32-worker jobs: 6.98 s wall, 2,785% average CPU, 124,736 KiB
  peak RSS;
- two concurrent 16-worker jobs: 6.60 s wall, 2,927% average CPU, 87,256 KiB
  maximum per-process peak RSS.

The concurrent schedule was 5.4% faster but used two simultaneous process
footprints. Seed 1 trained and seed 2 accepted no update; each output matched
its corresponding sequential run exactly.

## Recommendation

Use 32 Rayon workers for a single panel-heavy CPU corpus run on this host. When
wall time is the only objective and two or more independent artifacts are
ready, run at most two 16-worker corpus jobs concurrently. Keep the online
rank-margin chain and recurrent served-forward audit serial; splitting them
would change dependency order rather than expose independent work.
