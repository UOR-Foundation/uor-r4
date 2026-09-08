---
name: uor-research-architect
description: >-
  Lead AI Research Architect workflow for UOR-R4: executing native geometric LM training,
  running focused verification tests, checking resource ledgers, and recording BLAKE3 empirical receipts.
---

# UOR-R4 Research Architect Workflow

Use this skill when developing, training, benchmarking, or validating native geometric LM components in UOR-R4 under owner-directed recovery mode `#973`.

## 1. Preflight Verification & Resource Audit

Before executing any build or training cycle:
1. **Check Storage Margin:**
   Verify accounted storage does not violate the active cap (+128 MiB stop margin):
   ```bash
   # Verify current storage against the approved cap (4,445,962,240 bytes)
   git status
   ```
2. **Project Model Time:**
   Ensure projected training and test execution stays well within the remaining model budget (852.938 seconds remaining of 1,800s).

## 2. Focused Development Checks

Adhere to the focused verification policy in `AGENTS.md`:
* **Formatting:** `cargo fmt --check`
* **Typecheck Touched Package:** `cargo check -p <touched-package> --all-targets --offline`
* **Focused Unit Test:** `cargo test -p <touched-package> <focused-test> --offline`
* **Zero-Allocation Verification:** For `uor-r4-graph-runtime`, verify `#![no_std]` and zero-allocation proofs in `uor-r4-proof-model`.

## 3. Empirical Sealing & Receipt Protocol

When a training or verification run completes:
1. Compute and record BLAKE3 hashes for all generated model artifacts and test outputs:
   ```bash
   b3sum path/to/model.json
   ```
2. Update the corresponding issue record in `docs/` and `docs/integration/current-state.md` with:
   - Exact input/output CID
   - Number of training examples and validation outcomes
   - Exact elapsed wall time and peak memory consumption
3. If a negative result is observed, preserve the artifact and mark the outcome truthfully (never delete or retroactively adjust criteria).
