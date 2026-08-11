# VERIFICATION

Which axis of `just vv` discharges which class of claim. `AGENTS.md` defines the
rules; this maps them onto the commands that enforce them.

| `just` recipe | Enforces | ID classes |
| --- | --- | --- |
| `just fmt-check` | the diff is reviewable | --- |
| `just model` | R1, R4, R5 | `CM-01` |
| `just lint` | clippy at `-D warnings` | --- |
| `just test` | the workspace suite | --- |
| `just features` | every optional feature compiles, with its tests | --- |
| `just bdd` | R3 and R2's behavioural half | `CM-02`, `CM-03` |
| `just deny` | R6, over the dependency graph | --- |

The ID column is thin because the register is empty. It fills as capabilities
are added; an ID with no scenario, or a scenario with no test, fails `just bdd`.

## Every gate is falsifiable

A gate nobody has seen fail is indistinguishable from a gate that cannot. Before
adding one, plant the defect it exists to catch, confirm it fires, and add a row
here.

| Gate | Planted defect | Reported |
| --- | --- | --- |
| `check-model` (R1) | a `CONFORMANCE.md` that disagrees with the register | yes |
| `audit-deferral` (R4) | a deferral marker in a crate, and one in the gate's own source | yes, both |
| the honesty meta-gate (R2) | an ID with no test | armed by the register |

The last row says what it does on purpose. With an empty register there is no ID
whose test can be removed, so the plant has nothing to plant; the test prints
that rather than passing quietly, because a falsifiability check that cannot
falsify is the exact thing it exists to rule out. It re-arms with the first ID.

`audit-deferral` is worth the second column. It reads every crate *and* `xtask`,
so it reads its own source, and its markers are therefore spelled in halves: a
list of forbidden tokens written out in full matches itself, and exempting the
file would leave a hole precisely where a deferral parked in a gate would sit.
Both plants were run --- one in a crate, one in the gate --- and both were caught.

## What this suite does not establish

Anything about a dependency. A library imported here is gated in its own
repository; restating its guarantees would give a claim two sources, which is
what R1 forbids. What may be claimed here is what is built here.
