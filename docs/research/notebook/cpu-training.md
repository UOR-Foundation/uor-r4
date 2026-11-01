# Notebook: CPU training research (`cpu-training`)

This notebook records CPU training and evaluation research authorized by D84. CPU rows name the workstation hardware, the isolated module, the exact evaluator and the controls. GPU remains the qualification authority, so a CPU row is development evidence unless a later owner decision changes that boundary.

Background: [the CPU training repair program](../cpu-training-program.md).

## Results

Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-11-01 | [deepseek] Built `crates/uor-r4-cpu-train` and its independent `OuterL1V1` CPU decision core: checked canonical-root decoding, 256-cell admission, full-vector ranking, top-8 and target admission. Ran seven finite tests, 256-row parity against the existing host writer oracle, and a 32,768-row release benchmark on the Ryzen 9 9950X. | **Parity:** 256 / 256 top-8 and admitted-cell decisions matched exactly. **Benchmark rows/s:** 221,526 at 1 thread; 1,319,832 at 8; 2,332,450 at 16; 3,970,189 at 32. Every thread count returned checksum `4667609706970220960`. | component: exact host-oracle parity, distinct top-8, lower-token ties, malformed-input refusal, deterministic benchmark checksum. Stored-GPU row parity had no exact fixture and remains open. | **positive (component scope):** the independent CPU writer reproduces the host oracle and scales, but this is neither training throughput nor a trained-model parity result | [REPORT](../../loop/tasks/cpu-discrete-writer-core/deepseek-1/REPORT.md) |
