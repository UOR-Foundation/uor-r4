# Studio (`web/studio`) — the browser client

**Live:** https://uor-foundation.github.io/uor-r4/studio/ — published 30 October 2026 from gh-pages `d8e67c66` (wasm sha256 `d971fe7d…`, demo sha256 `d60f88d6…`; see the item's Resolution).

The Studio is the project's in-browser interface: editor, chat panel and model manager, with the
integer serving engine (`crates/uor-r4-integer`, `--features serving-wasm`) running in a Web
Worker. It is served from the project's documentation site at **`/studio/`**
(<https://uor-foundation.github.io/uor-r4/studio/>), live since 30 October 2026 from gh-pages `d8e67c66`.

This directory holds the page source. **The wasm bundle, the demo model and the tokenizer are
generated, never committed** (CHARTER §3.13 forbids committed binaries and corpora); the generated
files live in the gitignored `pkg/` and are produced by the build below or by
`scripts/studio/build-and-publish.sh`.

## What is published and what is dev-only

Published (copied by `scripts/stage-studio-site.sh`):

- `index.html`, `index.css`, `geometric_prime_router_webapp.html`, `coi-serviceworker.js`;
- everything under `assets/` (JS shims, tokenizer artifact, CSS, fonts);
- `pkg/` — the wasm-pack output plus `synthetic-stack.lut` and `tokenizer.json`;
- `models.json` — generated at publish time by the staging helper (see below).

Dev-only (never published): `tools/`, `verify_*.mjs`, `*.sh` and this README.

## Build the bundle and run the checks locally

```sh
export PATH="$HOME/.cargo/bin:$PATH"          # the PATH cargo may be a non-Rust shim
# 1. wasm engine (wasm-pack 0.15.0 used locally; the proposed CI workflow pins 0.13.1; scalar table kernel, no wasm SIMD)
RUSTFLAGS='-C target-feature=-simd128' wasm-pack build crates/uor-r4-integer \
  --target web --no-opt --out-dir ../../web/studio/pkg -- --locked --features serving-wasm
# 2. deterministic synthetic demo stack (seed 5, schema uor-r4.lut-stack/1) + its tokenizer
~/.cargo/bin/cargo run -q --locked -p uor-r4-integer --features serving-wasm \
  --bin mk-synthetic-stack -- 5 web/studio/pkg/synthetic-stack.lut
node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut \
  --out web/studio/pkg/tokenizer.json
node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut \
  --check web/studio/pkg/tokenizer.json
# 3. the checks the publisher runs at the same tree
node web/studio/verify_studio_worker.mjs
node web/studio/verify_studio_page.mjs --strict-router
node web/studio/verify_studio_page_load.mjs --strict-router   # needs local Chrome/Chromium
python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm \
  --serving --call-graph --tap
# 4. serve it (from the repository root)
ruby -run -ehttpd web/studio -p 8000            # open http://localhost:8000/
```

The page-load harnesses enforce the offline bar: **no external HTTP request at page load**
(same-origin requests only). The runtime trap fails closed when no Chrome/Chromium is available;
the publisher skips it with a warning only when no browser exists on the machine.

## Publishing (`scripts/studio/build-and-publish.sh`)

**How the docs site works today.** `.github/workflows/docs-site.yml` builds the mdBook and
replaces the root of the target branch, but it deliberately keeps a directory named `studio/`:

```sh
find site -mindepth 1 -maxdepth 1 ! -name .git ! -name studio -exec rm -rf {} +
```

So a built Studio published once to `gh-pages` / `gh-pages-preview` **persists across every later
book deploy**. The 30 October 2026 app now live at `/studio/` was published from gh-pages
`d8e67c66`, with the integer wasm bundle, synthetic demo and model-source panel verified by HTTP and
hash checks. Publishing a newer page means replacing that directory once.

**The publisher.** `scripts/studio/build-and-publish.sh` does the whole unit:

1. builds the wasm engine, the seed-5 synthetic demo stack and its stack-bound tokenizer into the
   gitignored `web/studio/pkg/`;
2. runs the worker, static page-load, runtime page-load and wasm-instruction checks;
3. stages the runtime files with `scripts/stage-studio-site.sh` (replacing `studio/` wholesale so
   no file from the retired app survives) and writes `models.json`;
4. makes a publish commit in a throwaway git worktree of `origin/gh-pages` or
   `origin/gh-pages-preview`, checks the GitHub Pages size limits, prints the staged tree, the
   previous head and the rollback line;
5. pushes only when `--push --yes` is given. **The default is a dry run, and `--push` without
   `--yes` refuses.** Live (`--target gh-pages --push --yes`) is refused unless the checkout is
   on an attached `main` branch whose HEAD is `origin/main`.

```sh
# dry run against the preview branch: builds, stages, commits locally, pushes nothing
scripts/studio/build-and-publish.sh --target gh-pages-preview --stage-dir /tmp/studio-staged

# after review, publish the live site from a clean, attached main checkout (owner/Lead only)
scripts/studio/build-and-publish.sh --target gh-pages --push --yes
```

**Why not in CI yet.** Making the Studio build part of the workflow needs
`.github/workflows/docs-site.yml` changed, and `.github/**` is an **owner-only path**
(`docs/harness/owner-paths.json`): a lab PR touching it fails the `owner-paths` gate. The Lead's
report carries the proposed workflow diff (add the wasm/demo/verify build and a
`studio/` replacement to the existing publish step, plus path triggers for `web/**` and
`crates/uor-r4-integer/**`) for the owner to land. Until then this script is the manual publisher;
the proposed workflow reuses the same `stage-studio-site.sh` helper, so the staged output cannot
drift from the script.

### `models.json` (schema `uor-r4.studio-models/1`)

Generated by `scripts/stage-studio-site.sh`; one entry per model source:

```json
{
  "schema": "uor-r4.studio-models/1",
  "sources": {
    "bundled": true,
    "local_file": true,
    "url": {"available": false, "note": "set public_model_url at dispatch to publish a public model URL slot"},
    "openai_compatible_api": {"configured_in_browser": true, "storage_key": "uor.studio.modelSources.v1"}
  },
  "models": [
    {"id": "synthetic-seed-5", "kind": "bundled", "label": "...", "url": "pkg/synthetic-stack.lut",
     "tokenizer_url": "pkg/tokenizer.json", "format": "UORLUT01", "sha256": "...", "size_bytes": 0},
    {"id": "public-model", "kind": "url", "label": "...", "url": "https://..."}
  ]
}
```

The model decision is: ship the bundled synthetic demo, a **local-file** loader, a **URL** loader,
and an **OpenAI-compatible API** manager with add-API, all in the Studio page; **no private HF
artifact is published**. The manifest's bundled entry keeps the page usable with no network, the
optional `url` entry (only when a publish supplies `--public-model-url`) is how a *public* hosted
model can be offered without a code change, and API configuration (base URL / model / key) lives in
`localStorage` under `uor.studio.modelSources.v1`, never in this manifest.

Interface note for the model-source layer: `assets/js/model_sources.js` currently hardcodes the
demo at `./pkg/synthetic-stack.lut` (exactly the file this publisher ships) and does not yet read
`models.json`. The 10-line consumption hook is: fetch `./models.json` when the model-sources panel
first opens (not at page load), map `models[]` entries onto the bundled/URL choices, and fall back
to the current hardcoded demo if the fetch fails. The page works either way, so the generated file
can land before that hook.

### Limits, rollback, and why no artifact is committed

- GitHub Pages limits: **100 MB per file** and **1 GB per published site**; the publisher enforces
  both and prints the file count and sizes, and the proposed workflow does the same.
- **Rollback:** the publisher prints the previous target head and an exact
  `git push --force-with-lease origin <previous-sha>:<branch>` line. Force-pushing a build branch
  to the known-good commit is the rollback; nothing is deleted.
- Build outputs are generated locally/runner-side and land only on the build branch. Nothing is
  committed to `main`: no wasm, no `.lut`, no tokenizer, no manifest.
- The wasm/synthetic demo prove the engine and the page end to end. They are a **synthetic
  stand-in**, not a trained model; the page's engine-status labels say so.
