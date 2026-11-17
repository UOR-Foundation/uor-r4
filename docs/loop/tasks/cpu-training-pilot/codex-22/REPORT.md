# REPORT — carry-hold receipt retention and platform coverage

Lab: codex. [ORDER](ORDER.md). Maintainer follow-up to [PR #3889](https://github.com/UOR-Foundation/uor-r4/pull/3889).

## What did not work

At `d824de345`, the batch success test expected a Linux descriptor operation on macOS, and both optional test hooks could truncate existing receipts. In this verification, the default Linux cross-build also failed to find Linux C headers for BLAKE3’s NEON implementation.

## What changed

Both platform tests use the same valid two-arm fixture. Linux tests the ordering and byte-identical repeat. Other targets require the documented Linux-containment refusal and verify that no report exists. The two optional receipt hooks call the same create-new writer. Its regression first retains a receipt, attempts to replace it with a shorter one and compares the saved bytes to the original.

## Measurements

On Apple silicon macOS all eight tests of the audit binary passed. A separate invocation exercised the actual single-report retention hook twice against the same destination. The second invocation failed with an existing-file error, and byte comparison confirmed the original SHA-256 `c2c83d0cbf500e46400e750327470ab40b6ee9cca3c868ad2fd77c9660074fc3` was preserved.

## Checks run at the head

- `cargo test --locked --offline -p uor-r4-cpu-train --bin cpu-carry-hold-audit` on macOS.
- `cargo check --locked --offline --target aarch64-unknown-linux-gnu -p uor-r4-cpu-train --tests --features blake3/pure` checked the Linux test code with the portable hash backend.
- `rustfmt --check --edition 2021` on the changed Rust file and `git diff --check` passed.

## Skills used

Source review, native host testing and Linux cross-compilation.

## Still open

The repaired Linux success test was type-checked. Runtime verification on Linux remains outstanding at this commit. The historical eight-checkpoint evidence and resource ledger retain their original recorded scope.

## Plain summary

A repeated test can no longer overwrite a retained receipt. The tests exercise the platform behavior the tool actually supports.
