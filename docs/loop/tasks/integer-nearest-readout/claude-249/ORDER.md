# ORDER: integer-nearest-readout, claude-249 (Q5's trace instrument, the integer side R)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `381ae4d98`.

## Goal

Done-when 3 now waits on Q5 alone. Q5 needs an exact trace of 184 clear-margin disagreements, under claude-247's [decision card](../claude-247/DECISION.md). The trace that rule needs does not exist at the base (the card's "What the instruments can and cannot do"). This unit builds the instrument's integer side, R; later units build the rest.

**What R's side must do, from the card:**
- **Record every decision of the five kinds** at every step, in the forward's order:
  - each layer's rate buckets in lane order, with the score `s_q16` each was chosen from;
  - each stream read at its depth, then the readout read: for each head, in head order, the query code, the key code and the kept set;
  - the turn memory's lookup;
  - the nearest: the pick, the runner-up and both scores at Q12, with R's Q12 query.
- **Write R's query at every reply offset,** so a later grader can score any token, including the other side's pick (`s_R(u) = −Σ_j |Q_j − T_{u,j}|`).
- **Force codes and rate buckets.** A forced set Φ, per window, replaces:
  - a rate bucket at (step, layer, lane) with a given entry;
  - a code at (step, depth, head, role) with a given word.

  A forced query code at step t replaces that row's query. A forced key code at step s replaces candidate s's key at every later row, because a key is the code of `x_{s−1}`, decided once at step s.
- **Run from `fbs-toks --forced`:**
  - `--decision-trace FILE` writes the trace of the rows it runs (`uor-r4.decision-trace/1`);
  - `--force FILE` (`uor-r4.decision-force/1`) runs only the rows it names, each under its own Φ.
  - The forced rows it writes keep claude-244's format, so I-R can compare them with the sealed files.

**What this unit does not do:**
- the float side (F and T), T's rounded copy, the replay driver, the near-tie grader and the sheet: later units, in that order;
- resuming a pass from saved state. The card allows it but does not require it; a pass runs from the window's start, about 0.16 s on R.
- R's code coordinates. R is never a step's reference side (the card: F for the structure step, T for the arithmetic step), so no rule reads them, and writing 64 coordinates per code per head per step would make each window's trace hundreds of times larger. The trace records each code's word.

**The schemas are shared.** `fbs_snapshot.rs` holds them, beside `HammingTrace`, so the float side's unit writes the same records (the float crate depends on `uor-r4-integer`).

## Cancel probe (run first)

- **Does an instrument already record or force R's decisions?** At `381ae4d98`:
  - `fbs-toks --hamming-trace` writes codes and kept sets only, with no buckets, memory lookup, nearest or query;
  - nothing in `stack/hamming.rs` or `stack/session.rs` forces a decision (`grep -i forc` finds only unrelated "brute force" comments).
- **Does an open order hold any of the files?** The ORDERs without a REPORT are this one, kimi-1, kimi-2 and kimi-43. None of the Kimi ones names `hamming.rs`, `session.rs`, `stack/tests.rs`, `fbs_snapshot` or `fbs-toks`, and no other PR is open.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`: forced codes in the copy and type heads' choices, and the record of the codes as used;
- `crates/uor-r4-integer/src/stack/session.rs`: the decision trace and forcing in `stack_step_fbs` and `stack_lanes`;
- `crates/uor-r4-integer/src/stack/tests.rs`: the tests;
- `crates/uor-r4-integer/src/fbs_snapshot.rs`: the two schemas;
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`: `--decision-trace` and `--force`;
- `docs/roadmap/items/integer-nearest-readout.md`;
- `docs/loop/tasks/integer-nearest-readout/claude-249/**`.

## Done when

1. **The trace.** `fbs-toks --forced … --decision-trace` writes `uor-r4.decision-trace/1`. At each step it holds the five kinds in the forward's order, with R's query at every reply offset. A session refuses the trace on an artifact off the card's path: not FBS, a recurrence that is not geometric, or a snapped artifact.
2. **The forcing.** `--force` replaces codes and buckets as above. Tests show:
   - a traced session, and one forced to its own recorded decisions, step exactly as an untraced one: the same tokens and queries at every step (I-R's test, and I-Force's second part);
   - forcing a side to its own decisions leaves its trace unchanged (I-Force);
   - a forced bucket and a forced query or key code take the forced value in the trace, and change what comes after (I-Force's first part). Negative gate: with the line that applies the forcing set aside, those tests fail;
   - a forced set naming a step, layer, lane, read, head or entry outside the model is refused.
3. **The served path is unchanged.** These pass:
   - the full `uor-r4-integer` suite;
   - the serving audit (`audit_zero_matmul_serving.py --stack`);
   - the WASM build;
   - `loop gates --full`.
4. **A host smoke run on R s1's artifact** (`claude-243/R/a7/a7r-s1.uorlut`). `--forced --decision-trace` runs on DEV s1 rows outside Q5's set, and its forced rows equal claude-244's sealed `forced-dev-s1.json` rows exactly. The time per window is recorded. No row of the set is run, so no trace value Q5 reads is seen before its sheet.
5. **A fresh exact-head review passes,** and the REPORT puts the negatives first. No notebook row: the unit measures no result.

## Stop and ask if

Stop and decide, never wait:
- the forcing needs a file outside this scope, such as `stack/memory.rs`. Memory lookups are never forced, so none is expected; if one is, it gets its own scope change first;
- any existing test, the serving audit or the WASM build fails with the trace off. Then the served path changed, and that is fixed before anything else;
- R's step makes a discrete decision the card does not list. That voids the card (its I-Complete), so it is recorded and the card is reopened before any further unit.
