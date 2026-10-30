# ORDER: m7-studio-live-model-sources, deepseek-2 (record the live publish and close the item)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: cdaed4330. Follows ORDER deepseek-1 (PR #3442, merged as cdaed4330).

## Goal

Record the post-merge live publish of the fixed Studio and set the item done per its Done-when 4: the built page, wasm bundle, demo stack and model sources are published to gh-pages and https://uor-foundation.github.io/uor-r4/studio/ serves them; the old 13 October router bundle is gone. This records are the only source change: item status done + Resolution section, docs/products/web-client.md present tense + live evidence, docs/research/notebook/m4-serving.md row evidence, web/studio/README.md live pointer. It closes #3440 (D70).

Evidence at publish: gh-pages bac74f13 -> d8e67c66976f19af86beb0dd1060f05143c56e30 (manual publisher, scripts/studio/build-and-publish.sh --target gh-pages --push --yes from an attached main at origin/main); Pages build "built"; live /studio/ status chip "Pre-alpha · M1 complete · 1 of 8 milestones"; pkg/uor_r4_integer_bg.wasm 200 sha256 d971fe7d…7882d7; pkg/synthetic-stack.lut 200 sha256 d60f88d6…92df; pkg/tokenizer.json 200 sha256 6758318d…c57b; models.json 200; pkg/uor_r4_wasm_router_bg.wasm 404 (old bundle removed); no jsdelivr/cdnjs scripts in the live page.

## Cancel probe (run first)

- Live /studio/ serves the new page and the wasm sha above; gh-pages head is d8e67c66 and its Pages build succeeded (re-check before applying).
- No source-code change is needed; if any of the live evidence has changed, stop and re-verify rather than recording stale numbers.

## Files in scope

- docs/roadmap/items/m7-studio-live-model-sources.md (status done + Resolution)
- docs/products/web-client.md, docs/research/notebook/m4-serving.md, web/studio/README.md
- docs/roadmap/README.md (tool-rendered from the item status change)
- docs/loop/tasks/m7-studio-live-model-sources/deepseek-2/**

## Done when

- The item is status: done with a Resolution section carrying the live URL, gh-pages commit and artifact hashes; docs and notebook say published with the same evidence; no rendered view is hand-edited (roadmap render only).
- roadmap check 115 items OK with done 87/ready 17; render --check current; claim-wording and git diff --check pass; uor-harness loop pr-check --local PASS.
- The issue #3440 is closed by this PR's Closes #3440; the five required checks carry policy: pass (run <id>).

## Stop and ask if

The live evidence no longer matches; a code change would be needed; or any owner path, bar, status beyond this item, or pinned receipt would move.
