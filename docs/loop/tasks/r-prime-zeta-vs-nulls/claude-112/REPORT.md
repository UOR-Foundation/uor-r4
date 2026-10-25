# REPORT: r-prime-zeta-vs-nulls, claude-112

Lab: claude. Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md). Sheet: [`r-prime-zeta-vs-nulls-2026-10-25.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-2026-10-25.md).
Closes #2881.

## What did not work, first

- **Prime-zeta does not beat the injective nulls at zero updates, so this configuration is negative (loss), D22.**
  - P8 wins against none of the quota-matched nulls. It loses to Q0 (b/c 122/173) and Q1 (122/179), and ties Q2 (122/121).
  - Against the random-injective nulls it loses to R0 and R1 and wins against R2 (125/94).
  - The configuration closed is checkpoint 0, K = 8, three seeds per null family, on the 14,287-row DEV panel. The mechanism stays ACTIVE. The trained comparison stays held, and this run does not test it.
- **The loss rests on nulls that landed high.** P_succ admission across null seeds runs from 126 to 184 for Q and from 96 to 188 for R. P8's 127 sits at the low end of that spread, near the K = 8 of 256 cells chance rate of about 120.
  - One hypothesis, untested here: at checkpoint 0, admission depends on each table's particular placement more than on its construction family.
  - This is descriptive, and decides nothing.
- **The tables admit nearly disjoint rows.** P8 shares 2 to 8 of its 127 admitted rows with any null, against about 3 to 6 expected if admission were independent of the table.
- **The notebook row could not be written to its page.** The installed loop guard refuses `docs/research/notebook/m1-prime-zeta.md`: it is outside this item's scope, and the guard fix that admits an item's own notebook page (#3189) is not in the installed binary. The filled row is in the lab README §5, ready to carry over.
- **The first FIT returned NOT FIT, with one Required finding.** The grader's update refusal read two fields that the crate always writes as 0, so that check could never fail. It was fixed in `f5155fe17` (see Measurements).
- **The run cost more per hour than the sheet planned.** The provider gave a 2×5090 at $2.176/h for a 1×5090 request. The cached build made the run short: about 9 minutes, about $0.33, against the sheet's $0.20–$0.60.
- **A second claim this session was cancelled by its probe.** `recurrence-gates-six-recurrence-fit` (#2890, draft #3243) was claimed after `loop next` offered it.
  - Antigravity had registered and FIT'd its run sheet 80 minutes earlier (#3234), with its results path under `results/antigravity/`.
  - Its runner, arms and recurrence export are not on `main`, and the item's scope holds no crate path.
  - #3243 is closed, and its branch and worktree are removed, with a note on #2890.

## What changed

Docs and a detached instrument only. This item's scope holds no crate path.
- **The D68 card** `DECISION.md` registers the bar and the instrument gates. A fresh decision review returned CHANGES in round 1 and APPROVE in round 2.
- **The final sheet** `docs/harness/prep/r-prime-zeta-vs-nulls-2026-10-25.md` registers that bar, pins the source, configs, job and grader, and carries the FIT record.
- **The grader** `docs/labs/r-prime-zeta-vs-nulls/instrument/` (`pz-nulls-grade`) is its own `[workspace]`.
  - It reads `uor-r4-discrete` by path, and reuses `address_key_grade::{paired_psucc, clears}`.
  - It has 11 tests: 10 unit tests plus a config check.
  - That config check confirms that the seven configs parse as `DriverConfig` under the pod's source, match their pinned bytes, and differ only in the table.
- **The job** `docs/labs/r-prime-zeta-vs-nulls/pz-nulls-job.sh`.
- **The grade** `docs/labs/r-prime-zeta-vs-nulls/grade.json`, and the lab README's §5 with its status line. Antigravity's text above them is unchanged.
- **The item is set `done`,** and the roadmap is re-rendered (ready 24, done 51).

## Measurements

- **Run:** `vast-57710234`, source `d1eea395d64f1d95e753f6800bfaae9c69c2f212`, job sha256 `820ee91645dcb3541a212304f0fb1a518f2e7834b074a4498a909c79b4c5e80e`, EXIT 0.
  - Each arm's eval took 27 to 28 s.
  - The outputs are in the HF store at `results/claude/prime-zeta-nulls-2026-10-25/run/`. All 68 files verified against SHA256SUMS after download, before the pod came down.
- **Instrument gates,** all passed in the job:
  - G-Rebuild: all 7 tables equal their pins.
  - G-Init: each checkpoint has 0 updates and 0 batches, its 131,072-word table equals its pin word for word, and routes and gates are identical across the arms.
  - G-Occ: as in the table below.
  - G-Exact: P8 read 427 / 127 / 38, before any null was evaluated.
- **The grade:** `grade.json`, sha256 `14cc2c62882069361c22632b94f867be480fda9bd822adff0490990b5ec37975`. Its verdict is `negative (loss)`.

| Arm | P_succ (of 3,847) | Full (of 14,287) | Top-1 | Top-8 | Collision pairs | Paired b / c against P8 | Outcome for P8 |
|---|---:|---:|---:|---:|---:|---:|---|
| P8 | 127 | 427 | 38 | 95 | 30,720 | — | — |
| Q0 | 178 | 564 | 36 | 74 | 30,720 | 122 / 173 | loss |
| Q1 | 184 | 560 | 49 | 133 | 30,720 | 122 / 179 | loss |
| Q2 | 126 | 696 | 52 | 158 | 30,720 | 122 / 121 | tie |
| R0 | 183 | 553 | 24 | 72 | 32,493 | 119 / 175 | loss |
| R1 | 188 | 485 | 25 | 116 | 32,802 | 125 / 186 | loss |
| R2 | 96 | 373 | 27 | 73 | 32,659 | 125 / 94 | win |

- **Every arm** has 4,096 distinct vectors, alias ceiling 14,287, zero tie mass and 256 occupied cells. The R arms have 3 to 8 tokens in their emptiest cell, and 26 to 27 in their fullest.
- **Matched-seed crowding, reported only:** Q0 against R0 is 169/174 (tie), Q1 against R1 is 159/163 (tie), and Q2 against R2 is 122/92 (win for Q2).

## Checks run at the head

- **The grader:** 11 tests pass. Three negative gates were run, each setting one line aside and then restoring it byte-identical:
  - the loss branch fails 2 tests;
  - the R-arm 256-cell check fails 1;
  - the `accepted_updates_in_model` refusal fails 1.
- **The decision review (D68):** round 1 CHANGES, round 2 APPROVE.
- **The FIT:**
  - Round 1 was NOT FIT. Its R1 finding: the update refusal read the literal-zero fields `updates_applied` and `updates_applied_by_evaluation`.
  - The fix reads the checkpoint counters, binds each eval and census report to its dev-b3 report by the model storage hash, and refuses absent hashes.
  - Round 2 was FIT, and the D67 pre-run review passed (#2881 `6401883120`).
- **`xtask roadmap check`:** 92 items OK.
- **Still to come:** `loop report --check`, `loop pr-check --local`, a fresh claims check of this REPORT and the README §5, and the exact-head review record.

## Skills used

- holding-the-goal: the outcome rule was fixed and registered before the run (§4.5). Instrument gates were classified apart from readings (§4.1, §4.2, D31), and the descriptive spread is reported without changing the registered verdict.

## Still open

- **The notebook row.** Carry the README §5 row into `docs/research/notebook/m1-prime-zeta.md`, once a unit or guard can write that page (#3189's guard fix, in an installed binary).
- **The mechanism page** `docs/mechanisms/prime-zeta-codebook.md` has no "tried" row for this result. Its path is outside this item's scope.
- **More null seeds, or a variance-matched bar,** would test whether P8 sits inside the null families' own spread. That needs a new sheet.
- **The trained comparison against the nulls** is still held.

## Plain summary

The project gives every word a fixed starting address built from prime numbers and the zeros of the zeta function. I checked whether that special construction helps the model's reader find the right word before any training, compared with simpler tables that only keep every word distinct. It does not. Two of the three simple tables that keep the same cell layout found more of the right words, and the third tied. The simple tables also differ a lot from one another, and the prime-zeta table falls at the low end of their range. So at this starting point, this configuration of the special construction buys nothing over a plain distinct-address table.
