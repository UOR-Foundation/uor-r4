# Before advertising or submitting this CUDA package

**Current decision: do not advertise or submit upstream until corrected independent replication and final review complete.** Host and single-card correctness pass. Final corrected measurements at `719a9a78` pass on RTX 5090/580.178.04 and RTX 4090/580.95.05: 504 correctness rows and 240 measurements per card, with per-launch oracle checks, complete-caller warmups, and the corrected clock boundary. All earlier measurements remain rejected and are not used for claims.

The useful community contribution is the combination of an exact integer inverted-file index and a mechanical PTX instruction/inventory gate. It does not require adopting UOR-R4's model or geometry. That usefulness still needs the following evidence.

## 1. Build and admit the real emitted module

On a compatible GPU machine, from a clean checkout:

```sh
UOR_NVCC_ARCH=compute_80 make cuda
```

Use the architecture appropriate for the actual card. Record the CUDA Toolkit, compiler, driver, GPU, architecture, command, source commit, and module hashes. Inspect the emitted PTX: the admission gate must accept exactly `uor_if_build`, `uor_if_update`, `uor_if_probe`, and `uor_if_full_scan`, and reject forbidden instructions or inventory drift. A host admission fixture is not a substitute for admission on the emitted bytes.

## 2. Execute the CUDA Driver API and prove parity

```sh
UOR_NVCC_ARCH=compute_80 make cuda-test
```

Demonstrate CPU/GPU agreement for all four operations, not just successful module loading. Cover the existing varied sizes and K values, exact ties, non-root refusal, stale/corrupt CSR, 64 incremental updates, removal/reinsertion, and all-absent/extreme cases. Run negative controls and repeated determinism checks. Report failures as failures; `CudaUnavailable` is not a pass.

## 3. Add a tuned, matched full-scan control

Benchmark against an optimized full-scan implementation using the same inputs, exact arithmetic, admission rule, candidate set, output, and transfer boundaries. Do not use the old spilling baseline as if it were a tuned comparator. Include single-cell, sparse, dense, small-V, large-V, and K-sensitive cases so the index's break-even region is visible. No general-kNN or changed-recall claim is licensed by this control.

## 4. Cover at least two architectures and two driver versions

Repeat correctness and measurements on at least two explicitly named NVIDIA architectures and two driver versions. Keep each result attached to its hardware, Toolkit, driver, architecture flags, and emitted module. Do not infer portability from PTX text or from one card.

## 5. Separate measurement scopes and publish raw evidence

For every configuration, publish raw per-run logs, warm-up and repetition counts, distributions or ranges, and the exact timing boundary. Report these separately:

- **Kernel-only:** admitted cubin execution on already resident inputs.
- **Direct writer/API:** index/probe work without the surrounding model.
- **End-to-end:** the complete declared caller, including transfers, allocation, synchronization, and construction costs.

Include register/resource use, occupancy, spill reports, memory traffic, and build/update break-even analysis where measured. Keep failures and variance. Do not extrapolate energy or cross-library performance without separate measurements.

The historical UOR-R4 bucket-writer's **45.5×** figure was a direct-writer ratio, not end-to-end speedup; its historical end-to-end figures were **1.68× forward** and **1.76× evaluate** on one RTX 5090 setup. Neither number belongs to this standalone package. See `docs/labs/m1-faster-writer/README.md` for that provenance.

## 6. Replicate independently and test an external consumer

Have a non-author repeat the correctness results on the declared hardware. Then have an external consumer copy only this directory into a clean checkout, follow the README without repository-specific knowledge, and build and run it. The existing clean-copy check proves host build independence, not independent hardware replication or external usability. Resolve their feedback before calling the package reusable.

## 7. Package the release evidence

Pin a source commit and version, retain MIT licensing and provenance, checksum source and emitted artifacts, and link the raw correctness and benchmark records. State precisely which architectures and operations passed and everything still NOT_RUN. Keep the telemetry consumer opt-in; do not silently add collection to a community example.

## 8. Then approach the community

Once those gates pass:

1. Open a **clarification issue** in NVIDIA's `cuda-samples` repository asking whether this self-contained example and its admission tooling fit their scope and contribution process. Do not open an architecture pull request or claim NVIDIA endorsement.
2. Start an **NVIDIA Developer Forums** thread with the minimal example, exact parity evidence, matched-control results, and a specific technical question.
3. Consider a **UOR Foundation Sandbox** application as a home for community development if maintainers can support it. Its existence would not imply NVIDIA acceptance.

A generic model-artifact-digest convention for OpenTelemetry is a separate proposal, not part of CUDA submission. No NVIDIA issue, pull request, forum post, Sandbox application, or performance announcement has been made by this unit.
