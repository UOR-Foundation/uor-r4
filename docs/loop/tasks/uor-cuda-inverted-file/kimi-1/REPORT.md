# REPORT: standalone UOR inverted-file CUDA package, kimi-1

Lab: kimi. Order: [ORDER.md](ORDER.md). References #3475.

This unit created a standalone CUDA C++17 example and its host evidence. No model training, model/API/worker/engine change, GPU evaluation, paid compute, production deployment, or Claude-owned file changed.

## What did not work

- The admission tests initially failed because C++17 lacks `std::span` and `starts_with`. I used C++17-compatible vectors and explicit prefix comparisons. A separate `.visible .entry` prefix off-by-one left entry names unrecognized; it was corrected after instrumenting the parser.
- The CSR validator initially missed stale annotations in the parallel `cells[token]` array. A new consistency check catches them. Removal/reinsertion and corrupted offsets are also covered.
- The first clean-copy hash comparison failed because one log contained compiler commands and the other did not. Normalizing to actual test output produced three identical results; no test was weakened.
- `nvcc` is unavailable, so AOT compilation and CUDA execution are **NOT_RUN**. The package reports this explicitly rather than treating the optional path as verified.
- The whole-branch review then found the CUDA link/loader defects, unsafe ABSENT updates, small distance sentinels, a tautological full-scan path, and reproducibility gaps. I made probe use actual CSR members while full scan independently enumerates tokens, separated explicit removal from non-root refusal, allocated update capacity, used `cuModuleLoad`, widened sentinels, added all-absent/corrupt/extreme controls, and added the reproducible `make evidence` bundle. Roadmap sync created the canonical issue #3475; the manual placeholder #3473 was closed instead of duplicating it.

## What changed

- Added a C++17 PTX admission gate with exact four-entry inventory and comment stripping.
- Added the fixed 16-root codebook, 256-cell CSR index, incremental updates, quota-limited probe, full-scan control, and typed invariant/refusal statuses.
- Added four matching CUDA entries and a minimal CUDA Driver API wrapper with explicit unavailable/failure separation.
- Added seeded and single-cell parity, non-root and K refusals, exact tie handling, 64 incremental updates, removal/reinsertion, and stale/corrupt/over-pruned controls.
- Added MIT license, source-commit provenance, README, CITATION, CODE_OF_CONDUCT, RESULTS, and the canonical lab record. The docs explicitly reject general-kNN, performance, portability, energy, and language-capability overclaims.

## Verification

- `make -s clean host-test` twice plus one clean-copy run: identical output, SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- Admission: **13/13 passed**. CPU parity/update/fault checks: **PASS**. Clean-copy build required no UOR-R4 checkout.
- `make cuda` and `make cuda-test`: **NOT_RUN**, `nvcc` unavailable. No generated PTX/cubin or GPU result is claimed.
- Documentation/scope checks and fresh exact-head review are recorded with the final delivery.

## Still open

- Compile and run CUDA parity on a supported GPU before any release or upstream submission.
- Measure a tuned full-scan baseline on at least two architectures, with repeated runs and separate kernel-only/end-to-end scopes, before any performance claim.
- Consider a Foundation Sandbox repository and NVIDIA/community feedback only after an external consumer can build and use the package. No such submission was made here.
