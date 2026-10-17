# `uor-r4-harness` Un-Dispatched Rule Wiring (`harness.*` and `prlint.gates_declared`) — 17 October 2026

**Scope:** Source-only harness rule-dispatch reconciliation (`antigravity` / `ag1`, milestone `#2029`). Zero model compute (`model_compute_executed: false`, `apply_calls: 0`, `$0` GPU spend). Training, M2, and Stage B remain **HELD**; Q4 remains **NOT_EXECUTABLE**; H1 magnitude remains **NOT_APPROVED**. No `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` target was modified, preserving all pinned `crates/uor-r4-discrete` and instrument SHA-256 digests.

---

## What Was Found

A cross-module audit of `crates/uor-r4-harness` comparing the 41 rules registered in `checkdispatch::RULES` (`crates/uor-r4-harness/src/checkdispatch.rs:22-67`) and `ruleinventory::INVENTORY` (`crates/uor-r4-harness/src/ruleinventory.rs:26-170`) against the ten per-module `all(...)` dispatchers invoked by `checkdispatch::per_module` (`crates/uor-r4-harness/src/checkdispatch.rs:230-250`) identified **four registered rules that were never called by their module dispatcher**:

1. **`harness.owner_rules_applied` (`crates/uor-r4-harness/src/checkdispatch.rs:37,103`, `crates/uor-r4-harness/src/ruleinventory.rs:160`):**
   - Registered in `RULES`, `MODULE_RULES["harness"]`, and `INVENTORY["harness"]`, and implemented in Python (`scripts/harness/checkrules.py:238-303` `_check_owner_rules_applied`), but **had no Rust implementation in `crates/uor-r4-harness/src/harnessrules.rs`**. Because `checkdispatch::run_scoped` (`crates/uor-r4-harness/src/checkdispatch.rs:266-278`) synthesizes `message: "ok"` whenever a registered rule ID produces no `Finding`, `uor-harness check` unconditionally reported `harness.owner_rules_applied: ok` without ever comparing `docs/harness/milestones.json`'s `all`-scoped `owner_rules` against `crate::prep::APPLICABLE_RULES`.
2. **`harness.entry_point` (`crates/uor-r4-harness/src/checkdispatch.rs:34,100`, `crates/uor-r4-harness/src/ruleinventory.rs:167`):**
   - Implemented in `crates/uor-r4-harness/src/doccheck.rs:185-213` (`doccheck::entry_point`), but never called by `harnessrules::all` (`crates/uor-r4-harness/src/harnessrules.rs:98-108`).
3. **`harness.documented_commands` (`crates/uor-r4-harness/src/checkdispatch.rs:33,99`, `crates/uor-r4-harness/src/ruleinventory.rs:156`):**
   - Implemented in `crates/uor-r4-harness/src/doccheck.rs:857-885` (`doccheck::documented_commands_rule`), but never called by `harnessrules::all` (`crates/uor-r4-harness/src/harnessrules.rs:98-108`).
4. **`prlint.gates_declared` (`crates/uor-r4-harness/src/checkdispatch.rs:55,141`, `crates/uor-r4-harness/src/ruleinventory.rs:103`):**
   - Registered in `RULES`, `MODULE_RULES["prlint"]`, and `INVENTORY["prlint"]`, and tested in `crates/uor-r4-harness/src/prgates.rs:1896-2044` (`DECLARED_GATES`), while `crates/uor-r4-harness/src/prlintrules.rs:259-261` (`prlintrules::all`) only invoked `interlock_holds(paths, claims)` (retaining a stale comment from before `prgates.rs` was ported).

---

## Changes Made

1. **`crates/uor-r4-harness/src/harnessrules.rs`:**
   - Implemented `pub fn owner_rules_applied(regs: &Registries) -> Vec<Finding>`, checking that every `all`-scoped `owner_rules` entry in `regs.owner_rules` (excluding `OWNER-*` sub-clauses, matching `scripts/harness/checkrules.py:260-268`) appears in `crate::prep::APPLICABLE_RULES`.
   - Implemented `pub fn entry_point(paths: &StatePaths) -> Vec<Finding>` and `pub fn documented_commands(paths: &StatePaths) -> Vec<Finding>`, adapting `crate::doccheck::entry_point` and `crate::doccheck::documented_commands_rule` to `checkdispatch::run_scoped`'s convention (forwarding `error` / non-empty-`fix` findings and returning `Vec::new()` when clean).
   - Wired `owner_rules_applied(regs)`, `entry_point(paths)`, and `documented_commands(paths)` into `harnessrules::all(paths, regs)`.
   - Added unit tests `an_unapplied_all_scoped_owner_rule_is_reported`, `entry_point_reports_stale_cli_and_stays_silent_when_clean`, and `documented_commands_reports_unknown_command_and_stays_silent_when_clean`.

2. **`crates/uor-r4-harness/src/prlintrules.rs`:**
   - Implemented `pub fn gates_declared_for(declared: &[&str], inventory_prlint: &[&str]) -> Vec<Finding>` and `pub fn gates_declared() -> Vec<Finding>`, reconciling `crate::prgates::DECLARED_GATES` against the `"prlint"` slice of `crate::ruleinventory::INVENTORY` (reporting duplicates or 1-to-1 mismatches in either direction as `prlint.gates_declared` `error` findings).
   - Wired `gates_declared()` into `prlintrules::all(paths, claims)`.
   - Added unit test `gates_declared_reconciles_declared_gates_with_inventory`.

---

## Verification

- `cargo test -p uor-r4-harness --lib harnessrules`
- `cargo test -p uor-r4-harness --lib prlintrules`
- `cargo test -p uor-r4-harness --lib checkdispatch`
- `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`
- `cargo fmt -p uor-r4-harness -- --check`
- `python3 scripts/check_claim_wording.py`
