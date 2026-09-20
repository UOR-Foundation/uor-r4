# DeepSeek execution prompt: learn useful contextual memory

**Reconciled with the completed PR #1314 handoff.** This is the single successor to that run. PR #1317 contains the principal audit and mechanism roadmap; this revision incorporates the final narrative and verified training population. No successor fit has run.

You are the implementation/research partner for the UOR-R4 Geometric Language Model. Complete one constructive step toward geometric attention: make exact contextual memory useful alongside the frozen local predictor. Use your judgment to select the smallest effective mechanism from the evidence below. You may improve the ranker, NoRead decision and bounded copy strength together; the previous instruction to add exactly one abstention signal with all other parameters frozen is superseded.

## Recover authority and current state

Refresh origin/main, live #973/#820 and relevant PRs, the actual model/resource ledger, retained report seals and free space. Work in an isolated full worktree; preserve owner and other active checkouts. Read AGENTS.md, DECISIONS.md D0-b/D1/D2/D3, canonical project-track/current-state, then these two documents completely:

1. [Correcting principal audit of PR #1314](occurrence-reader-audit-2026-09-20.md).
2. [Mechanism synthesis and geometric-attention roadmap](geometric-attention-mechanism-synthesis-2026-09-20.md).

PR #1314 merged as `fbf542aa4c1f7ed336cd028c6b7b3502cba4118c`; later commits may carry this prompt and corrections. Inspect the specific source and design/result receipts. Obtain additional history/research context as needed, using primary papers and original source. Historical authored grammar, old artifacts and dense reference mechanisms are donors, not interchangeable components of the current BPE model.

The goal remains a learned transformerless geometric language model for conversation/memory and coding/reasoning on consumer hardware. Rust training may use float/matmul. D0-b serving permits bounded <=4-bit additive/shift/table maps with no numerical-kernel multiplier or floating/transcendental work. Preserve prime/zeta/R4/S3/H4/icosian priorities while measuring each actual contribution. Do not adopt a transformer backbone, hidden teacher responses, a new authored parser or a generic architecture sweep.

## What the latest result actually says

Retain frozen E plus S's query-only local contribution with no S history fold. Retain the occurrence-reader-4 artifact and all earlier negative roots. The new ring has exact session/sequence identity, 128 token positions and at most 24 exact-key candidates, each carrying an observed successor.

The constructed fresh panel has reader accuracy 411/443 versus latest 279/443 on reader-relevant positions. On the authored slot-equivalence subset it is 99/120 versus latest 64/120. This is a useful selection result. Its geometry is eight-class token-slot equality, not yet directed H4 transport. Targets were constructed using the same partition, and the coefficient ablation changes exact-match weight as well as slot information.

Raw-text selection is **5 correct reads and 83 wrong reads out of 88**: five errors occur among ten covered positions, 78 among uncovered positions. The earlier 10/10 rank-hit claim was a counter defect. Synthetic all-position CE worsens +0.6713 bits/token; raw probe +2.7891. Thus ranking, abstention and integration strength are all legitimate learning targets. The fixed residual `2^14` at ten fractional bits is a 16-nat boost. Preserve these negative outcomes and the positive construction gain without promoting this candidate.

Construction training already had **737 read and 515 NoRead labels** among 1,252 examples. The full candidate-bearing fit population was 737 covered and 1,031 uncovered before negative subsampling. Therefore do not report missing NoRead supervision or an impossibility of learning it from synthetic data. Natural-text fit is intended to address distribution/feature/objective mismatch, not to add the first negative label. The synthetic margin-derived amplitude was a useful selector diagnostic; its successful source can overcome local margins, but its false-read cost was not optimized.

## Repair the instruments as part of the constructive task

Correct the specific defects in the audit; do not launch a broad replay campaign. Reaggregate saved vectors where sufficient and write a new correcting attempt, leaving sealed originals unchanged. In particular:

- Separate target-payload availability, correct selected payload, correct occurrence when identifiable, Read/NoRead, final token correctness and language loss. Multiple equal payloads need output-compatible credit; do not invent a unique source label.
- Make a real future-token intervention and actually analyze it. Compare prior scores, selected reference and payload. Compare ReadDisabled independently with the retained S-query-only scorer.
- Validate declared construction semantics; classify same-slot decoy ambiguity instead of asserting unique solvability. Use consistent denominators and paired comparisons against latest as well as local.
- Preserve exact identity information when disabling a nonidentity feature. A matched categorical partition/control must have targets defined independently of its code assignment.
- Hash the executable bytes properly. Compare complete exported/reloaded decisions, not only Read versus NoRead; bind source, compiler/config, data, tokenizer and artifact. Keep source sensitivity distinct from emitted-output sensitivity.
- Time a real baseline with no unused ring/admission/feature work and a complete reader path; divide by actual predictions. Report preprocessing, resident tables, scratch, ring, model bytes and memory traffic separately. Do not claim energy from a clock or an assumed power value.

## Construct and learn the next mechanism

Use fit-only natural-text positions with both useful and non-useful admitted candidates, together with enough of the retained constructed task to preserve useful binding behavior. Distinguish fit, tune, known regression panels and new evaluation documents/assignments. The eight raw windows and old fresh panel have now been inspected; replay them as regressions, not untouched final evidence. Do not tune on new evaluation outputs.

Begin with a short diagnostic of available causal features and contradictory observations. The complete inference action can be `NoRead` or `(selected occurrence, one of a few bounded copy strengths)`. Start from the existing ring/selector. Learn against actual post-injection next-token loss, or explain and validate a surrogate aligned with it. For a single boost, the exact offline loss difference is:

`log(1 + p_local(payload) * (exp(a) - 1)) - a * 1[payload == target]`.

This supports useful credit without treating every recurring key as a copy opportunity. The formula is for offline training/evaluation, not served transcendental arithmetic. Choose an exported finite action set or another equally bounded D0-b implementation. Check actual integer hard-forward behavior and saturation/overflow. Keep frozen local emission available when reading is harmful or no source exists.

A useful mathematical reference is `a_opt = clip(logit(r)-logit(p),0,A)`, where `p` is the local mass of the proposed payload and `r` is its true conditional next-token probability given causal observations and the selection process. This follows from the convex expected excess loss in the audit. It is an offline idealization; neither `r` nor normalized probabilities/logarithms must be served. Prefer direct observed action-loss fitting if that avoids a separate calibration component. All one-step candidate/strength costs are available offline from the next token, so no reinforcement-learning or reward-model machinery is required. Train and validate the actual quantized policy, including clipping/saturation.

Compare to a cheap globally fitted gain and always-NoRead. A causal-stratum gain table is allowed if it earns its cost; it is not mandatory. Its strata must come from serving-available information (candidate count, score separation, local payload score/margin, learned contextual state). Never use covered/uncovered, target equality, correct-source labels, corpus identity or authored evaluation-family names as policy features. If using predicted correctness, validate calibration after candidate selection and ranker changes; use held-out/cross-fitted predictions when needed. With varying local mass inside a stratum, optimize average action loss directly rather than subtracting logits of averaged probabilities. Sampling/weighting choices must represent the intended deployment objective, with natural-text and construction results reported separately.

You have discretion over the minimal contextual feature and optimizer/data dose. Plausible signals include relative source/query context, best-versus-runner-up evidence, local payload score margin, and a learned role/scope distinction. Existing `source_routing.rs`, `relational_attention/runtime.rs`, `occurrence_role.rs` and `query_participation.rs` offer finite operator/interface ideas. They do not authorize importing their authored word grammar. Expose a directed relative H4 element only if it preserves a needed distinction; do not call eight-slot equality a transported relation.

Keep the 128/24 memory bounds initially. If source coverage or an exact observation collision makes the objective unattainable, explain that evidence and choose one bounded correction within the full projection. Candidate coverage and ranking are different problems. Do not repeatedly fit a representation proven unable to distinguish the cases; conversely, do not inflate width, phase count or harmonic degree speculatively. Role/scope persistence can be the minimal intervention if the observed failure specifically requires it. No mandatory S7/harmonic implementation is part of this run.

## Decide from behavior and cost

Before seeing new evaluation, record the intended practical tradeoff and outcome decision. Retain the old thresholds as historical reporting columns; any new tolerance needs a prospective justification. Correctness, causality, exact references and honest evidence are not negotiable performance tolerances.

Compare frozen local, competent latest/exact-prefix, and learned reader on the same positions. Report coverage, conditional ranking, read precision, NoRead behavior, per-document paired loss, payload availability, generated continuations, and corrected direct-path cost. Include identical-recent-tail/different-older-binding cases, changed payloads, repeated keys, absent answers, conflicting contexts and held-out role/payload combinations. Show when the model follows a distant binding and when it leaves local generation alone. Teacher-forced gain without useful generated behavior remains a limited result.

Do not count collapse to always-NoRead as an attention gain. It is a useful baseline and may be the honest negative outcome, but a successful candidate must retain demonstrated causal contextual benefit as well as the declared natural-text tradeoff. Conversely, do not impose an arbitrary high-precision threshold: a bounded probability correction can be useful without winning every token's argmax. Report both loss and actual source-sensitive generated behavior. Any oracle choice made after viewing the next token is an opportunity bound only, not a learned result; use such a diagnostic only if it resolves the next implementation choice.

Use focused arithmetic/causality/export/continuation tests and compile/exercise the changed Rust path. Validate the actual trainer's disk continuation when you change its state. Do not repair unused legacy CPQK machinery unless you use QueryTrainer. No blanket historical suite is required for this unpromoted component; an actual model replacement would require relevant retained controls.

If the candidate helps only one subset, preserve and report that useful component and its limitation. If it is negative, record the smallest witnessed bottleneck and one justified successor. A few meaningful corrected attempts are permitted within the budget; stop unproductive refinement rather than accumulating threshold sweeps. Do not present another audit alone as completion of this constructive task.

## Resources, preservation and delivery

At principal review the ledger was 181238565/191300000 ms, with about 167.7 minutes left. This is a recovery pointer, not a fresh allowance. Read the live JSON before work. A reasonable initial planning ceiling is 90 minutes for the entire preparation/build/fit/controls/evaluation/corrections/checkpoint block, one model worker, <=8 GiB RSS, <=512 MiB new data/reports plus <=2 GiB incremental build, and the existing 128 MiB protected storage margin. Revise that complete projection from a small step-time measurement before committing to a longer fit; do not spend it merely because it exists.

Necessary local extensions are already owner-authorized: record the reason, increment, complete updated projection and cumulative limit before use. Retain charges across sessions and failures. No paid/external compute, deletion of unique research/checkpoints or owner checkout mutation follows from this authorization. Do not clear shared build caches routinely.

Claim each new report root exclusively, seal attempts and verify all files. Keep original artifacts and reports intact. Stage named paths, push a branch, create a protected PR with `References #973`, and verify the actual merge/source content before saying delivered. Synchronize README, canonical current state/plan where changed, evidence, continuation instructions, owning issues and any existing GitHub project items. A queue compatibility acknowledgement is not an executed test. Close no broad capability issue from this bounded result.

Conclude with the exact implementation choice and rationale; artifact/source/data identities; measured improvements and harms; actual outputs and controls; complete resource debit/balance; delivery receipts; and one evidence-supported next step in the geometric-attention roadmap. Be a research contributor: propose a better bounded mechanism when the data warrants it, while preserving the evidence that led there.
