# Offline radial read comparison

Fourth lab, September 26 local / September 27 UTC, 2026.

## Decision and source

The next attention comparison keeps the radial information in Claude's native
Lorentz reader and changes its distance-to-score law. This implements a research
question; it is not a new language result or a serving-policy change. The owning
[work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851517971)
records scope, ownership, distinct outcomes and cost before execution.

The native reader is derived from the continuous implementation in
[PR #1401 at dcf979df](https://github.com/UOR-Foundation/uor-r4/blob/dcf979df687ac867277ad8f3fc7dbc53393f67a8/crates/uor-r4-training/src/joint_model.rs).
The [independent review](fourth-lab-native-read-review-2026-09-26.md) preserves
the scope of that lab's reported results. Its converted transformer engine,
integer Lorentz kernel, wide learned serving scale and proposed D10 are not
adopted by this offline comparison. During development PR #1401 merged into
main, so the final increment extends that existing reader with LorentzAffine
and preserves its upstream behavior. It does not overwrite the concurrently
delivered Lorentz implementation with a second implementation.

## Mechanism and control

For query and key vectors, define

```text
q0 = sqrt(1 + ||q||²), k0 = sqrt(1 + ||k||²)
z  = max(q0*k0 - q·k, F32(1 + 1e-6))
Lorentz distance: d(z) = acosh(z)
score = exp(learned_log_beta) * (learned_offset - d(z))
```

The lifted cross-dot between `[q, -q0]` and `[k, k0]` is exactly `-z` before
clamping. Thus a lifted dot product followed by the same acosh kernel is the
same function, not a separate explanatory control. At read width 64, the lift
uses 65 deterministic features without additional learned feature weights.
Radius is query-dependent compatibility information; an angle-only code would
discard it. None of this algebra establishes predictive benefit.

The affine control uses a fixed reference derived from the existing initializer:

```text
d_star = lorentz_initial_offset(config)
z_star = cosh(d_star)
d_affine(z) = d_star + (z - z_star) / sinh(d_star)
score_affine = exp(learned_log_beta) * (learned_offset - d_affine(z))
```

The reference stays fixed when the learned offset changes. Both arms share the
query/key maps, radial lift, clamp, two learned scalars, admission, age terms,
NoRead, recurrent transport, values and copy/output composition. The affine arm
matches distance and derivative at the reference point. It does **not** match
every initial logit, attention entropy, NoRead mass or gradient. These residual
differences must be recorded in any experiment rather than hidden under a
claim of identical initialization. Shared learned matrices can be identical.

Lorentz and affine lifted scores rank raw candidates before age identically;
nonlinear spacing changes their weights and gradients. Adding learned age
after the score can also change final ordering. The nonlinear Lorentz law
produces approximately power-law weights in large `z`, whereas the affine
control gives an exponential penalty in `z`. Whether this helps distant
context and generated language is a hypothesis for the paired study.

## Research and serving boundary

`Dot` retains its old parameter inventory and omitted default configuration
field, preserving the declared old serialization contract. Lorentz preserves
Claude's exact continuous numerical contract so the reused operator does not
create an unnecessary checkpoint-format fork. LorentzAffine has its own
serialized operator identity and versioned continuous contract. Checkpoint
loading retains exact contract validation; a geometry label alone is insufficient.
Source-derived compatibility checks are separate from replay of Claude's
continuous trained checkpoints, which remain external to the newly shared
[native packet](../evidence/native-lorentz-packet-2026-09-26/README.md). Its four
packed QAT models are available, but are not a continuous parent for this control.

The new LorentzAffine control supports continuous offline training and
evaluation only. Its quantization, packed export and integer loading are
explicitly refused until the corresponding operator is implemented and adopted.
Existing Dot and upstream Lorentz paths retain their behavior. This prevents
an affine-control artifact from silently receiving another computation. The
current chatbot's persistent and dialogue readers are separate integration
paths; supporting a new geometry requires both, along with compatible value
width. Upstream now rejects unsupported Lorentz/width-128 conversational use.
The two cross-branch risks are recorded in the
[integration review comment](https://github.com/UOR-Foundation/uor-r4/pull/1401#issuecomment-5851495403).

The existing `joint-fit CAMPAIGN_JSON NEW_REPORT_ROOT {cpu|metal}` command reads
`Campaign.model` as `JointConfig`. Set `model.read_geometry` to `lorentz` or
`lorentz_affine` in a prospectively declared continuous research campaign, with
no quantization transition for this comparison. `joint-evaluate` reads the saved checkpoint identity.
This command wiring was inspected in source; no new paired campaign has been
prepared or executed for this implementation increment.

## Validation and next decision

Executed validation is recorded at delivery. Necessary implementation checks
cover forward/backward credit to query/key/scalars, checkpoint identity,
shared-reader dispatch, retained Dot behavior and unsupported-serving rejection.
They are functional checks, not fitted-model or language evidence.

Before any new fit, select one paired development comparison with fixed data,
draws, shared initial matrices, optimizer, full 256-token access and complete
resource projection. Keep training length, evaluation length, memory access
and vector width separate. Compare actual generated behavior along with loss
and read/copy diagnostics; retain the existing Dot result as an anchor.
Keep the selection policy identical between the reader arms. The concurrently
delivered [emission diagnostic](emission-selection-diagnostic-result-2026-09-27.md)
localizes several exposed failures to sampling departures, without executing
an alternative trajectory. Its separate same-checkpoint policy investigation
can proceed under its existing owner; it is neither a new reader gate nor
evidence that the radial score has fixed the prose failures.

- If the affine arm retains the useful benefit, prefer its simpler score law
  provisionally while retaining the radial geometric mechanism.
- If Lorentz improves useful generated behavior and loss, its nonlinear score
  family has evidence at that scope. Unique curvature, hierarchy and energy
  claims still require their own evidence.
- If only read/copy calibration changes, or lower loss does not improve emitted
  behavior, use the existing emission-localization track to select the next
  integration change.
- If both fail the language objective, preserve them and advance the independent
  language-interface or recurrence work. Do not automatically expand a sweep.

The optional signed-2I design remains available for a demonstrated unmet need;
this comparison does not demote the broader prime/zeta/R4 programme.
