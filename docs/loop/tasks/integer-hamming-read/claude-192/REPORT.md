# REPORT: integer-hamming-read, claude-192 (one link repaired so the documentation site publishes again)

Lab: claude. Order: [ORDER.md](ORDER.md). This is a one-link repair, not the item's step 2, which is still the next unit. The item has no GitHub issue of its own beyond #3385; claims and merges are noted on #2028.

**Speed.** No serving or training change.

## What did not work, first

- **The documentation site had not published since 2026-10-26T17:02Z.** Every `docs-site` run on `main` failed at `book-check` with 2 broken links. That includes the run after the documentation refresh (#3379, `54d8a3c36`). So the refresh reached the repository but not the site.
- **The cause is a mismatch between `book-stage` and mdBook,** not a missing file.
  - In `docs/research/notebook/m4-serving.md`, row 34 (antigravity-96's, #3287) has 155 backticks before its link to `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`. With an odd count, `book-stage`'s inline-code scan puts the link inside a code span and does not rewrite it to GitHub.
  - mdBook still renders it as a relative link, to a page the book does not stage.
  - The target file exists.
- **Not fixed here:** `book-stage`'s code-span scan, `xtask/src/book_stage.rs`, around line 181. It is outside this item's scope. A row with an unmatched backtick can break the site again.

## What changed

- **`docs/research/notebook/m4-serving.md`, row 34:** the one link `(../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md)` becomes `(https://github.com/UOR-Foundation/uor-r4/blob/main/docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md)`. That is the address `book-stage` would have written. No other character of the record changes.

## Measurements

- **At this head:** `cargo xtask book-stage` staged 104 pages and 70 figure files. `mdbook build` gave 0 errors. `cargo xtask book-check` gave 105 pages and 0 broken links.
- **On `main` (`54d8a3c36`), with the link as it was:** `book-check` gives 2 broken links, both to `harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.html`. That is the same failure as the `docs-site` run, so the check can fail.
- **`docs-site` history:** no success after 2026-10-26T17:02Z, in the 200 runs listed.

## Checks run at the head

- `cargo xtask book-stage`, `mdbook build`, `cargo xtask book-check`: as above.
- `uor-harness loop gates --quick`: green.
- `uor-harness loop report --check`: PASS.

## Skills used

- **holding-the-goal.** The owner asked for the website to be updated. The refresh merged without reaching the site, so the unit traced why the site could not publish, rather than calling the refresh done.

## Still open

- **After the merge:** the `docs-site` run on `main` must succeed (the ORDER's done-when 3). It is checked at `loop close --verify`.
- **`book-stage`'s code-span scan** should match CommonMark for an unmatched backtick. That is an `xtask` change for an item whose scope holds `xtask/src/book_stage.rs`.
- **The item's step 2:** the session hooks for the copy heads.

## Plain summary

The project's documentation website had not updated for three days, because one link in one page could not be resolved, so every publish attempt stopped. This changes that link to a full web address. The site's own build check now passes, and today's documentation update can publish.
