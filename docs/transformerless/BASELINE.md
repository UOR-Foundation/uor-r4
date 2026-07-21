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

Working assumption, to be finalized before the first graph certificate:

- **Continuity partition**: the existing teacher-generated story corpus (`Corpus`,
  `compiler.rs:51`) with its 80/20 construction/held-out split (`train_cut`), so new numbers are
  comparable to the cited P2 certificate.
- **Natural partition (OPEN)**: one redistributable natural corpus, pinned by CID with SPDX
  license recorded in PROV. Candidates: a Simple English Wikipedia sample or TinyStories val
  split. Selection is a Phase 0/2 open task — it must exist before the first HF fidelity
  certificate (issue #34) is meaningful.
- Corpus manifests live under `.uor-models/`; CIDs are referenced in artifact HEAD/PROV.
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
| **SmolLM2-135M HF path fidelity** | **none exists** | **missing** | evaluation-report tooling gap (README.md:203-208); issue #34, Phase 0/2 |

Important: the cited certificate belongs to the legacy llama2.c stories15M teacher, **not** to
the current default SmolLM2-135M-Instruct compile. The Gate C "baseline" for the graph must be
re-measured on the HF path once issue #34 lands.

### 3.2 Artifact sizes (fresh, 2026-07-21, `.uor-models/compiled/smollm2-135m-instruct/`)

| File | Bytes | Note |
|---|---|---|
| `tless_artifacts.bin` (TLA4) | 1,710,348 | codebooks, thresholds, class sigs (incl. certifier-only f32 `ctx_cb`) |
| `tless_store.bin` (TLS1) | 494,286 | graded evidence store |
| `tokenizer.bin` | 528,975 | byte-level BPE export |
| total deployed | 2,733,609 | vs. ~271 MB BF16 source (~99× smaller, nominal) |

Cited for the legacy artifact: 87.2× compression at 0.9692 mean cosine (full depth), 28.1×
end-to-end vs. 60.8 MB source (PROOF.md P5).

### 3.3 Runtime contract

| Metric | Value | Status |
|---|---|---|
| multiplies in runtime kernel | 0 (machine-checked source scan, witness P-4) | cited/enforced |
| integer ops per token | ~1.8×10⁵ | cited (PROOF.md P1, legacy path) |
| op census, SmolLM2 path (fresh, 32 greedy tokens, debug) | **144,498 avg ops/token**: adds 48,526 · xors 36,864 · shifts 11,662 · compares 1,334 · table-reads 46,112 | fresh — `tests/allocation_census.rs` (deterministic across runs) |
| allocations per generated token | **0** (asserted over 32 tokens across `assign_window`, `predict`, `predict_witness`, `generate_greedy_into`; `Runtime::new` also 0) | fresh, Gate B pattern proven |
| allocations at parse/load (fresh) | artifacts: 18 allocs / 1.71 MB; store: **57,092 allocs / 4.69 MB for a 494 KB container** (~9.5× — per-key `Vec<u8>` + `BTreeMap` nodes) | fresh; Phase 1 R4G1 packed layout is the fix |
| allocations on write path (`add_evidence` ×64) | 563 allocs / 44.7 KB | fresh, known; formalized as patch epochs in Phase 9 |
| bytes read / cache misses per token | deferred | needs pinned runner (D7) |
| per-token latency | pending | needs bench harness (criterion) |

### 3.4 Reproducibility

- κ-reproduction (byte-identical recompile) holds on macOS for the legacy path, fixtures in
  `crates/uor-r4-core/tests/fixtures/baseline_kappa.json` (`--release --test kappa_reproduction
  -- --ignored`). Cross-platform byte equality is **not** claimed today (platform SIMD FP in the
  compiler) — this is exactly what D2 addresses.

## 4. M.V.G. checkpoint targets (D1) — DRAFT

To be confirmed by the maintainer before Phase 0 exit. Absolute values are set relative to the
first HF-path certificate (issue #34); reference points are the cited legacy numbers above.

Draft pass conditions for the Phase-5 minimum viable graph, all on the declared distribution (§2):

1. Teacher-argmax agreement ≥ (HF-path TLA3 baseline + 5 percentage points), and in no case
   worse than that baseline (Gate C floor).
2. Bits/token ≤ HF-path baseline − 0.3.
3. Deployed artifact bytes ≤ 2× the current TLA4+TLS1 total (~5.5 MB).
4. Zero allocations per token step (hard requirement, Gate B).
5. Per-token latency ≤ 2× the current runtime on the pinned machine.
6. Novel/Contradictory fallback rate measured and reported; on-distribution rate < 20%.

Missing target 1–2 or 4 ⇒ stop or redesign. Missing 3/5/6 ⇒ redesign discussion.

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
