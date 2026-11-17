# REPORT for the M6 verifier temporary files

Lab: codex. Item: [m6-code-task-rustc-verifier](../../../../roadmap/items/m6-code-task-rustc-verifier.md). [ORDER](ORDER.md). Fixes #3876.

## What did not work

Two canonical test compilations previously could write task.rs in the same directory. The contribution resolved that collision. Reviewing the subprocess caller found a second clock-only name for stdout and stderr: two simultaneous calls could still return an I/O error at file creation.

## What changed

Preserved @Rajaykumar12's original commit from [#4038](https://github.com/UOR-Foundation/uor-r4/pull/4038). A process-wide sequence and exclusive directory creation give each verifier call its own directory. The compiler and test executable write their captures beside their source and binary, under distinct compile and run basenames. The directory guard cleans them up on either success or failure.

## Measurements

Three temporary-directory tests passed, including the 16-thread same-label test. Seven integration tests passed, including the expected compile error, assertion failure, timeout, refusals for missing test results or zero tests, and all 80 frozen canonical programs.

## Checks run at the head

Rust 1.97.1 on macOS arm64. Ran the harness library filter `m6_rustc_verifier` and integration target `panel_m6_rustc_verifier` at source `41fc05e72`. Harness formatting, the whitespace check and claim wording passed. The subsequent merge of main brought in an unrelated integer parity test and its records.

## Skills used

Source inspection and host validation under the ORDER.

## Still open

The unit covers trusted canonical verifier inputs and concurrent host test isolation. M6 model evaluation is outside this repair.

## Plain summary

Parallel verifier calls have separate sources, binaries and captured output. The frozen test panel passed on the maintainer host.
