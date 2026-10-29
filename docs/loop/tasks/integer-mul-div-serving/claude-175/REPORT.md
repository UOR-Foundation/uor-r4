# REPORT: integer-mul-div-serving, claude-175 (the audits allow integer multiply and divide)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays open. This unit is the audit half of done-when 1; claude-176 does the docs half.

**Speed.** Not applicable. This is an instrument change; no served code changed.

## What did not work, first

- **Main's browser build already failed the old audit.**
  - Under main's unchanged `audit_zero_matmul_wasm.py --serving --call-graph`, the WASM serving build at `48435fd53` passed 5 of 7 symbols. `step` and `IntegerStackSession::step` each reach 1 `i64.mul`, in function 50.
  - Neither audit script runs in CI (claude-174 found this), so nobody saw it.
  - D80 now allows it, and the new audit reports it rather than hiding it.
  - **The function is not identified.** The audit's `display_name` expects mangled names, but `wasm-bindgen` writes demangled ones (`uor_r4_integer[7bc29ba…]::…`), so it prints `inte::bc29ba8`. Naming it needs a WASM disassembler, and none is installed.
  - A guess, not checked: a multiply by a constant that is not a power of two. LLVM keeps it as `i64.mul` on WASM but lowers it to shift-and-add on AArch64, where the native audit passes.
- **Two builder subagents wrote nothing.**
  - The loop guard makes a subagent read-only unless it has a suborder in `local/claude/claude-175/orders/`. claude-157 hit the same thing.
  - Both builders also started near 115k tokens of context.
  - The Rust builder had already changed `report_output.rs` through a shell edit before the guard stopped it, leaving a partial change that did not compile, uncommitted, in its own worktree.
  - I took that diff as groundwork, finished both halves in the top session, and removed both builder worktrees.
- **`clippy -D warnings` already fails on `uor-r4-integer` at the base,** with rustc 1.97.1, in files this unit does not touch:
  - with default features, five files: `discrete_d11.rs` (11), `m6_exact_arithmetic.rs` (2), and `geometric_potential.rs`, `geometric_span.rs` and `lawful_likelihood.rs` (1 each);
  - with `serving-wasm`, 19 library files, among them `codec.rs` (9), `geometric_source_realizer.rs` (8), `model.rs` (5), `stack/kernels.rs` (2) and `stack/flock.rs` (2).

  None is in this diff, and the three changed Rust files have 0 warnings under either feature set. Not fixed here: out of scope.
- **The native script recognises only AArch64 mnemonics,** before and after this change. On an x86-64 build it would miss scalar multiplies and divides (they pass uncounted), and it would not detect x86-64 vector and float forms either (`pmulld`, `mulsd`).
  - So x86-64 builds are outside its audited scope. Today's audited builds are arm64, and its call-graph pass runs `otool`.
  - The Rust verifier does cover x86-64.
  - This is a gap in the old instrument, not one this change opens.
- **The 21 legacy WASM symbols are NOT_FOUND in the serving build,** so a whole-module run exits 1 as before, and the legacy rule is exercised only by the self-test's legacy case.
- **The public name `verify_step_objdump_zero_mul_zero_float` stays.** Its doc comment now says the name predates D80 and D81.

## What changed

- **`report_output.rs` (the Rust verifier, used only by the FBS stack):**
  - `is_forbidden_instruction` now passes the scalar AArch64 `mul`, `madd`, `msub`, `mneg`, `smull`, `umull`, `smaddl`, `umaddl`, `smsubl`, `umsubl`, `smnegl`, `umnegl`, `smulh`, `umulh`, `sdiv` and `udiv`, but only with no arrangement suffix and a general-register destination (new `is_general_register_destination`);
  - it also passes the x86-64 general-register multiply and divide families (new `is_x86_general_register_operands` for a bare `mul`) and calls to the compiler-rt integer helpers;
  - vector, dot-product and matrix multiplies, every float instruction (`fmov` included), float helpers and indirect calls still fail;
  - doc comments and the error text are updated.
