# INC-0139: Shell Law Sensitivity or Pivot to Angular Law

## Status
Queued next.

## Summary
Following INC-0138 REFINE finding: fixed H^4 geometry + adaptive shell activation
produces stable 2-shell structure, but shell assignment is norm-driven — it cannot
discriminate real embeddings from column-permuted controls.

This increment must answer a binary decision:

**Option A:** Can any shell law parameter (K, delta_r, shell_growth, shell_mode)
make the shell level discriminate real from col-perm embeddings?

**Option B:** Accept that shell assignment is fundamentally norm-driven, document
this as a Stage 2 constraint, and formally redirect Stage 2 effort toward improving
the Hopf-base angular law (where semantic structure was observed).

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing

## Mathematical Object Under Test
- First-factor H^4 routing manifold
- Shell law parameter sensitivity under geometry-only conditions (learn_so8=0, learn_scale=0)
- Test: whether any parameter makes shell_pmax or shell_entropy differ by >0.1
  between GEOM_ORIG and GEOM_COL_PERM

## Success Condition
A shell law parameter change (K, delta_r, adaptive_shell_growth, shell_mode)
causes |shell_pmax(GEOM_ORIG) - shell_pmax(GEOM_COL_PERM)| > 0.1, demonstrating
that the shell level can carry semantic discrimination. OR: an explicit decision
is recorded that shell assignment is norm-driven and Stage 2 is redirected to the
angular law.

## Falsification Condition
No tested shell law parameter separates real from col-perm embeddings at the
shell level by more than 0.05. This would establish norm-driven shell as a
fundamental constraint and formally close the shell-engineering sub-track.

## Trigger
INC-0138 Closed: REFINE (2026-03-13) — norm-driven shell finding requires
decision gate before further shell engineering.
