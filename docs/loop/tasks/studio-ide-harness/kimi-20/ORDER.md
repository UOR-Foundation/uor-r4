# ORDER: register the LexLean MVM workbench item, kimi-20

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569, registration parent only). New item: `studio-lexlean-mvm-workbench`. Base: `93e089ff256654e75a19641b7e2fbd2bc8e24524`. References #3569; never closes it or changes its acceptance.

## Owner approval and goal

The owner asked for LexLean's design, a fully working MVM terminal, GitHub project work through branch/edit/commit/PR, and desktop matching web. He said: **"Please use your best judgement when making decisions."** Register the design, plan and item; the top session decides acceptance and scope, never a subagent.

## Cancel probe (ran first)

`cargo run -q --locked -p xtask -- roadmap check` passed with 144 items. No `crates/uor-r4-mvm` exists at the base, no open PR overlaps Studio/Tauri/MVM paths, and #3569 is OPEN. A new item cannot anchor its own registration ORDER at the merge base, so use the established parent route. Missing implementation is work, not cancellation.

## Scope decision and files

Top-session D76 review accepts only the new-item/design/plan advisory outside the parent's scope, not an implementation waiver or parent acceptance change. Touch only the new item, its dated design/plan, this task folder and valid official roadmap output. Later code uses the merged item's ORDER. No model/API/engine/tokenizer changes, services, `.github`, rendered STATE/HANDOFF, credentials, deployment or owner-checkout changes.

## Binding implementation rulings

1. Reuse `AgentStore`/`AgentTaskStore` as the only task authority. MVM owns only bounded per-tab sandbox files, virtual Git, memory-only history and CoW state; it is not a model or task runtime.
2. Pin LexLean commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` and exact source hashes. Extract only the complete 337-line vsock module, never the generated application/model or AST Cargo evaluator. Encode initramfs as hash-pinned Rust source bytes; commit no binary.
3. `cc` must pass upstream exact-output native/browser tests and a second C program. Cargo/rustc without a real guest toolchain reports unavailable. Tasks 1–2 are a decisive preflight before UI work.
4. Give `WorkspaceEditor` an explicit MVM file source and every virtual file a revision. Preserve serialized path/saveId saves and refuse stale editor/terminal writes.
5. Use numeric persistence/protocol caps, transactional digest read-back and explicit corrupt/quota/version refusal. CoW images go directly from worker to IndexedDB; JSON carries only snapshot ID/digest. Guard each reload key with a page-lifetime Web Locks lease so duplicated tabs refuse or rotate rather than share writes. Persist no raw command history or tokens; refuse secret-like terminal input. Export no page-callable secret/proxy/raw-CoW API.
6. Keep real GitHub tokens out of MVM guest RAM and never register them with the sandbox. Secret tests use synthetic values. Pin `RepoTarget`; use base ref/blob SHA compare-and-swap, single-flight separately approved branch/edit/commit/PR actions, and refuse cross-repo/default-branch/unapproved writes.
7. Desktop reads/writes use only `AgentHost`-approved fixed `gh api` calls under unchanged CSP or report unavailable; Tauri token entry/direct fetch is disabled. Bind writes to the selected existing task/run without creating synthetic AgentStore records. Direct static-web token writes remain UNVERIFIED unless separately exercised live.
8. Default tests never write GitHub. Live smoke requires `UOR_GITHUB_SMOKE=1` and an exact dedicated non-production repository allowlist; record exact owner/repo and provisioning authority in `smoke-target.md`, never provision through the product, keep its PR open and do not claim model-driven bug fixing, production deployment or full parity.
9. Preserve protected selectors, mode-correct skip link, exact 1023/1024 behavior, keyboard/focus, CJK/RTL/IME and 390/1024/1440 acceptance. Generate MVM under ignored `node_modules/.cache/uor-r4-mvm/` and let Vite hash the `new URL(..., import.meta.url)` input into local `assets/*.wasm`; desktop acceptance requires real status/cc execution, not mounting alone.
10. Do not hand-edit generated roadmap output. Use red-first tests, serial writers and independent design/implementation/exact-head reviews.

## Done when

- The design, seven-task plan, item and this registration record pass roadmap validation and `git diff --check`; independent design findings are resolved or explicitly ruled.
- ORDER/REPORT, loop policy/full gates and required GitHub checks pass; fresh exact-head review and protected merge complete.
- After merge, official `roadmap sync --only studio-lexlean-mvm-workbench --apply` creates the issue, official render output is delivered through its separate path, and the item/issue identity is verified. Only then does implementation create its fresh ORDER/worktree.

## Stop and re-scope if

A check requires editing #3569 acceptance, undeclared source, a second runtime, credential persistence, a committed binary, a default-branch write, broadened desktop CSP, hand-edited generated output, or an unsupported capability claim.
