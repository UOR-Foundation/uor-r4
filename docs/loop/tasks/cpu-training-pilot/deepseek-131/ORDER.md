# ORDER: cpu-training-pilot, deepseek-131 (served final-gain BOS training)

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `7a77bce37`.

## Goal

Add a termination-aware training-through-serving path for the exported FBS
final gain. At training rows whose next corpus token is EOS, optimize the
complete served query so the decoder's BOS stop token ranks first; at every
other training row, protect the actual content token and reject added BOS
selection. Train on one document-disjoint public split and grade a disjoint
held-out split on both public model seeds against frozen, shuffled-label and
equal-move random controls.

This is D84 CPU development research. It cannot establish GPU qualification,
complete-reply correctness or useful conversation.

## Cancel probe (run first)

1. The public source artifacts have no Hamming readout, matching the existing
   cached final-gain replay limit.
2. Prior termination work tested post-query point, cell and orbit gates; none
   trained the final gain itself toward BOS.
3. The rank-margin trainer is not sufficient because it refuses updates when
   the target cell is not admitted, which is the dominant observed BOS failure.
4. Claude's open `one-model-end-to-end` P2b unit changes only its own float
   long-context preparation and ORDER, not this CPU crate or this experiment.

**Result at `7a77bce37`:** not cancelled. The public source artifacts have no
Hamming readout; deepseek-111 measured absent BOS admission on 93.75-97.15% of
generated decisions; deepseek-118/120/122 replaced post-query decisions but did
not train query geometry. The rank-margin update requires admission and cannot
address that majority. No open order claims this CPU final-gain path.

## Fixed construction

Use the existing 16-key public split: even document ordinals train, odd
document ordinals evaluate, up to eight examples per key, with no document
overlap. Derive opening/reply memory from training examples only.

For each teacher-forced training decision:

- a decision whose actual next token is corpus EOS id 1 is **terminal**;
- every other decision is **nonterminal** and retains its actual next token;
- the terminal objective is decoder BOS id 0, never corpus EOS;
- each decision is replayed through the complete cached path: final gain,
  Q24 opening/reply memory movement, Q12 conversion, writer admission and
  point fallback.

The trainer changes one final-gain code by exactly one signed unit per
candidate. It starts at the training-derived memory baseline, searches all
gain coordinates in both directions in deterministic order, and accepts at most
eight strict improvements. No terminal, unchanged, malformed or Hamming-read
artifact may run.

## Arms

1. frozen final gain;
2. true-label trained final gain;
3. shuffled-label trained final gain, preserving terminal/nonterminal counts
   and the accepted-move budget while moving terminal labels to a deterministic
   different subset;
4. equal-move random final gain with the same number of +/-1 moves; and
5. free-generation reporting for arms 1-4, using the established BOS/cap/context
   contract and a 16-token cap.

Every training decision is checked after every accepted candidate. Every
held-out arm is checked through the exact CPU/integer audit.

## Bar

Both seeds must pass:

- held-out terminal BOS top-1 is at least **25%**;
- terminal BOS top-1 beats both shuffled-label and equal-move random controls
  by at least `max(2, ceil(2 * sqrt(b + c)))`, where `b` is candidate-only
  correct and `c` is control-only correct;
- held-out nonterminal BOS selections are at most **1%** and no greater than
  frozen;
- held-out nonterminal content top-1 is not below frozen; and
- every arm retains zero query-coordinate, token and kept-cell mismatches.

Free-generation BOS, premature-BOS, cap, context, exact-reference, opening,
distinct-reply and repeated-4-gram counts are reported but are not the primary
termination bar.

## Instrument gates

- Input, source and binary hashes are pinned and verified.
- The training split contains at least one terminal and one nonterminal row;
  the evaluation split contains at least one terminal row.
- True and shuffled training labels preserve class counts, and the shuffled
  assignment changes at least one terminal decision.
- The true trainer accepts at least one move; the random control applies exactly
  that accepted-move count.
- Every accepted move strictly improves the registered terminal objective and
  passes all nonterminal safety gates.
- The output artifacts reparse, differ from their parents where a move was
  accepted, and carry no Hamming record.
- A seed-1 repeat reproduces every non-timing and non-resource report field.

## Steps

Implementation and tests precede a prospective FIT sheet and a fresh non-author
GO review. Only after that review may seeds 1 and 2 run. The run uses isolated
release targets, fresh output roots, an exact binary pin, time/RSS/storage caps
and copied compact evidence. A source, input, parity, control or repeat
mismatch is an instrument fault requiring a new FIT before retry.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-bos-gain-2026-11-04.md`
- `docs/labs/cpu-training-pilot-bos-gain-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-131/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving, integer-session, CUDA, Claude-owned, float-training or model
artifact source change is in scope.

## Done when

1. The cached final-gain trainer exposes exact terminal/nonterminal readings
   and deterministic target-BOS search while preserving the existing
   distance objective.
2. Focused tests cover labels, class-preserving shuffle, candidate safety,
   strict improvement, malformed/empty/no-terminal refusal, Hamming refusal,
   random-control move parity, cached/full-forward agreement and exact serving
   parity.
3. A prospective sheet records the fixed split, arms, controls, bar, resource
   caps and claim boundary, receives TEST FITNESS, and a fresh non-author
   pre-run review returns GO before public model evaluation.
4. Seeds 1 and 2 plus a seed-1 repeat complete or fail closed with an
   instrument classification; both negative and positive outcomes are retained.
5. The REPORT leads with the result, records free generation and exact parity,
   and links compact evidence; the notebook and roadmap preserve the scoped
   result.

## Stop and ask if

Stop and decide; never stop and wait.

- The source artifact or selected split differs, contains Hamming, or already
  contains memory.
- Cached and full-forward replay differ.
- Shuffled labels or the random control cannot preserve the registered
  counts/move budget or are behaviorally vacuous.
- Any accepted artifact loses exact CPU/integer parity.
- A fix would require a serving, integer-session, CUDA or Claude-owned source
  change. Record that boundary and leave the mechanism open for a new unit.
