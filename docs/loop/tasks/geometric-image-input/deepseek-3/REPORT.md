# REPORT: geometric-image-input, deepseek-3

Lab: deepseek. Item:
[geometric-image-input](../../../../roadmap/items/geometric-image-input.md)
(#3619). Order: [ORDER.md](ORDER.md). Base:
`55432f5f88e6c8abd349df88d4944c19bb0c7d23`.

## What did not work, first

- The first one-pixel PNG fixture was not decodable by Chrome. The browser test
  now uses a generated two-pixel RGBA PNG and fails on decode instead of
  treating an invalid fixture as an application fault.
- Direct CDP file assignment did not fire the page controller in the installed
  browser. The real-browser test now constructs a real browser `File`, assigns
  the picker's files and dispatches the same change event a picker emits.
- Making the whole composer send path asynchronous broke the existing typed
  generation adapter. Image preparation is now a deferred content promise that
  only API streaming resolves; ordinary text and local sends remain synchronous.
- Strict Clippy over `uor-r4-integer` still exits 101 on 51 pre-existing
  diagnostics. None names a changed file.

## What changed

Added `image_input.js`: raster type checks, 8 MiB per-image and 16 MiB total
bounds, exact-dimensional Canvas decode/downsample to at most 256 by 256, exact
RGBA extraction and a bounded preview. Studio attachments now distinguish text
from images, show image thumbnails/name/dimensions/size/removal, and retain only
text in localStorage. Session metadata asks for reattachment after reload; no
image bytes are restored.

Native Send resolves each image only after the explicit action, calls the
authenticated decomposition route, and attaches a canonical `uor_r4_image`.
Custom providers receive the bounded preview as ordinary `image_url`. Image-only
Send is supported for API providers, while local WASM explains that it ignores
image context. The required label
`R4 patch input - experimental; not visual understanding` appears in the
composer and context inspector.

The server now renders exact signed Q12 R4 bytes as compact base64 patch data
instead of verbose decimal lines. One 1 by 1 patch fits the 64-position
development context, while four images overflow it and return 413.

## Measurements

- 17 focused Node image/API/composer tests passed.
- 266 Studio tests passed across 31 files.
- Native loopback: 14 passed. Integer library: 451 passed, 1 ignored.
- Vision crate: 9 tests passed in the default parallel build.
- Real Chrome 157 against native `uor-chat`: exact metrics were
  **34 prompt / 6 completion / 40 total tokens**, 477.6 tok/s, TTFT 10.792 ms,
  12.562 ms elapsed, stop. Selecting the image produced no generation. The
  label matched exactly, reload retained no image bytes and showed the
  reattachment notice, reattachment restored the preview, and removal cleared
  it.
- Static page checks, zero-external runtime trap, WASM worker checks and desktop
  client checks passed. `npm run typecheck` and `npm run build` passed.

## Checks run at the head

- The evidence files under [evidence/](evidence/) retain the Node, Studio,
  native, WASM, page, desktop and browser outputs plus their SHA-256 manifest.
- `rustfmt --check` on changed Rust files, `node --check` on changed JavaScript,
  and `git diff --check`: passed.
- Strict Clippy on `uor-r4-vision`: passed. Integer Clippy has the 51
  pre-existing diagnostics reproduced on base and no changed-file diagnostic.
- `uor-harness loop gates --full`: all 11 gates passed on the first attempt.

## Skills used

No available Codex skill applied. L-019 was considered; this unit validates and
prepares browser input and adds no CPU model fallback.

## Still open

Visual understanding, semantic image answers, useful multimodal conversation,
M4 and M7 remain open. The image route makes R4 data available as experimental
context; it does not train the model to interpret that context.

## Plain summary

The Studio can now paste, drop or select an image, show it, remove it, and send
its deterministic R4 decomposition through the native local chat path only on
explicit Send. The browser keeps image bytes in memory, custom providers receive
ordinary image URLs, and the complete native/WASM/browser path is verified.
