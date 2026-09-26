# Geometric language model lab, phase 2: the route to a chat model, and what hyperbolic attention has and has not shown

2026-09-26 · Requested by the owner · References #820 · Branch `claude/blissful-wozniak-girwwq` (kept separate from `main`)

**Status.** An evidence note and proposed roadmap, not a decision record. It follows [cycle 3](hyperbolic-cycle3-2026-09-26.md) and applies the owner's decisions of 2026-09-26:
- weight transfer from an open transformer is approved, provided the served model performs no matmul;
- hyperbolic geometry is the lead mechanism;
- offline training is in scope;
- paid compute needs the owner's explicit approval.

**Labels.** **Measured** (run in the review sandbox), **Derived**, **Literature** (retrieved in this session; arXiv id given), **Hypothesis**. The lab had five specialist reviewers (mathematics, architecture, training and data, systems, experiment), two adversarial reviewers (feasibility/engineering and science/novelty) and the lead. Their reports and scratch experiments stay outside the repository. The numbers this note relies on are in [the evidence file](../evidence/lab-phase2-2026-09-26.json).

**New in the repository.** A Rust conversion tool in the training crate: `kappa_llama`, example `kappa-conversion`, and `scripts/kappa-m1-pilot.sh`, in commits `c67e230`, `50bee85` and `f43aa4e`.

## 0. Findings

1. **The route to a chat model is converting an open instruct model; the adversarial reviewers agree.**
   - *Literature.* Open small talk appears at 100–350M parameters only after 1–11T training tokens (MobileLLM 2402.14905, SmolLM2 2502.02737, LFM2 2511.23404).
   - *Derived.* An M1 trains roughly 0.2B tokens per week at 150M parameters, at an assumed and not yet measured Candle-Metal throughput.
   - Converting SmolLM2-135M/360M-Instruct (Apache-2.0) is the only route to that band inside the owner's budget. Training from scratch gives narrow-domain chat at best.
2. **The dot-to-hyperbolic "exact conversions" are flat-limit reparametrisations.** Three constructions reproduce a trained dot-product head, but only as curvature κ → 0 (§2.1).
   - *Derived, checked numerically.* To first order they are the dot score plus κ times a fixed quartic norm feature.
   - The meaningful measure is the dimensionless curvature `t = κ·mean|k|²`. A head is genuinely hyperbolic only near `t ≈ 1`.
   - *Literature.* Per-head learnable curvature initialised at zero with an exact Euclidean limit already exists (FPS-T 2309.04082; κ-GCN 1911.05076), as does RoPE as a spatial Lorentz rotation (HELM 2505.24722).
   - *Hypothesis: possibly new here.* The polarisation scores that reproduce a *pretrained* head, the hyperbolic norm-completion lift, Gromov-product attention, Busemann-landmark admission, and the flat-basin result in item 3.
3. **Curvature stays in the flat basin unless forced, and forcing it has not helped a converted model yet.**
   - *Measured, stand-in teacher, data objective, two seeds.* With curvature learnable from the exact start, it drifted only to `t ≈ 0.02` and matched the dot control to 0.0001 bits per byte.
   - Forced deep into curvature (one seed each), it was 0.0011–0.0015 bits per byte worse.
   - *Measured, toy retrieval heads on the code-scope tree.* An exact conversion keeps its teacher's blind spot (0.4% non-root). Annealing curvature in lifts it to 31%; a direct curved lift gives 52.5%. That task is hierarchical by construction.
   - The stand-in's heads are local byte mixers (68–98% of attention mass within 3 bytes, per the science reviewer; *Measured*), so the stand-in cannot tell geometries apart.
   - **The deciding test is now a zero-training probe on real SmolLM2 heads with a pre-registered rule (§4, M1a).** The tool implements it.
