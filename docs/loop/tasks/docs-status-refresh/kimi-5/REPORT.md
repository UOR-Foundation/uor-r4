# REPORT: correct the current CUDA module inventory, kimi-5

Lab: kimi. Order: [ORDER.md](ORDER.md). References #3380.

This is a documentation-only unit. No model training, GPU evaluation, paid compute, deployment, or kernel/source changes occurred.

## What did not work

- The guide said `discrete.cu` had 577 lines and 12 entries. Current source has 718 lines and 13; `discrete_carrier_select` was missing from the table.
- It described five generated modules and cubin assembly for six modules. Current `build.rs` assembles nine modules: the ordinary module plus eight specialized modules, with 42 entries total.
- The first mdBook build failed because `mermaid.min.js` had not been installed in the fresh worktree. Running the standard `mdbook-mermaid install .` setup fixed the environment; no check or source assertion was weakened.

## What changed

- Corrected the line and ordinary-entry counts and added carrier selection to the kernel table.
- Named all eight specialized modules and their entry counts, with 42 entries across nine modules.
- Clarified that specialized modules are explicitly opt-in and changed the cubin count from six to nine.
- Independently researched possible NVIDIA/community publication paths; those recommendations remain outside this documentation diff and require separate owner decisions, cleanup, and verification.

## Checks

- `cargo test -q --locked -p xtask`: **220 passed, 0 failed**.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: pass.
- `mdbook-mermaid install .`, `mdbook build`, and `cargo run -q --locked -p xtask -- book-check`: **105 pages, 0 broken links**.
- `git diff --check`: pass.

A fresh exact-head review follows before protected delivery.
