# REPORT: infra-loop-harness, claude-184 (item maintenance: the two standing 28 Oct directions)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904. The umbrella item stays open; its done-when does not move.

**Speed.** No serving change.

## What did not work, first

- **These two rows could have closed in claude-183, an hour earlier.** I read only the row `loop next` offered, the exporter direction, and its D80 companion. I did not read the whole table for other rows whose named units were already merged. That cost a second claim, a second review and a second merge for a two-row edit.
- **My first edit changed the determinism row's note but not its Status cell.** The row read `open` and "Done" at once, and `loop next` would have kept offering it. The decision review's first round caught it (High). That round also asked the note to say that R6's "with repeats" stays open (Medium). Both are fixed.
- **`loop next` gives GO on every `open` direction, including a standing criterion that no unit can finish.** The table has no status for "in force". This unit records the cost row as `done`, with "the criterion stays in force", which is the closest truthful status of the three. A status such as `standing` would need a harness change. It is listed under "Still open".

## What changed

- **`PLAN.md`:** two 28 Oct owner directions read `done`, each with what carries it.
  - **The cost row:** `fbs-cost-profile` measured cost per token (claude-157, claude-158), and `pipeline-map-revision` made it the map's ranking rule (claude-159). The criterion stays in force.
  - **The determinism row:** the archived prime and angular router is read in the map's own section (claude-159). The float-train, integer-serve route is the one `d11-exporter-float-trunk` closed at DEV scope (claude-182). The fixed-order backward makes training deterministic, and R6's "with repeats" stays open.

## Measurements

None: no run. Every claim cites a REPORT, an item's status or a section of the map.

## Checks run at the head

- **`uor-harness loop gates --full`:** 11 of 11 green.
- **No owner-direction row dated 28 Oct is still `open`:** all 8 quoted-direction rows read `done`.
- **The decision review (D68), fresh:**
  - **Round 1:** REJECT. The High was the status cell; the Medium was the open part of R6. It found the cost row sound, and `done` with "stays in force" the more honest status.
  - **Round 2:** see the review record.
- **`loop report --check` and `loop pr-check --local`:** run before the review record.

## Skills used

- **holding-the-goal.** Each row says what carries the direction, and that the cost criterion stays in force, so the status records no more than the records show.

## Still open

- **A status for standing criteria** in PLAN's owner-direction table, so `loop next` does not offer them as work. It needs a harness change, in `loopnext.rs` and the table's header, as its own unit.
- **R6's "with repeats"** (claude-183's list): a confirming run on new dual-c131 seeds trained with the fixed-order backward.
- **R7, a text path for the FBS artifact,** has no item yet.

## Plain summary

Two more of the owner's 28 October instructions are marked finished in the plan.
- The first, "I don't care what we serve as long as it is cheap", is a rule rather than a task. It is now how the project's map ranks every stage, by cost per token, so it is recorded as done and still in force.
- The second asked the labs to look into an archived routing design. The map now has a section on it.

The loop kept offering these two rows as work, though no unit could finish them.