4. **Hyperbolic geometry measurably helps where heads are *trained* with distance scores.**
   - *Measured.* At 64 key dimensions on this repository's code-scope tree, hyperbolic keys learn the hierarchy (85–90% non-root). Dot keys learn only "attend to the root" (0–2%).
   - *Measured, science reviewer, real cycle-3 read heads.* Distance-trained heads are indexable: query/top-key cosine 0.95, top-1 admitted at 99.2% with 5.5% of keys scored.
   - Real dot heads are not: cosine 0.31–0.33, 79% at 2.5% scored, about 25% scored needed for 99.8%. The synthetic near-copy results (99.8% at 2.8%) do not transfer to them.
   - So the geometry belongs in memory and index layers trained hyperbolic, not in converted dot heads.
5. **The cycle-3 gain is mostly a copy-distance effect.**
   - *Measured, science reviewer.* Controlling for log copy distance, the "enclosing scope" advantage survives in one of three seeds. The Lorentz read's gain grows with copy distance in all three: −0.013 to −0.027 nats per doubling.
   - The mathematics reviewer's 0.01–0.02-nat "oracle ceiling" measures what six scope features add to a small copy model. It is not a ceiling for a real LM.
6. **Energy: at 4 bits and 2K tokens of context a converted model is at about parity with llama.cpp Q4_0; the win needs long context or ternary weights.**
   - *Derived from published constants; nothing was measured on an M1.*
   - Recomputed on matched cores and key/value precision by the feasibility reviewer: 7.7 against 8.6 mJ per token on efficiency cores.
   - About 2× at 8K tokens with 3% of keys admitted.
   - Ternary weights project lower but need about 1.3 H100-days of quantization-aware training per attempt, which is paid compute.
   - The energy case does not depend on the attention geometry (science reviewer): it comes from table-driven low-bit weights, efficiency cores and cache compression, which also work for dot keys.
7. **Negative results.**
   - A power-law (horocycle) positional prior extrapolated worse than exponential decay (3.264 vs 3.226 nats at 2,048 tokens).
   - A conservative Hamiltonian read never settles on a key; with damping it costs about 100 softmaxes.
   - Primes, CRT and Galois-field hashing give no advantage over simple tabulation for exact content memory (all within 0.2% of the ideal collision count).
   - Two-level advertisement cells plateau at 99.6% admission.

   All *Measured*.

## 1. The route to a chat model

| Model | Size / tokens | Result (Literature, arXiv) |
|---|---|---|
| TinyStories | 1–80M | Narrow-domain fluency at 28–80M (2305.07759) |
| MobileLLM | 125M / 350M, 1T | MT-Bench 2.33 / 3.28 (2402.14905) |
| SmolLM2 | 135M (2T) / 360M (4T) | IFEval 29.9 / 41.0 (2502.02737) |
| LFM2-350M | 11T, logit distillation | IFEval 65.1 with 6 attention and 10 convolution layers (2511.23404) |
| Mamba-in-the-Llama | 8B instruct, 20B tokens | MT-Bench 7.35 / 6.86 at 50% / 25% attention; 5.64 for the fully Mamba2 variant (2408.15237) |
| LoLCATs | 8B, 40M tokens, LoRA | MMLU 52.8 vs 66.6; 23.8 without its sliding window (2410.10254) |

Conversions that keep softmax attention over the cache are the cheap regime; fully recurrent conversions lose most. This lab replaces only the attention score. Whether a converted model *also* needs geometric components, and in which layers, is the open question behind §4.

## 2. Converting dot-product attention

### 2.1 Three flat-limit constructions (Derived)

For one head of width `r`, curvature `κ`, and points lifted to the hyperboloid of curvature `−κ` with geodesic distance `d`:

