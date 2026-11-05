# ORDER: cpu-training-pilot, deepseek-183 (query-port root training)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `7787313b2`.

## Goal

Add a deterministic integer trainer that moves only the final-gain coordinates
which scale the writer query's first and last eight-coordinate ports, then run a
pre-registered two-artifact CPU comparison of that objective against matched
random-gain, shuffled-cell and frozen controls through the exact exported
serving path.

This directly targets Claude-289's diagnosed seed-2 failure: the query's two
ports fell to the roots' centre. It is a new objective, not another rank-margin
or candidate-ranker width experiment.

## Fixed construction

1. Build a cached exact trajectory from causal `RankMarginSample`s. For each
   target token `t`, the target cell is `t mod 256` and its two roots are
   `cell / 16` and `cell % 16`.
2. A trial changes one final-gain code by `±1` and replays only normalization,
   final gain, Q12 conversion and the writer through the patched artifact.
3. Candidate safety requires no fall in held-out-independent training target
   admission, top-1 or top-8. Among safe candidates, lower exact Q12 L1 distance
   from the query's two ports to the target cell's roots wins.
4. Only final-gain coordinates `0..8` and `W-8..W` are trainable. All other
   gain coordinates and model tables remain fixed.
5. Every accepted artifact is reparsed. CPU/serving query, token and kept-cell
   parity must remain exact.
6. Matched controls use the same move count: random port-gain moves and a
   shuffled-cell root objective.

No serving, CUDA, float-training, exporter or Claude-owned source changes.

## Tests

- cached replay equals a fresh complete forward;
- only port-gain coordinates move;
- a synthetic port-root objective accepts a safe improving move;
- admission/top-1 regressions are refused;
- shuffled-cell and random controls are deterministic and non-vacuous;
- malformed artifacts, Hamming-dependent artifacts and memory-bearing source
  artifacts fail closed for this bounded trainer;
- exact CPU/serving parity after every accepted move.

## Experiment

- use both public dual-c131 artifacts;
- use the existing repeated-key conversation split, with train and evaluation
  documents separated before trajectories are read;
- compare frozen, trained, shuffled-root and random-gain artifacts, with table
  port anchoring as a diagnostic arm only;
- report target-root distance, target admission, writer top-1/top-8, exact
  parity, moved coordinates, repeat identity, wall time and peak RSS.

## Bar

On both artifacts, the trained arm must:

- accept at least one changing port-gain move;
- reduce held-out target-port distance below frozen and random;
- not reduce held-out target admission or top-1 below frozen;
- beat the shuffled-root arm on target-root distance;
- retain zero CPU/serving parity mismatches;
- repeat after timing fields are removed.

## Done when

1. The trainer, driver and focused tests are complete.
2. A new pre-registration sheet records the exact inputs, controls, bar, cost,
   stop rule and implementation pin and reaches fresh-context FIT/GO.
3. Both artifacts run once and preserve sealed evidence.
4. The REPORT and notebook record the verdict without capability overclaim.
5. Reviewed merge, fresh-main verification and claim release complete.

## Stop rule

Stop on source/input drift, parity drift, a vacuous control, no accepted move on
either artifact, nondeterminism, a registered time/RSS stop, or any need to edit
serving, CUDA, float-training, exporter or Claude-owned source.
