# chat-v0 R2 — chunked parallel-scan dialogue block (design)

**Status:** design-only, 2026-09-25. Branch `codex/canonical-address-routing-20260925`, base
`5109861c`. No code, training, build or serving change is made by this document. It is the
vehicle-change design required by [R1c §6](chat-r1c-result-2026-09-25.md#6-recommended-next-action)
and rung R2 of the [CAR-LM ladder](car-lm-direction-update-2026-09-25.md#4-reordered-ladder-to-chat).

**Verdict up front.** The *current* joint cell has **no exact finite-state scan**: its gate,
transport and read inputs depend on the recurrent state, and its softmax read over all prior
events is not an associative summary. R2 must therefore introduce a declared **scan-native cell**
inside the joint family — within-chunk sequential, across-chunk an exact block-diagonal associative
monoid. This is a mechanism change, not a reparameterization of R1c. §3 enforces *implementation*
parity of the new cell; a separate matched run measures the *model-level* change against the R1c
width-576 curve. No serving code changes in R2.

## 1. The recurrence to chunk

Let `d=width`, `T=context`, `C=chunk`, `L=T/C`. `core_step` ([joint_model.rs:948](../crates/uor-r4-training/src/joint_model.rs))
computes, per position `t`, with state `s_{t-1} ∈ R^d` and token affine `x_t ∈ R^{3d}`:

```text
x_t   = e[tok_t] · W_inᵀ                                  (one matmul for all T)
r_t   = RMSNorm(s_{t-1}) · W_rᵀ          W_r = recurrent.state.weight (3d×d)
f_t   = x_t + r_t + b
u_t   = tanh(f_t[0:d]);  z_t = σ(f_t[d:2d]);  v_t = f_t[2d:3d]
g_t   = Tport(s_{t-1}, v_t)              per-R4-lane left quaternion, q=normalize(e0+0.1·v)
p_t   = (1−z_t)⊙g_t + z_t⊙u_t
a     = softmax([null(q_t); q_t·Kᵢᵀ/√r + age_i])   q_t = W_q·RMSNorm(p_t)
read_t= Σ_i a_i·V_i
m_t   = tanh(W_up·[p_t; read_t]);  ρ_t = σ(W_ρ·[p_t; read_t])
s_t   = (1−ρ_t)⊙p_t + ρ_t⊙m_t
c_t   = σ(W_c·[s_t; read_t]);  k_t = W_k·RMSNorm(s_t);  val_t = tanh(W_v·RMSNorm(s_t))
```

The terms split into three layers:

**(L) Associative.** If `(z,v,u)` are computed from `x_t` alone, then `s_t = A_t s_{t-1} + d_t`
with `A_t = diag(1−z_t)·blkdiag₄(R(q_t))` (block-diagonal 4×4 per lane) and `d_t = z_t⊙u_t`.
Define the chunk summary over positions `a..b`:

```text
Φ_{a→b} = A_b A_{b−1} … A_a           (block-diag 4×4 per lane)
ψ_{a→b} = Σ_{k=a}^{b} (A_b … A_{k+1}) d_k
(Φ,ψ)_{a→c} = (Φ,ψ)_{b→c} ⋆ (Φ,ψ)_{a→b}
            = ( Φ_{b→c}·Φ_{a→b},  Φ_{b→c}·ψ_{a→b} + ψ_{b→c} )
```

`⋆` is associative because it is matrix multiplication; it is the classic linear-attention monoid.
Each summary is `16·(d/4) + d = 5d` floats; one compose is `4d` MACs. The transport alone composes
exactly by quaternion multiplication, `R(q_a)R(q_b)=R(q_a⊗q_b)`; with the gate, each 4×4 block is a
general contraction with singular values ≤1 (stable). The scan materializes all `A_t,d_t` with one
batched matmul `X·[W_z;W_u;W_v]ᵀ`, then prefix-composes the `L` summaries in `log₂L` steps.

**(N) Non-associative.** `RMSNorm(s)`, the sigmoid/tanh gates, and the elementwise `interface`
quantizer (`joint_quantization::fake_quant`) make `A_t` state-dependent. This is why the R1c cell
cannot be chunked exactly. **Resolution:** keep (N) *per position inside the chunk*: chunk `c`
resolves its entry state from the scan, then runs its `C` exact positions; chunk-local work is
parallel across the `L` chunks. The cross-chunk summary uses only (L).

**(R) Not scanable.** The softmax read over *all* prior events and the exact-copy gather are global
(O(T²)/O(T)); softmax couples every key in its denominator, so no constant-size summary exists.
**Resolution:** replace with (i) a **local causal softmax over the last `W≤64` events** (per
position, parallel across chunks; matches the retained bounded-admission direction) and (ii) an
optional **linear long-range read** `q_tᵀ S_{t-1}`, with `S_t = Σᵢ γᵢ kᵢ vᵢᵀ` — a second exact scan
with diagonal decay. Local copy masses sum over the window; long-range mass is attributed by `S`'s
value memory. This is the Mamba-2/GLA duality and is the declared (R) replacement.

## 2. Interface (training crate only)

```text
File map
  crates/uor-r4-training/src/joint_scan.rs        NEW  scan cell, monoid, chunked forward/backward
  crates/uor-r4-training/src/dialogue.rs          + scan dispatch, probe subcommand
  crates/uor-r4-training/src/joint_model.rs       + new_dialogue_scan constructor (no serving crate touched)

crates/uor-r4-training/src/joint_scan.rs
  pub struct ScanConfig { pub chunk: usize, pub window: usize, pub mode: ScanMode }
  pub enum ScanMode { GatedTransport, GatedTransportLinearRead }
  pub struct ScanSummary { phi: Tensor /*(B, d/4, 4, 4)*/, psi: Tensor /*(B, d)*/ }
  fn compose(a: &ScanSummary, b: &ScanSummary) -> Result<ScanSummary>
  fn chunk_summaries(&self, affine: &Tensor, batch: usize, time: usize) -> Result<Vec<ScanSummary>>
  fn scan_prefix(summaries: &[ScanSummary]) -> Result<Vec<ScanSummary>>
  fn resolve_chunk(&self, entry: &ScanSummary, s0: &Tensor) -> Result<Tensor>
  fn scan_forward(&self, ids: &[u32], batch: usize, time: usize, mode: ReadMode, training: bool) -> Result<JointOutput>
  fn scan_block_gradients(&self, inputs, targets, masks, batch, time) -> Result<MaskedBatchGradients>  // compose with joint_parallel

impl JointModel
  pub fn new_dialogue_scan(config: JointConfig, scan: ScanConfig, device: &Device) -> Result<Self>

crates/uor-r4-training/src/dialogue.rs
  DialogueCampaign.scan: Option<ScanConfig>      #[serde(default)]  // absent => historical R1c cell
  validate: chunk ∈ {16,32,64,128}, window ∈ {32,64}, context % chunk == 0
  run_fit_inner: if cfg.scan.is_some() => JointModel::new_dialogue_scan + scan_forward/scan_block_gradients
```

Fixed R2 dimensions: `width=1024`, `context=256`, `chunk=64` (L=4), `read_width=64`, batch 16–24,
same `response_mask.u8`/`tokens.u16`, same `masked_nats` response-masked objective, same data seed.
Primary variant drops `recurrent.state.weight` from the transition (3d²=3,145,728 params at d=1024);
`recurrent.input.weight`/`recurrent.bias` carry `(z,u,v)`, and `read.*`/`update.*`/`copy.gate.*`
are reused. The optional linear-read adds no new shapes. Record the resulting parameter count in the
campaign rather than assuming the existing 13,778,306.

## 3. Numerics / gradients — parity harness

The new cell is a pure function; chunked and sequential are two evaluation orders and must be
bitwise-close. Prototype on CPU with f64-accumulated reference checks.

```text
Test file: crates/uor-r4-training/src/joint_scan.rs  #[cfg(test)] mod tests
  scan_chunked_matches_sequential_forward_and_loss   // width 128, context 64, batch 2, C∈{16,32,64}
  scan_chunked_matches_sequential_gradients          // every named parameter, relative error
  scan_chunk_summary_monoid_associativity            // compose groupings {a⋆b}⋆c vs a⋆{b⋆c}
  scan_chunk_size_invariance                         // C=16 vs 64 vs 256 on same fixed batch
  scan_session_matches_full_forward                  // incremental scan session == full scan_forward
  scan_causality_no_future_reads                     // perturb position t+k, assert t unchanged
  scan_finite_difference_check                       // 3 largest-|g| coords, h=1e-3, rel ≤ 3e-3
```

Harness: seed 7, deterministic non-degenerate ids (as `bounded_shared_graph_session_causality_and_gradients`),
one `scan_forward(..., training=true)`, `masked_nats` loss, one `backward()`. Compare per parameter:

```text
loss_equal:            |L_chunk − L_seq| ≤ 1e-6
gradient_parity:       max over params/elements |g_c − g_s| / (|g_s| + 1e-12) ≤ 1e-5
monoid_associativity:  ≤ 1e-6 relative (float reassociation only)
```

If a parameter exceeds 1e-5, first recompute both orders with f64 accumulation: a residual that
vanishes is reassociation (accept, record); a residual that persists is a structural bug (reject).
Run `cargo test --offline -p uor-r4-training --lib joint_scan::` and the width-1024 smoke
(`new_dialogue_scan` construction + 2-step forward). Report the executed numbers, not just pass/fail.

## 4. Throughput expectation (projection, to be measured)

R1c clean baseline: CPU+Accelerate, batch 24 → **804–841 sampled targets/s**, ~7.0 s/step, backward
84%. The sequential unroll is op-count/latency bound, not FLOP bound (flat batch 8→24; ~1 core busy).
C=64 replaces a 256-deep chain of tiny ops with `L=4` chunk matmuls + `log₂L=2` scan composes, each
batched over `B×C` rows and parallel across chunks/cores. **Projection 4–10× → ~3,200–8,000
targets/s**, one epoch (82.5M sampled targets) ~3–7 h. This is a projection only.

**Go/no-go probe** (predeclared): chunked targets/s **≥ 3×** the sequential targets/s on the same
host/batch/context **AND** all §3 parity tests pass, else **STOP** and use §6 alternatives.

```sh
BIN=target/release/uor-r4-training-metalacc
# paired probe campaigns differ only in "scan"; 20 steps, <=600 s, tiny eval, same seed/data
$BIN dialogue-fit campaigns/r2-scan-probe.json   /tmp/r2-scan-probe-1 cpu   # scan = {chunk:64,window:64,mode:"gated_transport"}
$BIN dialogue-fit campaigns/r2-seq-probe.json    /tmp/r2-seq-probe-1  cpu   # scan = null (R1c cell)
# compare report fields sampled_targets_per_second, phase_seconds.backward, and loss parity at step 20
```

## 5. Serving path

A chunked-trained artifact does **not** need to execute a scan at serving. The R2 cell's per-token
update `h_t = (1−z_t)⊙R(q_t)h_{t-1} + z_t⊙u_t` is elementwise add/blend plus the existing integer
quaternion transport; local softmax is already integer (`model.rs` `softmax(&scores,&tables)`), and
the linear long-range read is an integer accumulation. Under D0-b the declared kernel stays
add/sub/shift/table-read; no float or multiplier instruction. If R3 later serves precomputed chunk
summaries, composing two 4×4 blocks is 16 shift-add products per lane — still multiplier-free.

**Blocking width/context mismatch.** The current integer loader rejects the R2 artifact:
`crates/uor-r4-integer/src/config.rs:48` accepts only `width ∈ {128,256}`; `src/model.rs:187-188`
hard-fixes `context=256 && width=256 && read_width=64`; `src/model.rs:466` asserts normalization
width 256. R3 must extend width to 1024, keep `read_width=64`, add `chunk`/`window`/`mode` to the
packed manifest and loader checks, widen `STATE_BITS`/tables and the exporter, and re-run the
numerical bridge. R2 adds none of this.

## 6. Risks and alternatives

- **Low-bit export.** A composed chunk summary is a 4×4 contraction per lane. Serving per-token
  (recommended) never quantizes a composed matrix, so the R1c state grid is unchanged. If summaries
  are ever served, their singular values are ≤1 but the current Q11 state interface may clip; R3
  must audit it.
- **Chunk-boundary effects.** The input-only projection removes state-dependent gates; boundary
  statistics can shift. The `scan_chunk_size_invariance` test plus a matched C∈{32,64} held-out NLL
  check bound this. If model-level NLL regresses versus R1c at equal exposure, retain the option of
  re-injecting a state-dependent correction once per chunk (reuses `recurrent.state.weight`, at the
  cost of the parameter drop) — declared as a variant, not silently mixed in.
- **Memory.** Summaries cost `5d` per chunk per lane (~6 MB at B=24, d=1024, L=4); backward holds
  only `C` live steps instead of 256, so peak should fall, not rise. Measure host RSS.
- **If <3× is not reached.** In order: (i) the existing `cpu_gradient_shards` (measured 1.2–1.6×,
  not sufficient alone); (ii) chunked depthwise convolution / window-attention mixer instead of a
  scan; (iii) shorter context for the same compute. Record the measured ratio and pick the cheapest
  path that still carries ≥5M parameters and the response-masked objective; do not tune a failed
  vehicle in place.

**Affected components.** `joint_parallel::masked_batch_gradients` gains a scan sibling;
`joint_model` gains one constructor; the integer crate, tokenizer, data and frozen panel are
untouched. The R1c width-576 artifacts and the sealed panel `fresh` array remain preserved and
closed until a design is selected.
