# Shared Track B attention interface

Status on 2026-09-29: **source implemented; compilation, new tests and checkpoint parity NOT_RUN**.
The earlier stock
wrapper's executed checks are scoped to its [recorded revisions](track-b-conversion-result-2026-09-29.md),
not to this newly added implementation. [Draft PR #1518](https://github.com/UOR-Foundation/uor-r4/pull/1518)
carries both stages without claiming either checkpoint parity or B2 acceptance.
The [prospective storage correction](../evidence/track-b-storage-correction-2026-09-29.json)
now permits a bounded next phase: the plan reserves 30 GiB for Track B traces,
whereas our earlier card mistakenly treated that allocation as an untouchable
free-space floor. The new conservative lab guard is 24 GiB plus the owner's
128 MiB stop margin, with 256 MiB additional build and 96 MiB report allocation.
Earlier stopped attempts keep their original conditions and remain unqualified.

Lab 1's [shared-model ruling](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5898192281)
assigns one Candle model to Codex. DeepSeek supplies flock at the attention seam;
Anti-Gravity uses generation for teacher data. Source API availability is distinct
from a qualified numerical bridge.

## Model and plug-in contract

`track_b::model::TrackBModel` loads the original full-width checkpoint through the
existing safe Kappa loader after the strict conversion config checks. It preserves
frozen embeddings, projections, MLPs and tied output head. The forward is adapted
from Candle 0.9.2, with its source revision and MIT notice in the file. Primitive
RMS normalization, split-half RoPE and ordinary softmax retain autodiff paths.
The default is dense causal softmax; the result is all-position F32 logits
`[batch,time,vocabulary]`. It supports full prefixes up to the checkpoint's position
limit, including 2,048 positions; the authored lightweight 2,048-position check
is not an executed full-model test.

```rust
pub trait AttentionKernel {
    fn attend(
        &mut self,
        layer: usize,
        inputs: &AttentionQkv,
        positions: &AttentionPositions,
    ) -> uor_r4_training::Result<candle_core::Tensor>;
}
```

- Q has shape `[B,H,T,D]`; K and V retain native grouped-query shape `[B,KV,T,D]`.
- These are **after RoPE**. Absolute query/key ranges are explicit and currently
  both `0..T`; cached or incremental tails are unsupported.
- The interface also supplies the causal mask, RMS-plus-gain normalized input
  `[B,T,W]`, and shared cosine/sine tables. LoRA can therefore use the actual
  composed student's input and apply the same RoPE to projection deltas.
- The return is F32 `[B,H,T,D]` before frozen WO. The model applies WO, residual,
  MLP and later frozen layers without detaching the replacement's output.
- `capture_layer` stops after the selected layer's WO and returns detached
  normalized input and post-WO/pre-residual target. It skips that layer's MLP,
  later layers and the vocabulary head.

There is no adopted LoRA trainer, recurrent attention operator or flock adapter
in this change. A dense floating-point teacher is not native geometric serving.

## Harmonic features

`track_b::harmonics` implements the [reviewed proposal](track-b-harmonic-design-2026-09-29.md):
packed trace-free degree 0–3 bands and their positive shifted-power combination.
The six feature widths are 17/152/952 at d=16 and 33/560/6,512 at d=32. Normalization
uses a counted fixed unit-vector fallback near zero. The feature order has a
version string for later artifact binding.

Real-arithmetic positivity does not guarantee an F32 denominator floor.
Distributed antipodal tests check score signs and floor-relative error separately
from the ordinary Gram tolerance. A future recurrent operator must record and
reject nonpositive/nonfinite denominators and match an explicit causal kernel;
it may not hide instability with a clamp. No recurrent-state cost measurement
or operator-learning result is claimed here.

## Seeded batched generation

`TrackBModel::generate` takes tokenized prompts with stable seed IDs, explicit
sampling/options and a master seed. A versioned SHA-256 derivation selects a
separate Candle RNG stream per prompt; reordered batches retain those streams.
Options select greedy or positive-temperature top-k/top-p sampling, maximum new
tokens, maximum padded token allocation, PAD and EOS IDs. Duplicate seed IDs and
invalid options are errors. When combined, top-k masking precedes softmax so
top-p uses the renormalized retained distribution; cutoff ties select lower
token IDs first. Results include generated IDs, derived seed and
EOS/context/new-token stop reason. Newly generated EOS is retained; prompt EOS
does not stop generation.

The implementation recomputes complete prefixes and right-pads only future
positions. It selects each row's last real position, so future padding is masked
by ordinary causality. It does not infer padding from token values, important
because SmolLM2 PAD and EOS both use 2. Finished rows leave the active batch and
consume no more RNG. Sampling streams are stable; cross-device floating-point
token equality is unmeasured.

This is an F32 functional interface without a KV cache or throughput result.
F16 teacher-data generation, chat-template rendering and optimized cached
batching remain separate work. Callers must use the original tokenizer, bind
prompt/template/source identities and budget full-prefix attention and logits;
`max_padded_tokens` is an allocation guard, not a total RSS guarantee.

## Amended numerical gate and next execution

The fixed reference still has four windows of lengths 1/4/8/32, 45 positions,
49,152 vocabulary coordinates, two exact CPU workers, ≤1e-4 maximum absolute
error and a 600-second total wall bound. Neither windows nor tolerance are reduced.
For each CPU/Metal backend the future harness retains 53 stock-wrapper comparisons
and adds 94 shared-model rows: 45 full-prefix rows plus 49 batched rows. The second
batch row is a distinct `[1]` prefix followed by future padding; only its first
position is scored against the exact singleton reference.

The [prospective amendment](../evidence/track-b-shared-attention-projection-2026-09-29.json),
[registered on the board](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5898730739),
increases the retained-report projection from 32 to 96 MiB to preserve these
extra raw logits. It does not increase the time, thread, RSS, build-storage or
cumulative-ledger limits. The later prospective correction replaces our
30 GiB free-space interpretation with the conservative 24 GiB lab guard and
unchanged 128 MiB margin. Preflight accounts for all projected new space;
runtime observation fails closed, independently of the hard 600-second timer.
The observer stops and joins before report sealing. Next: compile and run the
named tests, then the unchanged-reference smoke in a new claimed root. B2
fitting remains gated on an actual passing shared-model result.
