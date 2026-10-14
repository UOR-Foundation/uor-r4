# Pre-registration — discrete admission construction controls

- Date: 2026-10-14. Milestone: M1 #2029; shared learning/output #2032.
- Owner: Codex/Sol; exact-source reviewer requested: Claude; PM: Kimi.
- TEST FITNESS: PENDING. No numerical execution is authorized by this draft.
- Numerical baseline: `e7d926fbe38537f4faa90b817705c61a780778d6` plus only the
  admission diff allowed by Amendment 2 of the
  [learning sheet](m1-discrete-rung0-training-2026-10-13.md).
- Integrated source, source digest, executable SHA256, PTX SHA256, geometry
  SHA256, CUDA architecture and compiler: NOT_PINNED. Bind before execution;
  old #2400 artifacts do not qualify this changed proposal receipt ABI.
- Training and M2 remain **HELD**. This sheet does not release either hold or
  authorize the 1,640-batch corpus pilot. It qualifies finite construction only.

## Base artifact

Integrated source for review: `73f2f7e21a671dc27f42d4c9380673dd23995780`,
[PR #2421](https://github.com/UOR-Foundation/uor-r4/pull/2421). Later sheet-only
format/pin updates do not change that executable source. Binary/PTX identities
remain NOT_BUILT and must be recorded before any numerical execution.

## Data split

Authored disposable literal fixtures only. No production TRAIN/DEV input enters
an apply call. Metadata tests use literal storage, mask, cursor and release-record
fixtures; these do not execute a host model. No trained artifact is inherited.

## Bar

All named construction controls must complete with their registered exact
witnesses/refusals and bounded apply inventory. No tolerance or performance bar
exists. A failure suspends construction qualification and is diagnosed under R12;
it cannot close or redirect a geometric mechanism.

## Minimum test power

This is construction qualification, not a statistical language experiment.
The 25 finite CUDA controls plus 17 metadata tests resolve the named ABI,
causality, identity, admission and persistence risks. They do not establish
learnability, corpus controller execution or production throughput. The
separately registered training sheet owns any later corpus dose and transfer.

## Stop rule

Training and M2 stay HELD. No corpus execution, no owner-release record creation,
no automatic continuation into the pilot. CUDA/contract faults stop launches
and preserve actual partial results/NOT_RUN. Owner caps and disk floor remain
hard boundaries; estimates are not reasons to kill a healthy admitted control.

## Question and boundaries

Does the admission diff preserve the qualified integer learning law, carry,
writer, proposal and update behavior while correctly reporting candidate
sensitivity and enforcing the controller's explicit admission boundaries?
The new controller also reports P1 (masked TRAIN target counts for token IDs
0..409) and P2 (completed-snapshot vector/cell cardinalities); these are
metadata instruments, not a new learning law or schedule.

All model arithmetic and sensitivity comparison execute on the GPU as exact
integers. No floating-point model/instrument, gradient, softmax, squashing,
matrix contraction, Python model dependency, NVRTC or CPU model fallback is
introduced. Host processing admits files/identities and reports completed
observations; P2 metadata counting is explicitly permitted by the learning
sheet. No production corpus may enter any apply call in this qualification.

The finite apply boundary from the
[original driver sheet](m1-discrete-driver-finite-2026-10-13.md) remains literal:
eleven attempted calls, eight accepted disposable updates, one foreign refusal,
one stale refusal and one equal-loss no-op. Models and post-apply snapshots
remain in process and are dropped; none is saved, archived or promoted.
No new apply calls or dose extension are introduced by the sensitivity checks.

## Exact composed CUDA inventory

Run the following **25 existing named controls once** at the integrated source,
with the added assertions below. The lists emitted by `list-controls`,
`list-flock-controls` and `list-driver-controls` must exactly match these lists.
Each suite gets its own fresh, exclusively created report directory.

| Suite / command | Exact names |
| --- | --- |
| Route / `qualify` — 11 | `exact_writer`, `virtual_proposals`, `context_transport`, `isolated_windows`, `collision_and_omitted_target`, `nonvacuity`, `artifact_and_identity`, `inventory_bounds`, `variable_length_prefixes`, `target_independent_admission`, `token_relabelling_preserves_geometry` |
| Flock / `qualify-flock` — 6 | `flock_identity_read`, `flock_gate_counterfactual`, `flock_transport_witnesses`, `flock_payload_witness`, `flock_selection_and_eligibility`, `flock_refusals_and_noop` |
| Driver / `qualify-driver` — 8 | `isolated_apply_boundary`, `route_apply_reality`, `gate_apply_reality`, `fixture_loop_resume`, `native_top8_ties`, `count_reference_excludes_dev`, `seeded_initialization`, `corpus_schedule_resume` |

Every original literal witness and domain-refusal expectation is retained from
the original driver and Flock sheets. The following sensitivity assertions are
inside existing controls, not additional independent test-count claims:

| Existing control | Added witness |
| --- | --- |
| `isolated_apply_boundary` | Improving VectorBlock0 ordinal7, 640→544 reports sensitivity `true`. Its existing one-leaf Route proposal has all16 costs544, CURRENT selected, sensitivity `false`. |
| `isolated_apply_boundary` | One additional **read-only** proposal at the original V2/B2/C1/kept2 state: query token0, target0, mutate rival token1's first block (`VectorBlock(2)`). CURRENT remains ordinal0 with before0/after0/epoch0, while sensitivity is `true`. All nonidentity candidates are nonimproving; at least one is worse. Tied candidates are allowed by the action stabilizer. This distinguishes cost sensitivity from acceptance without any CPU ranking or extra apply. |
| `route_apply_reality` | Existing improving Route ordinal1, 640→256 reports sensitivity `true`. |
| `gate_apply_reality` | Existing improving Gate ordinal2, 448→64 reports sensitivity `true`. Active Gate inventory remains ordinals0..2; padding3..15 is excluded by the CUDA sensitivity loop. |
| `flock_refusals_and_noop` | Existing one-token Gate proposal has equal active costs0..2, CURRENT selected and MAX padding3..15; sensitivity must be `false`, proving padding cannot manufacture sensitivity. No extra GPU call is added. |

The added proposal uses one extra carry of one physical token and sixteen
active candidate lanes over one authored row, including the implementation's
baseline replays. Driver proposal attempts become eleven: ten completed
proposals and the existing wholly-ineligible status8 refusal. There remain
eleven apply calls/eight accepted fixture updates and zero corpus updates.
The extra proposal is not applied, timed as production throughput or retained
as a learning artifact. No new `measure-cost` run is part of this sheet.

## Controller metadata inventory and source review

Run **17 non-CUDA Rust metadata tests**: the ten baseline tests plus seven
admission/controller/report tests. Before FIT, confirm these exact names and count
against integrated `cargo test -p uor-r4-discrete --no-default-features --lib --
--list` output. **This draft remains PENDING until that inventory and the exact
integrated source are confirmed.** No host test executes model computation.

| Module | Exact test names |
| --- | --- |
| `corpus::tests` — 3 | `corpus_header_masks_documents_and_tail_are_metadata`, `corpus_refuses_hash_mask_header_and_missing_bos`, `tokenizer_bos_is_explicit_metadata` |
| `metadata::tests` — 2 | `every_admitted_context_preserves_ordered_atom_coverage`, `triple_and_nine_atom_closure_retain_exact_child_indices` |
| `tests` — 3 | `apply_calls_are_confined_to_bounded_fixture_and_admitted_transition`, `configuration_and_hold_are_metadata_only`, `read_configuration_is_explicit_bounded_metadata` |
| `training::tests` — 4 | `complete_coordinate_cycle_and_resume_are_metadata`, `checkpoint_metadata_refuses_swapped_config_data_and_cursor`, `pilot_slice_reports_masked_reference_share_without_reordering`, `snapshot_metadata_distinguishes_interior_patterns_from_writer_cells` |
| `controller::tests` — 5 | `controller_is_pilot_only_and_unknown_release_fields_refuse`, `pilot_configuration_rejects_unregistered_seed_dose_or_data`, `release_records_bind_owner_review_scope_and_every_identity`, `resume_counters_require_exact_pilot_cursor_and_baseline`, `immutable_checkpoint_output_refuses_reuse` |

The new checks and source review cover these concrete construction risks:

- Missing owner release, incomplete source/artifact/protocol binding, wrong
  configuration and expanded pilot dose are refused before a CUDA context.
- Default training and corpus-update admission remain HELD. Merely providing
  a FIT sheet does not mint an update capability or release the hold.
- The registered ascending pilot coordinates remain VectorBlock0..1639;
  Route reports attempted0 / sensitivity NOT_RUN / accepted0.
- Per-kind attempted/sensitive/accepted observations distinguish tied no-op,
  sensitive no-op and accepted update, without recomputing model costs on host.
- P1 uses the completed masked TRAIN count-reference observations, not DEV or
  unmasked prompt frequency, and preserves the exact count numerator and
  denominator. P2 counts full vectors and first/last-block cell keys over the
  completed snapshot without a CPU forward or geometric scoring fallback.
- Restored trained checkpoints retain exact data/config/source lineage;
  evaluation does not silently reject a valid trained epoch, change its state
  or accept a foreign source identity.

Some of these boundaries also need exact-source review; a metadata test is not
evidence that the corpus controller or production-shape learned evaluation ran.
The review must trace CLI admission → private capability → bounded transition
→ CUDA apply, checkpoint/evaluator bindings and the report assembly. The corpus
pilot remains NOT_RUN and is outside this finite execution scope.

## Steps

1. Finish the allowed diff and freeze its exact source, metadata-test inventory
   and composed CUDA control names. Obtain Claude's prospective exact-source
   FIT, including answers to the four questions below.
2. Root records scoped qualification admission. Use at most one owned admitted
   CUDA12.8+ Ubuntu24 pod within all-lab limits of four pods and $8/hour; inspect
   live leases first. No new rental or numerical run follows from this draft.
3. Compile release AOT; record architecture, nvcc/Rust/driver/device and binary,
   PTX, source and geometry digests. Admit actual PTX with the existing exact
   eleven-entry integer instruction checker. A build failure is a blocker.
4. Execute each composed suite once in its fresh directory. Require all25
   controls PASS with no missing/ignored/skipped/NOT_RUN entries and the exact
   eleven/eight apply dose. A CUDA fault stops further launches, preserving
   remaining NOT_RUN and actual partial receipts. Any D29/D32 refusal inside
   a control is a blocker, except the specifically registered domain-refusal
   witnesses. Generic CUDA failure never satisfies such a witness.
5. Preserve source/build logs, stdout/stderr/exit codes and complete or partial
   control JSON. Charge setup, build, controls, retries and preservation to the
   existing cumulative ledger. Verify archive round-trip before pod deletion.

The prior composed controls completed within the earlier 78-second control and
cost-command interval, but that interval also included a separate cost run;
it is not a standalone timing bound for this new source. The added witness is
one V2/B2/C1 proposal, not a production-shaped proposal. This sheet sets no
speed bar or arbitrary timeout and claims no measured new cost. Setup/build
and provider time remain unknown until observed; a healthy admitted run is not
killed merely because an estimate expires. Actual hard stops are owner caps,
disk floor, source/contract mismatch and failed/unsafe CUDA.

## R12 author answers

1. **Quantity:** construction correctness of unchanged integer operations,
   receipt sensitivity distinct from acceptance, strict hold/admission and
   checkpoint/report identities. No language quality or mechanism merit.
2. **Computable now:** yes for the completed finite CUDA fixtures and metadata
   interfaces once integrated source builds and its inventory is pinned. There
   is no corpus-training or production-throughput inference in these controls.
3. **Can the instrument meet its bar?** The witnesses use exact integer
   identities. All-tied, improving and nonimproving-but-sensitive receipts are
   different attainable cases. Exact refusal/lineage and synthetic metadata
   fixtures have no precision/noise threshold. A wrong fixture or instrument
   invalidates that evidence and is repaired; it does not reject geometry.
4. **Should it run now?** Yes after exact-source FIT and scoped admission,
   because the proposal receipt ABI and controller boundary changed. Source
   review plus finite regression resolves that concrete risk before any owner
   release. No fresh corpus dose, speed ranking or mechanism closure is needed.

## Independent review and outcome

TEST FITNESS: PENDING

Claude exact-source verdict and R12 answers: PENDING. Final source/artifact and
metadata-test pins: PENDING. Numerical controls: NOT_RUN. Corpus pilot: NOT_RUN.
Training and M2: HELD. A failed control suspends qualification, never closes or
redirects a mechanism; attribution follows instrument, fixture, bar, mechanism.
