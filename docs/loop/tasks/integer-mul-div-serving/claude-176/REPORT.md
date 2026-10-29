# REPORT: integer-mul-div-serving, claude-176 (the docs say what D80 and D81 allow)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays open: done-when 1 is now met (claude-175 did the audits, this unit the docs), and done-when 2 to 4 remain.

**Speed.** Not applicable. This unit changed docs only and measured nothing, so there is no notebook row.

## What did not work, first

- **Many lines that mention "no multiply" are not rule statements,** so this unit leaves them. They are dated audit results, or descriptions of how today's kernels multiply without the instruction. Both stay true until the kernels unit (done-when 2) switches the kernels over, and its last bullet updates them then:
  - `lut-serving.md:3, 15, 17, 40, 54, 76, 106`;
  - `serving-and-export.md:31, 37, 54, 78`;
  - `pipeline-map.md:264, 321`;
  - `architecture.md:57` (the counts) and `84`;
  - `mechanisms/README.md:104` (the counts);
  - `README.md:51`;
  - `glossary.md:134` (the counts) and `276`;
  - `what-is-uor-r4.md:76`, `where-we-are.md:33`, `faq.md:31` and `transformer-vs-geometry.md:43`;
  - `products/README.md:34, 35`, `language-model.md:3, 34` and `findings.md:89`;
  - the three figures.
- **Lines left as written:** history and dated rows (`lut-serving.md:21–29, 61, 70, 77–78`), the names of old things (D11's title and anchor, "the zero-mul audit" as a search term on `lut-serving.md:102`), other pages' descriptions (`lut-serving.md:116`), and the GPU note on `pipeline-map.md:220`.
- **`pipeline-map.md` cites notebook rows by line number, and those numbers drift.** Rows go in at the top of `m4-serving.md`, so each new row moves every row below it down by one.
  - It had four references to "line 17" and one to "line 23". None points at the row it meant: line 17 is now claude-164's snap row, and line 23 claude-135's forced-snap control.
  - This unit changed only the reference on the line it edited, to "claude-175's row", which cannot drift.
  - The other four (`pipeline-map.md:64, 205, 208, 216`) are left as found, for a later fix.
- **The heading "The zero-multiply audit"** in `serving-and-export.md` is now "The serving audit". No page linked to its anchor (`git grep the-zero-multiply-audit` finds nothing).

## What changed

- **Rule pages:**
  - `docs/rules/README.md` (§3.6 row);
  - `decision-map.md`: the D11 and I1–I5 rows now say D80 and D81 amend them;
  - `HARD-RULES.md`: the D11 row, and its check column, which now names each audit and what it allows;
  - `project-track.md` (the D11 line).
- **D11's R2** reads "amended by D80 and D81" in `lut-serving.md`, `serving-and-export.md`, `language-model.md` and `glossary.md`.
- **The audit's description** now says the FBS stack's audits allow scalar integer multiply and divide and still refuse float and vector or matrix multiplies (`lut-serving.md:9, 46`, `mechanisms/README.md:104`, `serving-and-export.md` §"The serving audit"). The last says its numbers were measured before that change.
- **Geometry guide** (`docs/geometry/README.md:16, 93`): the multiplier-free table is now described as how today's kernels multiply, not as the rule.
- **`gpu-and-cuda.md:62`, `pipeline-map.md:217, 338, 361` and `architecture.md:57`:** the rule as amended. Each says the kernels do not use the instructions yet.
- **Summaries:** `README.md:85`, `CITATION.cff` (its keyword is now "integer-only inference", and the abstract no longer says "no multiplier"), `what-is-uor-r4.md:53`, `products/README.md:25`, `releases-and-artifacts.md:58` and `desktop-client.md:109`.
- **`docs/integration/DECISIONS.md`:** D11's fate line gains "Amended by D80 and D81 (28 Oct 2026)". It was applied as [`d11-pointer.patch`](d11-pointer.patch) under the owner's permission quoted in the ORDER.

## Measurements

None: this unit changed docs only.

## Checks run at the head

- `cargo xtask links`, `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.
- A re-sweep of the 23 scope pages: every remaining "no multiply" line is in one of the lists above.

## Skills used

- **holding-the-goal.** Each changed line says what D80 and D81 allow and that floats stay banned. None claims the kernels already use the instructions.

## Still open

- **Done-when 2, the kernels:** use the hardware multiply in the products and the snap's square, and the divide in the norm, exact against the old kernels. In the same unit, refresh every line listed under "What did not work" with the new audit counts.
- **Done-when 3 (speed) and 4 (notebook and REPORT per unit).**
- **A later fix:** `pipeline-map.md`'s four drifting references to `m4-serving.md` line numbers.

## Plain summary

The rule pages, guides and summaries now say what Casey's two decisions allow: the engine may use the chip's whole-number multiply and divide, and decimals are still banned. Lines that report today's audit results, or explain how today's engine multiplies without the instruction, are still true, so they change when the engine actually changes.
