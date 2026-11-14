# ORDER: implement the LexLean MVM workbench, kimi-2

Lab: kimi. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939). Base: `be21fa82347cee867e0ed9503a3a5023837f8c5f`. References #3939 until every acceptance row is met; then Closes #3939 and sets item `done` in the same PR.

## Owner goal

The owner requested LexLean's three-pane UI, real MVM, virtual files/Git/build/cc/vsock, approved GitHub PR workflow and desktop parity: **"Please use your best judgement when making decisions."** Preserve the browser task runtime and ModelPort boundary. Fixtures are not model capability.

## Cancel probe and provenance correction (ran first)

#3939 is OPEN and no competing PR overlaps these paths. Direct source reading proves `r4_spiralcore_vsock` is `lexlean_generated.rs:45,751-45,938`; `r4_compositional_gllm` begins at `45,940`. The registered 337-line/46,087 range is a documentation defect. Correct item/design/plan to the actual 188-line module and selectively sync #3939 before product code. Never import GLLM lines or edit historical registration records.

Cancel only if native/browser exact-output MVM cannot pass without canned behavior, or the complete item already exists. Missing adapters/tests are work.

## Scope

Only declared item paths: root Cargo manifests; `crates/uor-r4-mvm/**`; named Studio/Tauri source/build/tests; this task folder; item, design/plan, FILEMAP and named product/current-state docs. Generated views use their separate official path. No model/API/engine/tokenizer, services, `.github`, Tauri CSP, owner checkout or another lab's work.

## Binding contracts

1. `AgentStore`/`AgentTaskStore` alone own tasks. MVM owns only bounded per-tab files, virtual Git, memory-only history and CoW. Replay is synthetic.
2. Pin approved commit/hashes. Import only `mvm.rs`, `mvm_cc.rs`, initramfs and `lexlean_generated.rs:45,751-45,938`, preserving licenses/notices. Initramfs becomes Rust source, never a committed binary. No new external package.
3. Tasks 1–2 precede UI: exact native/browser C output, real RV64 ELF, files/CoW/vsock and invalid-input refusal. Cargo/rustc without a real guest toolchain report unavailable; no AST Cargo success.
4. Enforce numeric caps. CoW goes directly worker→IndexedDB; JSON carries snapshot ID/digest. Web Locks prevents duplicated-tab writes. Persist no raw history/tokens; refuse secret-like input and page secret/proxy/raw-CoW APIs. Browser vsock is in-memory only.
5. `WorkspaceEditor` gets an MVM file source and per-file revisions. Preserve selectors/skip link, path/saveId saves, 1023/1024, keyboard/focus/CJK/RTL/IME and 390/1024/1440 behavior. Desktop must execute status/cc.
6. Pin `RepoTarget`; use base ref/blob SHA CAS and single-flight separate branch/edit/commit/PR approvals. Refuse cross-repo/default-branch/unapproved writes. Tokens are memory-only and never registered with MVM. Desktop uses `AgentHost`-approved fixed `gh api` or unavailable, bound to an existing task/run.
7. Default tests never write GitHub. Smoke needs `UOR_GITHUB_SMOKE=1`, an exact non-production allowlist and committed evidence. Drive static web through loopback `gh`, test desktop approval separately and leave the PR open. Direct token writes remain UNVERIFIED unless exercised.

## Execution and done checks

Use red-first tests, serial scoped commits, a ledger and actual counts/logs. Correct provenance and sync #3939 before the crate; Tasks 1–2 must pass before UI. Finish native/WASM/browser/desktop acceptance, virtual Git/editor consistency, approved smoke, product docs, full gates/policy, exact-head review, protected merge and fresh-main verification. Report every NOT_RUN/UNVERIFIED row. Clean only reproducible inactive artifacts after preserving evidence.

## Stop and re-scope if

A requirement needs a second runtime, new external package, generated-model import, canned build/PR success, persisted credential, default-branch/production write, broader CSP, committed binary, undeclared source or unsupported claim.
