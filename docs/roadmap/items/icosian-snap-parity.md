---
id: icosian-snap-parity
title: Reproduce the sealed S1.4 icosian transport comparison
milestone: research
area: evaluation
status: ready
open_to: anyone
needs_gpu: true
depends_on: []
---

## Why it matters

The archived kernel has a sealed result that an external researcher can reproduce with public inputs.

## What is missing

The model files, held-out tokens and SHA256SUMS are public. The original question about obtaining inputs is resolved. The registered comparison still needs an independent replay receipt against the sealed counts. [Issue 2655](https://github.com/UOR-Foundation/uor-r4/issues/2655#issuecomment-6300190469) · [Issue 2655](https://github.com/UOR-Foundation/uor-r4/issues/2655#issuecomment-6391158177)

## Done when

Under the reviewed sheet, report selections, root mismatches and top-1 agreement against the sealed comparison, with hashes and hardware identity. Record a differing result without tuning the registered comparison.

## How to start

Use the contributor instructions and public inputs linked below. Attach hardware specifications and use the registered non-author FIT for the model execution.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/2655)

<details>
<summary>Original report by Cyborg-Hawk-AI on 2026-10-18</summary>

## Goal

Serve the 2I icosian transport snap (the S4-adopted main line's transport) as a D11 kernel, with parity against the sealed snap-parity record.

## What exists (measured)

The snap path (`stack_snap_select`, `stack_snap_rotation`) is Z[φ] shift-add argmax over the 120 roots + the 120×120 group table (`uor-r4-core/src/native_geometric/learner/group_table.rs`), with the exact Z[φ] tie-break. On x86_64 it audits R2-clean:

- `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack` = **FULL PASS** (0 forbidden instructions, 44 symbol ranges).

## Frozen gate (pre-registration)

Parity against `docs/evidence/s1-4-snap-parity-2026-09-29.json` (the sealed record, top1 agreement 0.9919, root mismatches 2477/1177878). Pre-registering the run before any fit, per the rules.

## Cost

The parity run needs the S4-arm artifact and the chat-v0 held-out set, which I do not have. The kernel and the native audit are in PR #2654.

</details>
