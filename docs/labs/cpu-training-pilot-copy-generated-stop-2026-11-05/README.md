# Generated-trajectory copy stop gate

This record fits a geometric stop gate on the actual longest-first copy
trajectories from panel 2711 and evaluates it unchanged on panel 2712. The gate
uses the existing 256-bit sign-signature, eight-prototype-per-class Hamming
learner. The terminal label is the normalized reference length.

The main result is negative: on both artifact seeds the dev gate produced
**0/449** exact replies. It sometimes fired at the reference length, but those
bodies were not exact. Training-panel premature stops were **34** and **31**
against a 4-row 1% limit; dev premature stops were **3** and **8**.

The oracle ceiling remains 11/453 train and 13/449 dev. The current gate does
not recover it. This closes the fixed all-reference-length labeling with eight
signed prototypes; it does not close geometric termination generally.

Attempts 1–5 are preserved:

- attempt 1 used the wrong report schema literal;
- attempt 2 completed but wrote transient `/dev/fd` report paths;
- attempt 3 stopped after report reads were moved to hashes because a process
  substitution could not be read twice;
- attempt 4 reproduced the result but omitted per-row fired decisions and
  selected-trace hashes;
- attempt 5 added those records but counted no-fire cost only to the reference
  horizon.

Attempt 6 is final, includes per-row audit records and a byte-identical repeat
after timing removal.
`no_fire` means no gate firing through the reference horizon; token cost is
charged to the complete 64-token selected trace.
Published evidence is in [`evidence/`](evidence/).

## Exact-body follow-up

Deepseek-159 filtered training to the 11 trajectories whose generated body
already equalled the reference before fitting the same gate. Both artifact
seeds then fit **7/11** terminal rows with zero false stops on 308 selected
nonterminal rows. On the unchanged dev panel the gate produced only **1/13**
exact replies on each seed. Seed 2 also emitted **9** premature stops against
the four-stop limit.

This shows that the filtered training state is separable in sample but does
not transfer. The shuffled, deepseek-157 all-reference and no-gate controls
each produced zero exact dev replies. Attempt 7 is the sealed exact-body run,
with a byte-identical fresh-process repeat for each artifact seed. The result
closes exact-body filtering for this eight-prototype gate, not geometric
termination. See [deepseek-159](../../loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md).
