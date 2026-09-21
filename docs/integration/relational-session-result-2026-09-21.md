# Learned relational access and content-dependent session control

September 21, 2026. Executes the [learned relational-control brief](deepseek-learned-relation-control-step-2026-09-21.md)
and the [grounded-session review](grounded-session-review-2026-09-21.md), based on the **reviewed head
`5821c48d`** of PR #1340. That PR was still queued at the time of this work, so protection was
preserved, the reviewed head was read directly, and this branch is based on it. Isolation worktree
`.worktrees/learned-relation-control`; owner checkout untouched. New module
`learner/relational_session.rs` and mode `--mode=relational-session`.
[Evidence](../evidence/relational-session-2026-09-21.json).

## The task

A **supplied memory world** is a set of `(role, key, value)` token triples. The record's role token
encodes the relation while the request names the relation, so the role/relation correspondence must be
**learned**, not read off. Every entity has four plausible facts (one per relation), so matching the
entity identity alone cannot resolve the request.

A request is `(relation, entity)`. The session:

1. admits records whose key **exactly equals** the request entity (an exact identity join, separate
   from ranking);
2. ranks the admitted candidates with the learned compatibility and picks one;
3. **gates** on the learned compatibility: an admitted candidate that does not answer the requested
   relation leaves the request explicitly unresolved rather than emitting an incompatible fact;
4. reads the retrieved value; **if that value is itself a key of the world** it follows the learned
   follow-up relation and continues, otherwise it emits and stops.

The role/relation correspondence, the ranking weights, the follow-up map and the continuation policy
are all learned from declared development traces; served inputs never contain a target, a gold hop
count, a next-source pointer or a fixture family.

## Results

| Arm | development /128 | **fresh final /128** | correct depths |
| --- | ---: | ---: | ---: |
| **learned relational (primary)** | **128** | **128** | **128** |
| matched categorical compatibility | - | 128 | - |
| matched cyclic (`mod 120`) | - | 128 | - |
| relation match frozen (exact-key only) | - | **0** | - |
| continuation frozen to always-continue | - | **0** | - |
| reads disabled | - | **0** | - |

Final worlds are a **fresh draw** with new record assignments, not the development worlds. Every
request emits one decoded answer token (for example `able`, `ong`). The three lesions show each
learned component is necessary: without the learned relation match the session cannot select the
answering fact, without the content-dependent continuation it cannot stop on a literal fact, and with
no read there is nothing to answer from.

The matched categorical and cyclic controls reach the **same 128/128**. On this task the geometric
compatibility is *equally* competent, not superior; that is reported as a tie rather than an
advantage.

## Causal interventions

| Intervention | Observed |
| --- | --- |
| one-position source edit (record index 0, all other records fixed) | first retained value, follow-up relation query, selected second occurrence and final answer all change together, matching the independently derived expectation |
| request relation changed, entity and every record fixed | selected fact and answer change, matching the independent expectation |
| required record removed | **explicitly unresolved, no emission** (16/16 worlds, and the single-case intervention) |
| plausible distractor added (same key, unseen role) | answer and selected roles preserved |
| pause/resume mid-session | identical on 128/128 requests |
| two interleaved frames | independent; advancing one leaves the other unchanged |
| owned capture vs live reference | the owned payload stays usable after its origin was overwritten, while the separate origin liveness check reports `false` |

## Composition, session and evidence discipline

The model artifact is 140 bytes, exported and **independently reloaded with full-predictor parity on
256 requests** before any reported result; the reload contract rejects malformed bytes, invalid action
tags, out-of-vocabulary tokens and a terminal frame that still carries a pending action. The frame
serializes request/control state, the owned operand, the retained result, the pending action and the
output position, so restoring it continues the remaining work rather than rebuilding the answer.
Per-request events record the world/version, relation, entity, selected occurrences, retained values,
actions, emitted token and terminal reason, with decoded answers.

## Deviations

I kept the constructive Q8 factorizer from `grounded_session.rs` as a retained component but did **not**
force world relations into it: supervisor/office style relations are memory joins, and the review is
explicit that non-injective world relations need exact memory plus typed operations. Geometry carries a
relation as a group element and scores *relational compatibility*; it does not compute the fact. The
follow-up relation is a learned finite map because a supplied-program schedule is what the previous
milestone already had.

## Not established

The token layout, record triples and follow-up convention are supplied; only the correspondence, the
ranking and the continuation decision are learned. The world is small (8 entities, 4 relations, 4
literals) and relations are single-hop joins. The categorical control ties the group arm, so no
geometric advantage is claimed, and no encoding-normalized or whole-path cost advantage is claimed.
Energy `UNAVAILABLE`; whole-path D0-b not claimed. Retained roots
`relational-session-{1,2,3,4}` (0 unlisted each), with `-4` delivered.
