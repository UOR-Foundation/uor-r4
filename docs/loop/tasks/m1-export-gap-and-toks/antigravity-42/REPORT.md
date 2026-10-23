# REPORT: m1-export-gap-and-toks, antigravity-42

## Negatives and open items (read first)

- **Negative / open:** No trained discrete model checkpoint (`uor-r4.discrete-spiralcore-fbs/2`) has cleared Stage 3 (`m1-close-saved-model`) yet (`m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, and `m1-trunk-assembly-and-training` have not produced a passing checkpoint, and training remains **HELD** under D40). Per step 1 of [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md) ("Check the progress of `m1-d11-exporter-discrete` and of a saved trained model; do not start a run without both"), no GPU DEV scoring or laptop timing run is executed in this unit.
- **Open:** [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) is at `TEST FITNESS: PENDING` and requires a non-author `TEST FITNESS: FIT` review (CHARTER §4.5), owner/PM registration of the Stage 4 bar and two-seed band on `#2839` / `#2029` (CHARTER §5.2, D39 item 5), and a saved trained discrete checkpoint before execution. [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md) therefore remains `status: ready`. References #2839, #2029.

## Files changed

- `docs/loop/tasks/m1-export-gap-and-toks/antigravity-42/ORDER.md` (lines `1-37`): task order and cancel probe for `antigravity-42`.
- `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` (lines `1-145`): Stage 4 one-page draft pre-registration sheet (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- `docs/labs/m1-export-gap-and-toks/README.md` (lines `1-53`): Stage 4 design note, dependency audit (`m1-d11-exporter-discrete` `done` in `#2998`; trained model checkpoint still pending), and compute boundary summary.
- `docs/mechanisms/lut-serving.md` (line `28`): linked the Stage 4 pre-registration sheet and lab record under the mechanism history's `Next` bullet.
- `docs/loop/tasks/m1-export-gap-and-toks/antigravity-42/REPORT.md` (lines `1-47`): this task report.

## Commands run and outputs

1. Cancel probe:
   - `uor-harness loop gates --quick` interrupted with `SIGINT` after `150 ms` -> `rc = -2`.
2. Pre-registration and documented-command rule checks:
   - `uor-harness check --only prep` -> `0 error` (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` passes `prep.sheet_shape` across all 8 required blocks; expected `warn prep.fit_before_run` on `PENDING`).
   - `uor-harness check --only harness.documented` -> `1 rules, 0 failed (0 error, 0 warn)`.
3. Docs link and archive verification:
   - `~/.cargo/bin/cargo run -p xtask -- check` -> `ok: docs, archive, figures, and roadmap checks passed`.
4. Loop gates and PR checks:
   - `uor-harness loop gates --quick` -> `fmt-claim-wording GREEN`, `fmt-harness GREEN`, `claim-wording GREEN`.
   - `uor-harness loop report --check` -> `PASS`.
   - `uor-harness loop pr-check --local` -> `verdict: PASS`.

## Ratchet counts

- D23 (`softmax`/`softplus`/`sigmoid`/`tanh`/`gelu`/`rope`): `0` added (`0` delta).
- D25 (`matmul`/`gemm`): `0` added (`0` delta).
- D32 (`f32`/`f64` in core/discrete/integer): `0` added (`0` delta).

## Verification log

- `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md:39-44`: pins `ft/dev/tokens.u16` (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`) and `ft/dev/response_mask.u8` (`a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`) and the two-seed K16 noise bands (`\Delta_{\text{noise,top1}} = 398`, `\Delta_{\text{noise,top8}} = 748`, `\Delta_{\text{noise,adm}} = 930`).
- `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md:62-67`: separates GPU research evaluation (CHARTER §3.5) from laptop static audits and product serving throughput timing (CHARTER §3.6).