| Construction | Score | Near κ = 0 |
|---|---|---|
| Key-norm homotopy (architecture reviewer) | `(|k|² − d(q,k)²) / (2√r)` | `(⟨q,k⟩ − |q|²/2)/√r + κF/√r`, with `F = (|q|²−|k|²)²/8 + |q−k|⁴/24` |
| Intrinsic polarization (lead) | `(d(o,k)² − d(q,k)²) / (2√r)`, `o` the origin | same, minus `κ|k|⁴/(6√r)` |
| Norm completion (mathematics reviewer) | separate completion coordinates give `q₀ = Q`, `k₀ = C`, `z = QC − q·k` | the dot score as `QC → ∞`, with every key on one hyperbolic sphere |

- *Measured, the science reviewer's check.* With the first-order term the error falls from 7.5×10⁻² to 3.5×10⁻⁵ at κ = 10⁻⁵.
- RoPE rotates spatial coordinates only, a hyperboloid isometry, so the scores stay relative-position scores.
- Norm completion makes the radius constant, so it cannot carry hierarchy.
- The **Gromov product** `(q|k)_o = ½(d(o,q) + d(o,k) − d(q,k))` is not exact at the start. In a rooted tree it is the depth of the lowest common ancestor.

### 2.2 Stand-in conversion (Measured)

Teacher: a 3-layer, width-128, single-head RoPE attention byte model (607,488 parameters), 1.9897 bits per byte on WikiText-2 validation.

| Score | Zero-shot | After 400 steps of Q/K + temperature distillation |
|---|---:|---:|
| dot (drift floor) | 1.9897 | 1.9909 |
| intrinsic polarization (log ε = −6) | 1.9897 | 1.9905 |
| cosine, fitted scale | 2.0511 | 1.9908 |
| Gromov product, fitted scale | 2.0490 | 1.9907 |
| Euclidean distance | 3.9259 | 1.9929 |
| committed Lorentz read `β(δ − d)` | 4.0115 | 1.9946 |

Every score recovers because these heads are local byte mixers: a content-free, distance-only attention pattern alone reaches 2.92 bits per byte against 4.22 with no attention (science reviewer). The table shows that the distillation machinery works. It does not show that geometry swaps are cheap for SmolLM2's sink, induction or retrieval heads.

Drift away from the teacher as curvature grows (mean row KL at log ε = −3, *Measured*):

| Layer | Intrinsic | Key-norm |
|---|---:|---:|
| 0 | 0.032 | 0.024 |
| 1 | 0.056 | 0.24 |
| 2 | 0.086 | 0.53 |

On synthetic sink heads the ordering reverses (intrinsic KL 3.75 at κ = 1, key-norm 0; architecture reviewer), so real heads must choose.

### 2.3 The flat basin (Measured)

Toy retrieval heads on the code-scope tree (mathematics reviewer, one seed). Non-root accuracy at 48 / 96 / 192 stored nodes:

| Arm | Non-root accuracy |
|---|---|
| Dot teacher | 0.0 / 1.7 / 0.4% |
| Exact conversion, curvature learnable | 0.0 / 1.4 / 0.4% |
| Curvature annealed into the curved regime | 35 / 35 / 31% |
| Direct curved lift (raw accuracy first collapses to about 1%) | 59 / 57 / 53% |
| Hyperbolic head trained from scratch | 85.5% at 192 |

Stand-in, data objective, 600 steps. Validation bits per byte:

| Arm | Seed 1 | Seed 2 |
|---|---:|---:|
| dot | 1.9480 | 1.9458 |
| intrinsic, learnable from exact (ends at `t ≈ 0.02`) | 1.9479 | 1.9457 |
| intrinsic, curvature floor raised to log ε = −2 (`t ≈ 3–9`) | 1.9491 | — |
| Gromov product, direct curved conversion | 1.9495 | — |

## 3. The Rust conversion tool

`crates/uor-r4-training/src/kappa_llama.rs` and `examples/kappa-conversion.rs`:

