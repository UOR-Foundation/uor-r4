# ORDER: legal-construction-exact-solve, claude-50 (load the saved case exactly and solve its root relaxation)

Lab: claude. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `bca96e43d`. Item PR (partial: `References #2828`).

## Goal

This is unit 3a of the design note on #2828 (6351533475), built on the exact solver in #3005.

1. **Settle the program's inputs from the original source,** not by inference. The source is `protected_joint_vector.rs` and `protected_legal_set.rs` at commit `ebd6ecaa41bd1a3ae90e0f8af3b8363724fb3e82`, which is in the local object store. It fixes which saved files hold `g` and `m`, the family order of the 1,920 coefficients, how `q0` is derived, and which of the 380 guard rows enter.
2. **Load them bit-exactly** from the verified case under `local/claude/legal-case/` (archive md5 `5a965dad…`). Check each file against the hashes the archive itself records (`protected-margin-receipt.json`, `manifest.json`).
3. **Make the LP scale to N = 1,920,** with a bounded-variable simplex or an equivalent exact reduction, still exact and still float-free.
4. **Solve the root LP relaxation exactly,** and record its optimum and the wall time.

**Why the root decides a lot.** The relaxation contains every legal integer move. If its optimum is exactly 0, no legal move strictly lowers `g·δ`, so `δ = 0` is the exact optimum, objective 0, and the item's done-when is met. If its optimum is negative, branch-and-bound is the next unit.

## Cancel probe (run first)

At `bca96e43d`, does a saved-case loader or root result already exist under `docs/labs/legal-construction-exact-solve/`? If one does, the task is void. Class: instrument.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**`
- This task folder

## Done when

1. The loader's file mapping cites the original source, and all inputs match their recorded hashes.
2. The new LP agrees exactly with the #3005 tableau on the existing finite controls and the brute-force suite.
3. The root relaxation's exact optimum, its wall time and, if it is 0, the proof note are recorded in `docs/labs/legal-construction-exact-solve/README.md`. It is called instrument, and states that a solver failure is a blocker, not a mechanism negative.
4. **The checks pass:** crate tests, clippy `-D warnings`, `report --check` and `pr-check --local`. No float appears outside the input conversion.

## Stop and ask if

- The root LP does not finish in about 2 hours of laptop time. Record the timing and stop; a different method is a new design call.
- The original source contradicts the design note's program, for example different guard semantics.
