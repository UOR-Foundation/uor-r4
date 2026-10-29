# REPORT: integer-mul-div-serving, claude-178 (the docs report what the engine does now)

Lab: claude. Order: [ORDER.md](ORDER.md). **The item is done:** done-when 1 (claude-175, claude-176), 2 (claude-177 and this unit) and 3 (claude-177) are met, with a REPORT for each unit.

**Speed.** Not applicable. This unit changed docs only and measured nothing; claude-177 measured 4,829 / 4,827 tok/s.

## What did not work, first

- **Five lines from claude-176 became false when claude-177 merged.** They said "today's kernels do not use them yet": `lut-serving.md:9`, `serving-and-export.md:3`, `architecture.md:57`, `pipeline-map.md:217` and `geometry/README.md:93`. My first sweep missed them because it skipped every line that mentions D80. All five now say the stack's kernels use the instructions since claude-177.
- **The weight dot product still needs no multiply.** The 4-bit weight projection reads a table of the activation's multiples (`stack_activation_tables`), which claude-177 did not change. So `lut-serving.md:38`, `serving-and-export.md:31, 35`, `integer-export.svg`'s step and `lut-serving.svg` panel A stay as written. Only the products of two runtime values moved to the hardware multiply.
- **Left as written:** history and dated rows, the `pipeline-map.md` table of other mechanisms' costs (264–271), and its mechanism-options row (321).

## What changed

- **Lines that said today's engine has no multiply** now say it has no float, with the integer multiply and divide allowed and counted since claude-177:
  - `README.md:51`;
  - `mechanisms/README.md:104`;
  - `glossary.md:134, 276`;
  - `what-is-uor-r4.md:76`, `where-we-are.md:33`, `faq.md:31` and `transformer-vs-geometry.md:43`;
  - `products/README.md:34, 35` and `language-model.md:3, 34`;
  - `findings.md:89`;
  - `lut-serving.md:3, 54`;
  - `architecture.md:84`.
- **The products of two runtime values** are described as one hardware multiply since claude-177. The table form is named as what they used before, and as the tests' reference: `lut-serving.md:40`, `serving-and-export.md:37`, `geometry/README.md:93`.
- **`serving-and-export.md`'s audit section** gains claude-177's counts:
  - native: 44 symbol ranges, with 47 multiplies and 1 divide allowed (65 and 9 in the call graph);
  - WebAssembly: 7 of 7, with `step` at 14 multiplies and 9 divides.

  Its summary row now reads "Native serving audit".
- **Figures:**
  - `lut-serving.svg`: the title is now "an exact whole-number engine", and panel C's counts are dated 18 October, with the figure's description updated to match. Its caption in `lut-serving.md` says panel A shows the weight dot product.
  - `integer-export.svg`: the counter reads "float instructions: 0".
  - `hero-how-it-thinks.svg`: the band reads "add, subtract, multiply, compare, table look-up. No decimals." All three still parse (`xmllint`), and no generator writes them.
- **`docs/roadmap/items/integer-mul-div-serving.md`:** `status: done`; the roadmap README is re-rendered.

## Measurements

None: this unit changed docs only.

## Checks run at the head

- `roadmap check` (109 items OK) and `roadmap render`; `xmllint` on the three figures.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** Every changed line reports the engine as claude-177 left it: no float, the scalar integer multiply and divide in the stack's step, and the weight projection still by table.

## Still open

- **The next serving lever:** the icosian snap's scan, 46% of a token (`fbs-serving-vector-kernel`).
- **The exporter (R6), per Casey's "fix the exporter".**

## Plain summary

The pages that said the engine never multiplies now say what is true since the last change: it uses the chip's whole-number multiply and divide, and still never uses decimals. This closes the work item that carried Casey's two decisions into the engine.