- **Loading.** A Hugging Face Llama checkpoint: BF16 or F32 safetensors, grouped-query attention, tied or untied output head. Biases, RoPE scaling and interleaved RoPE are refused.
- **Scores.** `dot`, `key_norm` and `intrinsic`, plus the exact first-order kinds `*_linear` (the matched-capacity control). Every head has a learnable temperature.
- **Modes:**
  - `tokenize`: the checkpoint's own BPE.
  - `probe`: agreement with dot over a grid of `t`.
  - `drive`: the zero-training test below.
  - `train`: next-token loss or KL to a dot teacher; optional query/key training; gradient accumulation; curvature annealing to a dimensionless `t` whose floor is recomputed from the current keys every step, so shrinking the keys cannot escape it.
  - `sample`: SmolLM2's chat template and default system turn; refuses mismatched saved variables.
- **`drive`, per head of the real checkpoint:**
  - key-norm statistics;
  - the cosine from each query to its top key;
  - the attention mass in the top 2% and 5% of keys, which is the ceiling for any index;
  - the flat-limit curvature drive `dL/dt` of both first-order kinds, from one backward pass, under next-token loss and under KL to a teacher. The self-distillation drive must vanish; it is reported as a check;
  - the zero-shot loss change when one layer's heads are set to `t = 0.1` and `t = 1`.
