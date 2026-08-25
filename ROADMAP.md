# Roadmap

Capability checklist. This is the **product** roadmap; the **research**
programme — what has been measured, refuted and left open — lives in
[docs/RESEARCH.md](docs/RESEARCH.md), and the two are tracked separately on
purpose. Research direction is set by measurement and changes faster than this
list does.

_Last reviewed: 2026-08-24 (#933 reopened S0 after a scorer-scope audit: RF-31 is
NOT ESTABLISHED at normative deployed-serving scope pending an exact R4G1Runtime
re-emission and teacher-free census). Prior: 2026-08-22 (S4 closed `LIMIT`)._

> **Post-v0.1 intelligence sequencing lives in the
> [R4 Intelligence Completion Plan](docs/r4_intelligence_completion_plan.md)** —
> the readable mirror of GitHub programme root #820 (stages S0–S7 plus the
> cross-cutting F0 formal lane). This file stays the **product** capability
> checklist and defers the ordering of intelligence work to that plan.

## Next up (recommended sequencing)

The immediate prerequisite is **#933**, the reopened S0 truth-boundary child.
ADR-0001 still names `R4G1Runtime` as the sole normative production
candidate/token selector, but #908 measured `R4Engine` in a reference/off-serving
harness and #910 did not add SKMX/PSIB consumption to `R4G1Runtime`. The existing
29.702% result therefore remains valid empirical reference evidence, not a
deployed-serving result. #933 must unify the production decode surfaces, bind a
versioned deployed-quality report, re-emit the canonical broad graph, and record a
teacher-free RATIFY/LIMIT/RETIRE/UNAVAILABLE verdict before downstream work resumes.

Current order:

1. **#933 — restore normative R4G1 serving reachability and re-ratify RF-31.**
   The #933 implementation restores RF-31's structural scope as
   `normative-runtime` / `deployed-serving`, while its empirical quality remains
   NOT ESTABLISHED until the exact canonical sample/census records RATIFY,
   LIMIT, RETIRE, or UNAVAILABLE.
2. **Then follow the native dependency chain #932 → #931 → #839.** No hours-class
   teacher or parity run is part of #933; its quality evidence is teacher-free over
   the already-recorded canonical labels.
3. **Deferred research and product backlog remains unchanged:** the #784
   context-code-convergence family, the #653 phase-2 formal remnants, the
   #759→#460 codebook-collision root cause, a future vacuity-robust
   route-attention contract, and eventual `uor-r4` alias removal.

## Landed

- [x] **Wrap transformerless into r4** — *done 2026-07-18*. The full
  transformerless program is integrated into
  [`uor-r4-core::transformerless`](crates/uor-r4-core/src/transformerless),
  rebased onto the UOR substrate (`src/tless_uor.rs`: uor-addr addressing,
  `TlessAxis`, per-prediction `Grounded` witnesses), and exposed at
  `/api/tless/{predict,index,generate}`. The old repository is superseded.
- [x] **`up`/`run` locally** — `./uor-r4-cli` orchestrates download → compile →
  score → serve → interactive client in one command.
- [x] **API `/v1/chat`** — OpenAI-compatible `/v1/chat/completions`, plus
  `/v1/models`, `/v1/status`, `/v1/reload`, `/v1/corpus`.
- [x] **Web-based chatbot** — the browser dashboard (`index.html`) with engine
  selector, telemetry, tokens/sec speed metric, semantic map and dev-mode
  toggle.
- [x] **Attestable / audit logging / provenance** — UOR attestation envelopes
  (`uor_address`, `artifact_cid`, `store_cid`, `attestation_cid`),
  `POST /api/uor/verify`, per-turn audit log rendered by `r4 audit`.
- [x] **GNAF vendored (#653 phase 1, PR #742)** — the pinned
  `WASM-GEMM-GNAF` Lean4 proof project is vendored at
  `proofs/wasm-gemm-gnaf/` with full provenance
  (`docs/gnaf_import_provenance.md`). It is a proof that a WASM GEMM kernel
  is cost-optimal, unrelated to text generation; source and provenance only
  — not wired into any pipeline, not in the deployed dependency graph. Phase
  2 (integration matrix, PR #765) landed and #653 closed 2026-08-17; its
  "defer-open" items are currently untracked (see Next up item 3).
- [x] **Ready-by-default serving epic (#655, closed 2026-08-19)** — A–F
  complete: uor-matmul-owned compile arithmetic; shared bundle loader +
  sidecar manifest/verifier; Production/Experimental engine profiles
  (Production default, r4g1-only admission, typed declines); **`r4` as the
  canonical served identity** on every surface (`uor-r4` a deprecated request
  alias); **seeded sampling as the decode default** (pinned seed, greedy
  opt-out; 15/15 valid on the declared in-domain canary vs 0/15 greedy); the
  CLI ask path applying the **same deployed D4 abstention policy** as the
  server tier (#811). Execution records:
  `docs/serving_default_flip_655_f.md` + the #655 close comment.
- [x] **Release v0.1 + pipeline (#741, closed 2026-08-19)** — tag-triggered
  workflow (draft release binding code SHA + contract version; native CLI for
  linux x86_64 + macOS arm64; wasm frontend), the packaged model bundle as an
  attested GitHub Release asset, and the explicit hard-verified
  `r4 install-release --tag v0.1` fetch, proven end to end against the live
  release. `docs/RELEASE_PIPELINE.md`.

## Capabilities

- [~] Text-based AI — compiles, serves, and ships as a verified release;
  **output quality remains the central research problem**. The #755 corpus-
  ordering fix plus the seeded-sampling decode default moved baseline answers
  from word-salad/digit-attractors to valid English sentences (15/15 on the
  declared canary, 2026-08-19), but distinctness is weak (#784: distinct
  prompts converge onto similar completions) and factual quality is
  research-grade; the canonical local bundles still predate the #755
  recompile. See
  [Which track can actually produce coherent text](docs/RESEARCH.md#which-track-can-actually-produce-coherent-text--the-honest-current-answer).
- [ ] Image
- [ ] Audio

Stretch:

- [ ] Video
- [ ] vLLM

## Tooling

- [ ] agentic harness
- [ ] agentic tooling (loop-based calls)
- [ ] `SKILL.md`
- [ ] collaborative prompting
- [ ] Tauri desktop shell

## Cloud service

- [ ] `deploy` (to cloud) — command-based; stack
- [ ] analytics
- [ ] persistent and transient VMs
- [ ] egress/ingress replacement

## Self-host

- [ ] deployment to your own cloud — tooling

## Hologram

- [ ] app SDK

## AI features

- [ ] Identity
- [ ] RBAC / policy
- [ ] analytics
- [ ] storage — document tokenization
- [ ] compression
- [ ] context (storage)
