# ORDER: m2-fresh-heldout-panel, antigravity-22

Lab: antigravity. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `fb6df367b`.

## Goal

The item's done-when reads:

> A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

## Cancel probe (run first)

At `fb6df367b`:
1. `docs/harness/panels/m2-reply512.json` lists `"noise": "unstated..."` and only the opened 128-row transfer draw (`"fresh-transfer"`, `0/128`, exposed in `#2164` and below the 200-row floor of `CHARTER.md` §4.3).
2. Neither `docs/harness/panels/m2-fresh256.json` nor `docs/labs/m2-fresh-heldout-panel/` exists at `fb6df367b`.

Result: no unexposed `>= 200`-row M2 reply panel or measured development noise spread is registered at `fb6df367b`. Task is live. Class: instrument.

## Calls

- Post the written registration draft on `#2030` (and `#2849`) specifying how the 256 fresh questions (`128` histories × `2` opposite-role queries across `16` bank blocks: `64` `length2`, `64` `length4`, `64` `length8`, `32` `update`, `32` `reassert`) are deterministically generated, how exact and near-duplicate overlap is excluded against `dev512`, the opened `fresh128` draw, and every training/curriculum corpus, the measured noise spread on `dev512` from the sealed `saved437` parent (`archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json` and `archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json`), and that the existing deterministic grader (`uor-r4-core::native_geometric` / `StackStore` / `FrozenAnswers` membership) is unchanged.
- Build the standalone zero-float, zero-D23/D25 Rust panel generator, noise-spread analyzer, and overlap/seal verifier in `docs/labs/m2-fresh-heldout-panel/instrument/`.
- Freeze the sealed 256-row held-out panel (`m2-fresh256`, `ceil(0.40 * 256) = 103/256`) and measured development spread in `docs/harness/panels/m2-reply512.json` and `docs/harness/panels/m2-fresh256.json` without committing raw panel corpus files (`docs/reference/storage.md`).
- Update `docs/reference/storage.md`, `docs/mechanisms/native-geometric.md`, and `docs/labs/m2-fresh-heldout-panel/README.md` (including the 6-column research notebook row, noting that `docs/research/notebook/m2-grounded-reply.md` is outside `m2-fresh-heldout-panel.md`'s `scope:` frontmatter).
- Obtain a fresh-context non-author review of the overlap exclusion and seal before opening for merge.

## Files in scope

- `data/**`
- `docs/harness/panels/**`
- `docs/mechanisms/native-geometric.md`
- `docs/labs/m2-fresh-heldout-panel/**`
- `docs/reference/storage.md`
- `docs/roadmap/items/m2-fresh-heldout-panel.md` and `docs/roadmap/README.md`
- This task folder (`docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/**`)

## Done when

- `cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml` passes all unit, overlap-exclusion, noise-spread, tokenizer-budget, and negative-mutation tests (`0` floats, `0` D23/D25 sites).
- `docs/harness/panels/m2-reply512.json`, `docs/harness/panels/m2-fresh256.json`, `docs/reference/storage.md`, `docs/mechanisms/native-geometric.md`, and `docs/labs/m2-fresh-heldout-panel/README.md` record the frozen 256-row panel hashes, measured `dev512` spread, and `>= 103/256` (`>= 40%`) bar.
- Fresh-context non-author review verifies the overlap exclusion and seal; `uor-harness loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope or any owner path (`docs/harness/milestones.json`) needs to change, or if any model evaluation/training compute is needed.
