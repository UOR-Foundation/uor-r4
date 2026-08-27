# UOR-R4 documentation map

This page is the entrypoint for documentation under `docs/`. The current
project is the **route-native geometric-intelligence programme**: canonical
lexical data receives factorable geometric addresses, spherical-harmonic spin
state carries causal context, and a bounded recursive route hierarchy is
intended to replace learned attention, inference, and reasoning machinery.

**Current product truth:** #958 established a source-free storage/recall and
bounded-query foundation. It did **not** establish geometric attention, a
source-free decoder, or working source-free chat. The dependency order is
`#961 -> #952 -> #953 -> #954 -> #955`, followed by bounded product/release work
in `#962-#965`. Do not use an older TLA/R4G1, graph-compiler, or hybrid
source-model document to infer a later capability.

## Start here

1. [Geometric Intelligence Programme](geometric_intelligence_programme.md) —
   current purpose, complete mechanism set, sequencing, and claim boundaries.
2. [ADR-0004: route hierarchy](adr/0004-geometric-intelligence-route-hierarchy.md)
   — local, sentence, paragraph, conversation, and global causal state.
3. [Geometric Intelligence Evaluation Policy](geometric_intelligence_evaluation.md)
   — decision-bearing probes, matched controls, anti-recall evaluation, and
   long-run limits.
4. [Glossary](transformerless/GLOSSARY.md) and
   [Formal Vocabulary](formal_vocabulary.md) — canonical terms, notation, and
   permitted claim language.
5. [Research ledger](RESEARCH.md) — measured outcomes and their exact scope.

## Status classes

Every living entrypoint should fit one of these classes. A document's own
status banner wins when it is more specific.

### Current authority

These documents define what the project is building now.

| Document | Authority |
|---|---|
| [Geometric Intelligence Programme](geometric_intelligence_programme.md) | Post-#958 architecture, work order, mechanisms, and product boundary. |
| [ADR-0003](adr/0003-fixed-zeta-prime-route-attention.md) | Fixed-zeta prime-route foundation retained by #958. |
| [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md) | Recursive causal route scopes and the load-bearing paired-H4/E8 bridge. |
| [Geometric Intelligence Evaluation Policy](geometric_intelligence_evaluation.md) | What may be evaluated, in what order, and at what cost. |
| [Formal Vocabulary](formal_vocabulary.md) | Normative claim classes and notation. |
| [Glossary](transformerless/GLOSSARY.md) | Structural terminology across current and preserved lanes. |

If these disagree with an older plan, use the current programme and ADRs. Live
GitHub dependencies still govern implementation eligibility.

### Active implementation documentation

These describe current code surfaces or the immediate route-native build, but
do not independently authorize capability claims.

| Document | Scope |
|---|---|
| [Model lifecycle](MODEL_LIFECYCLE.md) | Current target lifecycle first; preserved teacher/TLA/R4G1 commands afterward. |
| [Configuration](CONFIGURATION.md) | Workspace option inventory spanning current foundation and preserved runtimes. |
| [Compiler stage DAG](compiler_stage_dag.md) | Existing deterministic compiler scheduling reference. |
| [Compiler concurrency](compiler_concurrency_config.md) | Existing worker/configuration reference; not proof that route-native product work is parallel. |
| [Compiler memory budget](compiler_memory_budget.md) | Existing resource-control reference. |
| [Prime-route #958 qualification](prime_route_attention_qualification_958.md) | Foundation implementation and terminal `RETAIN_STORAGE_RECALL_ONLY` boundary. |

The current capability sequence is:

```text
#961 lexical geometry/state plumbing
    -> #952 complete recursive geometric attention
    -> #953 source-free grammatical inference and generation
    -> #954 correctness and typed abstention
    -> #955 bounded reasoning
    -> #962-#965 product integration, serving purity, cost, and release
```

Lexical serialization is prerequisite plumbing, not attention. Attention is
not inference. Readable output is not correctness, and correctness is not
reasoning.

### Historical research and measurement records

These records are preserved evidence. Their positive and negative results
remain valid only for their declared artifact, population, selector, and
execution path; they do not define current sequencing.

- [Geometric Causal Decoder Roadmap](geometric_causal_decoder_plan.md) —
  historical #948-#958 sequence, superseded by the current programme.
- [R4 Intelligence Completion Plan](r4_intelligence_completion_plan.md) —
  historical S0-S7 programme.
- [R4-native architecture](r4_native_architecture.md) — historical 2026-08
  research/design record.
- [Graph compiler implementation plan](r4_graph_compiler_implementation_plan.md)
  — preserved historical engineering plan.
- [Research ledger](RESEARCH.md) and issue-numbered Markdown/JSON records —
  append-only measurement evidence.
- [Prime-router geometric-context evidence](prime_router_geometric_context_evidence.md)
  — ancestor trajectory evidence; it used Ollama for language and is not
  source-free generation evidence.

An issue-numbered result file is not a living roadmap unless a current
authority document explicitly adopts it.

### Preserved runtime references

These documents still describe retained, testable components. They are not the
route-native intelligence architecture and do not show that source-free chat
works.

| Document | Preserved scope |
|---|---|
| [Transformerless cross-compiler](transformerless/TRANSFORMERLESS.md) | TLA table compiler and historical multiplication-free runtime. |
| [R4G1 wire format](transformerless/R4G1.md) | Packed graph container and parser/runtime contract. |
| [Inference operation contract](inference_contract.md) | CPU-only integer hot-path rules for the preserved TLA/R4G1 runtime. |
| [Local-only contract](transformerless/LOCAL_ONLY.md) | Provider boundary for the preserved transformerless serving lane. |
| [Release pipeline](RELEASE_PIPELINE.md) | Existing R4G1/TLA bundle packaging and distribution. |
| [Serving model discovery](SERVING_MODEL_DISCOVERY.md) | Historical audit of legacy loader/cascade surfaces. |
| [Minimal client](minimal_client.md) | Existing terminal/API surface; not evidence of route-native answers. |
| [Proof and certificate](transformerless/PROOF.md) | Scoped historical runtime witnesses and measurements. |

## Read by task

- **Understand the goal:** programme -> ADR-0004 -> glossary.
- **Implement #961 or later:** programme stage -> applicable ADR -> evaluation
  policy -> live GitHub dependency.
- **Interpret a number:** research ledger -> named measurement record -> exact
  artifact and denominator.
- **Operate preserved tooling:** model lifecycle -> configuration -> relevant
  runtime reference.
- **Explain the project:** [ELI5](explainers/ELI5.md) or
  [Undergraduate](explainers/UNDERGRADUATE.md), then the programme.

## Documentation rule

New living documents should link to the programme rather than restating its
mechanisms. Historical records remain intact; add a status/scope banner when
their original present-tense wording could be mistaken for current authority.