- **The pre-registered rule.** Curvature training is warranted in a layer only if `−Σ_h dL/dt_h` exceeds the measured zero-shot cost at `t = 1`.
- **Checks.** Ten focused tests: flat-limit and first-order agreement, RoPE invariance, statistics, the drive and its vanishing self-distillation gradient, gradients, the arcosh series switch, the curvature floor, and distillation toward a flat teacher. Every mode ran end-to-end on a synthetic bf16 grouped-query checkpoint.
- **Memory** (measured by the feasibility reviewer; the tool's defaults follow from it). The curved scores keep about thirty `(batch, heads, time, time)` tensors per layer. Training a 135M student with a 360M teacher therefore needs `batch=1` with accumulation at 256 tokens on a 16 GB machine, or 128 tokens on 8 GB. Contexts of 2K or more need a chunked implementation.
- **Not done.** No real checkpoint has been converted. The SmolLM2 weights are not reachable from the review environment (Hugging Face egress is blocked); they are on the owner's Mac.

## 4. Roadmap

| # | Where | Work | Decision |
|---|---|---|---|
| M1a | owner's M1, about 1 h | `scripts/kappa-m1-pilot.sh`: probes, `drive` with SmolLM2-360M as teacher, a chat sample | Pre-registered: if no layer passes the drive rule, skip curvature training, keep dot heads, and report the backbone as a quantized transformer |
| M1c | owner's M1, about 1 h | llama.cpp SmolLM2-135M Q4_0 with a q8_0 cache at 2K and 8K tokens, on a performance core and under `taskpolicy -c background`, joules per token from two run lengths | If Q4_0 on efficiency cores is within 1.5× of the projected converted path at 2K, the energy case rests on M4 or ternary weights |
| M1b | owner's M1, only if M1a passes | Distillation from 360M: dot, `intrinsic_linear`, intrinsic learnable, intrinsic and key-norm annealed to `t = 1`; 3 seeds, paired data | Do curved heads beat *both* controls on held-out KL/NLL beyond seed noise? |
| M2 | M1 days, or GPU if approved | 4-bit QAT distillation including integer activations, table nonlinearities and a ≤4-bit output head | Chat quality at 4 bits. **Gated on owner decision 2** |
| M3 | here and M1 | Integer serving; chat CLI; measured joules per token | Against the M1c baseline |
| M4 | here and M1 | Long context: dump real SmolLM2 queries and keys, test query-aware indexes, and train a native hyperbolic memory/retrieval layer over the converted backbone | The geometry's best-supported role (§0.4). ≥99% of attention mass at ≤5% scored on real heads? |
| M5 | M1 | Parameter memory: none, learned product keys, E8 or sign sub-codebooks, n-gram addressed | ≥0.02 nats or factual-recall gain at equal active parameters |

## 5. What the adversarial review changed

| Defect | Severity | Response |
|---|---|---|
| "Exact conversions" are hyperbolic in name only near κ = 0; curvature stays in the flat basin | Critical (science) | Relabelled as flat-limit reparametrisations with prior art. Added the first-order control and the zero-training drive test with a pre-registered rule. The owner's backbone decision moved before M2 |
| M1b did not fit an M1 (about 27 GB for curved arms at batch 4×256) | Critical (feasibility) | Batch 1 with gradient accumulation by default; memory guidance; drive before training |
| The energy thesis compared unlike configurations | Critical (feasibility) | Restated as parity at 4 bits and 2K; the energy gate moved to M1c/M4 |
| Real dot heads are not indexable like synthetic ones | Major (science) | M4 gated on real dumps; geometry assigned to trained memory/index layers |
| Stand-in cannot discriminate between geometries | Major (science) | §2.2 scoped as a machinery check |
| Enclosing-scope gain confounded with copy distance; "ceiling" mislabelled | Major (science) | §0.5 restated |
| Raw κ is not scale-free; the annealing floor could be escaped by shrinking keys | Major (feasibility) | Curvature stated in `t`; the floor is recomputed from the current keys each step |
| `sample` silently ignored saved weights; missing system turn | Major / minor (feasibility) | Fixed (`f43aa4e`) |
| Latent NaN in the unused series branch | Minor (feasibility) | Fixed |
| Literature labels | Minor | Mamba-in-the-Llama's 5.64 is the fully Mamba2 variant; T-MAC's 70% is its 2-bit best case (20.6% at 4 bits) |

## 6. Owner decisions

1. **Hugging Face access.** Allow Hugging Face (huggingface.co and its download hosts) in this environment, or keep real-checkpoint work on the M1.
2. **"No transformer backbone at serving", before M2.** The likeliest near-term chat model is a 4-bit, integer-served SmolLM2 whose heads may stay dot. Options:
   - (a) accept a converted backbone and restate the rule as "no dense float matmul and no multiplier at serving", with geometric memory and index layers added;
   - (b) keep the rule and pursue a native model, which reaches narrow chat at M1 budgets;
   - (c) take (a) now as the chat vehicle, while native geometric components replace parts of it as they prove themselves.
3. **Paid compute.** Ternary quantization-aware training is about 1.3 H100-days per attempt; 4-bit is about one GPU-hour plus overhead.
4. **SIMD.** One small audited SIMD crate with `unsafe` for the NEON table kernels, outside the frozen `forbid(unsafe_code)` crates? Without it the weight path stays 12–16× slower (*Measured*, x86 ratios).
5. **Multipliers for activation products.** The systems reviewer found that tables cost about 1.8× a hardware multiply for element-wise products, and that the multiplier's energy share is negligible. D0-b covers weight maps; attention q·k, gating and value mixing are activation-by-activation products. Allow a hardware integer multiply there?
6. **D5.** Are memory layers addressed by fixed geometric codes allowed? That is sparse parameter access without a learned gate.
7. **Evaluation.** Judge geometry by event-level metrics (long-range copies, entity and scope resolution, long-memory recall) alongside mean NLL?

## 7. Cost of this phase

- **Machine.** One shared review container: 4 cores, 15 GB, no GPU. It ran from about 16:20 to 18:15 UTC, at load 10–30. No paid or external compute.
- **Lead's runs.** The lead's single-threaded stand-in conversion and fine-tune runs recorded 2.9 wall-clock hours in total, under contention: 7,943 s and 2,580 s.
- **Reviewers' runs.** Their costs are in their reports. The experiment reviewer used about 1.2 CPU-hours. The science reviewer's five checks each took under a minute.
- **Not included.** The four context-256 cycle-3 runs are still in progress and are not part of this note.
- **Storage.** Phase-2 scratch outputs are 0.2 GB outside the repository, plus a shared 1.7 GB build cache. The repository gained two Rust source files, one script, this note and a 32 KB evidence file.
