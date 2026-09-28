# Capability API and WebAssembly helper module

**Date**: September 28, 2026
**Owning Lab**: Lab 3 (Anti-Gravity); corrected by Lab 1 per the owner's direction (PR #1452 review)
**Track**: T3 (Mission runtime and measured efficiency)
**Issues Referenced**: #1172, #1173, #963, #964, #820
**Evidence Artifact**: `docs/evidence/capability-api-wasm-audit-2026-09-28.json`

---

## 1. Summary

1. **Capability API (`IntegerCapabilityApi`).** `crates/uor-r4-integer/src/capability_api.rs`, schema `uor-r4.integer-capability-api/1`. It is a lifetime-free session container over one loaded bundle. It provides metadata, a scoped capability status matrix, session creation with an optional persistent system prompt, user-turn ingestion, token generation and session save/restore.
   - It is safe Rust under the crate's `#![forbid(unsafe_code)]`.
   - The metadata **declares** the D11 numerical contract and states that the API does not certify it. Qualification requires the ARM64 instruction audit of the delivered release binary and a measured, sealed report. The API no longer returns the literal "D11 Certified" and "M1 Qualified" statements or the hard-coded `zero_*` flags.
2. **WebAssembly module (`uor_r4_integer.wasm`).** A frozen `wasm32-unknown-unknown` build with 13 `uor_wasm_*` helper exports. It exports no load, session or step entry point, so it is **not a serving runtime**. It is **not D11-clean**: the model step and the Hopf helpers' callees contain `i64.mul`.
3. **Audit tool (`scripts/audit_zero_matmul_wasm.py`).** The original 20/20 PASS covered 20 function bodies (the 13 exported helpers and 7 internal kernels) **without following calls**, with a decoder defect. It is withdrawn. The corrected tool and its re-audit are described in sections 3 and 4.

## 2. Capability API behaviour

- **Restore.** `restore_session` keeps the saved sampler state, sampling policy and read mode. It previously reset them to seed 0, greedy and enabled.
- **Empty prompt.** `ingest_user_turn` rejects an empty prompt before it changes the session. Previously a rejected call had already advanced the turn counter.
- **Turn end.** `generate_text` returns an error when committing a sampled turn-end or EOS token fails. Previously that error was discarded.
- **Tests.** `crates/uor-r4-integer/tests/capability_api_tests.rs` runs three tests on the synthetic byte-vocabulary bundle on every run:
  - the metadata declares the contract without certifying it;
  - a restored session keeps its sampler, policy and read mode and continues with identical text and saved state;
  - an empty prompt leaves the saved session byte-identical.
- **Fixture tests.** Two width-576 tests need the local `dialogue-child-bundle-1` fixture. They are ignored by default and fail when run without it.
  - The lifecycle test covers metadata, session creation, ingestion, generation, save and restore. It passes: the restored session keeps its sampler, policy and read mode, and it saves identically.
  - The continuation test is a **known failure**, and its ignore reason says so. `uor-r4.integer-session/1` does not serialize the width-576 value stores (`persistent_values_576`, `dialogue_values_576`, `l2_pages_576`), and `ChatSession::from_serialized` rebuilds them empty. A restored width-576 session therefore reads empty values and diverges at its first step.
  - This gap is in the existing session schema, not in the capability API. It affects every width-576 restore, including `uor-chat`'s. Closing it needs a schema change and a decision about existing width-576 saves.

## 3. The frozen module

| Field | Value |
|---|---|
| Path | `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor_r4_integer.wasm` |
| SHA-256 | `a41c652795e65821bc59381c77d10339e0c9edd3f4aa068b2c7e79b608f1b3ac` |
| Size | 1,664,885 bytes (an earlier record said 1,684,344) |
| Build commit | **unrecorded**. The folder name `698bdda` predates `wasm.rs`, which was first committed in `a8d59d31`; the file was written 86 seconds before that commit. |
| Compiler | rustc 1.97.1, from the module's `producers` section |
| Target features | bulk-memory, bulk-memory-opt, call-indirect-overlong, multivalue, mutable-globals, nontrapping-fptoint, reference-types, sign-ext. There is no `simd128`, although the earlier record listed `+simd128` among the flags. |
| Exports | 2,142 function exports through `--export-all`, including the 13 helpers |

The helpers are not `#[no_mangle]`, because the crate forbids unsafe code, so a module exposes them only through `--export-all`. The crate no longer declares a `cdylib` crate type. A module build must request one explicitly, for example:

```bash
cargo rustc -p uor-r4-integer --lib --release --target wasm32-unknown-unknown \
  --crate-type cdylib -- -C link-arg=--export-all
```

## 4. Instruction re-audit of the frozen module

The corrected auditor decodes every instruction of each audited function. It counts Class I (integer multiply), Class II (integer divide/remainder) and Class III (floating point, including the 0xFC 0..7 saturating truncations). It reports every other 0xFC-prefixed instruction, and every SIMD (0xFD) and atomic (0xFE) instruction, as an *unclassified* finding, which fails the symbol. It takes the build commit as an argument and does not read the checked-out HEAD. The audited symbols are the original 20 plus `IntegerModel::step_conversational_into`, the model step, which no helper reaches.

| Scope | Clean symbols | Findings |
|---|---:|---|
| Root bodies only (`--tap`) | 19 of 21 | The model step: 38 × `i64.mul`, plus `memory.copy`/`memory.fill`. `uor_wasm_score_token_salience_default_key`: one `memory.fill`, which the original decoder mis-sized. |
| Direct calls followed (`--tap --call-graph`) | 14 of 21 | The Hopf X/Y/Z helpers: 12 × `i64.mul` each, 8 in `UnitS3Q30::hopf_project` and 4 in `UnitS3Q30::from_i32_coords`. `project_vocab`: 5 × `i64.mul`. `project_vocab_with_products`: 1. The model step: 56 in total. |

Each `i64.mul` in `hopf_project` and `from_i32_coords` is preceded by `i64.const 3`. This is the `(u << 1) + u` step of `mul_i32_radix4` compiled to a multiply by 3. The `core::hint::black_box` added to that loop does not prevent it, so the earlier statement that it kept the Hopf helpers multiplier-free is withdrawn.

The call-graph scope excludes the native auditor's allowlist (allocation, formatting, panic, I/O, tokenizer and loading code), imported functions and indirect call targets. A clean result is a scoped static measurement of named symbols in one artifact, not a serving qualification.

## 5. Pages Studio (#1173)

The module cannot serve Pages Studio as it stands, because it has no load, session or step export. Earlier text said the module was ready to link and that ingestion, generation and state serialization run in it. That text is withdrawn. A browser serving path would need:

- exported session entry points;
- a build that records its commit and flags;
- an instruction audit that follows calls and finds no Class I–III or unclassified instruction in the step path.
