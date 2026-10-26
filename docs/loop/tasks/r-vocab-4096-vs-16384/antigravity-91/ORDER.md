# ORDER: r-vocab-4096-vs-16384, antigravity-91

Lab: antigravity. Item: [r-vocab-4096-vs-16384](../../../../roadmap/items/r-vocab-4096-vs-16384.md). Base: `5a104c6f8`.

## Goal

Complete roadmap item `r-vocab-4096-vs-16384` (`#2887`, "Compare 4,096 and 16,384 tokenizer pieces", unlocked by `#2840` `m1-faster-writer` in `5a104c6f8`). The item's done-when reads:

> Matched runs at 4,096 and 16,384 pieces with the same registered bars, recorded as notebook rows.

## Cancel probe (run first)

1. **Verify existing matched `V = 4,096` vs `V = 16,384` runs and tests before any new GPU spend (`CHARTER.md` §3.5, §4.2):**
   - **Matched GPU writer & serving-cost runs (`#2840`, `5a104c6f8`):** Under the registered sheet `docs/harness/prep/m1-faster-writer-2026-10-26.md` (`TEST FITNESS: FIT`, `D68`, `D67`), `writer-bucket-qualify` ran matched GPU evaluations at `V = 4,096` and `V = 16,384` (`B = 4`, `K in {1, 8, 16}`, 2 seeds x 32 rows per `(V, K)` plus uniform occupancy `16` vs `64` tokens/cell at `K = 8`) across two independent `sm_120` builds (`Build A` `37419bdc303b79ac4beac0fab0ea292d74dc34ac9623754457e9107d89a40fd0` and `Build B` `df583e45f9093df9b551486b0f909349de12df9bb5280955e7121b9d494fc528`) on `1x NVIDIA GeForce RTX 5090` (`vast-57796936`), recorded in `docs/labs/m1-faster-writer/README.md` and `docs/research/notebook/m1-discrete-attention.md` row 79. Re-verify finite controls with `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls --no-fail-fast`.
   - **Matched BPE tokenizer & numeric-boundary runs (`#2872`, `number-aware-tokenizer`):** Run `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-tokenizer --no-fail-fast` (`23/23` tests pass, including `reaches_16384_on_bounded_synthetic_text_and_preserves_runtime_roundtrip` and `number_aware_three_arm_comparison_and_digit_piece_census` across `V = 4,096` and `V = 16,384`: `ctrl1` `Digits(true)+ByteLevel`, `ctrl2` plain `ByteLevel`, `trt` `Digits(false)+ByteLevel`, 2 runs/arm on the `170,176`-byte deterministic benchmark corpus `05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`).
   - **Discrete-driver (`B = 4`) capacity audit at `V = 16,384` (`classification: instrument` / architectural boundary):** Verify the five explicit `V <= 8,192` / `V <= 4,096` guards in `crates/uor-r4-discrete/src/lib.rs:83`, `prime_zeta.rs:108`, `address_cells.rs:322,384`, `discrete.cu:633`, and `crates/uor-r4-integer/src/discrete_d11.rs:667` that cap `StructuredV1` (`256` cells x `16` tied interior roots = `4,096`) and `Spec::validate()` (`2..=8192`), proving why a full discrete-driver training run at `V = 16,384` cannot execute without first widening interior-block indexing (`(k / 16, k % 16)` at `B = 4`) and raising the `8,192` validator cap, and why no redundant GPU pod lease should be spent (`CHARTER.md` §3.5: *"Spend GPU only where a run can change a decision"*).
   - **Cancel probe result:** `23 / 23` `uor-r4-tokenizer` tests pass and `8 / 8` `writer_bucket_controls` tests pass (`classification: mechanism` on the matched `V = 4,096` vs `V = 16,384` GPU writer & BPE comparison; `classification: instrument` on the 5 discrete-driver `V <= 4,096` / `8,192` guards).

## Files in scope

- `docs/harness/prep/r-vocab-4096-vs-16384*`
- `docs/labs/r-vocab-4096-vs-16384/**`
- `docs/loop/tasks/r-vocab-4096-vs-16384/**`
- `docs/roadmap/items/r-vocab-4096-vs-16384.md` (status line)
- `docs/roadmap/README.md` (rendered via `xtask roadmap render`)

## Done when

1. Cancel probe (`cargo test -p uor-r4-tokenizer --no-fail-fast` and `uor-r4-discrete` capacity audit) passes (`classification: mechanism` on the matched `V = 4,096` vs `V = 16,384` GPU writer & BPE comparison; `classification: instrument` on the 5 discrete-driver `V <= 4,096` / `8,192` guards).
2. One-page comparison sheet `docs/harness/prep/r-vocab-4096-vs-16384-2026-10-26.md` and D68 decision card `docs/loop/tasks/r-vocab-4096-vs-16384/antigravity-91/DECISION.md` pass fresh-context `TEST FITNESS: FIT` and `D68` decision review, with `[antigravity] owner log:` posted on `#2887`.
3. `docs/labs/r-vocab-4096-vs-16384/README.md` and `docs/loop/tasks/r-vocab-4096-vs-16384/antigravity-91/REPORT.md` record the matched `4,096` vs `16,384` runs against the registered bars (leading with `tokens/s`) and the notebook rows (`docs/research/notebook/m1-discrete-attention.md` row 79 and `data-tokenizer` row specification), `r-vocab-4096-vs-16384.md` is set to `status: done`, and all gates pass.

## Stop and ask if

- Any check or review requires touching files outside the declared scope, or changing a `CHARTER.md` §3 constraint or GPU cap.
