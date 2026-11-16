# REPORT: end-to-end-integration, claude-396

Lab: claude. Order: [ORDER.md](ORDER.md). Parent: [codex-2](../codex-2/REPORT.md) (PR #4013).

## What did not work, first

- **The bare pair score fails the null control.** With no frame change at all, `spin-raw` finds the target on only 124 of 200 trials at K = 8, and 70 of 200 at K = 32. A larger key wins on the bare score. Only the normalised score `s·|s| / (|a_q|² |a_j|²)` is asserted. The key's norm is its own self-pair, so the normalised score is still frame invariant.
- **The planted win is close to guaranteed by construction.** The target is the query's own feature seen in another frame, so its pair score equals the query's self-pair exactly. This probe tests the mechanics: that the score keeps a match under a frame change, and that raw-coordinate scorers lose it. It does not test learned retrieval.
- **The margin shrinks as K grows.** For the planted case, the min/median spin margins are 13/351 thousandths at K = 8 and 1/82 at K = 32. For the null case they are 1/278 and 1/102. At K = 32 the closest distractor is already within 0.001 of the target on some trial.
- Random small `Z[φ]` features stand in for real states. No token or trained state enters the probe.

## What changed

- `crates/uor-r4-integer/tests/sh_spin_retrieval_probe.rs` (new, test only). The probe copies codex-2's exact arithmetic: `Z[φ]`, the 120 binary-icosahedral quaternions and the sandwich action. Each trial plants a query and K candidates:
  - the **target** is the query's feature in another frame (pose `q_q g`, feature `D(g)⁻¹ a_q`);
  - the **distractors** are random small features with random poses;
  - in the **planted** case, one decoy copies the query's coordinates under a pose that is not frame consistent.
- There are four scorers, all exact: `spin` (normalised), `spin-raw`, `l1` on the raw coordinates, and `popcount` on their sign bits. A tie at the top never counts as correct.
- This ORDER, this REPORT and one notebook row. No source change, no dependency.

## Measurements

`cargo test --locked -p uor-r4-integer --test sh_spin_retrieval_probe -- --nocapture`, 200 trials a case:

| case | K | scorer | correct | target-tied | wrong | tie@top | spin margin min/median (1/1000) |
|---|---|---|---|---|---|---|---|
| null | 8 | spin | 200 | 0 | 0 | 0 | 1/278 |
| null | 8 | spin-raw | 124 | 1 | 75 | 2 | – |
| null | 8 | l1 | 200 | 0 | 0 | 0 | – |
| null | 8 | popcount | 178 | 22 | 0 | 22 | – |
| null | 32 | spin | 200 | 0 | 0 | 0 | 1/102 |
| null | 32 | spin-raw | 70 | 0 | 130 | 0 | – |
| null | 32 | l1 | 200 | 0 | 0 | 0 | – |
| null | 32 | popcount | 114 | 86 | 0 | 86 | – |
| planted | 8 | spin | 200 | 0 | 0 | 0 | 13/351 |
| planted | 8 | spin-raw | 143 | 0 | 57 | 0 | – |
| planted | 8 | l1 | 0 | 0 | 200 | 0 | – |
| planted | 8 | popcount | 0 | 5 | 195 | 20 | – |
| planted | 32 | spin | 200 | 0 | 0 | 0 | 1/82 |
| planted | 32 | spin-raw | 59 | 0 | 141 | 1 | – |
| planted | 32 | l1 | 0 | 0 | 200 | 0 | – |
| planted | 32 | popcount | 0 | 2 | 198 | 83 | – |

**Classification (§4.2):** a mechanics and instrument result, not a model result.
- Under a frame change, the raw-coordinate scorers are blind by construction: L1 and popcount get 0 of 200.
- The normalised spin-1 score keeps the match on every trial.
- With no frame change, L1 finds the target as well as the spin score does. So the spin score's advantage is frame invariance alone, not sharper retrieval.

## Checks run at the head

- The probe: 3 passed, the table above. This is the second run, and it matches the first cell for cell.
- `git diff --check` is clean. A grep of the added docs for the D23 interlock's banned words finds none.
- `loop pr-check --local` and the exact-head review are recorded on the PR.

## Skills used

- `solution-integrity`: the null control runs before the planted case. That is how the failure of the bare score was found, rather than hidden by the planted wins.
- `scope-guard`: one new test file and docs; no source, schema, dependency or mechanism status.

## Still open

- **What this unlocks.** The l = 2 and 3 tables may be built, on two conditions: the score is normalised, and a margin-versus-K check runs at real key counts first.
- Margins at K ≥ 128, where the median margin may close.
- Real states rather than random `Z[φ]` features: the token-to-coefficient map is not built.
- Whether the normalisation (a division by two squared norms) can be kept integer and fixed-width at serving width.

## Plain summary

The spin-1 score finds a key that has been moved into another frame, and L1 and popcount cannot: 200 of 200 against 0 of 200. That holds only once the score is divided by both keys' sizes, since the bare score loses even with no frame change. The win is built into the setup, and the gap to the next-best key shrinks as the key count grows. So the next spin unit measures that gap at real key counts before building the higher tables.
