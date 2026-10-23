# REPORT: legal-construction-exact-solve, deepseek-57 (exact single-coefficient census and a feasible legal change)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2828. Base: `2035770c9`. `References #2828`; the item stays `ready`.

## What did not work, first

- **The prior instruments are still blocked and this unit does not repair them.** The exact dense tableau returned no result in 120 s; the bounded-variable revised simplex ran 7,800 s without a basis; the bounded-precision search took 48,025 pivots and returned no basis to certify. The recorded next step was a structural reduction, which this unit follows.
- **The first idea (dual certificate for the zero move) was dropped on evidence.** A nonnegative `y` with `J^T y = g` would prove `g.delta >= 0` for every feasible `delta` and make the zero move optimal. claude-50's phase-2 objective had already gone negative, and this unit exhibits a feasible `delta` with `g.delta < 0`, so no such certificate exists; the case did not fall to the cheap proof.
- **Single-coefficient moves barely exist.** The exact census shows 1,883 of 1,920 columns carry both signs, so only 37 columns permit any single-coefficient move at all; a single-move-only constructor is far from the whole problem. The joint greedy pass is what produced the result.
- **The first version was missing independent verification.** It reported the incremental objective/slack only; a fresh recomputation from `g` and `J` was added and is now mandatory (it agrees exactly). A second run is required to confirm determinism and did.

## What changed

| File | Change |
|---|---|
| `docs/labs/legal-construction-exact-solve/solver/src/bin/single-move.rs` | new exact instrument: legal-destination enumeration, column sign census, exact best single move, greedy joint pass with exact rational slack and per-column feasible interval, mandatory independent recomputation from `g` and `J`, assignment output; two unit tests (destinations keep + both quarter endpoints; sign census separates directions) |
| `docs/labs/legal-construction-exact-solve/README.md` | Unit 4 record: method, census, best single move, the 41-move feasible assignment with its exact objective and active-guard count, limits (no optimality proof), re-run commands |
| `docs/research/notebook/legal-construction.md` | one result row for this unit |
| this folder | ORDER and REPORT |

## Measurements

- Load: 1,920 coefficients, 380 guards, 20 fractional coefficients, 384/384 archive hashes verified, 1.68 s.
- Census (0.13 s): 1,883 both-sign columns, 11 positive-only, 17 negative-only, 9 all-zero; 37 columns admit a single-coefficient move.
- Best single move: coefficient 959, code `q = 7`, `delta = 2/1`, objective change `-8791175/67108864` (about -0.131), exact guard minimum `0/1`.
- Greedy joint pass: **41 accepted moves, no improving legal move remains**, objective **-12316323019/17179869184 (about -0.716904)**, exact guard minimum `0/1`, **318 of 380 guards exactly active**, 39.3 s including the independent recomputation.
- Independent recomputation from `g` and `J`: objective and guard minimum identical (`matches_incremental=true`); second run identical assignment and numbers.
- Crate tests: all suites pass (14 result lines, including the two new tests); build clean.

## Checks run at the head

`cargo test --locked` and `cargo build --locked --bin single-move` in the standalone solver crate; `single-move` on the saved case with the independent recomputation and a determinism rerun; `uor-harness ratchet d23/d25/d32` (baseline); `loop report --check`; `loop pr-check --local`; counted non-author review needed (code PR).

## Skills used

- `solution-integrity`: the cheap dual-certificate idea was tested and discarded on evidence; a feasible move is reported as an upper bound, never as an optimum; the assignment is recomputed independently and determinism rerun.
- `scope-guard`: only the item's lab folder and notebook row changed; archived code untouched; no Python; no bar/status/hold touched.

## Still open

- **No optimality or infeasibility proof, so the item stays `ready`.** The greedy pass is a local search: it stops when no single-coefficient improvement remains, and a multi-coordinate improvement may exist. The immediate next step is a branch-and-bound seeded by this incumbent with an exact LP/dual bound, or a face-reduction proof that the 318 active guards certify optimality.
- The actual commit rule (native scoring, 17 reference cases, strict-loss lowering) is not evaluated here; this unit only solves the saved guard-constrained linear problem. Training and rental remain HELD.

## Plain summary

- An exact sign-test census plus a greedy exact-slack search found the first legal move on the saved legal-construction case: 41 coefficient changes, objective about -0.7169, every one of the 380 guard rows exactly satisfied (318 tight), independently recomputed and deterministic. It is a feasible improving assignment, an upper bound on the discrete optimum, not an optimum; the item stays open.