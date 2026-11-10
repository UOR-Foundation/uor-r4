# ORDER: establish the browser-owned Studio item, kimi-1

Lab: kimi. Item: [studio-cursor-hosted](../../../../roadmap/items/studio-cursor-hosted.md); official issue sync follows. Base: `d4f1e3470ed2681847f33028de3258b5053b63f8`. References #3569; never closes it.

## Owner approval and goal

The owner said verbatim: **"I approve of this plan, please implement it end to end autonomously"** (2026-11-09, Kimi chat). Establish the approved new local browser-owned-runtime item and resolve reviewed implementation defects before code. This top-session decision follows the charter, product evidence, #3569, Agent-96's documentation approval and Agent-97's architecture review. No subagent decides acceptance or scope.

## Cancel probe (ran first)

Class: instrument/docs; no model compute. At base, #3569 is OPEN, no Cursor-hosted PR/item exists, and its item matches `origin/main`. Baseline: 11/11 gates GREEN (`local/kimi/cursor-implementation/baseline-gates.log`); pinned `npm ci` passed.

## Scope

Only the new item, official generated roadmap output/metadata, approved spec/plan corrections, and this item's task folder. Later code uses only item-declared paths. No service, API/model/engine/worker/tokenizer, `.github`, rendered STATE/HANDOFF, publication, credentials, real R4 generation, or owner-checkout changes.

## Binding implementation rulings

1. Add `ComposerSurface.tsx` in Task 7. Typed mode has one visible task list and one `AgentRuntime` send path; no parallel legacy `SessionList` or composer send.
2. Give the typed rail independent 1024-aware state. Test `1023_defaults_closed`, `1024_defaults_open`, close/reopen and manual close at 1024; do not change legacy AppShell.
3. Retarget the one skip link to `#harness-main` typed and `#mainColumn` legacy. Scope protected-selector assertions to visible elements under `#studio-typed-root`.
4. Before choosing transport, test concurrent NDJSON-read + POST-intent with flushed frames and real browser headers. If blocked, use `agent_host_poll`; no second listener. Require Host + token; validate Origin when supplied, not assumed on same-origin GET.
5. Native approval is authoritative. No page HMAC key; receipts are opaque host capabilities or asymmetrically verified. Test `page_cannot_approve_its_own_valid_intent` and `receipt_is_opaque_or_asymmetrically_verified`.
6. Resume emits `run.resumed` in a new generation at the next sequence; never repeat start markers. Relax the fixture-only invariant for resume and test continuation without duplicate output.
7. "No tasks yet" is normal; `runtime_unavailable` is initialization failure. Preserve the provider-key exception without copying it to IndexedDB. Keep `run.expired`/`quota_exceeded` out of local UI as NOT_RUN.
8. Test parser duplicate keys or narrow that claim. Define quota refusal/last-verified-snapshot recovery, multi-tab lease races or enforced single-tab refusal, and rejection of float/negative/non-ASCII tool arguments.
9. Each task uses red-first tests, fresh implementer/reviewer, serial writers and declared scope. Replay fixtures are not model evidence.

Ruling: these corrections preserve the approved architecture; if wrong, rework the affected task rather than add a service/model dependency.

## Done when

- `cargo run -p xtask -- roadmap check` passes; `cargo run -p xtask -- roadmap render` produces the official change.
- The item preserves #3569 and excludes hosted service, real R4 generation, useful-model evaluation, production deployment and full-parity claims.
- ORDER/REPORT and repository checks pass; exact-head review and protected merge complete. Official issue sync precedes Task 1 code.

## Stop and re-scope if

A check requires editing #3569 acceptance, an undeclared source, a service/credential/model dependency, hand-edited generated output, or an unsupported capability claim.
