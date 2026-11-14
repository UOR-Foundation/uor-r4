# uor-r4-mvm

RV64IMAC Linux-userland emulator for the UOR Studio workbench. This is a software emulator with an in-memory AF_VSOCK policy boundary, not a hardware hypervisor, model runtime, or host network transport.

## Upstream provenance

Vendored from `Casey-allard/lexlean-r4` commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` under MIT OR Apache-2.0. The upstream license texts are preserved in `LICENSE-MIT` and `LICENSE-APACHE`; generated-code comments identify `R4.SpiralCoreVsock` (`src/R4/SpiralCoreVsock.lex.tex`).

| Input | Upstream identity |
|---|---|
| `crates/wasm/src/mvm.rs` | 98,525 bytes; SHA-256 `2789ec57daa7f92dbd094e5d2c38a8ae343b4f38bd09cd2ed6bc377fa7d67206` |
| `crates/wasm/src/mvm_cc.rs` | 75,765 bytes; SHA-256 `19390475442bbede4332bf38776706d42aa09e2daa9df4e014d140c6af5ce12f` |
| `crates/wasm/src/lexlean_generated.rs` | 2,206,963 bytes; SHA-256 `acc9a89d65bb43bdb61a720fd90570dacf46a43adb9340fcbc427a9245e247e8` |
| `lexlean_generated.rs:45,751-45,938` only | 188-line `r4_spiralcore_vsock` module; SHA-256 `102c1c859413c4d4a1470d8fecb4dcc3dfa8fa0ea50c402468f22718ad0b478f` |
| `crates/wasm/src/mvm_initramfs.bin` | 12,164 bytes; SHA-256 `071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c` |
| `crates/wasm/tests/mvm_spiralcore_sandbox.rs` | 9,573 bytes; SHA-256 `9b99e2d64f32475decc5c10db658677a4b9b211f8d6963e3668fbb471a653c7b` |

The module ends at line 45,938. `r4_compositional_gllm` begins at line 45,940 and is excluded, as are every other generated-file line, LexLean `api.rs`, `lib.rs`, `wasm_ast_eval.rs`, `execute_cargo_json`, the generated application/model and AST-evaluated Cargo behavior.

## UOR modifications

- `mvm_cc.rs` is byte-identical to upstream. `spiralcore_vsock.rs` contains only the exact 188-line module above.
- `mvm.rs` has three declared adaptations: `crate::lexlean_generated::r4_spiralcore_vsock` becomes `crate::spiralcore_vsock::r4_spiralcore_vsock`; `include_bytes!("mvm_initramfs.bin")` becomes `crate::mvm_initramfs::MVM_INITRAMFS`; and `include!("mvm_persistence.rs")` adds complete CoW export/import. The provenance test reverses only those strings and reproduces the upstream hash.
- `mvm_initramfs.rs` is generated from the pinned binary into a Rust byte slice, so no compiled initramfs binary is committed. Its bytes and SHA-256 are tested.
- Vendored modules are `#[rustfmt::skip]` so package formatting cannot change upstream hashes; new tests and adapter code remain rustfmt-checked.
- `lib.rs` exposes the native engine/compiler and safe module re-exports. No page-callable host-secret, proxy-egress or raw-CoW import API is exposed by this native crate's Studio worker boundary.
- `mvm_persistence.rs` preserves inode modes/kinds, directories, sectors, allocation counters and process counters. WASM snapshots serialize maps as plain objects and CoW goes directly from the worker to IndexedDB; only snapshot IDs/digests cross the JSON protocol.
- Cargo/rustc are not emulated by an AST evaluator. Without a genuine guest Rust toolchain they must report unavailable. C compilation and execution must emit genuine `EM_RISCV` output and exact nontrivial results.

Native and browser-WASM acceptance are verified separately; this provenance record alone is not a capability claim.
