# Track B draft preservation

All five sources are unregistered. Only the std-only cost estimator has been
compiled and exercised: its exact source passed nine accounting fixtures on
September 29. The other four drafts remain **UNCOMPILED / NOT_RUN**. No token
corpus, fitted operator, independent checkpoint replay or model result exists
from these drafts. The shared Candle model build remains blocked by library-load
policy; accounting fixtures do not qualify that model or its parity bridge.

| Draft | Preserved scope and limits | Execution status |
| --- | --- | --- |
| [track_b_source_data.rs](track_b_source_data.rs) | Bounded two-pass preparation with the original tokenizer, document-separated splits and exact-text duplicate exclusion. Requires a direct `blake3` dependency. UORT v1 has a 64-byte header: consumers must use `MmapCorpusReader` and manifest payload-token offsets; the historical headerless reader is incompatible. | UNCOMPILED / NOT_RUN |
| [track_b_transfer.rs](track_b_transfer.rs) | Full-width frozen Q/K/V/O with paired rank-4, alpha-4 Q/K/V LoRA. Dense control and positive harmonic polynomial arms; separate bounded detached recurrence. Training forms a quadratic Gram matrix. Independent per-query-head key maps require nine states, despite three native KV heads. | UNCOMPILED / NOT_RUN |
| [track_b_fit.rs](track_b_fit.rs) | Isolated teacher-forced layer fit using actual teacher post-WO targets, explicit document-disjoint windows, paired seed and fixed schedule of at most 500 updates. Reuses `NamedAdamW`, requires a real resource observer, and preserves parameters, moments and configuration hashes. A caller must claim/seal its report through `report_output`. Student composition, public hooked early-exit capture and a runnable coordinator are unfinished. | UNCOMPILED / NOT_RUN |
| [track_b_hybrid.rs](track_b_hybrid.rs) | Replaces selected harmonic scores with raw reciprocal-rank weights before one normalization. Detached recurrence correction changes both numerator and mass. Requires supplied unique causal ranked support; it does not implement or validate the Lorentz selector. Shared selector integration, actual fitting and composed generation are absent. | UNCOMPILED / NOT_RUN |
| [track_b_cost.rs](track_b_cost.rs) | Checked analytic state, matrix, parameter and logical traffic accounting. Current implementation costs remain separate from counterfactual sharing/fusion. No model, Candle, backward-cost, timing or allocator implementation. | Exact std-only source compiled; 9/9 accounting fixtures PASS |

The fit's reduction decision covers the isolated-layer MSE gate only: a positive
replacement baseline must improve by at least 50%; the dense control's zero
baseline is explicitly non-applicable. Evaluation roles distinguish development,
frozen confirmation and independent final held-out data. Proposed zero/constructor-
mean predictor baselines and fixed norm-bin diagnostics are not yet implemented;
they do not add owner acceptance gates. All-layer hybrid NLL/KL and the matched
dense comparison remain separate work.

The hybrid draft also retains a source-only regression for prescribed reciprocal
rank mass. Three value lanes isolate selected mass, first-selected weight and
background mass at causal prefix lengths 8, 32 and 128, with fixed support size 8.
It compares dense replacement and recurrent correction with a closed-form F64
oracle under a declared F32 tolerance. The correction input is aggregated in F64
and rounded once to F32; long-prefix F32 recurrence accumulation error remains a
separate diagnostic. The fixture remains **UNCOMPILED / NOT_RUN**;
it does not exercise feature projection, the external selector or learned model
behavior, and it changes no arm or acceptance gate. The independently reviewed
[information/mass analysis](https://github.com/UOR-Foundation/uor-r4/blob/e259ec6e0bd5793a1c134b4c7b41095b8e340bdf/research-reviews/20260930-track-b-information-mass/track-b-information-mass-audit.md)
records its mathematical motivation and scope.

The source-data and transfer drafts were recovered internally after the owner's
September 29 workspace-image restoration. The data draft's original recovered
hash was `93458b4718be3919848567ca4ff1379fa9929ae64c838efe453617d8766c9866`;
its metadata reserve and non-UTF8 argument handling were then corrected. Transfer
was restored from recorded successful patches; no prior external checksum
survives for an independent byte comparison.

The [preservation evidence](../../../docs/evidence/track-b-b2-drafts-2026-09-29.md)
records all current hashes, executed estimator receipts, analytic tables and
remaining NOT_RUN work. The estimator's frozen source header still says NOT_RUN;
the receipt supersedes that statement only for its exact accounting fixtures.
Register and compile Candle drafts under the next declared implementation step.
Fitting remains gated on the shared model's unchanged exact-reference parity
test. No policy, goal, gate threshold or resource allowance is changed here.
