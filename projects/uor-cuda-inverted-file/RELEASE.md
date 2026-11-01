# UOR Inverted-File CUDA Example 0.1.0

A downloadable source-and-evidence release of the standalone exact integer inverted-file index, CPU oracle, optional CUDA Driver API implementation, and build-time PTX admission gate. This is a scoped community-review release, not the UOR-R4 model's M8 release and not an NVIDIA-endorsed sample.

## Assets and verification

- `uor-cuda-inverted-file-0.1.0.tar.gz`: the complete standalone project, its CUDA lab evidence and task records, and the linked telemetry conventions. No generated binaries, model checkpoint, corpus, or third-party data are included.
- `SHA256SUMS`: the SHA-256 manifest for the archive.

Download both assets, then verify and extract them:

```sh
shasum -a 256 -c SHA256SUMS
tar -xzf uor-cuda-inverted-file-0.1.0.tar.gz
cd uor-cuda-inverted-file-0.1.0/projects/uor-cuda-inverted-file
make host-test
make telemetry-test
make evidence
```

The optional CUDA build requires `nvcc`, a compatible NVIDIA driver, and a supported GPU. Choose the architecture for the target card, for example:

```sh
UOR_NVCC_ARCH=compute_120 make cuda
UOR_NVCC_ARCH=compute_120 make cuda-test
```

A missing toolchain or GPU is `NOT_RUN`, not a passing CUDA result. See `README.md` for the API and `examples/otel/README.md` for the separate, explicit loopback collector experiment.

## Source identity

Release tag: `cuda-inverted-file-v0.1.0`, targeting the protected `main` merge commit that contains this release definition. The measured CUDA implementation is `719a9a7861deb595dfc5269ffec8a5e7e53cb39b`; the only later changes in the standalone project before this release metadata were documentation, so the released CUDA/telemetry implementation is unchanged.

## Measured evidence

- Host and clean-copy checks: 13/13 PTX admission checks plus all host parity, update, refusal, and fault checks.
- RTX 5090/580.178.04 and RTX 4090/580.95.05: 504 correctness rows and 240 measurements per card, with per-launch oracle checks and complete-caller warmups.
- Independent corrected replication on another RTX 5090/595.71.05: the same 504 correctness rows and 240 measurements passed.
- Earlier measurements remain rejected for publication-race and benchmark protocol/timing defects; they are not used for these results.

| Card / driver | Operation | Kernel median (ms) | Direct API median (ms) | Complete-caller median (ms) |
|---|---|---:|---:|---:|
| RTX 5090, 580.178.04 | full scan | 0.109856 | 115.493 | 115.407 |
| RTX 5090, 580.178.04 | probe | 0.160144 | 115.678 | 115.620 |
| RTX 4090, 580.95.05 | full scan | 0.114944 | 188.424 | 188.272 |
| RTX 4090, 580.95.05 | probe | 0.190448 | 188.447 | 188.839 |

Probe/full-scan kernel-median ratios are 1.09×, 1.69×, 1.76×, and 1.81× on the RTX 5090 and 1.13×, 1.73×, 1.98×, and 1.90× on the RTX 4090 at V = 64, 256, 1024, and 4096. These are same-card, same-configuration **kernel-only** ratios; wrapper overhead dominates the direct and complete-caller scopes.

Raw evidence commits:

- RTX 5090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/acc77e6f8b7f290deb4bab28a549be4083f3c9c4
- RTX 4090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/89e10d8b248383b152f2db81cd421e000b7bdf10
- Independent replication: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/dddfebf8a5ccd006a5f9e0045e1e46b2feeacb82

## Not claimed

This is not general exact kNN: bucket probe and full scan consider only the K admitted cells, preserving recall. No end-to-end speedup, cross-card ranking, energy result, cross-library comparison, production deployment, language-model capability, or NVIDIA acceptance is claimed. Architectures, drivers, sizes, concurrency, and fault cases outside the recorded grid remain untested.

`OUTREACH.md` contains optional community-discussion drafts. Publishing this release does not submit them or request NVIDIA architecture changes.
