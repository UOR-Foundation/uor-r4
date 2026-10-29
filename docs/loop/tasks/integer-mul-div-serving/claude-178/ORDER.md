# ORDER: integer-mul-div-serving, claude-178

Lab: claude. Item: [integer-mul-div-serving](../../../../roadmap/items/integer-mul-div-serving.md). Base: `e0bbacc43`.

## Goal

The item's done-when reads:

> 1. **Docs and audits agree with §3.6.**
>    - Every page listed above that states the rule, summaries included, says integer multiply and divide are allowed and floats are not.
>    - The native and WASM audits allow integer multiply- and divide-class instructions, and keep refusing every float instruction.
>    - The native audit keeps refusing `fmov` for now. Float and vector values share one register file, so it cannot tell an integer move from a float one. `fbs-serving-vector-kernel` may revisit this.
>    - Host tests show a float instruction still fails each verifier, and an integer `mul` or `udiv` now passes. The Rust verifiers are tested in `tests/m5_full_path_cost_controls.rs`, and each audit script in its own `--self-test`: the WASM audit already has one, and the native audit gains one.
> 2. **The kernels use them, exactly.**
>    - `stack_mul_u64`, `stack_mul_u128`, `mul_i128` and `stack_hamilton`'s products use the hardware multiply.
>    - The snap's `ℤ[φ]` comparison squares by the hardware multiply.
>    - The norm's long division uses the hardware divide.
>    - Each new kernel equals the old one by exhaustive or random host tests, and every reply on claude-126's export is byte-identical.
>    - The summaries and figures that report a zero-multiply audit are updated with the new audit result in the same unit.
> 3. **Speed:** `fbs-toks --parts` on this M1, against claude-169's sealed binary (sha256 `8a7a83c7…805a`), run alternately. Decode tok/s, the parts, and a `sample` profile with its total read from the thread line.
> 4. One notebook row and a REPORT for each unit. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## Cancel probe (run first)

To fill in.

## Files in scope

- `scripts/audit_zero_matmul_serving.py`
- `scripts/audit_zero_matmul_wasm.py`
- `crates/uor-r4-integer/src/report_output.rs`
- `crates/uor-r4-integer/src/bin/uor-r4-stack.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`
- `crates/uor-r4-integer/tests/capability_api_tests.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/rules/README.md`
- `docs/rules/decision-map.md`
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/README.md`
- `docs/model/serving-and-export.md`
- `docs/model/pipeline-map.md`
- `docs/model/architecture.md`
- `docs/model/gpu-and-cuda.md`
- `README.md`
- `CITATION.cff`
- `docs/start/faq.md`
- `docs/contributing/HARD-RULES.md`
- `docs/integration/project-track.md`
- `docs/geometry/README.md`
- `docs/start/glossary.md`
- `docs/start/what-is-uor-r4.md`
- `docs/start/where-we-are.md`
- `docs/start/transformer-vs-geometry.md`
- `docs/products/README.md`
- `docs/products/language-model.md`
- `docs/products/releases-and-artifacts.md`
- `docs/products/desktop-client.md`
- `docs/research/findings.md`
- `docs/figures/mechanisms/lut-serving.svg`
- `docs/figures/model/integer-export.svg`
- `docs/figures/start/hero-how-it-thinks.svg`
- `docs/loop/tasks/integer-mul-div-serving/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-mul-div-serving.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
