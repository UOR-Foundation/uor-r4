# BVG-K Stage C2 Block Apply & Stage T Pre-Registration (`#2835`)

- **Lab:** `antigravity` ([`#2835`](https://github.com/UOR-Foundation/uor-r4/issues/2835), PR [`#3204`](https://github.com/UOR-Foundation/uor-r4/pull/3204))
- **Milestone:** M1 (`#2029`) — `m1-discrete-attention` (Step 3 on the M1 path in `PLAN.md`)
- **Status:** **COMPLETE (`status: done`) — Stage C2 finite CUDA controls PASS (`8/8` controls, `3,419` fixture updates, receipt `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) and the one-page Stage T pre-registration sheet ([`../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md)) is registered with a non-author `TEST FITNESS: FIT` review and D68 decision review.**
- **Stage C1 Qualification Receipt:** [`../bvgk-rowsk-scoring-2026-10-18/evidence/bvgk-c1-20261018/report/rowsk-qualification.json`](../bvgk-rowsk-scoring-2026-10-18/evidence/bvgk-c1-20261018/report/rowsk-qualification.json) (`sha256 = ebdf0ef4ddccfdce80758c9c7d9185ca66bb7689897d22503e63727d8779ef5b`, `5,130 / 5,130` coordinate parity, `16.25x` scoring speedup)
- **Stage C2 Qualification Receipt:** [`../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`](../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json) (`sha256 = 2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `8 / 8` controls `V0`, `RC`, `UT`, `JR`, `DC`, `V3`, `RF`, `C1R` pass on RTX 5090)
- **Stage T Pre-Registration Sheet:** [`../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md) — **TEST FITNESS: FIT**, **D68 DECISION REVIEW: PASS** ([`#2835` comment `6395412609`](https://github.com/UOR-Foundation/uor-r4/issues/2835#issuecomment-6395412609), reviewer `bb65a68d-8e8b-4e09-8d64-f624827d645f`)

---

## 1. Stage C2 Block Apply Qualification Summary (Already Met)

Stage C2 (`crates/uor-r4-discrete/src/{bvgk_block.rs,bvgk_device.rs,bvgk_qualification.rs,bvgk_source.rs}`) qualified the opt-in BVG-K block proposal (`propose_block`) and one-epoch set commit (`apply_block`) on an RTX 5090 (`2026-10-19`, receipt `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`):

| Control | What It Verifies | Measured Result | Verdict |
| :--- | :--- | :--- | :--- |
| **`V0`** | Source, PTX, cubin, and loaded kernel register/memory resource parity | `0` register or local/shared/const memory delta | `PASS` |
| **`RC`** | Scratch re-carry window splitting (`split_windows`) reproduces original carried batch | Exact match across all 4 cells (`Route` 2/4 blocks, `OccupiedCellRankHingeV1`, `Relation`) | `PASS` |
| **`UT`** | `propose_block` leaves original device state (`table`, `routes`, `gates`) and `epoch` byte-identical | `192` proposals (`150` with `|S_t| >= 2`), `0` state/epoch mutations | `PASS` |
| **`JR`** | Scratch-clone joint loss `L(S_t)` matches independent sequential forced single-coordinate applies | `150 / 150` multi-coordinate proposals match exactly | `PASS` |
| **`DC` (`V4` + `V5`)** | Decision rule (`S_t` iff `L(S_t) < L({best})`, else `{best}`), commit mask (`V4`), and committed loss (`V5`) | `135` joint wins, `15` fallback-to-best, `4` single, `38` empty; all match rule | `PASS` |
| **`V3`** | One-epoch `apply_block` state matches sequential forced single-coordinate applies in storage, with `epoch` advancing by `1` | `64 / 64` commits (`50` joint commits) match byte-for-byte in `table`, `routes`, and `gates` | `PASS` |
| **`RF`** | Host and device refusals (empty/oversized block, repeated parameter, mixed kinds, stale epoch, out-of-range lane) | `12 / 12` typed refusals with state and epoch unchanged | `PASS` |
| **`C1R`** | Stage C1 `V2` K-coordinate scoring regression (`5,130` coordinates across `342` blocks) | `5,130 / 5,130` exact parity (`0` mismatches) | `PASS` |

---

## 2. Resolved Prerequisites for Stage T (`#2831` and `#3179`)

The 19 October draft sheet (`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`) held Stage T on two preconditions that are now resolved on `main`:

1. **`m1-a8-full-dose-rerun` (`#2831`, merged as `dc5f409b0`) — Qualified Base Lineage & Controller**:
   - The A8 address-structured codebook (`docs/labs/m1-a8-registration-and-runbook-staging/evidence/inherited/address-structured-v1.json`, `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) under active D40 64-batch whole-interval admission-preserving rollback (`AuditPolicy::registered()`: `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`) passed `B3_init` (`1,351 / 3,847` DEV `P_succ`, `4,244 / 14,287` DEV Full, `1,211 / 3,847` TRAIN anchor) and passed `Q1` (`batch = 4,096`) on `FrozenZero` (`DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top1_correct = 871 / 14,287`, `6.55x` over initial `133` and `+408` over the prior best trained discrete `463 / 14,287`).
   - `#2831` also isolated why `K = 1` single-coordinate updates subsequently hit `STOP_FOR_DIAGNOSIS` (`consecutive_rejections`): scoring a single coordinate per 64-row local batch while auditing the 3,847-row global anchor every 64 batches rolls back the entire 64-batch window whenever one move inside the window breaches the anchor floor. Scoring `K = 16` coordinates per batch and committing verified joint blocks (`L(S_t) < L({best})`) directly addresses this `K = 1` acceptance-and-credit bottleneck.
2. **`m1-w2-code-utilization` (`#3179`, merged in PR `#3193`) — Admission-Matched Next-Word Comparators**:
   - On the stored W2 captures (`caseyallard/uor-r4-store:results/claude/w2-capacity-2026-10-25/record.json`, `6971797111a08414596f32050d08a427f3e4f9595826a75587966086986b6928`, summarized in `docs/labs/m1-w2-capacity-capture/README.md`, and `caseyallard/uor-r4-store:results/claude/w2-utilization-2026-10-25/record.json`, `d9aa6cd1a9a46876d4e50f1143d481e810fdcfb216beeccf6422ce1b7d70db90`, summarized in `docs/labs/m1-w2-code-utilization/README.md`), full-panel comparators are Unigram `U = 1,226 / 14,287`, Bigram `G = 6,300 / 14,287`, Code table `T = 3,414 / 14,287`, and Q4 Writer `W = 149 / 14,287`.
   - Restricted to the `1,920` writer-admitted rows (`A_W`), the **admission-matched code table (`restricted table`) scores `1,138 / 14,287` (`1,138 / 1,920`)** and the **admission-matched bigram (`restricted bigram`) scores `1,482 / 14,287` (`1,482 / 1,920`)** (`QA = A1a`, `QB = B2`, `QC = C1`).
   - This pins the Stage 2 bar for Stage T (`docs/roadmap/items/m1-bvg-k-block-apply-c2.md` & `m1-bvg-k-stage-t-training.md`): **beat the unigram table (`1,226 / 14,287`), then the admission-matched next-word table (the version W2 produces: `1,138 / 14,287` restricted code table and `1,482 / 14,287` restricted bigram on the baseline, plus the checkpoint-matched restricted tables on the checkpoint's admitted rows `A_W`, not the `6,300` full-panel table), on two seeds (`~$12` budget).**

---

## 3. How Stage T Covers the Three C2 Uncovered Items

The three items left outside Stage C2's fixture scope (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`) are registered in Section 4 of [`m1-bvgk-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md) as mandatory Step 0 GPU controls before any Stage T training result is read:

1. **`AKd` (BVG-K `apply_block` vs ordinary single-move `Device::apply` kernel)**: Across the first `32` single-move commits (`|commit_ordinals| == 1`), a shadow device restored from the pre-commit snapshot applies the same coordinate via ordinary `Device::apply` (`discrete.cu`); `table`, `routes`, `gates`, `epoch`, and snapshot SHA-256 match byte-for-byte.
2. **`V1d` (`K = 1` identity with today's `transition` law)**: Across the first `24` batches on the production A8 configuration, `transition_block` at `K = 1` on a shadow device matches the `T1` arm's `transition` bitwise in state hash, `epoch`, `accepted_updates`, and schedule cursor.
3. **Driver-level controls (`KId`, `V8d`, `V9d`)**: `KId` verifies `model.epoch() == accepted_updates`, `committed_moves == sum(moves)`, `|commit_ordinals| == moves`, fresh device score equals `committed_loss < l_before`, and all-ineligible batch no-op; `V8d` verifies `FrozenZeroV1` keeps all relation trits `0` with `0` relation ordinals committed; `V9d` verifies restoring the batch-`64` checkpoint and running to batch `128` matches an uninterrupted `128`-batch run bitwise.
