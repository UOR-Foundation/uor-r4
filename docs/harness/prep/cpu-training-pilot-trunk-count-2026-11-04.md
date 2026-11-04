# Public 19-table CPU count training

**Status:** PENDING

## Question

Can the independent CPU path derive exact trunk training rows from a public M1
artifact, add them to all 13 sufficient-statistic count tables, export a new
19-table artifact, and preserve exact CPU/integer serving decisions on
document-disjoint evaluation rows?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API. This is not GPU qualification or language-quality evidence.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `TBD_BEFORE_FIT` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

The artifact is
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/t16-frozen-zero-seed1-trunk.d11`.
The corpus files are in the same directory.

## Data split and arms

Use the existing document-disjoint masked-corpus split with evaluation stride
2 and a fixed cap. Training rows are taken only from the training split;
evaluation rows only from the held-out split. The exact train/eval counts are
fixed by a selection-only run before FIT.

Arms:

1. `no_update`: the source 19-table artifact;
2. `additive`: exact target counts from the training rows;
3. `shuffled`: the same training prefixes with a deterministic non-identity
   permutation of target assignments.

Every arm evaluates the same held-out rows through both `CpuDiscreteTrunkReader`
and `IntegerDiscreteModel`.

## Training procedure

For each training prefix and target, derive:

- the base CPU read code;
- active recurrence, shuffled-order recurrence and unit-normalization codes;
- the final query token and supervised target.

Increment the unigram and all 12 sparse tables exactly as `TrunkTables::build`
does. Counts are checked for overflow. `unigram_best_global` follows count
descending then smaller token id. Export with `export_trunk_d11`; re-import and
require every snapshot/routes/gates/step/norm value and all tables to match.

## Bar

Instrument and mechanism gates:

- 19 tables remain present and parse;
- codebook, routes, gates, recurrence candidates and normalization candidates
  are unchanged;
- exact arm's decoded count tables equal source tables plus the independently
  tallied training rows;
- shuffled and exact differ in target assignments, while their prefix-derived
  codes and non-count sections stay equal;
- every arm has zero query-coordinate, admitted-cell, top-8 and token
  mismatches against integer serving on every held-out row;
- a fresh seed-1 repeat matches every non-timing field byte for byte;
- training and shuffled artifacts are deterministic across construction.

The capability readings (admitted, top-1, top-8) are descriptive. A change is
not a positive language result and no change is not a training failure.

## Steps

The final FIT revision must replace this block with one executable build/run
recipe naming source pin, binary hash, fresh roots, input hashes, exact split
counts, timeout/RSS bounds, output inventory and repeat canonicalization.

## Minimum test power

One public trained artifact is sufficient for writeback correctness. Held-out
coverage must include every selected evaluation row and all four decision
surfaces: Q12 query, admitted cells, top-8 and token. A second seed or a
capability bar is not required for this host mechanism unit.

## Cost and stop

Local workstation only. Final storage/time bounds are fixed before FIT. Stop on
artifact drift, semantic-section change, count mismatch/overflow,
nondeterminism, output-root reuse or CPU/serving divergence.

## Claim boundary

This proves CPU writeback over the full public 19-table family. It does not show
that added counts improve held-out quality, complete replies, fresh phrasing,
useful chat or GPU-qualified training.

## TEST FITNESS

TEST FITNESS: PENDING
