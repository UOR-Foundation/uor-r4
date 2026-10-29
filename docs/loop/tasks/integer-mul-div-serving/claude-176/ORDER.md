# ORDER: integer-mul-div-serving, claude-176 (the docs say what D80 and D81 allow)

Lab: claude. Item: [integer-mul-div-serving](../../../../roadmap/items/integer-mul-div-serving.md). Base: `de19e3755`.

## Goal

This unit is the **docs half** of the item's done-when 1; claude-175 merged the audit half. The item's words:

> - Every page listed above that states the rule, summaries included, says integer multiply and divide are allowed and floats are not.

The item also says the D11 entry in `docs/integration/DECISIONS.md` lacks an "amended by D80, D81" pointer, to be written by patch under the owner's permission.

## Which lines change here, and which wait

- **Change now:** every line that states the serving rule, or describes the serving audit, as current: the rule pages, D11's R2 wherever listed, the audit's description, the geometry guide, and the summaries that state the rule.
- **Wait for the kernels unit** (done-when 2's last bullet): dated audit results ("44 native symbol ranges", "7 of 7"), descriptions of today's table-based products, and the three figures. All stay true until the kernels use the multiplier.
- **Left as written:** history, dated rows, and names of old things.

## Cancel probe (run first)

Run at `de19e3755`:
- **The pages still state the old rule:**
  - `docs/rules/README.md:41` says "no float, transcendental, multiplier or integer-divide instruction";
  - `HARD-RULES.md:15` says "no multiplier instruction";
  - `docs/geometry/README.md:16` and `:93` say "no multiplier instruction".
- **CHARTER §3.6 and the D80 and D81 rows** in `DECISIONS.md` already state the new rule. The D11 entry (line 72) has no amendment pointer.
- **No other open PR** touches these pages.

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `docs/rules/README.md`, `docs/rules/decision-map.md`, `docs/contributing/HARD-RULES.md`, `docs/integration/project-track.md`
- `docs/mechanisms/lut-serving.md`, `docs/mechanisms/README.md`
- `docs/model/serving-and-export.md`, `docs/model/pipeline-map.md`, `docs/model/architecture.md`, `docs/model/gpu-and-cuda.md`
- `docs/geometry/README.md`
- `README.md`, `CITATION.cff`, `docs/start/glossary.md`, `docs/start/what-is-uor-r4.md`
- `docs/products/README.md`, `docs/products/language-model.md`, `docs/products/releases-and-artifacts.md`, `docs/products/desktop-client.md`
- `docs/integration/DECISIONS.md`: the D11 entry's pointer only. It is an owner path and outside the item's `scope:` list; see below.
- `docs/loop/tasks/integer-mul-div-serving/claude-176/**`

## Owner approval

Owner path changed: `docs/integration/DECISIONS.md`, one line under D11.

Casey, in Claude's chat by 2026-10-29T03:20:23Z:
- "allow divide, hold on floats for now as you have recommended and fix the exporter";
- his standing permission for this rule change's owner-path edits, from D80: "i give you permissing to fix the stuff you need to to make this work".

Written as for D81: an edited copy under `local/claude/claude-176/`, its difference kept as `d11-pointer.patch` here and applied with `git apply`.

## Done when

1. Each "change now" line above says that served integer code may use the hardware integer multiply and divide since D80 and D81, and that floats stay banned. No line claims the kernels already use them.
2. D11's entry in `DECISIONS.md` points to D80 and D81.
3. `cargo xtask links` and `loop gates --full` pass. A REPORT. No notebook row, since nothing was measured.

## Stop and ask if

Stop and decide, never wait: a line is ambiguous between rule and measurement. It is then left for the kernels unit and named in the REPORT.
