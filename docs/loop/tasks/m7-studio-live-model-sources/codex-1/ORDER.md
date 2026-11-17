# Restore a verified public M1 prediction in Studio

Lab: codex. References [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md), #3440. Base: 9465b92d6e11ca0ee913ebe34aaaddff0cdf97f5.

## Cancel probe

The live catalog contains only the synthetic demo. A pinned M1 seed 1 URL loads the 19-table trunk with the published digest, but the prediction panel refuses text without its matching tokenizer. The default composer routed a request to the browser task fixture. Both defects reproduced in a fresh Chromium session.

## Scope and done check

Trace the existing publication gate, match public weights and tokenizer to their release evidence, and exercise text prediction through the normal rendered Studio controls. Repair the smallest necessary Studio and publishing surfaces, with focused regression checks. Record the artifact identity, tokenizer binding, a real top-eight result, the compute boundary and the publication state. The parent session owns review and release.

## Stop list

Use only the already released M1 artifact. Preserve existing gates and evidence. No research training, panel scoring, private artifacts, paid resources, changes to owner rules, workflow edits or direct public writes.
