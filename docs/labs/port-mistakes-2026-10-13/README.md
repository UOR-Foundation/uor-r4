# D30 port 5: `mistakes` is Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 5 of the series

## What moved

`crates/uor-r4-harness/src/mistakes.rs` — new, **12 tests**. `uor-harness mistakes list|find` is Rust
and **out of the delegated set**. `show` and `capture` still delegate **by name** — `capture` writes a
**lesson file plus a guard**, a bigger surface than reading one, and it belongs in its own unit.

**Verified against Python on a real file** — identical match set:

```
$ uor-harness mistakes find -f crates/uor-r4-harness/src/status.rs
2 lesson(s) apply:
  [block] host-fallback-is-a-defect - A host-side fallback presented as a cost, not as a defect
      why: path crates/uor-r4-harness/src/status.rs
      guard: prlint.no_cpu_model_compute
  [info] portability-constraint-is-interpreter-discovery - ...
```

Same two lessons, same severities, same guards, same `why`. Python truncates the lesson body and
prints it; Rust omits it (see limitations).

## The bug worth naming: `fnmatch` is not a shell glob

**Python's `fnmatch` `*` crosses `/`, and so does `?`.** A shell glob does not. I assumed shell
semantics, so **`*team.rs*` did not match `crates/x/src/team.rs`** — meaning a lesson written against
a bare filename **silently stopped firing**.

**A guard that stops firing is indistinguishable from a guard that passes.** That is the same failure
class as this whole line, and it is now pinned by a test with Python's answers quoted in it:
`fnmatch('src/*.rs', 'src/deep/team.rs')` is True; `fnmatch('a?c', 'a/c')` is True.

## Two tests of mine that were wrong, not the code

Both in the same direction — **I checked the rule I had in mind rather than the one written down**:

1. `query_tokens("stale-binary_x")` — I asserted it split into three. **`-` and `_` are token
   characters**, so it is one token.
2. `query_tokens("ab cd")` — I asserted `cd` survived. The length test is on the **token**, and two
   characters is not more than three, so neither survives.

The code was right both times. Worth recording because a wrong test that gets "fixed" to match wrong
code is how a rule quietly changes.

## Rules the port keeps

- **`status` gates everything** — only `active` and `warn` lessons surface. A closed lesson is history,
  and resurfacing it trains readers to skim.
- **Query tokens are longer than three characters.** `the` appears in almost every trigger, so
  admitting it would match everything and mean nothing.
- **Results sort `block` before `warn` before `info`** — the reader is about to edit code and needs the
  stopping condition first.
- **`why` names which rule fired.** A lesson surfaced for the wrong reason teaches the wrong thing.
- **The tests load the real `docs/harness/mistakes/`**, and one asserts that a real harness source file
  surfaces at least one lesson — **otherwise the command is decorative**.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **194 passed** (12 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| `mistakes find -f status.rs` vs Python | **identical match set** |
| delegation notice for `mistakes list` | gone |

## Limitations, stated

- **`capture` is not ported** — it writes a lesson **plus a guard**, and porting it means porting the
  guard-convention and the front-matter validation with it. `show` still delegates too.
- **The human output is not identical to Python's.** Python prints the truncated lesson body and the
  evidence list; Rust prints id, severity, title, `why` and guard. **The match set is identical, which
  is what `find` is for**, but a script parsing the extra lines breaks.
- **The four `check` rules `mistakes.py` defines** (registry validity, evidence exists, guard covered,
  stale) are **not ported**. They run today through the Python `check`, so a Rust `check` that skipped
  them would be a silent downgrade — which is why `check` stays delegated rather than half-ported.
- **`fnmatch` is implemented for `*` and `?` only.** Character classes (`[abc]`) are treated as
  literals, silently.
- **The tests are not compared against Python for the real registry** — the match set was compared by
  hand on one file. A lesson whose pattern relies on a construct I did not implement would diverge
  without a test noticing.
