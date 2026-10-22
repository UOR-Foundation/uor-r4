# REPORT: m3-nonrelation-prose-panel, antigravity-10

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2856.

## What did not work, first

- In the initial draft of `data/panels/nonrelation-prose-v1.txt`, row `nrp-028` had a missing space (`"thedistant"` instead of `"the distant"`), which was fixed before freezing the SHA-256 digest (`7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`).
- In the first compile of `crates/uor-r4-training/src/nonrelation_prose_panel.rs`, `StackStore::new(16)` failed with `error[E0061]: this function takes 2 arguments but 1 argument was supplied` (`StackStore::new(lineage, capacity)` takes `(u64, usize)`) and matching `Written::Corrected` / `Written::Conflict` as enum variants failed with `error[E0223]: ambiguous associated type` because `Written` in `scoped_memory.rs` is a struct with fields `action: Update` and `conflict: bool`.
- As a deliberate negative-gate test of the `#1718` false-overwrite detector, removing `written.action == Update::Correct` from `overwrote_stored_fact` in `apply_prose_action` caused `nonrelation_prose_panel::tests::grader_detects_false_assert_and_false_overwrite_including_issue_1718_relation_4` to fail as expected (`panicked at crates/uor-r4-training/src/nonrelation_prose_panel.rs:704:9: assertion left == right failed: left: 2, right: 3`).

## What changed

- Posted the design note on `#2856` (`https://github.com/UOR-Foundation/uor-r4/issues/2856#issuecomment-6337042540`) and the proposed pass-bar decision card to the PM/owner on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`, CHARTER §5.2).
- Added the frozen 50-turn plain-text non-relation prose panel at `data/panels/nonrelation-prose-v1.txt` (SHA-256 `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`, registered in `data/panels/MANIFEST.sha256`), with 10 turns in each of the 5 categories (`small_talk`, `opinion`, `nonrelation_question`, `anaphoric`, `statement_near_miss`) split into 25 `in_domain` probes and 25 `heldout_v2_disjoint` turns.
- Added `crates/uor-r4-training/src/nonrelation_prose_panel.rs` (wired into `crates/uor-r4-training/src/lib.rs` under `#[cfg(test)]`) implementing `ProsePanel::load_frozen`, `seed_world_v2_store`, `apply_prose_action`, `grade_prose_panel`, and 4 unit tests covering SHA-256 verification, clean `Unresolved` grading, false-assert and `#1718` false-overwrite detection on relation 4 (`hometown`), and 4-word run disjointness of the held-out slice from `milestone_world.rs` and `milestone_world_v2.rs`.
- Added the one-page pre-registration sheet `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`) and lab record `docs/labs/m3-nonrelation-prose-panel/README.md`.

## Measurements

- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib nonrelation_prose_panel` on macOS aarch64: 4 passed, 0 failed in 0.06s.
- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib` on macOS aarch64: 42 passed (38 existing + 4 new in `nonrelation_prose_panel::tests`), 0 failed in 0.32s.
- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training` on macOS aarch64: 67 passed total across 3 test binaries (`lib`: 42, `dialogue-recall-corpus` bin: 25, `doc-tests`: 0), 0 failed.
- `shasum -a 256 -c data/panels/MANIFEST.sha256`: 31/31 files OK (including `nonrelation-prose-v1.txt`).
- `uor-harness ratchet d23`: OK (`4451` remaining across `165` files, baseline `4451`, `+0`).
- `uor-harness ratchet d25`: OK (`93` remaining across `26` files, baseline `93`, `+0`).

## Gate results

See `receipts.jsonl` in this folder, bound to the final commit via `uor-harness loop gates --full` and `uor-harness loop report --receipts`.

## What remains open

- Running the `world=v2` compiler over `data/panels/nonrelation-prose-v1.txt` to record baseline `false_assert` and `false_overwrite` rates is a GPU evaluation run and stays gated on a non-author FIT review of `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md`, the owner's pass-bar decision (`#2028`), and a typed D40 owner release (CHARTER §4.5, §5.1, §5.2).

## Decisions requested

- Owner / PM decision on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`): confirm the pass bar (`false_overwrite == 0 / 50`, `false_assert == 0 / 50`, `clean_unresolved == 50 / 50`) for `nonrelation-prose-v1.txt`.
