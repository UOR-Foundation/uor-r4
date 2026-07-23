# Baseline and Working Assumptions — R⁴ Graph Compiler

Phase 0 deliverable of `docs/r4_graph_compiler_implementation_plan.md` (§5 Phase 0).
Date: 2026-07-21. Status: living document; numbers are marked **fresh** (measured this phase),
**cited** (from prior certificates), or **pending** (harness in flight / tool missing).

## 1. Working assumptions for decisions D1–D8

Adopted as working assumptions on maintainer go-ahead (2026-07-21); reversible until the plan is
formally committed. Full text: plan §2.

- **D1** — Commit scope is the M0–M5 tranche. Go/no-go at the M.V.G. checkpoint (§4 below).
- **D2** — Reproducibility semantics: canonical deterministic compiler mode (normative scalar FP,
  pinned seeds, no platform BLAS) for certificate-bearing artifacts; platform-accelerated modes
  allowed for local iteration, validated by behavioral equivalence (PDF §15). Recorded in
  `docs/transformerless/R4G1.md` §7.
- **D3** — Evaluation distribution: declared in §2 below.
- **D4** — Runtime fallback: consult EXCT exact-residual evidence, then abstain with explicit
  status; manifest-declared per status. Chat UX decided when Phase 5 lands the manifest.
- **D5** — Phases 6/7 are trigger-gated on measured region counts / bytes-read counters, not
  scheduled.
- **D6** — Deployment target: **revised by maintainer 2026-07-21** — the primary deployment target
  is another platform (to be named before Phase 1 freezes R4G1 widths and runtime features); wasm
  is demoted to optional. Caller-owned-bytes-first remains as a target-neutral design rule.
- **D7** — Process: backlog filed as GitHub issues #11–#34 under Phase milestones on
  `UOR-Foundation/uor-r4`; work continues on `feature/proof-carrying-semantic-routing` with the
  graph path cfg-gated; benchmark/reproducibility gates run on the pinned dev machine until a
  pinned CI runner exists.
- **D8** — This plan serves the research thesis. Product-usefulness gate (chat quality eval)
  required before funding Phases 8–10.

## 2. Evaluation distribution declaration (D3)

**Decided 2026-07-22 (maintainer):**

- **Continuity partition**: the existing teacher-generated story corpus (`Corpus`,
  `compiler.rs:51`) with its 80/20 construction/held-out split (`train_cut`), so new numbers are
  comparable to the cited P2 certificate.
- **Natural partition**: a pinned **Simple English Wikipedia** sample, license **CC BY-SA 4.0**
  (recorded in PROV as SPDX `CC-BY-SA-4.0`). Sizing and split rules are fixed at first use
  (target: a few thousand articles, construction/held-out by content hash), pinned by CID; the
  corpus manifest lives under `.uor-models/` and its CID is referenced in artifact HEAD/PROV.
  (Alternative considered: TinyStories validation split — cleaner licensing-wise but keeps the
  evaluation synthetic; rejected in favor of a genuinely natural distribution.)
- All fidelity claims carry: distribution id, n, confidence interval, slices, seeds, stopping
  rule (Gate K). No claim generalizes beyond the declared distribution.

## 3. Baseline measurements

### 3.1 Fidelity

