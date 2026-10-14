# `status.registry_vs_github`: a check that cannot run no longer reports as a pass

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the sibling defect named in `docs/labs/skipped-is-not-a-pass-2026-10-13/README.md`

## The defect, and the test that pinned it

Both unavailability paths returned **`severity="info"`, `ok=True`**:

```python
return [RuleResult(..., severity="info", ok=True, message=f"GitHub not checked: ...")]
return [RuleResult(..., severity="info", ok=True, message="GitHub unavailable, ...")]
```

**So a lab without GitHub access got a green `check` on the rule whose entire job is to notice that
`milestones.json` and the milestone issue have drifted apart.**

**And an existing test asserted exactly that**, named for it:

```python
def test_registry_vs_github_degrades_to_info_without_gh(self) -> None:
    ...
    self.assertTrue(results[0].ok)
    self.assertEqual(results[0].severity, "info")
```

**The test named the defect and pinned it as intended behaviour.** It is updated to pin the new one.

## Fixed

**`warn` with `ok=False`** on both paths, messages opening **`the registry was NOT cross-checked
against GitHub`**. **`error` would be wrong, and the argument is stronger here than for a missing
binary**: GitHub rate-limits, and a lab working offline is not violating anything — **a
permanently-red check gets ignored.**

**What changes is that neither case is silent.** Both paths verified by forcing the failure.

## What the fix revealed, which is larger than the fix

**With GitHub available, the rule returns 8 findings — and every one is `ok=True`, `info`:**

```
[M1] newest card (2026-10-13T23:26:14Z) has no machine-readable number; registry says ...
[M2] newest card (2026-10-13T02:52:05Z) has no machine-readable number; registry says ...
... 8 total; ok values: [True]; severities: ['info']
```

**The rule cannot fail.** Its comparison never completes, because every milestone's newest card lacks
the machine-readable number it looks for. **I checked the cards rather than assuming the parser was
stale** — the newest card on #2029 is a prose work card with no numeric field. **So the parser is
right, and the rule has never cross-checked anything.**

**This is the same shape a third time, and larger**: the unavailability paths were the smaller half.
**A comparison that cannot complete because the input never carries the field is reporting green on
every run — not because the registry agrees with GitHub, but because nothing was ever compared.**

**I am not changing that behaviour here.** `info` may be the deliberate design for "the card has no
number", and **deciding what the rule should do when no card is machine-readable is a policy
question, not a port.** It is recorded on #2028 with the measurement so it can be decided rather than
rediscovered.

## Verification

| check | result |
|---|---|
| raising lookup | `warn`, `ok=False`, `NOT cross-checked` |
| GitHub reported unavailable | `warn`, `ok=False`, `NOT cross-checked` |
| the old test, renamed and updated | passes, pinning the new behaviour |
| `python3 -m unittest discover -s tests/harness -t .` | **326 passed**, twice |
| `uor-harness check` | 0 errors |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |

## Limitations, stated

- **`parity.py` fails during this unit, and not because of this change.** `render agreement: line 3:
  rust '- date: 2026-10-14' != python '- date: 2026-10-13'`. **Root cause: Rust's `prep::today()`
  slices `now_rfc3339()[..10]` — the UTC date — while Python's `today()` uses local time.** In US
  evenings they differ by a day, so **parity fails for roughly the last five hours of every US
  working day.** Not fixed here: it is a real defect, **separate from this change**, and I would
  rather report it precisely than bundle it. **A verification that fails on a schedule is one people
  learn to ignore.**
- **The two unavailability cases still share one severity.** *"Rate-limited right now"* and *"this
  registry has disagreed with its issue for a week"* both produce one warning. **They are no longer
  silent, but they are not yet distinguished** — and distinguishing them needs the comparison to
  work first.
- **The suite is flaky under concurrent lab activity.** Two runs during this unit reported failures
  and two reported `OK`; **I repeated before concluding.** A single red run in this worktree is not
  evidence of a regression, and **the registered rule count moved between 53 and 56 across runs.**
