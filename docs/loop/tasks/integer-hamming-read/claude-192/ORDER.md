# ORDER: integer-hamming-read, claude-192

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `54d8a3c36`.

## Goal


**This unit (claude-192): a one-link repair, not step 2.** The documentation site has not published since 26 Oct 2026. Every `docs-site` run fails `book-check` on 2 broken links, both from one link in `docs/research/notebook/m4-serving.md` (antigravity-96's row, #3287). That notebook is this item's file. The owner asked on 29 Oct for "all of the documentation on the website updated", and the refresh (#3379) merged without reaching the site. Step 2 of the item is unchanged and is the next unit.

The item's done-when, for reference:
The item's done-when reads:

> 1. The integer runtime runs the whole read as claude-146's REPORT describes it, all four heads of both reads, under D11's rules (CHARTER §3.6: integer only, no float or transcendental instruction; since D80 and D81 the hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them, and until then products go through D11's tables):
>    - **copy heads:** keys and queries from the table's points through the key rotation. These depend only on the token, so their codes can be built once per vocabulary entry;
>    - **type heads:** queries from the residual through the query rotation, and payloads from the residual at `s` through the value rotation. These rotate the state at run time, so each product goes through D11's tables;
>    - the codes packed into 64-bit words; the match by `popcount(q XOR k)`; the 16 nearest earlier positions `1 ≤ s < t`; the weights `max(0, m − d)/m`; the stream read after layer 2 and the move on the readout, each product through the tables.
> 2. **Parity,** against the float forward of the same stored checkpoint (claude-146's language runs, `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`), on DEV. No D11 tolerance exists for this read yet, so the unit's prep sheet pre-registers the bar for two numbers, and the REPORT gives both:
>    - the share of (position, head, read) kept sets equal to float's. One sign flip in an integer rotation changes a code, so this is not assumed to be exact;
>    - top-1 agreement between the integer and float decoded ids.
>    
>    The float reference is computed on the GPU (CHARTER §3.5); the integer side runs on the laptop, as serving does.
> 3. **Speed on this laptop:** `fbs-toks` decode tokens/s against context, with the read and without, on the same artifact.
> 4. One notebook row; the result on main.

## Cancel probe (run first)

Cancel if the site already publishes, or if the broken links come from somewhere else.
- **`docs-site` runs:** none succeeded after 2026-10-26T17:02Z; 200 runs were listed. The last, at `54d8a3c36`, failed `book-check` with 2 broken links, both to `harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.html`.
- **The cause:** row 34 has 155 backticks before the link. That odd count makes `book-stage` treat the link as inside an inline code span, so it is not rewritten to GitHub. mdBook still renders it as a link, to a page the book does not stage. The target `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` exists.

Result: **go.** An instrument fault, in the site's staging, not a mechanism.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `docs/harness/prep/integer-hamming-read*`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-hamming-read.md`

## Done when

1. The one link is absolute (`https://github.com/UOR-Foundation/uor-r4/blob/main/...`), so `book-stage` passes it through. Nothing else in the row changes.
2. `book-stage`, `mdbook build` and `book-check` pass locally: 0 broken links. On `main`, the same check gives 2.
3. After the merge, the `docs-site` run on `main` succeeds.

## Stop and ask if

- **Any other change to the record's text is needed.** Then stop and leave the record as written; make only the link change.
