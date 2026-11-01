# REPORT: repair Studio design cutover scope and handoff, kimi-2

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent used only for merge-base validation). Delivery item: `studio-typed-shell-upgrade` (#3500). Order: [ORDER.md](ORDER.md). Final text head: this report's commit; exact-head evidence is posted on PR #3509.

## What did not work

- Fresh delivery review at `0e5060513` found that `legacy.html` and existing verifiers were absent from item scope, so the designed cutover could not pass loop scope or retarget old checks.
- The first ORDER did not explain why the completed parent was required for merge-base validation, and the PR body could auto-reference #3440. The first REPORT also stopped at an earlier head and omitted two known verification gaps.
- No lifecycle design decision was rejected: the reviewer accepted the five builder resolutions. No implementation, browser data, dependency, publisher, deployment, or model/engine/worker/tokenizer/API change occurred.

## What changed

Added `web/studio/legacy.html` and `web/studio/verify_*.mjs` to #3500's scope. Defined one explicit retarget contract: `verify_studio_page.mjs`, `verify_studio_page_load.mjs`, `verify_studio_worker.mjs`, `verify_desktop_client.mjs`, and `verify_m7_parity.mjs` default to `index.html` and accept `UOR_STUDIO_PAGE=legacy.html`; runtime page-load maps it to `/studio/legacy.html`. New checks use the built index and no assertion is weakened.

Clarified that the completed parent appears in this PR's ORDER only because #3500 does not yet exist at the merge base. The PR body and public handoff reference only #3500; later implementation orders use #3500 after registration merges. This REPORT replaces the earlier handoff without a stale head and names the exact-head PR evidence and both verification gaps.

Repair content: repair ORDER plus the item/design changes and canonical sync/render. Final policy output reports exact size; no size cap applies.

## Measurements

- Roadmap remains at 126 items, with #3500 ready and #3440 done.
- Scope now contains the local legacy page and all five existing verifiers, while excluding all nine harness shared files.
- `node web/studio/verify_studio_page.mjs --strict-router` is rerun at the final report head; exact output is posted on the PR. It remains a static check, not a new browser or accessibility result.

## Checks run at the head

After this final report commit, run the static page check, full 11 gates, claim wording, roadmap/render/diff checks, and local policy. Post the exact head/tree and outputs on PR #3509 before fresh review and queueing. No npm install, production Vite build, browser run, model load, publisher run, or real-data migration was performed.

## Skills used

Fresh adversarial delivery review, systematic debugging, scope guard, solution integrity, and direct verifier/source inspection. No model compute or paid resource was used.

## Still open

#3500 remains ready for its foundation ORDER after this design merges. The pre-existing R4G1 assertion stays out of scope and must be disclosed rather than counted as green. The real File System Access picker still requires manual/headed verification; only import-files/download-save and a stubbed directory handle are automated. Production publication remains a separate owner decision. Unrelated sync warnings were left untouched.

## Plain summary

The design now names every file and verifier needed for its eventual cutover, explains who owns this registration PR, and carries honest final-head evidence and verification limits. It still implements nothing.
