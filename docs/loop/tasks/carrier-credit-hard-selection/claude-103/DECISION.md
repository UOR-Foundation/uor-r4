# Decision card (D68): a new sheet for carrier credit on the live learner, and this item's scope

Lab: claude, claude-103. Item: [`carrier-credit-hard-selection`](../../../../roadmap/items/carrier-credit-hard-selection.md) (#2819). Status: round 2, revised after decision review round 1 (CHANGES).

## Why this test, and why now

- **The chain.**
  - M1 closes on the saved-model path (D79). Its training doses are the A8 full dose rerun (#2831) and BVG-K Stage T (#2836), and both train with the discrete learner's canonical coordinate sweep (`training.rs`:585–610).
  - This item is `milestone: research`, and no item depends on it. `loop next` lists it as the only item claude can claim now (unblocks 0).
  - The question it answers bears on those doses directly: does choosing *which* coordinate the exact move tries, by the objective's own carrier credit, beat the fixed sweep and an uninformed in-batch choice at a matched dose?
- **The consumer.** A pass gives #2831 and #2836 a measured schedule for their next registered dose. That is a new sheet for each, not an edit to a running one. A control-only win tells them the gain is in-batch selection, not credit. A fail tells them the canonical sweep is not their bottleneck.
- **The mechanism.** `docs/mechanisms/carrier-credit.md` is PARKED. Its reopen conditions 1 ("hard selection by address or integer counterfactual moves") and 5 ("an integer counterfactual form on the discrete path") name exactly this test.

## The facts this rests on

1. **The archived sheet cannot run lawfully.** `archive/labs/unified-geometric-integration-2026-10-11/PREREGISTRATION.md`:16–27 starts from saved437 and runs 128 updates at batch 8.
   - Its objective is the "equal-episode logmeanexp of the existing strongest-wrong pooled negative-log-mass margin0 softplus", which CHARTER §3.2 bans in losses.
   - ON and OFF differ only in "the new v3 field-to-carrier/prototype adjoints" (`:23`), which are float gradients.
   - Its 437/512 start was itself panel fitting: 0 of 128 on fresh rows (`docs/mechanisms/carrier-credit.md`:3).
2. **Its code is compiled out.** `crates/uor-r4-training/src/lib.rs` marks the carrier modules `#[cfg(any())]`, and the runner example requires `historical-reference` (`Cargo.toml`:27, 53–55). Float training is legal only in the separate float module (§3.7, D37, #2823), which is not extracted yet.
3. **The softmax "sampled choice" was never run.** It is `Mode::Sampled` in `whole_path_learning.rs` near 2324, and its file calls itself an "offline CPU reference … not a qualified CUDA producer". The executed arms held the other carriers "at the actual hard codes" (`README.md`:131).
4. **saved437 is only in the iCloud cold store** (`results/codex/codex-m2-fresh-437-20261010.tar`).
5. **The only live learner is `crates/uor-r4-discrete`** (ACTIVE).
   - It tries all 16 legal states of one coordinate against the integer rank-margin hinge, and commits on strict improvement.
   - The canonical sweep visits V·B = 4,096 × 4 vector blocks in order.
   - In the 14 Oct pilot, 1,179 of 1,640 vector proposals had some candidate cost different from identity, and 236 accepted (`archive/labs/a2-pilot-2026-10-14/README.md`:29–31). The DEV count reference was 1,226 top-1 and 4,060 top-8.
   - CUDA `discrete_evaluate` already writes per-row hinges, `top1_correct` and the top-8 admitted ids (`device.rs`:1565–1650).
6. **A new schedule must live inside the crate.** `Device::apply` is `pub(crate)` (`device.rs`:2007), and a move commits only through `PreparedDriver::transition` with a controller admission (`training.rs`:2579–2690). D67 staging has a precedent in `bin/a8-qualify-stage.rs`:513–545.

## Decision (recommended)

**A. A new sheet, honestly named.**
- The done-when is met by a new pre-registered sheet with its own FIT (§4.5: archived float sheets "return only as new sheets"). It tests the discrete counterpart of carrier credit (reopen condition 5).
- It reruns the archived sheet's ON/OFF × two-seed design, not its objective, start state or carriers.
- The three arms become six: three schedules × seeds 1001 and 2001.
- The item's done-when text is updated to say this (see C).

**B. The hard-selection credit step.** It replaces the sampled choice.
- **The credit signal.** Each batch, CUDA evaluates it with `discrete_evaluate`. A row with a positive integer hinge (token plus address, the geometry's own rank-margin objective over E8-root cells) credits that hinge to its target and to its strongest admitted rival (`top8[0]` when top-1 is wrong). These are the two carriers whose cells the margin is measured between.
- **The selection.** The tried coordinate is a vector block of the token with the largest credit, ties to the lowest id. The block is `completed_batches mod B`.
- **Bound to the GPU (§3.5).** The credit scatter-add and the argmax run as one nvcc-built kernel in `discrete.cu`, with no NVRTC (owner, 13 Oct). The host receives only the 2-word result `(token, credit)`. A test fails if the selection path downloads per-row results.
- Nothing is sampled, and there is no probability, softmax or matrix product.

**C. Arms.** Every arm uses the same objective, 16-state counterfactual kernel, commit rule, batch order, corpus, start state and dose. Each arm runs at seeds 1001 and 2001.
- **ON:** the credit selection in B.
- **CONTROL:** a seeded, uniformly drawn target token from the batch, with the same block rule. It separates credit from "any in-batch schedule".
- **OFF:** the canonical sweep.
- **Start state:** a fresh model (seeded-device init) on the registered discrete corpus and spec, with the dose, the corpus and the fresh-row slice fixed in the prep sheet.

**D. Scope fix, through one `infra-loop-harness` unit.**
- The scope is `crates/uor-r4-discrete/**`. It is the convention of 18 items, and the pin updates reach tests across the crate. A narrower list would force another umbrella unit the first time the guard refuses a pinned test.
- `notebook: docs/research/notebook/m1-discrete-attention.md` is bound.
- The done-when wording is amended per A.
- This is this item's own scope fix (D68), following the `023910c63` precedent. It is not a new infrastructure item under D77.

**E. The bar** is fixed in the prep sheet before any run, and reviewed for FIT. It is ON against CONTROL at both seeds, on fresh rows (§4.3), beside the count-table reference (§4.4). It reports wall time and tokens/s, because ON and CONTROL pay one extra evaluation per batch. OFF is reported as the reference.

## What each outcome unlocks

| Outcome | Meaning | Unlocks |
|---|---|---|
| An instrument check fails | instrument (§4.2): classified first | repair, rerun; no verdict |
| ON beats CONTROL at both seeds on fresh rows | carrier credit adds over in-batch choice | reopen conditions 1 and 5 met; mechanism PARKED → READY; a schedule sheet for #2831 and #2836 |
| ON beats OFF, but not CONTROL | a schedule effect, not credit | mechanism stays PARKED; the in-batch schedule goes to #2831 and #2836 as their finding |
| ON wins on DEV but not on fresh rows | panel fit | no pass; configuration negative (D22) |
| No arm beats OFF | the sweep is not the bottleneck at this dose | configuration negative (D22); #2831 and #2836 keep the sweep |

A pass still leaves reopen condition 2 (one typed binding and frame) and the "all 4,096 choices scored with physical copy aliases" part of condition 4 open, and they are named in the REPORT. Condition 3 is met by the corpus and the fresh rows. Condition 6, the M2 hold, is superseded by D67.

## Rejected

- **Replay the archived sheet:** its banned objective and compiled-out float code rule this out.
- **Port it to the float module:** #2823 is not extracted, and the objective would still be banned.
- **Close the item unrun:** a blocker is fixed, not parked.
- **Narrow the crate scope to named files:** rejected for the umbrella churn above.

## Risks and reversal

- **Pin churn.**
  - Editing `training.rs` moves `driver_source_sha256`, which `prime_zeta.rs`' bin tests pin. The full discrete suite must run, and the pins must move in the same PR.
  - A negative gate is required: a test that the canonical schedule's config and lineage bytes stay byte-identical.
  - The new kernel is pinned by its PTX bytes, never by the compiled-source sha, which changes with every rebuild.
  - A run built from a fixed commit is unaffected.
- **Reversal cost:** low. It is one schedule variant behind config, plus the item-file lines.
