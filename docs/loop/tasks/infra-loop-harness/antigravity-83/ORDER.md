# ORDER: infra-loop-harness, antigravity-83

- Item: `infra-loop-harness` ([docs/roadmap/items/infra-loop-harness.md](../../../roadmap/items/infra-loop-harness.md), issue #2926)
- Lab: `antigravity`
- Branch: `antigravity/unblock-m1-m4-sheet-scopes`
- Base: `12facdf9471aa228459e9dd6c2cef91b2ce9bdf8`

## Goal

Narrow the `scope:` of four ready roadmap items whose code and host controls are already merged on `main`, so their pre-registration sheet / `TEST FITNESS: FIT` and evaluation units are not blocked by `PR #3231` (`docs/mechanisms/hamilton-gate.md`) or `PR #3230` (`crates/uor-r4-discrete/**`), following the `claude-105`/`claude-108` (`#3223`, `#3229`) and `antigravity-80` (`#3232`) precedent:

1. `docs/roadmap/items/m1-normalization-mechanism.md` (`#2842`, milestone M1): remove `docs/mechanisms/hamilton-gate.md` (which overlaps `PR #3231` and is not edited by the `m1-normalization-mechanism` sheet + `TEST FITNESS: FIT` close unit).
2. `docs/roadmap/items/m4-flock-coupling-prereg.md` (`#2860`, milestone M4): remove `crates/uor-r4-discrete/**` (its 7/7 host controls in `flock_coupling_controls.rs` are merged on `main`; only the sheet + `TEST FITNESS: FIT` + notebook row remain), and add `notebook: docs/research/notebook/m1-discrete-attention.md` and `docs/loop/tasks/m4-flock-coupling-prereg/**`.
3. `docs/roadmap/items/r-prime-zeta-vs-nulls.md` (`#2881`, milestone research): remove `crates/uor-r4-discrete/**` (its constructor, CLI, and 7/7 host tests are merged on `main`; only the sheet + `TEST FITNESS: FIT` and zero-update evaluation battery remain), and add `notebook: docs/research/notebook/m1-prime-zeta.md` and `docs/loop/tasks/r-prime-zeta-vs-nulls/**`.
4. `docs/roadmap/items/r-read-off-vs-read-on.md` (`#2885`, milestone research): remove `crates/uor-r4-discrete/**` (its kernel switch, 7/7 host controls, and battery script are merged on `main`; only the sheet + `TEST FITNESS: FIT` and zero-update evaluation battery remain), and add `notebook: docs/research/notebook/m1-discrete-attention.md` and `docs/loop/tasks/r-read-off-vs-read-on/**`.

## Cancel probe

- Command:
  ```sh
  uor-harness loop claim infra-loop-harness --lab-topic unblock-m1-m4-sheet-scopes
  gh pr diff 3169 --name-only
  gh pr diff 3230 --name-only
  gh pr diff 3231 --name-only
  ```
- Cancel if: any of the four target item files overlaps an open PR (`#3169`, `#3230`, `#3231`), or any of the four items still requires unmerged edits in `docs/mechanisms/hamilton-gate.md` or `crates/uor-r4-discrete/**` before its sheet/`TEST FITNESS: FIT` unit.
- Class: `instrument`
- Result: **PASS (2026-10-25).** `uor-harness loop claim` refused `infra-loop-harness` only because idle draft `PR #3169` (`codex-1`) holds the umbrella item (`PLAN.md` and `docs/integration/DECISIONS.md`). Per the umbrella exception, `gh pr diff` on `#3169`, `#3230`, and `#3231` confirms zero file overlap with `docs/roadmap/items/{m1-normalization-mechanism,m4-flock-coupling-prereg,r-prime-zeta-vs-nulls,r-read-off-vs-read-on}.md`.

## Files and areas in scope

- `docs/roadmap/items/m1-normalization-mechanism.md`
- `docs/roadmap/items/m4-flock-coupling-prereg.md`
- `docs/roadmap/items/r-prime-zeta-vs-nulls.md`
- `docs/roadmap/items/r-read-off-vs-read-on.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-83/**`

## Done when

- All four item files have their scopes narrowed to their docs/prep/labs/task paths with notes recording the D68 scope change, a fresh-context D68 decision review approves the change and is logged for Casey on `#2926`, and `uor-harness loop gates --full` is green.
- Class: `host`

## Stop and ask

- Stop and decide via D68 decision review (or propose in the owner's queue if a CHARTER §3 constraint or GPU cap is implicated) if any scope change conflicts with `uor-harness check` or an active PR.
