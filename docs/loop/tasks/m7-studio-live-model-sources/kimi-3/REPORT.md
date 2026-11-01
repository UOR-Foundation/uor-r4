# REPORT: align Studio design final-head records, kimi-3

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent used only for merge-base validation). Delivery item: `studio-typed-shell-upgrade` (#3500). Order: [ORDER.md](ORDER.md). Final text head: this report's commit; exact-head evidence is posted on PR #3509.

## What did not work

- Fresh review at `7c371d729` accepted the design/scope but found the active REPORT still said 126 items after merge-forward, used future-tense checks, and left the design's older base unqualified. The active record is corrected to the 134-item merged state and actual outcomes below.
- The first record-repair commit added `kimi-3/ORDER.md` before its required REPORT existed, so local policy refused it as missing. This REPORT completes that required record; it does not hide the failed check.
- No lifecycle or cutover design decision was rejected. No implementation, browser data, dependency, publisher, deployment, or model/engine/worker/tokenizer/API change occurred.

## What changed

Added `web/studio/legacy.html` and `web/studio/verify_*.mjs` to #3500's scope. Defined the retarget contract: all five verifiers default to `index.html` and accept `UOR_STUDIO_PAGE=legacy.html`; runtime page-load maps it to `/studio/legacy.html`. New checks use the built index and no assertion is weakened.

Clarified that the completed parent appears in this PR's ORDER only because #3500 does not yet exist at the merge base. The PR body and public handoff reference only #3500; later implementation orders use #3500 after registration merges. The design now says reviewed at `381ae4d98` and merge-forwarded to `03800ffd2`, with Studio source unchanged. This REPORT records the 134-item roadmap and exact-head checks without a stale self-hash.

Final exact-head policy size: docs 8 files +433/-0, generated 1 file +5/-2, other 1 file +1/-0. Total: 10 files, 439 insertions and 2 deletions. No size cap applies.

## Measurements

- Roadmap has 134 items: ready 28, held 10, decision 1, done 95. #3500 is ready once, #3440 remains done, and all eight unrelated CPU registrations are retained.
- Scope contains the local legacy page and all five existing verifiers while excluding all nine harness shared files.
- `git diff 381ae4d98..03800ffd2 -- web/studio` is empty; merge-forward did not change the reviewed implementation.
- `node web/studio/verify_studio_page.mjs --strict-router`: PASS, both strict pages clean, no failures. This is a static check, not a new browser or accessibility result.

## Checks run at the head

- Full `uor-harness loop gates --full`: 11/11 GREEN at the final text head; exact ledger path and Git tree are posted on PR #3509.
- `loop pr-check --local`: PASS. Only explicitly reviewed new item/design/canonical-map scope advisories, the rebase-created pre-ORDER design commit, and known FILEMAP advisories remain. Exact size is above.
- Claim wording: PASS. Roadmap check: 134 items OK. Roadmap render: current. `git diff --check`: PASS.
- No npm install, production Vite build, browser run, model load, publisher run, or real-data migration was performed.

## Skills used

Fresh adversarial delivery review, systematic debugging, scope guard, solution integrity, and direct verifier/source inspection. No model compute or paid resource was used.

## Still open

#3500 remains ready for its foundation ORDER after this design merges. The pre-existing R4G1 assertion stays out of scope and must be disclosed rather than counted as green. The real File System Access picker still requires manual/headed verification; only import-files/download-save and a stubbed directory handle are automated. Production publication remains a separate owner decision. Unrelated sync warnings were left untouched.

## Plain summary

The design names every file and verifier needed for its eventual cutover, explains who owns this registration PR, and carries the actual 134-item roadmap and final-head check evidence. It still implements nothing.
