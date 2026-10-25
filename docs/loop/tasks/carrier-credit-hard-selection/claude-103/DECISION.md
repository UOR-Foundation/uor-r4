# Decision card (D68): where the three-arm sheet is rerun, and this item's scope

Lab: claude, claude-103. Item: [`carrier-credit-hard-selection`](../../../../roadmap/items/carrier-credit-hard-selection.md) (#2819). Status: proposed, pending a fresh-context decision review.

## The facts this rests on

1. **The sheet as written cannot run lawfully.** The three arms are in `archive/labs/unified-geometric-integration-2026-10-11/PREREGISTRATION.md`:16–27. They start from saved437 and run 128 updates (two epochs of 512 rows, batch 8).
   - The objective is the "equal-episode logmeanexp of the existing strongest-wrong pooled negative-log-mass margin0 softplus". That is a log-mean-exp over softplus of a log mass, in the loss, which CHARTER §3.2 bans.
   - ON and OFF differ only in "the new v3 field-to-carrier/prototype adjoints" (`:23`), which are float gradients.
2. **Its code is compiled out.**
   - `crates/uor-r4-training/src/lib.rs` marks `whole_path_learning`, `geometric_bank_generate`, `geometric_continuation_learning` and their siblings `#[cfg(any())]` ("Preserved historical module").
   - The runner example `geometric-frozen-map-fit` requires the `historical-reference` feature.
   - Float training is legal only in the separately scanned float module (§3.7, D37, #2823), and that module has not been extracted yet.
3. **The softmax "sampled choice" was never run.** It is `Mode::Sampled` in `whole_path_learning.rs`:2315–2330 (`candle_nn::ops::softmax` over temperature-scaled logits), and its file calls itself an "offline CPU reference … not a qualified CUDA producer". The executed arms used greedy native choices (`README.md`:131, `AUDIT.md`:146–153).
4. **saved437 sits only in the iCloud cold store** (`results/codex/codex-m2-fresh-437-20261010.tar`), not in the HF store.
5. **The only live learner is `crates/uor-r4-discrete`.** `docs/mechanisms/discrete-learning.md` marks it ACTIVE.
   - It tries all 16 legal states of one coordinate against the integer rank-margin hinge, and commits only on strict improvement.
   - Its schedule is a fixed sweep, `coordinate_at` in `training.rs`:585–610: canonically all V·B = 16,384 vector blocks, then the reads. Most batches therefore try a block whose token never appears in the batch. The 14 Oct pilot accepted 236 moves in 1,640 batches.
   - The CUDA evaluation already returns, per row, the token and address hinges, `top1_correct` and the top-8 admitted ids (`device.rs`:1565–1650).
6. **A new schedule must live inside the crate.** `Device::apply` is `pub(crate)` (`device.rs`:2007). A move commits only through `PreparedDriver::transition`, with a controller admission (`training.rs`:2579–2690). D67 staging, where the merged binary writes the release record against the lab's own pre-run review, has a precedent in `bin/a8-qualify-stage.rs`:513–545.

## Decision (recommended)

**A. Rerun the sheet's design on the live learner.** Keep the design: carrier credit ON against OFF, matched batches, and seeds 1001 and 2001. Run all four arms; the fourth (ON, seed 2001) never ran (`README.md`:42–44). Use one registered discrete configuration, named in the prep sheet. This is a re-registration of the sheet, not a replay.

**B. The hard-selection credit step.** This replaces the sampled choice.
- On each batch, CUDA evaluates the batch first. A row with a positive hinge credits its integer hinge to two tokens: its target, and its strongest admitted rival (`top8[0]` when top-1 is wrong).
- The coordinate that the exact counterfactual move tries is a vector block of the token with the largest integer credit. Ties go to the lowest token id. Across selections of the same token, the block cycles through its B blocks.
- The selection runs as integer CUDA work. Nothing is sampled, and there is no probability or softmax.
- **OFF** keeps the canonical schedule.
- Everything else is identical in both arms: the objective, the 16-state counterfactual kernel, the commit rule, the batch order and the dose.

**C. Scope fix.** Add `crates/uor-r4-discrete/**` to this item's `scope:`, following the convention 18 items already use. Bind `notebook: docs/research/notebook/m1-discrete-attention.md`, the live learner's page. These land through one `infra-loop-harness` unit, because the guard reads scope from `origin/main`.

**D. The bar and the decision table live in the prep sheet,** fixed before any run and reviewed for FIT.

## What each outcome unlocks

- **Pass** (the bar is set in the sheet):
  - It meets reopen condition 1 on `docs/mechanisms/carrier-credit.md`: a rule-compliant replacement for the sampled choice, checked end to end.
  - It gives the M1 path's training runs (A8 full dose, Stage T) a candidate schedule, which they can test at their own dose.
  - The mechanism moves from PARKED to READY, with the measurement.
- **Fail:** this configuration closes (D22). The mechanism stays PARKED, with its remaining reopen conditions named.

## Rejected

- **Replay the sheet as written:** its banned objective and compiled-out float code rule this out.
- **Port it to the float module:** #2823 is not extracted, and the objective would still be banned.
- **Close the item unrun:** a blocker is fixed, not parked.

## Risks and reversal

- **Pin churn.** Editing `training.rs` moves `driver_source_sha256`, which the bin tests in `prime_zeta.rs` pin.
  - The full discrete suite must run, and the pins must be updated in the same PR.
  - A run already built from a fixed commit is unaffected.
  - Default schedules keep their config and lineage bytes. The new variant is serialized only when chosen.
- **Reversal cost:** low. It is one schedule variant behind config, plus one scope line.
