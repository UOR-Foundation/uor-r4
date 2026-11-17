# Public M1 seed-1 browser prediction

Lab: codex. References #3440 and the [ORDER](ORDER.md).

## What failed on production

On 17 November 2026, the production catalog SHA-256 was `fa33aa230b37b1d788983f9adb910f35d4562679db5351a5cf32e758d0c5ed79` and listed only the synthetic fixture. Chrome's URL loader downloaded the correct M1 artifact but omitted its tokenizer. The legacy predictor refused the text request. The public typed workbench's Send action created a completed synthetic task with no prediction transcript.

## What the staged client did

1. Downloaded the previously released M1 seed-1 model only after the browser operator chose **Models → Load M1 next-token predictor (seed 1)**. The public revision was `1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55`. The model was 86,536,132 bytes, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`. Its tokenizer was 265,393 bytes, SHA-256 `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`. Both releases were also independently downloaded anonymously and hashed. [Immutable release](https://huggingface.co/caseyallard/uor-r4-m1-model/tree/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55).
2. Selected **Predict next tokens** after disabling Chrome's network. The prompt “The capital of France is” encoded to `[413,2743,3695,372,3704,1651,435]`. The visible ordered result was `[223,952,2359,2471,23,263,964,1520]`. The native Rust `IntegerDiscreteModel::evaluate_prefix` on the same public model and IDs returned the same ordering. The top candidate decoded to a space. Chrome observed zero off-origin requests before the load click. Only Hugging Face and its CDN received GETs during model loading.
3. The M1-specific publishing selector takes no operator-provided model path, URL, digest or schema. It checks pinned records for this exact release, including the seed-1 GPU score (14,287/14,287 rows agreed and B4 passed) and the separate seed-1 CPU/trunk parity record (616 rows, 19,712 query coordinates, zero top-eight disagreements). The generic FBS path still requires its served-forward receipt. The scope is a historical D11 next-token predictor. This unit measured no training, new panel, native-state milestone or chat capability.

## Changes after adversarial review

The initial selector emitted the correct model pins but lacked a discrete served-forward receipt. The original browser smoke accepted unrelated requests during the download. The parent approved the [bounded schema-specific release requirement and fixture scope](DECISION.md).

On an Apple M1 with macOS 27.2.1, the read-only `CpuDiscreteTrunkReader::decide` and `IntegerDiscreteModel::evaluate_prefix` each read the exact public bytes and seven IDs above. The [receipt](../../../../../scripts/studio/public-m1-served-forward.json) retains both raw decisions, including 32 signed Q12 coordinates, eight ordered cells, eight ranked token IDs and top-1. SHA-256 of the prefix followed by the native query, cells, candidates and top-1, each encoded as a 32-bit little-endian word, is `a009e0cbdc2a485583803ba9c9cc92e00810a2497c9413fc4a40057331e4cbbe`. The validator compares both raw records, checks that digest, matches the measured browser candidates, binds the historical panel’s manifest/token/mask digests, and pins all three historical evidence files. The served receipt SHA-256 is `ff370aefd6ba8062e2316c5478a8d428b8e220f8c5da2d47f7b0eeed24bfe479`. Mutations of either model identity, workload, output order, or both supposedly agreeing query words fail. The browser smoke checks each external method, HTTPS origin and original URL of every redirect chain, and downloads and verifies the same-origin receipt.

## Reproduction and handoff

The typed publisher builds the runtime catalog from the pinned selector by default. After review the staged catalog SHA-256 is `fae4cc14401055e511edfc76b64cf7f03622ff29e9acff6cc6613a872282c19f` and includes a same-origin served-forward receipt. The earlier publisher dry run staged 17 files. The revised candidate adds the receipt as its eighteenth file. The generated serving WASM SHA-256 was `a62f959dcbc6062b7bc147830749e1a1b05e350e9e6bdb4509e409d9949b91c3`. Its serving-call audit passed seven checks. The static page scan, Chrome load traps, synthetic worker, all 1,003 typed tests, TypeScript checks, 40 model-source tests and three catalog tests, 16 served-forward tests and the current harness native-space rule passed. The release-gate test fixture now supplies the runtime allowlist files its positive case needs.

```sh
scripts/studio/build-and-publish.sh --typed --target gh-pages-preview --stage-dir <empty-directory>
# Serve the staged directory with a local HTTP server, then run:
node web/studio/verify_public_m1.mjs http://127.0.0.1:<port>/
```

The parent session owns production publication from freshly merged main. Its separate live check uses `node web/studio/verify_public_m1.mjs https://uor-foundation.github.io/uor-r4/studio/` after Pages serves the candidate.
