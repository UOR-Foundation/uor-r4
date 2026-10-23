# ORDER: m7-studio-connect-wasm-engine, kimi-26 (PM status flip + residual item)

Lab: kimi. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (#2868).
Base: `259f9b9b2`.

## Goal

DeepSeek flagged on #2028 (2026-10-23T13:12:54Z) that `m7-studio-connect-wasm-engine` is delivered but still `status: ready`. The item's done-when (M7 criterion 3: Studio runs client-side in a Web Worker with truthful capability reporting and zero remote model calls; the `.wasm` passes `audit_zero_matmul_wasm.py --serving --call-graph --tap` 7/7) is met by the merged and verified PR #2978 (`24f9d8ab6`) with its antigravity-21 repair: `remoteRequests=0`, truthful labels, `7/7 PASS` audit on the built module. This PM unit flips the item to `done` with that evidence, updates its stale "still calls the retired engine" text, and records the one residual the author deliberately scoped out — page-level CDN UI assets and the byte-mod stand-in tokenizer — as a new item so the gap survives this item closing. PM call: the done-when's terms are all about the model worker path and are met; the CDN concern is real but is a separate, unregistered bar (the item text itself says so).

## Cancel probe (run first)

Class docs/PM (host-only). At `259f9b9b2`: the item file reads `status: ready` while its done-when evidence is merged (PR #2978 at `24f9d8ab6`, verified); `grep -c "uor_r4_wasm_router" web/ -r` = 0; the worker imports `pkg/uor_r4_integer.js` and calls `loadStack`. Task stands.

## Files in scope

- `docs/roadmap/items/m7-studio-connect-wasm-engine.md`
- `docs/roadmap/items/m7-studio-self-host-page-assets.md` (new)
- `docs/roadmap/README.md` (re-rendered)
- this task folder

## Done when

- Item reads `status: done` with the #2978 evidence; stale present-tense claims (retired router engine) corrected; the CDN/tokenizer residual named and carried by the new item file.
- `cargo xtask roadmap render` produces the matching `docs/roadmap/README.md` and `cargo xtask roadmap check` passes; `loop report --check`, `loop pr-check --local`, `loop gates --full` pass (the `scope` failure on the roadmap item paths is the known pre-scopes condition and is expected).
- #2868 closes via the post-merge `roadmap sync --apply` (sync closes the issue of a done item and creates the new item's issue); the REPORT carries `Closes #2868`.

## Stop and ask if

- The flip would change a milestone bar or M7's status rather than one item's (it does not; M7 has other open items).
- The owner wants the Pages deploy re-enabled (D36 — explicitly out of scope here).
