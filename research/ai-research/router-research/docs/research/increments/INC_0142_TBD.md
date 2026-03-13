# INC-0142: Semantic Embedding Proxy Task — Stage 2 Routing with Structured Embeddings

## Status
Queued.

## Trigger
INC-0141 Closed: KILL (2026-03-13). INC-0136–0141 exhausted all routing paths on the
wikitext2 hash embedding. Mathematical proof: chi_u and delta are scale-invariant; the
hash embedding is isotropic by construction. No 4D Hopf subspace of a hash feature
produces semantic angular concentration. Stage 2 is proxy-task-blocked on hash embeddings.

## Hypothesis
The wikitext2 hash embedding proxy task was the failure mode, not the H^4 Hopf routing
geometry itself. Semantically structured embeddings (where nearby tokens in the embedding
space share semantic content) would have within-pair Hopf angular correlations that reflect
semantic clustering rather than hash-induced isotropy.

If the H^4 Hopf sector routing law is valid, testing with semantically structured
embeddings should show pmax_after(ORIG) > pmax_after(COL_PERM) by at least 0.2 ratio.

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection
- Whether semantic embedding clustering maps onto concentrated Hopf sectors
- Whether col-perm (which destroys semantic nearest-neighbor structure) produces
  measurably less concentrated sector routing than the original embedding

## Proposed Embedding Source
One of:
1. **GloVe 100-dim** (public, small, known semantic clustering) — simplest option
   - Download: glove.6B.100d.txt (~822 MB)
   - Map wikitext2 tokens to GloVe vectors → semantic embedding matrix
2. **LM activation proxy** — intermediate language model hidden states
   - Slightly more complex; richer semantic structure
3. **Word2Vec skip-gram** — similar to GloVe but different training
   - Well-studied semantic properties

Recommended starting point: GloVe 100-dim (simple, self-contained, widely validated).

## Experiment Design
- sector_mode: phase4d_hopf_base
- learn_so8: 0, learn_scale: 0
- adaptive_shell_growth: 0.0 (pure angular isolation, matching INC-0140/INC-0141)
- phase4_dims: TBD — run dim search on semantic embedding to find phase4_dims with
  strongest within-pair correlation (same approach as INC-0141 pre-screen)
- Routes:
  - SEM_ORIG: semantic embedding, input_transform=none
  - SEM_COL_PERM: semantic embedding, input_transform=col_perm
  - SEM_GAUSSIAN: semantic embedding, input_transform=gaussian
  - CTRL_HASH: hash embedding (wikitext2 default), input_transform=none (INC-0141 baseline)

## Success Condition
|pmax_after(SEM_ORIG) − pmax_after(SEM_COL_PERM)| / mean > 0.2

This would demonstrate:
- The H^4 Hopf routing law IS semantically discriminative given appropriate embeddings
- Stage 2 wikitext2 failures were proxy-task failures, not geometry failures
- Stage 2 can be closed as PARTIAL-PASS with caveat on embedding requirements

## Falsification Condition
pmax_after|diff|/mean < 0.1 with semantic embeddings — routing does not discriminate
real from col-perm even with semantically structured input. This would imply:
- The H^4 Hopf sector routing law is structurally wrong for this proxy task
- Stage 2 may require a fundamentally different sector law (not fixed Hopf subspace projection)
- The kill-list Stage 2 criterion needs re-scoping

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing

## Blocker Requirements Before Starting
- [ ] Choose embedding source (GloVe recommended)
- [ ] Download / prepare embedding matrix aligned to wikitext2 vocabulary
- [ ] Wrap as a new proxy task data file (.npz format matching wikitext2_proxy)
- [ ] Run pre-screen dim search: full pairwise correlation matrix, find best 4D subspace
- [ ] Verify TV(ORIG vs PERM) > 0.05 in pre-screen before running full sweep

## Notes
This increment is a prerequisite reconstruction, not a lateral investigation. The
cascade from INC-0136–0141 provides clear evidence that the proxy task (not the routing
geometry) was the failure mode. INC-0142 is the minimal next honest test.