- **`tests/m5_full_path_cost_controls.rs`:**
  - Three probes planted a scalar `mul` or `udiv` under the old rule. They now plant a vector `mul.4s` or an `fadd`, so each still checks what it checked: kernel detection, callee reachability, the alloc callee.
  - Six lines the old list rejected (`smnegl`, `divq`, the three `mulx` forms, `bl <__multi3>`) move to the new test's allowed list.
  - The new test `stack_disassembly_allows_scalar_integer_mul_div` covers both sets, including a compiler-rt divide body.
- **`scripts/audit_zero_matmul_serving.py`:**
  - In `--stack` mode, scalar integer multiply and divide lines are taken out of Class I and II, in both the symbol ranges and the call graph, and their counts are printed (text and TAP).
  - The other modes are unchanged.
  - New `--self-test`, with stack scalar tests that also assert the non-stack modes still refuse a scalar `mul`.
- **`scripts/audit_zero_matmul_wasm.py`:**
  - `SERVING_WASM_SYMBOLS` (the stack's browser serving surface) fail only on float, unclassified (SIMD included) or decode findings. They report their multiply and divide counts, and the result records `integer_mul_div_allowed`.
  - Legacy symbols keep the zero-multiply rule.
  - `--self-test` gains three cases: allowed and passing, allowed with a float and failing, and a legacy symbol failing.
- **`bin/uor-r4-stack.rs`:** one comment.
- **Notebook:** one row in `docs/research/notebook/m4-serving.md`.

## Measurements

Run on this M1 at code `aa3f59deb` (base `48435fd53`). Sealed to `caseyallard/uor-r4-store:results/claude/integer-mul-div-audits-2026-10-29/`; `SHA256SUMS` was read back OK.

| Check | Result |
|---|---|
| Native release `uor-r4-stack` (`d12e793b0f32e1195ac1626b93da1bac1128f0964a2da34d9737812e88a6a922`), new `--stack` audit | FULL PASS: 44 symbol ranges, 93 call-graph functions, 0 / 0 scalar multiplies / divides allowed |
| `uor-r4-stack`'s own objdump check (m5 suite, `serving-wasm`) | passes: 120 symbols, 17,268 instructions |
| WASM serving build (`ed17bb735e4393f0e960bdc0cb90a012f22efc699cc2e84939f77f9c1fb0274e`), main's audit | 5 of 7 (`step` and `IntegerStackSession::step` FAIL on 1 `i64.mul`) |
| The same build, new audit | 7 of 7 PASS, the `i64.mul` reported |
| New m5 test with the verifier change set aside | fails on `mul x8, x9, x10` |
| Native `--self-test` with the allowance disabled | fails: "'mul x8, x9, x10' is not a scalar mul" |
| WASM `--self-test` with the old status rule | fails: "D80/D81 allowance: FAIL mul=1 div=1" |

## Checks run at the head

- `cargo test -p uor-r4-integer --test m5_full_path_cost_controls --no-fail-fast`, with and without `serving-wasm`: 2 and 4 passed, 0 failed.
- `cargo fmt --check`: clean. Clippy on the three changed files: 0 warnings (see above for the base's own errors).
- Both `--self-test`s: PASS. `python3 -m py_compile` on both scripts: clean.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The change allows exactly D80 and D81's scalar products and quotients. Floats, `fmov`, vector forms and dense matrix multiplication (§3.3) stay refused.

## Still open

- **claude-176, the docs half of done-when 1:** the rule pages, summaries and figures listed in the item. It also adds the D11 "amended by D80, D81" pointer in `docs/integration/DECISIONS.md`, by patch under the owner's permission.
- **Done-when 2 (the kernels) and 3 (speed).**
- **Candidates for later items, not done here:** a CI job that runs both `--self-test`s, and teaching the WASM audit's name display to read demangled names.

## Plain summary

Casey allowed the laptop engine to use the chip's own multiply and divide. The three safety checks that inspect the compiled engine still forbade them, so they now allow plain one-number-at-a-time multiplies and divides, and still forbid decimals and the vector forms. While checking, the browser version of today's engine turned out to already contain one multiply that the old check would have rejected. Nobody noticed, because no automatic job runs that check.
