# DESIGN: evidence-bounded LexLean reuse

## Intent

Make the Studio more useful without mislabeling a heuristic dispatcher or an incompatible artifact as a coherent default model. Preserve the existing UOR-R4 research model and all current client safeguards.

## Findings that cancel the default swap

- The public LexLean page loads its 188,728,472-byte pack and computes `17 + 25 = 42` through an exact-arithmetic path. The same live session answers `Explain in two sentences what you are.` with an unrelated Hunger Games review and `What number did I just ask you to compute?` with Fibonacci code.
- The strong arithmetic/code replies come from deterministic tools/templates before the LUT model. The model is not a transformer, but its deployed `/8` header, memory key length and sigmoid-table semantics differ from the current UOR engine and charter.
- There is no hosted OpenAI-compatible inference endpoint to select as an external API baseline. Importing the WASM would add a second runtime and its heuristic corpus; converting only weights would not reproduce the observed composer behavior.

## Approaches

1. **Full deployed WASM as default:** rejected at this scope. Incompatible artifact semantics, second runtime, unresolved weights/corpus provenance, table-form squashing and unreliable ordinary chat.
2. **Model-only conversion:** rejected as a quick port. The header and memory layout differ and the observed usefulness is not model-only. A future export/retrain needs its own item and matched acceptance evidence.
3. **External OpenAI baseline:** not currently available; the static page has no inference API. It would remain non-default even if a user-hosted server appears.
4. **Repair and shared exact tool (selected):** fix the real silent loader failure, keep the synthetic default honest, and expose the existing UOR geometric arithmetic engine through a small shared Rust function in the browser worker and Tauri host.

## Implementation

- Add a strict bounded parser for one decimal `+` or `-` expression with only recognized optional instruction wording. Return `None` for ordinary prompts and an explicit error for recognized invalid/underflow requests.
- Format the numeric answer first, then `R4 geometric exact arithmetic — tool, not model generation` and the actual `PlaceStepRecord` trace. Reuse `GeometricArithmeticEngine`; do not copy LexLean's solver or probability labels.
- Export the function to WASM. The worker handles it before model state/tokenization and emits one text result with zero model completion tokens; ordinary generation remains on the existing path.
- Add a Tauri RPC command with the same shared function. `desktop_openai.js` tries it before starting a sidecar request and returns a normal JSON fallback envelope with zero model usage. The sidecar remains unchanged for all other prompts.
- Keep the exact-tool exchange visible locally but remove its user prompt and assistant result from subsequent OpenAI context. A real native probe first proved that removing only the answer still left an 81-position turn; the paired-filter regression failed before the repair and now leaves only the ordinary follow-up.
- Convert worker filename/expected-hash mismatches into an explicit error. Add a no-storage/default regression; `primary` remains display metadata, not an automatic selector.

## Verification and remaining limits

Prove parser/tool negatives and positives first, then the actual WASM worker, Tauri command/transport, real browser and packaged app. Compare arithmetic and ordinary prompts side by side; confirm no fabricated token usage and no special-token sanitization. Full existing suites and hosted Pages checks are required. This adds a deterministic tool, not a conversational model, visual understanding, a new milestone, or a qualified trained bundle.
