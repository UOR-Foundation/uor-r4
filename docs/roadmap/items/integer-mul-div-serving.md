---
id: integer-mul-div-serving
title: Serve with the hardware integer multiply and divide (D80, D81), exactly
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [fbs-serving-kernels]
done_class: host
notebook: docs/research/notebook/m4-serving.md
scope: [scripts/audit_zero_matmul_serving.py, scripts/audit_zero_matmul_wasm.py, crates/uor-r4-integer/src/report_output.rs, crates/uor-r4-integer/src/bin/uor-r4-stack.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs, crates/uor-r4-integer/tests/capability_api_tests.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/tests.rs, docs/rules/README.md, docs/rules/decision-map.md, docs/mechanisms/lut-serving.md, docs/mechanisms/README.md, docs/model/serving-and-export.md, docs/model/pipeline-map.md, docs/model/architecture.md, docs/model/gpu-and-cuda.md, README.md, CITATION.cff, docs/start/faq.md, docs/contributing/HARD-RULES.md, docs/integration/project-track.md, docs/geometry/README.md, docs/start/glossary.md, docs/start/what-is-uor-r4.md, docs/start/where-we-are.md, docs/start/transformer-vs-geometry.md, docs/products/README.md, docs/products/language-model.md, docs/products/releases-and-artifacts.md, docs/products/desktop-client.md, docs/research/findings.md, docs/figures/mechanisms/lut-serving.svg, docs/figures/model/integer-export.svg, docs/figures/start/hero-how-it-thinks.svg, docs/loop/tasks/integer-mul-div-serving/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/integer-mul-div-serving.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "sounds like reverse d11 is the answer" (he chose "Allow integer multiply (Recommended)"), then "allow divide, hold on floats for now as you have recommended and fix the exporter".
- **The rules:** D80 and D81 amend CHARTER §3.6. The served artifact has no float or transcendental instruction, but may use the hardware integer multiply and divide. Integer products and quotients are exact, so serving stays deterministic.
- **What the old rule cost:** six exact units took the served FBS token on the M1 from 270 to 3,042–3,044 tok/s ([`fbs-serving-kernels`](fbs-serving-kernels.md)). The remaining cost is mostly exact products done without a multiplier.
  - Each `stack_mul_u64` or `stack_mul_u128` builds a table of sixteen multiples, then reads it once per radix-16 digit.
  - `stack_hamilton` holds 21% of a token in claude-169's profile.
  - The icosian snap's scan holds 32%.
  - The RMS norm runs a restoring long division and a digit-by-digit square root. `stack_rms_norm_fine` holds 43 of the profile's 940 samples (about 5%).

## What is missing

- **The audits still refuse what the charter now allows.**
  - `scripts/audit_zero_matmul_serving.py --stack` refuses multiply- and divide-class instructions.
  - `report_output.rs`' disassembly verifiers (`verify_step_disassembly_text`, `verify_step_objdump_zero_mul_zero_float`) refuse `mul`, `sdiv`, `udiv` and the `__udivti3` builtin. The `uor-r4-stack` cost check and `tests/m5_full_path_cost_controls.rs` use them.
  - `scripts/audit_zero_matmul_wasm.py` counts `i64.mul` and `i64.div_u` in the browser build of the same step (`stack::IntegerStackSession::step`, `stack_argmax`) and expects zero. The browser build is served too, so D80 and D81 apply to it.
  - All of them must keep refusing every float instruction.
- **The docs still state the old rule as current.**
  - **Rule pages:** `docs/rules/README.md` (§3.6's plain-language row), `docs/rules/decision-map.md` (D11 and I1–I5), `docs/contributing/HARD-RULES.md` (the D11 row) and `docs/integration/project-track.md` (its D11 line).
  - **Mechanism and model pages:**
    - `docs/mechanisms/lut-serving.md` (R2) and its figure `docs/figures/mechanisms/lut-serving.svg`;
    - `docs/mechanisms/README.md` (the D11 row: "An audit proves no multiply instruction");
    - `docs/model/serving-and-export.md`, `pipeline-map.md` and `architecture.md`, and the figure `docs/figures/model/integer-export.svg`;
    - `docs/model/gpu-and-cuda.md` ("the serving contract, where no multiply instruction may appear at all");
    - `docs/geometry/README.md` (lines 16 and 93; line 93 prescribes the sixteen-multiples table).
  - **Summaries:** `README.md` and `CITATION.cff` (its keyword and abstract); `docs/start/glossary.md`, `what-is-uor-r4.md`, `where-we-are.md`, `faq.md` and `transformer-vs-geometry.md`; the hero figure `docs/figures/start/hero-how-it-thinks.svg`; `docs/products/README.md`, `language-model.md`, `releases-and-artifacts.md` and `desktop-client.md`; and `docs/research/findings.md`. Some state the rule. Others report a zero-multiply audit as the current result, which stops being true once the kernels use the multiplier.
- **Left out on purpose:**
  - **Other engines and their checks:** R4G1 and TLA (`docs/transformerless/**`, `docs/inference_contract.md`, `docs/minimal_client.md`); the VSA chat binary's `scripts/serving_multiplier_check.py`; the WASM Hopf engine (`wasm_serve.rs`); and the zero-multiply checks in `uor-r4-core`, `uor-r4-graph-certify`, `uor-r4-wasm-router` and `uor-r4-api`.
  - **Descriptions of code that still has no multiply:** the comments in `stack/chat.rs`, `stack/flock.rs` and `stack/simd.rs`, and the geometry figures drawn by `docs/figures/geometry/generate.py`.
  - **History and dated records:** `docs/research/history.md`, the research lines, `docs/integration/current-state.md`, done items, `docs/labs/**`, `docs/loop/tasks/**` and `archive/**`.

  The D11 entry in `docs/integration/DECISIONS.md` also lacks an "amended by D80, D81" pointer. That file is an owner path, so the pointer is written by patch under the owner's permission for this change, as D80 and D81 were.
- **The kernels still use the multiplier-free forms.**

## Done when

1. **Docs and audits agree with §3.6.**
   - Every page listed above that states the rule, summaries included, says integer multiply and divide are allowed and floats are not.
   - The native and WASM audits allow integer multiply- and divide-class instructions, and keep refusing every float instruction.
   - The native audit keeps refusing `fmov` for now. Float and vector values share one register file, so it cannot tell an integer move from a float one. `fbs-serving-vector-kernel` may revisit this.
   - Host tests show a float instruction still fails each verifier, and an integer `mul` or `udiv` now passes. The Rust verifiers are tested in `tests/m5_full_path_cost_controls.rs`, and each audit script in its own `--self-test`: the WASM audit already has one, and the native audit gains one.
2. **The kernels use them, exactly.**
   - `stack_mul_u64`, `stack_mul_u128`, `mul_i128` and `stack_hamilton`'s products use the hardware multiply.
   - The snap's `ℤ[φ]` comparison squares by the hardware multiply.
   - The norm's long division uses the hardware divide.
   - Each new kernel equals the old one by exhaustive or random host tests, and every reply on claude-126's export is byte-identical.
   - The summaries and figures that report a zero-multiply audit are updated with the new audit result in the same unit.
3. **Speed:** `fbs-toks --parts` on this M1, against claude-169's sealed binary (sha256 `8a7a83c7…805a`), run alternately. Decode tok/s, the parts, and a `sample` profile with its total read from the thread line.
4. One notebook row and a REPORT for each unit. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## How to start

Read:
- CHARTER §3.6 and the D80 and D81 rows in `docs/integration/DECISIONS.md`;
- [claude-172](../../loop/tasks/infra-loop-harness/claude-172/REPORT.md)'s REPORT, which lists the doc lines;
- `report_output.rs` around `verify_step_disassembly_text`;
- the forbidden patterns in `scripts/audit_zero_matmul_serving.py`, and `CLASS_I_OPS`, the division opcodes and the stack symbols in `scripts/audit_zero_matmul_wasm.py`;
- the `multiples_u128` / `mul_by_u64` family, `stack_hamilton`, `snap_difference_positive` and `stack_rms_norm_fine` in `crates/uor-r4-integer/src/stack/kernels.rs`.

The baseline binary and artifact are claude-169's (see `fbs-serving-vector-kernel`'s How to start).

## Rules that apply

- **Serving stays integer** (CHARTER §3.6): no float or transcendental instruction, including in vector form.
- **No dense matrix multiplication** (§3.3). The hardware multiply forms single exact products, never a matrix product.
- **Exact or nothing:** a faster kernel that changes one token, cell or state is a defect.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above); D80 and D81.
- [`fbs-serving-kernels`](fbs-serving-kernels.md), claude-169's REPORT and claude-172's REPORT.
