# ORDER: integer-mul-div-serving, claude-178 (the docs report what the engine does now)

Lab: claude. Item: [integer-mul-div-serving](../../../../roadmap/items/integer-mul-div-serving.md). Base: `e0bbacc43`.

## Goal

The last bullet of done-when 2: "The summaries and figures that report a zero-multiply audit are updated with the new audit result in the same unit". claude-177 changed the kernels and left these lines for this short unit.

Since claude-177, the FBS stack's step uses the hardware integer multiply and divide.
- Native `--stack`: 47 multiplies and 1 divide allowed in 44 symbol ranges, no float.
- WASM: `step` at 14 multiplies and 9 divides, 7 of 7 PASS.

So lines that say today's engine has "no multiply", and descriptions of products read from a sixteen-multiples table, are now false. Dated records stay dated.

## Cancel probe (run first)

Run at `e0bbacc43`:
- claude-177 is merged, and its notebook row holds the new counts.
- The lines claude-176 listed still state the zero-multiply results as current.

Both held. **Not cancelled.**

## Files in scope

The pages claude-176's REPORT lists under "What did not work": `lut-serving.md`, `serving-and-export.md`, `pipeline-map.md`, `architecture.md`, `mechanisms/README.md`, `README.md`, `glossary.md`, `what-is-uor-r4.md`, `where-we-are.md`, `faq.md`, `transformer-vs-geometry.md`, `products/README.md`, `language-model.md`, `findings.md`, and the three figures. Also `docs/roadmap/items/integer-mul-div-serving.md` (the status) and this task folder.

## Done when

1. Each listed line states the current result, or is dated as a past one.
2. The item's done-when 1–3 are met, so `status: done` and the README is re-rendered.
3. `loop gates --full`, a fresh review and a REPORT.

## Stop and ask if

Stop and decide, never wait: a line's meaning is unclear. Date it rather than guess.
