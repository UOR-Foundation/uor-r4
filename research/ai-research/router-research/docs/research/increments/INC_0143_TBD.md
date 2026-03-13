# INC-0143: Finalize PPMI-SVD Discrimination — Stage 2 Close

## Status
Queued.

## Trigger
INC-0142 Closed: KEEP (2026-03-13). PPMI-SVD semantic embeddings confirm H^4 Hopf routing
discriminates real from col-perm: rel_diff=31.2%, z≈4.2, correct ordering ORIG > COL_PERM >
GAUSSIAN across both seeds. Stage 2 geometry hypothesis not falsified.

Two options are open for Stage 2 full closure:
A. **Finalize (4 seeds)** — confirm INC-0142 result is stable, not a 2-seed artifact
B. **Production embedding test** — confirm discrimination holds with GloVe or LM-activation
   embeddings (richer semantic structure than PPMI-SVD on PTB)

## Hypothesis
The INC-0142 confirm result (rel_diff=31.2%, z≈4.2) will replicate across 4 seeds (option A)
and/or with a production-quality embedding (option B).

If both hold, Stage 2 can be closed as PARTIAL-PASS with the caveat that production routing
requires semantically structured input embeddings — pure hash features are insufficient.

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection (same as INC-0142)
- Whether the INC-0142 discrimination result is seed-stable (option A)
- Whether the same routing subspace works with GloVe/LM-activation embeddings (option B)

## Experiment Design

### Option A: Finalize (4 seeds)
- Same config as INC-0142 confirm, but seeds [0, 1, 2, 3]
- Routes: SEM_ORIG, SEM_COL_PERM, SEM_GAUSSIAN
- phase4_dims: 3,65,2,21 (INC-0142 confirmed subspace)
- Expected: mean rel_diff remains > 0.2 across 4 seeds

### Option B: GloVe 100D Production Embedding
- Download GloVe 6B.100d (if network available) or use a pre-filtered PTB vocabulary subset
- Map PTB vocab → GloVe 100D vectors, context embedding = mean-pool L2-normalised
- Run dim search on GloVe embeddings → find best phase4_dims
- Routes: GLOVE_ORIG, GLOVE_COL_PERM, GLOVE_GAUSSIAN, PPMI_ORIG (INC-0142 reference)

## Success Condition (Stage 2 CLOSE)
- Option A: mean rel_diff > 0.2 across all 4 seeds; no seed individually fails
- Option B: GLOVE_ORIG > GLOVE_COL_PERM by rel_diff > 0.2

Either option alone is sufficient to close Stage 2 as PARTIAL-PASS.

## Falsification Condition
- Option A: 4-seed mean rel_diff < 0.1 — INC-0142 was a 2-seed artifact
- Option B: GloVe also fails to discriminate — Stage 2 geometry law may be structurally wrong

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing

## Notes
The recommended path is Option A first (fastest, no download needed), then Option B if
resources allow. If Option A passes, Stage 2 is closed as PARTIAL-PASS and development
can proceed to Stages 3/4 with the constraint that production architecture requires
semantic input embeddings for routing to function correctly.
