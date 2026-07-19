# Roadmap

p0:

- [x] Wrap in transformerless into r4 (or inverse) — **done 2026-07-18**: the
  full transformerless program is integrated into
  [`uor-r4-core::transformerless`](crates/uor-r4-core/src/transformerless)
  (κ-reproduction proven bit-exactly),
  rebased onto the UOR substrate (`src/tless_uor.rs`: uor-addr addressing,
  `TlessAxis`, per-prediction `Grounded` witnesses), and exposed at
  `/api/tless/{predict,index,generate}`. The old repository is superseded.

Capabilities:

- [~] Text-based AI
- [ ] Image
- [ ] Audio

Stretch capabilities:

- [ ] Video
- [ ] vLLM

Tooling:

- [ ] `up`/`run` (locally)
- [ ] API `/v1/chat` (server/locally)
- [ ] agentic harness
- [ ] web-based chatbot
- [ ] agentic tooling (loop-based calls)
- [ ] `SKILL.md`
- [ ] collaborative prompting
- [ ] tauri

Cloud service:

- [ ] `deploy` (to cloud)
    - [ ] command-based
    - [ ] stack
- [ ] analytics
- [ ] persistent and transient vms
- [ ] egress/ingress replacement
- [ ] attestable, traceability, audit logging, data provanance

Self-host:

- [ ] deployment to your own cloud
    - [ ] tooling

Hologram:

- [ ] app SDK

AI Features:

- [ ] Identity
- [ ] RBAC/policy
- [ ] analytics
- [ ] storage
    - [ ] document tokenization
- [ ] compression
- [ ] context (storage)