| Metric | Value | Status | Source |
|---|---|---|---|
| top-1 accuracy | 28.9% | cited | PROOF.md P2 (legacy stories15M teacher, ~10⁵ store keys) |
| teacher-argmax agreement | 31.7% | cited | PROOF.md P2 |
| bits/token (WB) | 6.54 (teacher floor 1.5960, ceiling 70.4%) | cited | PROOF.md P2 |
| store keys | 89,200 | cited | PROOF.md P2 |
| HF-path evaluation tooling | exists | landed | PR #41 (`evaluate-report`); issue #34 closed |
| **Gate C harness (Phase 4)** | TLA3 store baseline 31.7% / 11.88 bits-token | fresh, 2026-07-22 | `r4 transformerless score`, fixture corpus, 30,036 held-out positions — reproduces the P2 agreement anchor; bits/token is the scorer's own accounting, not the P2 WB metric |
| **Gate C: graph formula v1 (Σ-over-cloud)** | **0.3% / 70.47 bits-token** | fresh, unfavorable | correlated sibling-subtree residual stacking (issue #64, redesign in flight) |
| **Gate C: graph formula v2 (chain-telescoped + EXCT precedence)** | measured on re-compile | pending | issue #64 redesign implemented; re-run `r4 transformerless score` on the fixture corpus after recompiling the graph artifact |

Important: the cited certificate belongs to the legacy llama2.c stories15M teacher, **not** to
the current default SmolLM2-135M-Instruct compile. The Gate C harness reproduces its 31.7%
agreement anchor on the fixture corpus; HF-path certificates for the SmolLM2 compile are
producible via the PR #41 tooling on the D3 distribution (§2).

### 3.2 Artifact sizes (fresh, 2026-07-21, `.uor-models/compiled/smollm2-135m-instruct/`)

| File | Bytes | Note |
|---|---|---|
| `tless_artifacts.bin` (TLA4) | 1,710,348 | codebooks, thresholds, class sigs (incl. certifier-only f32 `ctx_cb`) |
| `tless_store.bin` (TLS1) | 494,286 | graded evidence store — **stale: u16-era entries (6 B); current u32 parser rejects it. Regenerate via recompile; incident validates R4G1's versioning rules** |
| `tokenizer.bin` | 528,975 | byte-level BPE export |
| total deployed | 2,733,609 | vs. ~271 MB BF16 source (~99× smaller, nominal) |

Cited for the legacy artifact: 87.2× compression at 0.9692 mean cosine (full depth), 28.1×
end-to-end vs. 60.8 MB source (PROOF.md P5).

### 3.3 Runtime contract

All fresh numbers verified 2026-07-21 by `tests/allocation_census.rs` against the real SmolLM2
artifacts (deterministic across runs; debug profile).

| Metric | Value | Status |
|---|---|---|
| multiplies in runtime kernel | 0 (machine-checked source scan, witness P-4) | cited/enforced |
| integer ops per token | ~1.8×10⁵ | cited (PROOF.md P1, legacy path) |
| op census, SmolLM2 path (32 greedy tokens) | **144,496 avg ops/token**: adds 48,530 · xors 36,864 · shifts 11,666 · compares 1,324 · table-reads 46,112 | fresh |
| allocations per generated token, steady state | **0** (asserted over 32 tokens across `assign_window`, `predict`, `predict_witness`, `generate_greedy_into`; `Runtime::new` also 0) | fresh, Gate B pattern holds |
| allocations, warm-up (finding) | **5 allocs / 496 B during the first ~34 predictions** — `Runtime.recent` (repetition guard) is a `Vec` that grows to steady-state capacity. The "allocation-free hot path" is amortized, not unconditional. Graph-runtime fixed-capacity `RuntimeState` (Phase 5) removes this by construction | fresh |
| allocations at parse/load | artifacts: 18 allocs / 1.71 MB; store (real, legacy TLS1-u16 parse): **57,498 allocs / 5.40 MB for a 494 KB container** (~10.9× — per-key `Vec<u8>` + `BTreeMap` nodes) | fresh; Phase 1 R4G1 packed layout is the fix |
| allocations on write path (`add_evidence` ×64) | 563 allocs / 51.1 KB | fresh, known; formalized as patch epochs in Phase 9 |
| bytes read / cache misses per token | deferred | needs pinned runner (D7) |
| per-token latency | pending | needs bench harness (criterion) |

### 3.4 Reproducibility

- κ-reproduction (byte-identical recompile) holds on macOS for the legacy path, fixtures in
  `crates/uor-r4-core/tests/fixtures/baseline_kappa.json` (`--release --test kappa_reproduction
  -- --ignored`). Cross-platform byte equality is **not** claimed today (platform SIMD FP in the
  compiler) — this is exactly what D2 addresses.
- **Baseline anchor moved 2026-07-21** (maintainer decision): the pin was stale from
  `b142c93`-era after two deliberate compiler redesigns — `5baa7c0` (phase-10: u32 token IDs, new
  corpus record layout with top-3 tokens/weights, oracle separation) and `bbdd596` (hash-index
  RVQ projection, relational prefixes). Investigation before re-pinning: (1) compiler determinism
  verified — two independent compiles produced identical 27-κ sets; (2) stage-0 drift traced to
  those redesigns, not to nondeterminism or platform wobble. The same u32 migration is what
  invalidated the on-disk TLS1 store (§3.2). Re-pinning helper: `dump_baseline_kappa` in
  `tests/kappa_reproduction.rs`. Lesson recorded for Gate E: an unversioned baseline plus a
  redesigned compiler = a broken reproduction gate; R4G1's HEAD records compiler identity for
  exactly this reason.

## 4. M.V.G. checkpoint targets (D1) — CONFIRMED

**Confirmed by the maintainer 2026-07-22 ("all defaults", unamended).** Absolute values are set
relative to the first HF-path certificate (PR #41 tooling); reference points are the cited
legacy numbers above. These are the go/no-go contract for the Phase-5 checkpoint review:
missing 1–2 or 4 ⇒ stop or redesign; missing 3/5/6 ⇒ redesign discussion.

Pass conditions for the Phase-5 minimum viable graph, all on the declared distribution (§2):

1. Teacher-argmax agreement ≥ (HF-path TLA3 baseline + 5 percentage points), and in no case
   worse than that baseline (Gate C floor).
2. Bits/token ≤ HF-path baseline − 0.3.
3. Deployed artifact bytes ≤ 2× the current TLA4+TLS1 total (~5.5 MB).
4. Zero allocations per token step (hard requirement, Gate B).
5. Per-token latency ≤ 2× the current runtime on the pinned machine.
6. Novel/Contradictory fallback rate measured and reported; on-distribution rate < 20%.

Missing target 1–2 or 4 ⇒ stop or redesign. Missing 3/5/6 ⇒ redesign discussion.

### 4.1 Canonical bits/token definition (issue #65 Chain 4, item 13)

Two bits/token figures appear in this document; they are distinct metrics and both are reported:

| Metric | Definition | Where reported | Current value |
|---|---|---|---|
| **Gate C bits/token** | Scorer's own accounting: for each held-out position the scorer scores all candidates; bits/token = −log₂(P_scorer(next_token)) where P_scorer comes from the scored candidate distribution. The scorer's candidate set may not include the held-out token (→ floor probability). This is the metric used in `evaluate_gate_c` and the `score_report.json`. | Gate C table (§3.1 above), `score_report.json`, `r4 transformerless score` output | 11.88 (TLA3 baseline), 70.47 (graph formula v1) |
| **Witten-Bell (WB) bits/token** | Full-vocabulary probability under the Witten-Bell smoothed graded store, as in the P2 certificate and `evaluate-report`. Computed over the full vocabulary, not just the candidate set. This is the metric cited in PROOF.md. | `instruction-eval.json`, `evaluate-report` command | 6.54 (P2 WB, legacy llama2.c teacher) |

**Decision**: the Gate C bits/token (scorer accounting) is the normative quality metric for the
graph path because it measures what the scorer would actually select, not the WB smoothing floor.
Target M.V.G. §4 item 2 ("bits/token ≤ HF-path baseline − 0.3") uses the **Gate C bits/token**.
The WB figure is retained in certificates for research continuity but is NOT the gating metric.

## 5. Threat-model note (backlog #22)

Full adversarial model: `docs/transformerless/THREAT_MODEL.md`. Headline threats: crafted region
activation, overlap poisoning, frontier/candidate exhaustion, fallback denial-of-service, integer
saturation, collision with privileged concepts. Defenses: strict fan-out/frontier limits,
validated routing bytecode, checked integer semantics, bounded patch layers, adversarial collision
suites, separation of semantic routes from cryptographic identity.

## 6. Phase 0 exit status

- [x] Glossary frozen (`docs/transformerless/GLOSSARY.md`)
- [x] R4G1 RFC drafted (`docs/transformerless/R4G1.md`)
- [x] D1–D8 working assumptions recorded (§1)
- [x] Backlog filed as issues #11–#34 with phase milestones (D7)
- [~] Baseline measurements: artifact sizes, allocation census, and op census fresh
  (`tests/allocation_census.rs`); fidelity pending issue #34; bytes-read/latency pending bench setup
- [x] Threat model written (`docs/transformerless/THREAT_MODEL.md`)
- [ ] M.V.G. targets confirmed by maintainer (§4)
