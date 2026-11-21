# Frozen order-free reader with native Relation state

The fixed reader was written by free2 on GPU. This diagnostic asks whether an unchanged reader can choose better frozen form rows from the production Relation read on paired new wording. The existing lexical selector remains the baseline. The served writer processes the same visible original prompt, in the same model, for every arm.

## Pins and boundaries

- Base source is `4372ed414487c68cdd79bb680ac8cc110c5df631`. The staged source manifest binds the modifications and isolated CLI source. Bind the built binary and all used inputs in each dispatch.
- Frozen carrier SHA-256 `2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302`. It validates the parent hash and saved reader on load. Reader SHA-256 `9db7e8348bb0bdadee7502a35716d39dacb8e9c25e57f76338fe3fea91e1227b`. The existing package receipt binds the parent and tokenizer. The same integer token IDs are passed to both sides.
- TRAIN tokens SHA-256 `bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24`. TRAIN response mask SHA-256 `89e0732b5858363e31a9729627e55e775107ababd4fda19ecc55e2aa9cb47e0f`. Select up to 4,096 evenly spaced starts of contiguous marked replies, using the reader's window and BOS fence. Read canonical UORT64 with the same parser as conjunction-probe reader-save. The mask identifies onsets only. No current reply IDs or held rows are read as inputs to selection. Prior visible assistant tokens may be part of an earlier conversation context. Exact duplicate state keys give the most frequent frozen form-address list and tied lists are omitted.
- The same shared CPU artifact writer is used for all arms, with the lexical branch also checked against the unchanged normal artifact reader. CPU host code enumerates lexical row IDs, constructs CUDA identity/window metadata, tallies exact TRAIN state/list joins, handles files and scores released references. The frozen row file is unmodified. CUDA executes native carried encoding and Relation pre-gate read, and computes the diagnostic bounded nearest-neighbor decisions, including exact TRAIN readback, all knockout lookups and donor decisions. The existing pre-gate kernel's diagnostic writer uses constant target ID zero and its prediction and loss are discarded. The replacement selection bypasses the later joint count/pointer gate.
- CUDA reads use 128-row batches, at most 192 causal IDs, four 8-coordinate blocks and 4,096 prototypes. Geometry is signed integer words in the existing four-block codebook. Nearest selection sums integer coordinate distances and abstains on conflicting labels at the same minimum distance. The abstention uses the unchanged suffix-ladder path. The knockout zeroes route actions and gates as in `with_state_transport_knockout`; Relation can still carry history through its other operations.
- Freeze the blinded panel hash and its adjacent opposite-role pair order under the parent sheet. Every predictor record has exactly `id` and `prompt`. The separate adjudicator opens references only after the predictions are sealed. Donor is the adjacent paired row at index `i ^ 1`. Its selected form IDs are used with the recipient's original prompt and writer.
- Output includes raw vectors and SHA-256 over consecutive little-endian i32 coordinates for each active, knockout and donor state. Each arm carries its form IDs, nearest distance and prototype ordinal when applicable, abstention flag, full emitted IDs and writer trace. The first wrong offset is zero-based, and missing expected or actual tokens appear as null. Exact, first-token and pair/family evidence are interpreted under the parent panel's predeclared gates. This mixed device/product diagnostic cannot close a GPU writer or production wording claim.

## Finite gates and compute

The no-held finite test verifies planted UORT first and last response onsets, rejects a shifted mask and raw payload, then brings up the actual artifact and 16 fixed four-ID probes with the same two-ID suffix. Every full and knockout pre-gate CUDA vector must match its native CPU observer bit-for-bit, and every knockout vector must equal the ordinary production knockout read. There must be at least one changed state at fixed suffix with transport active, plus at least one alteration under knockout. The two real differing states must select different synthetic keys at zero distance and conflicting exact keys must abstain. A finite failure blocks TRAIN and held dispatch. The finite test has no natural-language accuracy interpretation.

Use one physical CUDA device sequentially after any active production lease clears. Expected engineering ceiling is 1 GPU, under 1 GiB incremental device memory, finite under 30 seconds and under 2 minutes per fit or prediction phase after the executable has been built. These are pre-run estimates pending receipts. Keep logs and processes within the assigned scope, and use only an admitted device/lease. AOT build uses nvcc and the repository admission tool for the device's compute target. The final binary and receipt must identify compiler, device, source manifest, carrier and input hashes. Record per-phase /usr/bin/time -v and available nvidia-smi device/usage snapshots. Reserve new output paths. Do not run held prediction until the independent GO and the masked positive/writer package review are recorded.

## Commands

From the staged source root, use the actual device architecture in place of `compute_89`.

```sh
PATH=/usr/local/cuda/bin:$PATH CARGO_TARGET_DIR=../target-cuda UOR_NVCC_ARCH=compute_89 \
  cargo build --locked --release --manifest-path local/codex/state-cli/Cargo.toml
```

After the corresponding release, run each in a fresh run directory.

```sh
question-causal-state-cuda finite "$artifact" finite.json
question-causal-state-cuda fit "$artifact" "$train_tokens" "$train_mask" 4096 train-index.json
question-causal-state-cuda predict "$artifact" train-index.json "$sealed_id_only_panel" 48 predictions.json
sha256sum predictions.json > predictions.sha256
question-causal-state-cuda adjudicate predictions.json "$released_references" first-wrong.json
```
