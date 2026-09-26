# G0a result: canonical-address cache cheap-predictability feasibility

**Status:** branch-local evaluation-only result, 2026-09-25. References the CAR-LM
plan ([§3, §6, §7](canonical-address-routing-plan-2026-09-25.md#7-gates-cheap-first-each-falsifiable))
and the D8 rung-1/rung-2 parents. No training, no artifact change, no serving
claim. Branch `codex/canonical-address-routing-20260925` at base `8a9c3dcf`.

**Verdict: the hand-built canonical-address cache does not meet the project's
n-gram quality bar. Do not advance to G0b composition or G1 learned routing on
this address design as built.** It passes only against an order-2/3 raw-count
strawman; it fails against the retained evaluator-v2 order-5 count baseline on
the same population and is dominated on quality-per-op by a two-token
exact-context cache.

## 1. What was measured and how

The task's state cannot be read from the checkpoint (it is not persisted), so a
new **evaluation-only** subcommand, `joint-cache-eval`, was added to
`uor-r4-training`. It loads the retained continuous checkpoint through the
unchanged learner, runs the read-enabled forward over the exposed evaluator-v2
development blocks, and records the recurrent state (`JointOutput.states`) and
the model distribution at every target position. The token-budget controls and
n-gram tables run beside it. No optimizer step, no data change, no artifact
write.

- **State source:** `JointModel::forward(..., ReadMode::Enabled, false)` per
  256-token block, batch 8. `states[b, p, :]` is the post-read/write state that
  emits the target at input offset `b*256+p+1`.
- **Canonical address arms:** the task under test.
  - `group2i120`: signed aggregate of the 64 R4 lanes -> nearest exact `2I` root
    (120 elements) using the existing `canonical_h4_roots_q30` table; `q` and
    `-q` stay distinct.
  - `voronoi{V}`, `V ∈ {64,120,256,1024}`: deterministic k-means over the full
    256-d recurrent state, fit on the 64 tune blocks only (12 Lloyd iterations).
- **Cache:** causal, keyed by address; warmed from the 64 tune blocks, then for
  each comparison position predicts the majority prior next token (first-prior
  reported separately) with an add-α smoothing and unigram backoff on a miss.
  A frozen tune-only (static) cache is reported too.
- **Controls at matched access:** exact-context cache of the last `k` tokens,
  `k ∈ {2,4,8}` (block resets); order-2/order-3 interpolated count n-gram fit on
  a 6M-token prefix of the first retained train store; the learner's own
  readout; whole-prefix NoRead; unigram.
- **Ops:** cache path counts address computation plus one table read, using the
  `uor-r4-integer` inspection convention. "Sticky" means the address computation
  is paid only when the quantised address changes; otherwise 1 compare. Dense
  denominator is the retained step-1 measurement, **1,672,448** learned-code
  inspections per full-read step.

Commands (worktree `/Users/casey.allard/uor-r4/.worktrees/canonical-address-routing-20260925`):

```sh
UOR_BUILD_SOURCE_COMMIT=8a9c3dcf5159d0b32873a04652eedf8fdcf3dc13 \
  cargo build --release --offline -p uor-r4-training --bin uor-r4-training --features cpu-accelerate

cargo test --release --offline -p uor-r4-training --lib --features cpu-accelerate joint_cache_eval

target/release/uor-r4-training joint-cache-eval \
  /Users/casey.allard/uor-r4/.uor-models/investigations/quantized-recurrent-20260925/fit-quaternion-continuous-1/checkpoint-final \
  docs/integration/reference-evaluator-v2.json \
  /Users/casey.allard/uor-r4/.uor-models/investigations/canonical-address-cache-20260925/quaternion-continuous-1/attempt-1 \
  cpu 8 6000000
```

Identities: executable SHA-256
`77c8dae2f44054617b315f2f1480558e51a56fa81ab004473f0c3666429fe160`; checkpoint
weights `00ee447cf7b00644c078d03794f1127cf68e772ec6cf6c236fdb53a8a2d27885`;
evaluator `d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62`;
1,678,466 parameters; 233,472 comparison targets; 390 s wall. The result was
produced with that tested binary; `rustfmt` was applied afterwards and a rebuild
compiles cleanly and passes the five focused tests, but the rebuilt binary has a
different SHA-256 (`b2df56dc…`), so release builds here are not bitwise
reproducible and the tested-binary identity above is the binding one.

## 2. Instrument check (the learner reproduces)

| Arm | NLL (nats/token) | top-1 |
|---|---:|---:|
| This run, read-enabled | 2.0905 | 51.23% |
| Retained rung-1 quaternion read-enabled | 2.1104 | — |
| This run, whole-prefix NoRead | 2.5782 | 44.38% |
| Retained rung-1 quaternion NoRead | 2.5617 | — |

The readout reproduces the retained artifact to within the continuous-1 vs rung-1
checkpoint difference and feature-arithmetic noise, so the state dump is
instrumented correctly.

## 3. Pareto: quality vs ops (233,472 comparison targets)

| arm | kind | hit% | top-1% | NLL | sticky ops/token | % dense | % replaced (vocab) | literal G0a |
|---|---|---:|---:|---:|---:|---:|---:|---|
| group2i120 | fixed group cell | 100.0 | 10.75 | 5.7999 | 993 | 0.059 | 0.095 | pass |
| voronoi64 | k-means state | 100.0 | 22.91 | 4.1985 | 16,385 | 0.980 | 1.563 | pass |
| voronoi120 | k-means state | 100.0 | 24.43 | 4.1866 | 30,721 | 1.837 | 2.930 | pass |
| voronoi256 | k-means state | 100.0 | 26.77 | 4.2418 | 65,537 | 3.919 | 6.250 | pass |
| voronoi1024 | k-means state | 100.0 | 31.57 | 4.6608 | 262,145 | 15.674 | 24.999 | **cost miss** |
| exact_context_k2 | token hash | 82.5 | 29.76 | 5.6906 | 4 | 0.0002 | — | pass |
| exact_context_k4 | token hash | 30.7 | 19.23 | 5.9884 | 6 | 0.0004 | — | quality miss |
| exact_context_k8 | token hash | 4.6 | 9.98 | 5.8830 | 10 | 0.0006 | — | quality miss |
| **model readout** | recurrent | — | 51.23 | 2.0905 | 1,672,448 | 100 | — | reference |
| model NoRead | recurrent | — | 44.38 | 2.5782 | 1,672,448 | 100 | — | control |
| unigram | count | — | 7.50 | — | 1 | ~0 | — | floor |
| ngram order-2 | count | — | 5.49 | 6.0242 | 3 | ~0 | — | control |
| ngram order-3 | count | — | 3.49 | 6.0666 | 4 | ~0 | — | control |
| **evaluator-v2 order-5 count (retained)** | count | — | — | **2.4056** | ~1 | ~0 | — | project n-gram |

Two facts govern the verdict.

1. **The order-2/3 comparator is a strawman.** Its NLL (6.02) is worse than the
   model's NoRead (2.58) and its top-1 (5.5%) is below the unigram floor (7.5%):
   the raw interpolated counts have no absolute discounting, so singleton
   contexts dominate and its calibrated `λ = 0.3` sits at the grid edge. The
   project's actual count baseline on this exact comparison tail is the retained
   evaluator-v2 **order-5** model at **2.4056** nats
   ([joint-recurrent-result](joint-recurrent-result-2026-09-25.md#natural-likelihood-and-combined-read-contribution)),
   i.e. 3.6 nats better than the cache arms' best.
2. **Quality-per-op is dominated by the cheap token cache.** `exact_context_k2`
   reaches 29.76% top-1 at 4 ops; `voronoi1024` reaches 31.57% at 262,145 ops —
   a ~65,000x cost for +1.8 points. The state address does carry real signal
   (31.6% vs 7.5% unigram) but not a competitive one.

## 4. Hit rate is degenerate; address stability is the revealing number

Every address arm reports **100% causal hit** and **100% static hit**: the 64
tune blocks already contain every cluster of every codebook up to V=1024, so the
criterion "hit ≥ 50%" is met by construction and measures nothing about fresh
predictability. The informative quantity is **address stability** — the fraction
of consecutive positions whose quantised address is unchanged:

| arm | address stability% | hit% on stable addresses |
|---|---:|---:|
| group2i120 | 5.01 | 100 |
| voronoi64 | 6.96 | 100 |
| voronoi120 | 5.17 | 100 |
| voronoi256 | 3.26 | 100 |
| voronoi1024 | 1.71 | 100 |

Because stability is ≤7%, the event-driven "spend energy in proportion to
surprise" amortisation gives essentially **nothing**: sticky median ops equal
always-recompute ops for every arm. The continuous recurrent state moves to a
different cell almost every token, including at the coarse 120-element group
cell. Causal hit rate by position-in-block and by gap since the last
same-address occurrence is therefore flat at 100% for every address arm (full
arrays in the sealed report and evidence JSON); the non-degenerate variants are
the exact-context caches:

| arm | hit% 0-15 | 16-31 | 32-63 | 64-127 | 128-191 | 192-255 |
|---|---:|---:|---:|---:|---:|---:|
| exact_context_k2 | 77.0 | 83.4 | 82.9 | 82.4 | 83.2 | 82.7 |
| exact_context_k4 | 25.9 | 30.9 | 31.9 | 30.4 | 31.5 | 30.7 |
| exact_context_k8 | 2.7 | 4.6 | 5.1 | 4.7 | 5.0 | 4.5 |

## 5. G0a verdict against the predeclared criteria

Predeclared: quality = cache NLL ≤ the n-gram's and top-1 ≥ the n-gram's; cost =
sticky median cache ops ≤ 10% of 1,672,448; hit = causal hit ≥ 50%.

- **Against the order-2/3 comparator as implemented:** passes for
  `group2i120`, `voronoi64/120/256`; `voronoi1024` fails cost. This is the
  literal pass recorded by the command's own `verdict`.
- **Against the project's n-gram-of-record (evaluator-v2 order-5 count,
  2.4056 NLL):** **every** cache arm fails quality (best 4.1866) and the
  exact-context controls fail too. The predeclared **kill criterion** — "cache +
  compose cannot match an n-gram's quality per op" — is met.

The latter governs. G0a therefore returns the **kill branch for a hand-built
continuous-state address cache**; the mechanism is not falsified as a source of
extra information (the state beats the unigram and the two-token cache), but it
is falsified as a cheap route to n-gram-level quality.

## 6. Limitations

- Cache-path ops count address computation plus table read; the recurrence that
  produces the address is not counted, while the dense denominator counts the
  full learned step. This is the plan's framing and is generous to the cache.
- The order-2/3 n-gram is raw interpolated counts, not the evaluator-v2
  discounted/pruned count model; only the retained order-5 number is used as the
  project bar, and it is a retained measurement rather than a re-execution here.
- The 100% hit rate is a tune-saturation artifact, not a fresh-text hit rate.
- Floating-point offline learner state; `2I` is used only as a fixed codebook,
  not as a canonical integer/group state. No integer or energy claim.
- One seed, one arm (quaternion), exposed development text. The matched ordinary
  arm is **NOT_RUN**. Cache arms also lack a shuffled-state negative control.
- No fresh final evaluation; the comparison tail was previously exposed.

## 7. Files changed

- `crates/uor-r4-training/src/joint_cache_eval.rs` (new, evaluation-only).
- `crates/uor-r4-training/src/lib.rs` (+1 `pub mod`).
- `crates/uor-r4-training/src/joint_campaign.rs` (+4-line dispatch).
- `docs/evidence/canonical-address-cache-2026-09-25.json` (sealed command output).
- this document.

Sealed local report root:
`/Users/casey.allard/uor-r4/.uor-models/investigations/canonical-address-cache-20260925/quaternion-continuous-1/attempt-1`
(`attempt.json`, `cache-eval.json` SHA-256 `7cc2ff0b…`, `summary.md`, `manifest.json`).

## 8. Resource cost

Build 6m04s (cold local-crate chain under `cpu-accelerate`), tests 0m30s, one
evaluation run 390 s wall at up to ~2.2% RSS (~256 MB state buffer) under a
concurrent host load average of ~26. Report 100 KB. No external compute, no
checkpoint or corpus write.

## 9. Immediate next step

**Kill G0b composition and G1 learned routing as the next work on this address
design.** The cheap-predictability premise fails on the retained learner: the
address is unstable (≤7%) and the cache leaves ~1.8 nats against the count
baseline at a cost that is already 4-16% of dense.

If the lead wants to preserve one branch before closing it, the cheapest
decision-bearing successor is a **product address** (state cell × recent-token
code, no training), evaluated against the same population with one predeclared
bar: **NLL ≤ 2.4056 and top-1 ≥ the retained order-5 count at ≤10% dense ops**.
If the product address cannot beat the count model — which already achieves
2.4056 at ~O(1) ops — then the state carries no exploitable next-token
information beyond local context and the branch closes. A genuinely different
question, not this one, is whether the state adds *long-range* information that
a 5-gram lacks (the plan's G2 contest); that is the only remaining reason to
keep state-conditioned addressing alive.
