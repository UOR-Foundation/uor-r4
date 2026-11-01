# REPORT: standalone CUDA hardware correctness, kimi-2

Lab: kimi. Item: #3475. Order: [ORDER.md](ORDER.md). Passing source: `18a1c8ad79a468025b0e82e79705c7266abac03c`.

## Negatives and limits first

- Three FIT rounds stopped pre-lease work for an unreachable ABSENT→cell branch, a zero-middle fixture that could not test ranking, a short member readback, and evidence/acceptance-count defects. All were repaired before the final gate; no performance conclusion was drawn from them.
- Attempt 1 at `71c84dfa3` passed host/AOT/admission but failed C++ compilation on three const count addresses. No parity ran. It is sealed at `local/kimi/cuda-release-1/`.
- Attempt 2 at `aef414859` passed 498-row executions, but delivery review found weakened old-cell bounds, contradictory status text, and incomplete rebuild/negative artifacts. The driver now pre-validates structure, the kernel restores full bounds without closing ABSENT insertion, and a driver-side corrupt-CSR refusal row was added. Attempt 2 remains sealed at `local/kimi/cuda-release-2/`.
- The acceptance arithmetic was corrected to 84 rows/configuration × 6 = **504**, not 499. The planted offset refusal is driver-side; kernel-side corrupt-CSR rejection is **NOT_RUN**.
- `cuobjdump -arch` produced an empty file and is not evidence. Architecture evidence is compute capability 12.0, the `compute_120`/`sm_120` build flags, and successful four-entry Driver API loading on the named card.
- The original negative-command record used a non-replayable shorthand. The sealed record was not rewritten; a separate [erratum](https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/32b4ad06c04ff680de04ecb5a566d5c43433b176) supplies the actual `printf "\n.visible .entry planted_extra()\n" >> …` mutation and was verified after download.
- No performance, tuned-comparator, second architecture/driver, energy, cross-library, independent-replication, external-consumer, or release result was produced. Additional corrupt variants, all-absent CUDA scans, K=0/17, V=1/4096, and concurrent kernels remain NOT_RUN. CPU and CUDA are separate transcriptions, so shared design errors remain outside parity detection.

## Measured result

On `vast-58646251`, one NVIDIA GeForce RTX 5090 (compute capability 12.0, driver 580.178.04) with CUDA 12.8.93:

- Clean host oracle: 13/13 admission checks and all host parity/refusal/fault checks passed.
- AOT compilation: PASS. Emitted PTX admission: exactly four entries. A planted fifth entry was rejected; the modified PTX, exact command, and rejection log are archived.
- CUDA Driver API smoke: all build/update/probe/full-scan checks passed.
- Three `--out` executions: **504/504 rows passed in each; 1,512 total rows; all three files byte-identical**. Each includes the driver-side corrupt-CSR refusal.
- A clean rebuild produced identical PTX and cubin hashes, with `clean-rebuild.log`, `module-rebuilt.sha256`, and `clean-rebuild-check.log` retained. PTX SHA-256: `7cf388d8367038478fb07b69c352a386aff3b338f4ebaefaa54c927b3d1fe156`. Cubin SHA-256: `feb4cc77ba686865b2219398067d8f840b522886b878b9be48f0ef05ba792bc8`.
- Reinsertion matched a fresh CPU build exactly. Probe used the GPU-built CSR; full scan intentionally used a CPU-built control CSR.
- Evidence was pulled with SHA-256 verification to `local/kimi/cuda-release-3/`, uploaded to the [project store](https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/e2d9b16cce8668138cf1ca04cad4d1c7c3fd4dd2), and verified again after download. The lease was released and the pod deleted.

The three leases were each quoted at $0.563/h. Provider invoice totals were not collected, so no exact dollar spend is claimed. Machine-readable results are in `docs/labs/uor-cuda-inverted-file/gpu-sm120-results.json`.

## Decision

The declared single-card correctness gate passes with full bounds checks and a self-proving archive. This unlocks only the separately reviewed matched full-scan performance instrument and additional-architecture work. It is not a performance, portability, production, or public-release qualification. The package's advertising hold remains in force.
