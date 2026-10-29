# REPORT: fbs-serving-kernels, claude-170 (decision: the exact kernels meet the item; set it done)

Lab: claude. Order: [ORDER.md](ORDER.md). The item is set `done`.

**Speed.** Not applicable: a decision unit, no run. The item's last measured decode on the M1 is 3,042–3,044 tok/s on claude-126's seed-1 export (claude-169), against 270 at claude-163, with every reply byte-identical.

## What did not work, first

- **The item's bar named a solution before its own first unit measured the problem.** Done-when 2 asked for "a vector kernel … in `stack/fbs_simd.rs`, behind a runtime switch". It was written when claude-158's guess was that the route gathers dominate.
  - claude-163's split showed the gathers at 0.5%. The cost sat in the icosian snap and the Hamilton product.
  - Exact structure cut those by 11.3×. A vector kernel was never written.
  - Closing the item needs its done-when amended. That is a decision, not a result.
- **No vector kernel exists.** Whether one would add speed on top of the structured kernels is not measured. In claude-169's profile the snap holds 32% and the Hamilton product 21%, so there is room. The audit makes vector code hard: it refuses any `fmov`, as claude-168 met twice.
- **The original done-when 2's test clause is met at the kernel level, not by one end-to-end test.** That clause asked for a host test holding "every token, kept cell, query and session state equal to the scalar `step_native` over a synthetic trunk and a stored artifact".
  - The units held each changed kernel equal to its predecessor on exhaustive and random inputs. A kernel that is equal on every input leaves every state unchanged.
  - The session twin's tests (`fbs_stage_tests`) still hold the timed step equal to `step_native` on a synthetic trunk.
  - On the stored artifact, the replies, which are the tokens, are byte-identical in every A/B. Kept cells and queries were not compared directly on it.

## What changed

- **`docs/roadmap/items/fbs-serving-kernels.md`:**
  - **Done-when 2** now reads "a faster exact kernel for the largest part: a vector kernel the audit allows, or an exact structured kernel". Its tests are each changed kernel equal to the old one, the session twin equal to `step_native`, byte-identical replies on a stored artifact, and the audit passing.
  - **Done-when 3** now allows the previous unit's sealed binary, run alternately, as the "without" arm.
  - **A `Done (claude-170)` note** names the units that met each done-when, and the open follow-ups (a NEON kernel, fewer golden candidates).
  - **Status:** `done`.
  - **Title:** "Split each served FBS layer, then make its largest parts faster, exactly".
- **`docs/roadmap/README.md`:** re-rendered: ready 17 → 16, done 79 → 80.

## Measurements

None: no run.

## Checks run at the head

- `cargo xtask roadmap check`: 107 items, ready 16, held 10, decision 1, done 80.
- `roadmap render --check`: current.
- The fresh decision review and `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The owner's direction is a cheap served token. The item's purpose is met at 11.3×, exactly, and the lock it held on the serving files is now free for the correctness items.

## Still open

- **A NEON kernel the audit allows, and fewer golden candidates:** open follow-ups, not bars.
- **The serving path's correctness gaps,** which the map ranks above further speed:
  - the FBS export gap (map R6, `d11-exporter-float-trunk`);
  - the integer look-back read (`integer-hamming-read`);
  - the turn memory in the artifact (`turn-memory-integer`);
  - the writer (`writer-admission`).

## Plain summary

The project's work item for making the laptop model faster is now marked finished. It had asked for one particular technique, special vector instructions, but measuring showed the time went elsewhere, and six exact changes made the model 11 times faster without that technique. The item's goal now says what it was really for, and the effort moves to making the served model correct.
