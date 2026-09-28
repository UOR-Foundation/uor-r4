# Release-bundle manifest: native width-576 dialogue model (unsealed)

**Date**: September 28, 2026
**Owning Lab**: Lab 3 (Anti-Gravity); corrected by Lab 1 per the owner's direction (PR #1452 review)
**Track**: T3 (Mission runtime and measured efficiency)
**Issues Referenced**: #965, #963, #1172, #1173, #964, #820
**Evidence Artifact**: `docs/evidence/release-bundle-manifest-dialogue576-2026-09-28.json`

---

## 1. Status

`scripts/package_release_bundle_integer.py` regenerated the manifest (schema `uor-r4.release-bundle-manifest/2`). It records only what the packager read or ran:

- the eleven bundle files;
- the model shape from `bundle.json`;
- the two frozen binaries and their instruction audits as run;
- the recorded full-path cost report.

It is **not sealed** and is **not a qualification**. The numerical contract is recorded as a declaration (D11), not as a certification.

The earlier version of this record is withdrawn in five respects:

- It presented unmeasured performance figures (67.1 ms cold load, 3.16 ms per step, a 23.22 MB REPL RSS). The packager produced them as literal defaults, because it read keys the harness never writes.
- It described the WebAssembly module as a serving runtime with a 20/20 PASS audit.
- Its digest table listed nine SHA-256 prefixes that match no file.
- It described SHA-256 values labelled `blake3` as BLAKE3 digests.
- It asserted cache fit from an analytic byte count.

## 2. Bundle files

Root: `/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1`. It holds 11 files, 4,356,152 bytes in total. Digests are SHA-256; the manifest adds BLAKE3 only when the optional `blake3` Python module is installed, and it was not.

| Path | Bytes | SHA-256 |
|---|---:|---|
| `attempt.json` | 810 | `7e9646c21f8ce7789bfa343dab376cf53347e4e7bcfa8475acbc52210ab4c72f` |
| `bundle.json` | 6,512 | `55b3fa54a424e46681bfde5cd257ff0da51e0f560fef550fa6d84d07f698562a` |
| `manifest.json` | 1,749 | `7da97234e9d6cc2708318741a362e70d8bb6c2900f47c34d2a4d9e00b6802c3e` |
| `model/hard-model.json` | 326,250 | `eb975d2b0b6349852af8841fd5fc10f9336a1ea39023e09e618658ff94204682` |
| `model/hard-parameters.bin` | 2,725,956 | `245a6fd1b5cbbbaa16ca3e6dbfa2837909216a0a4044ab9cc8edd9957a05510b` |
| `model/hard-parameters.json` | 135,189 | `8d58183f1e335d2420af6467f86fef26bf269afaa16b815a1d178d4421d4ac88` |
| `tables/attempt.json` | 401 | `bdd27be28c7a329e750100c958faee4975e224c0b06a9cb2e13c772b93bad2ba` |
| `tables/manifest.json` | 621 | `bc3742071a1501adc7223d21740bf2bde91345946e911ae6b71b4993627a4955` |
| `tables/tables.bin` | 1,048,560 | `993e63edc97a4c988b586bc52b5e9fcf133e675caceb7ed04727e7248bb30ff2` |
| `tables/tables.json` | 647 | `2d16e17cd34c5a07681a7950a22c4b9a67df5208b28304ab8b7fbe94461c150d` |
| `tokenizer.json` | 109,457 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

Model shape from `bundle.json`: width 576, vocabulary 4,096, context 256, dot read geometry.

## 3. Binaries and instruction audits

### A. Native `uor-chat` (the serving binary)

- **Path**: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor-chat`
- **SHA-256**: `4a4253a174d5af324fde647324f7830210bcb7bb0f6d5f2bdd94dcafe0c68aa4`; 1,465,168 bytes.
- **Build commit, compiler and flags**: unrecorded, and passed to the auditor as `UNRECORDED`.
  - The folder is named after `698bdda`, and the file was written 21 seconds after that commit. The working-tree state is unknown.
  - The binary predates this PR's source changes.
- **Audit**: `python3 scripts/audit_zero_matmul_serving.py <uor-chat> --strict-arm64 --tap`, which exits 0 with TAP 5/5 ok.
  - It checked 55 matched symbol ranges and 113 call-graph functions, with 0 violations.
  - It lists 13 declared symbols as not audited (no separate symbol; possibly inlined or absent).

### B. WebAssembly `uor_r4_integer.wasm` (a helper module, not a serving runtime)

- **Path**: same folder; **SHA-256** `a41c652795e65821bc59381c77d10339e0c9edd3f4aa068b2c7e79b608f1b3ac`; 1,664,885 bytes.
- **Build commit**: unrecorded. The folder name predates `wasm.rs`.
- **Exports**: no load, session or step entry point.
- **Audit**: `python3 scripts/audit_zero_matmul_wasm.py <module> --tap --call-graph`, which exits 1 with 14 of 21 symbols clean. The model step and the Hopf helpers' callees contain `i64.mul`.
- See the [capability API and WebAssembly record](capability-api-wasm-runtime-2026-09-28.md).

The packager stops without writing a manifest if the native audit fails. It records the WebAssembly audit without gating on it, because that module is not the serving binary.

## 4. Recorded cost run

The manifest reads `docs/evidence/full-path-m1-cost-dialogue576-2026-09-28.json` through its `metrics` and `parity_gate` keys, with no defaults. That report comes from one single-thread run under unknown machine load, with no sealed report root. It is not a qualification, and it does not meet the declared ceilings:

| Metric | Recorded | Declared ceiling | Meets |
|---|---:|---:|:---:|
| Cold load | 1,530.49 ms | ≤ 250 ms | no |
| Step latency, mean | 7.315 ms | ≤ 4 ms | no |
| Step latency, p90 | 13.237 ms | ≤ 4 ms | no |
| Step latency, p50 | 5.170 ms | — | — |
| Throughput | 130.6 tok/s | — | — |
| Prompt ingestion | 7.788 ms/token | — | — |
| Tokenizer encode | 43.93 µs/token | — | — |
| Session save / restore | 0.165 / 0.322 ms | — | — |
| Peak process RSS | 22.36 MB | — | — |
| Replay parity | 38 requests, 58 turns, 1,433 decisions, 0 departures | exact | yes |

- The live `uor-chat` REPL RSS was never recorded, so it is `UNAVAILABLE`.
- Bytes touched per token (1,821,872) is an analytic count, not a measurement, and it omits the recurrent, read-value and update weight reads.
- Energy is `UNAVAILABLE`.
- Details: [full-path cost record](full-path-m1-cost-dialogue576-2026-09-28.md).

## 5. Regenerating and checking

```bash
# Manifest (build identities default to UNRECORDED; pass them when known)
python3 scripts/package_release_bundle_integer.py [--arm64-build-commit SHA] \
  [--arm64-compiler TEXT] [--arm64-flags TEXT] [--wasm-build-commit SHA] [--m1-cost REPORT]

# Instruction audits
python3 scripts/audit_zero_matmul_serving.py <uor-chat> --strict-arm64 --tap
python3 scripts/audit_zero_matmul_wasm.py <module> --tap --call-graph

# Capability API tests (synthetic bundle; the fixture tests need --ignored)
cargo test -p uor-r4-integer --release --test capability_api_tests

# Full-path cost harness into a new sealed report root
UOR_R4_M1_COST_REPORT_ROOT=/new/attempt/dir UOR_R4_SOURCE_COMMIT=<sha> \
  cargo test -p uor-r4-integer --release --test full_path_m1_cost -- --ignored --nocapture
```
