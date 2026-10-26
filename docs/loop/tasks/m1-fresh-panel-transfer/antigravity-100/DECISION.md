# D68 Decision Card: Holding Sealed Fresh-Panel Transfer Until Two-Seed DEV Bars (a)–(d) Pass (`m1-fresh-panel-transfer`, `#2841`)

- **Lab:** `antigravity`
- **Order:** [`docs/loop/tasks/m1-fresh-panel-transfer/antigravity-100/ORDER.md`](ORDER.md)
- **Item / Issue:** [`docs/roadmap/items/m1-fresh-panel-transfer.md`](../../../../roadmap/items/m1-fresh-panel-transfer.md) ([`#2841`](https://github.com/UOR-Foundation/uor-r4/issues/2841))
- **Pre-Registration Sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md) (`TEST FITNESS: FIT`, [`#2841` comment `6423732020`](https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-6423732020))
- **Authority:** `CHARTER.md` §3.5 (*"Spend GPU only where a run can change a decision"*), §4.3, §4.8, §5.1, D67, D68, D69

---

## 1. Whole-Project Context & What `#2841` Governs

1. **Role of `#2841` on the M1 Path (`PLAN.md` Step 7, `CHARTER.md` §4.8):**
   - `m1-fresh-panel-transfer` (`#2841`) is M1 closing bar (e):
     > *"(e) Fresh transfer. The gain holds on a fresh, sealed test panel registered in advance."*
   - Read with `CHARTER.md` §4.8 bar (d), "the gain" is integer `top-1` through the `D11` (`UORLUT01`) export beating both the unigram table (`1,226 / 14,287` on `ft/dev`) and the `T2` context-free successor table (`6,300 / 14,287` on `ft/dev`) on each of two seeds, with `tokens/s` reported.
   - Step 4 of [`docs/roadmap/items/m1-fresh-panel-transfer.md`](../../../../roadmap/items/m1-fresh-panel-transfer.md) states a strict scientific integrity rule:
     > *"4. Do not run it until `m1-export-gap-and-toks` has produced a saved model that passes the earlier bars on two seeds, and a fresh-context pre-run review has passed after its FIT (CHARTER §5.1, D67, D77). Score once. Do not re-draw or re-open the slice; use DEV for any earlier look."*
2. **Completed Preparation in `antigravity-99` (`PR #3292`, merged `31ddf59081a30baaa1ee5f7855fbf1b2d5717b7d`):**
   - Step 1: Implemented `refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`, and `d11-scorer fresh-score` in [`docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs`](../../../../labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs) (`18 / 18` unit tests passing).
   - Step 2: Posted the design note on `#2841` ([`#issuecomment-6423580146`](https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-6423580146)).
   - Step 3: Authored [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md), verified all stored fresh archive digests (`0b7e754c…`, `a1cda529…`, `df5d074a…`, `c2d4c7e5…`, `b41b8344…`, `4bd108ad…`, `82ead8db…`, `8287d20d…`) by streaming SHA-256 without extracting or scoring any row (`$0` GPU spend), and recorded non-author `TEST FITNESS: FIT` and D68 approval ([`#issuecomment-6423732020`](https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-6423732020)).
   - Step 5: Added [`docs/labs/m1-fresh-panel-transfer/README.md`](../../../../labs/m1-fresh-panel-transfer/README.md) and a notebook row in [`docs/research/notebook/m1-discrete-attention.md`](../../../../research/notebook/m1-discrete-attention.md).

---

## 2. Two-Seed `ft/dev` Audit of All Current Candidate Model Families

Under `G-Fresh-Gate` (`validate_fresh_transfer_gate` in `fresh_transfer.rs:107-220`), opening `fresh_v2` (`0b7e754c…` / `a1cda529…`, `14,479` eligible targets) requires a two-seed DEV receipt on `ft/dev` (`14,287` rows) where each seed satisfies:
$$\text{d11\_top1}(s) > 1,226 \quad\land\quad \text{d11\_top1}(s) \ge 6,300 + \max\bigl(24,\; \bigl|\text{d11\_top1}(1) - \text{d11\_top1}(2)\bigr|\bigr)$$

Auditing all five two-seed discrete / exported model families currently sealed on `main`:

| Candidate Family | Upstream Unit / PR | Seed 1 `ft/dev top-1` (`/ 14,287`) | Seed 2 `ft/dev top-1` (`/ 14,287`) | Beats `DEV Unigram` (`> 1,226`)? | Beats `DEV T2 + floor` (`>= 6,300 + max(24, \|s1 - s2\|)`)? | `G-Fresh-Gate` Verdict |
| :--- | :--- | ---: | ---: | :---: | :---: | :--- |
| **`t16-frozen-zero` (`UORLUT01`, `OuterL1V1`)** | `#2839` (`PR #3287`) | `455` (`4495412e…`) | `506` (`24fb26d9…`) | No (`<= 1,226`) | No (`< 6,351`) | `SEALED_FRESH_PANEL_REFUSED` |
| **`t16-relation` (`UORLUT01`, `OuterL1V1`)** | `#2839` (`PR #3287`) | `371` (`8ca0eb53…`) | `337` (`589726bd…`) | No (`<= 1,226`) | No (`< 6,334`) | `SEALED_FRESH_PANEL_REFUSED` |
| **`WithinCellCodeRerankerV1` (`T16_fz` / `T16_rel`)** | `#3267` (`PR #3280`) | `2,140` (`T16_fz`) | `2,140` (`T16_fz`) | **Yes (`+914`)** | No (`< 6,324`) | `SEALED_FRESH_PANEL_REFUSED` |
| **`Trunk-FZ` `Level 3` (`trunk-model.d11`)** | `#2844` (`PR #3250`) | `2,988` | `2,989` | **Yes (`+1,762`)** | No (`< 6,324`) | `SEALED_FRESH_PANEL_REFUSED` |
| **`Trunk-Rel` `Level 3` (`trunk-model.d11`)** | `#2844` (`PR #3250`) | `2,227` | `2,226` | **Yes (`+1,000`)** | No (`< 6,324`) | `SEALED_FRESH_PANEL_REFUSED` |

1. While `#3267` (`2,140`) and `#2844` (`2,988–2,989` and `2,227–2,226`) beat the `ft/dev` unigram baseline (`1,226 / 14,287`) by `+914` to `+1,763` rows, none yet reaches the `ft/dev` `T2` successor baseline (`6,300 / 14,287`).
2. Opening or scoring `fresh_v2` (`0b7e754c…` / `a1cda529…`) before a two-seed model passes `>= 6,300 + max(24, |s1 - s2|)` on `ft/dev` would violate Step 4 of `m1-fresh-panel-transfer.md` and unseal the single-shot transfer panel prematurely.
3. Conversely, leaving `docs/roadmap/items/m1-fresh-panel-transfer.md` at `status: ready` while its `depends_on: [m1-export-gap-and-toks]` is `done` causes `uor-harness loop next` to offer `m1-fresh-panel-transfer` (`M1 path, unblocks 9`) on every cycle ahead of ready research items (`r-ranking-with-admission-intact` `#2884` and `m4-lawful-likelihood-bpb` `#2855`).

---

## 3. Decision & Action Plan

1. **Approve transitioning `docs/roadmap/items/m1-fresh-panel-transfer.md` from `status: ready` to `status: held` under CHARTER §5.1 and D68:**
   - Codify all five candidate model families in `verify_current_exported_candidates_hold_fresh_seal()` (`docs/labs/m1-d11-artifact-scorer/instrument/src/fresh_transfer.rs`) and verify via a failing-first test in `tests/stage1.rs` that `validate_fresh_transfer_gate` refuses all five with `SEALED_FRESH_PANEL_REFUSED`.
   - Set `docs/roadmap/items/m1-fresh-panel-transfer.md` to `status: held` with explicit reopen condition: reopen to `status: ready` and run `d11-scorer fresh-score` on 1x RTX 5090 as soon as a saved two-seed `UORLUT01` model passes M1 bars (a)–(d) (`d11_top1 > 1,226` and `d11_top1 >= 6,300 + max(24, |s1 - s2|)` on `ft/dev` `14,287` rows with `B2` zero export gap, `B4` pass, and `tokens/s` reported) and passes a fresh-context D67 pre-run review.
   - Re-render `docs/roadmap/README.md` so `uor-harness loop next` immediately advances to the next ready item (`r-ranking-with-admission-intact`, `#2884`).
2. **Cost to Reverse:** Zero (`status: held -> ready` is a 1-line edit in `docs/roadmap/items/m1-fresh-panel-transfer.md`, and `d11-scorer fresh-score` is already built and tested).
