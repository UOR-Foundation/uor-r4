# ORDER: infra-loop-harness, deepseek-75 (record the M7 Studio live/model-sources item)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance; records PR, as in claude-212 and claude-210). Base: 5ec6e36a8. Owner direction, 2026-10-30 18:14Z: fix the AI coding Studio hosted on GitHub Pages as /studio/, make it cleaner, and fix the wasm so it hosts a current model or offers API/other-API options.

## Goal

Record that work as one M7 roadmap item (m7-studio-live-model-sources) and create its issue through roadmap sync --apply. Add the item file, its generated issue mapping and the rendered page. No Studio code is changed in this records PR; the implementation follows in its own unit with an ORDER naming this item.

Owner answers recorded on #2028 this session: (a) ship the bundled synthetic demo plus a local-model loader (file or URL) and OpenAI-compatible API providers with an add-API form; do not publish the private HF artifacts; (b) publish the fixed page live on gh-pages after review. The .github/** workflow stays an owner path; the build/publish helper will live under scripts/studio/.

## Cancel probe (run first)

- No item docs/roadmap/items/m7-studio-live-model-sources.md exists at the base, and no open issue carries its marker.
- The live /studio/ is still the 13 Oct deploy while the source web/studio is wired to uor-r4-integer; the docs-site workflow preserves studio/ and never builds the wasm bundle (read .github/workflows/docs-site.yml).
- No existing M7 item (m7-studio-connect-wasm-engine, m7-studio-self-host-page-assets, m7-parity-library-cli-browser) covers live publish plus model-source/API management.

If any fails, stop and report.

## Files in scope

- docs/roadmap/items/m7-studio-live-model-sources.md (new, hand-written)
- docs/roadmap/issues.json (written only by roadmap sync --apply)
- docs/roadmap/README.md (written only by roadmap render / sync --apply)
- docs/loop/tasks/infra-loop-harness/deepseek-75/**

## Done when

- The item exists and roadmap check reports 115 items OK; render --check clean.
- roadmap sync --apply created exactly one M7 issue and wrote issues.json; the dry run before it showed exactly one create and no unexpected updates.
- No owner path, bar, status, rendered view (other than the tool-rendered roadmap page) or pinned receipt changed; no code changed.
- REPORT records the commands, counts, the sync's GitHub writes and the limits.

## Stop and ask if

A file outside scope is needed; the sync plan shows an unexpected create/update/close; or the item needs a decision row rather than an item file.
