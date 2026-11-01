# Which loaded model artifact produced this result?

Status: **draft for external discussion, not an accepted OpenTelemetry convention and not an upstream submission**. The local UOR consumer and official Collector Contrib 0.164.0 interoperability test are evidence that OTLP/HTTP JSON can carry this information, not that these names have community approval.

## Problem

A serving system can report a model name without identifying the bytes that produced a response. Operators of self-hosted models need to answer a different question: **which exact loaded artifact, tokenizer, or related payload was active?** A name such as `gen_ai.request.model` does not answer it, and `service.version` is not a substitute.

The generic need applies across runtimes and model formats. It does not depend on UOR geometry, prime addresses, or any particular hashing library.

## Proposed discussion shape

Discuss optional **resource** attributes for loaded artifact identity:

| Draft key | Type | Meaning |
|---|---|---|
| `gen_ai.model.artifact.digest` | string | Full content digest, including an algorithm prefix such as `sha256:`. Never a trace or span ID. |
| `gen_ai.model.artifact.digest.algorithm` | string | Digest algorithm, making encoding/policy explicit even when a backend expects a bare digest. |
| `gen_ai.model.artifact.byte_size` | int | Exact number of payload bytes covered by the digest. |
| `gen_ai.model.artifact.identity_kind` | string | What the bytes represent, such as `weights`, `weights+tokenizer`, or `codebook`; prevents a payload digest being mistaken for a model, executable, or source revision. |

Names are proposals for review, not claims that these keys exist today. The current GenAI registry has model-name fields but no loaded-artifact digest convention.

Example for a small synthetic model payload (not the UOR codebook):

```text
gen_ai.model.artifact.digest = "sha256:<64 lowercase hex characters>"
gen_ai.model.artifact.digest.algorithm = "sha256"
gen_ai.model.artifact.byte_size = 1048576
gen_ai.model.artifact.identity_kind = "weights"
```

The digest covers the exact loaded payload bytes. A different tokenizer, representation, or executable needs its own explicitly scoped identity rather than overloading this value. Trace IDs remain random 16-byte values and span IDs random 8-byte values. SHA-256/BLAKE3 digests stay full metadata values.

## Privacy and cardinality

- Do not collect prompt/completion text or hash user turns. Low-entropy user content can be brute-forced even when hashed.
- Put artifact identity on the resource, not on every operation and not in metric labels. A content digest is high-cardinality metadata, not a useful metric dimension.
- Do not use the digest as a trace/span ID, persistent machine identifier, or cross-request user correlator.
- Keep collection opt-in. A local file or explicit collector is an experiment, not a default phone-home path.

## Evidence and current gaps

The standalone C++ consumer's four-span trace was accepted by official Collector Contrib `0.164.0`, with HTTP 200, correct parent links, no dropped attributes, and the exact codebook SHA-256/512-byte metadata decoded. That validates transport and identity separation for this example. It does **not** validate the proposed `gen_ai.*` names, an actual loaded language model, an OpenTelemetry SDK, multiple runtimes, production export, or upstream acceptance.

Before an upstream PR, provide at least one real loaded-model consumer and a second independent runtime/example. The UOR fixed-codebook example is useful transport evidence but is not itself evidence of model-instrumentation coverage.

## Upstream route

The current `semantic-conventions-genai` [contributing guide](https://github.com/open-telemetry/semantic-conventions-genai/blob/main/CONTRIBUTING.md) requires YAML model changes under `model/`, generated documentation, `make generate-all`, `make check-policies`, reference-scenario updates, and a Towncrier fragment; link checks use `mise run links`. New contributors also need the CLA and community prerequisites. Keep the change small.

A responsible sequence is:

1. Open a discussion issue with the generic operator problem, draft names, and the real collector evidence; ask whether resource-level artifact identity fits the GenAI SIG and whether the algorithm should be embedded only in the digest rather than duplicated as a second attribute.
2. Add the loaded-model and second-runtime examples plus privacy/cardinality review before proposing YAML.
3. If invited, make a small PR with the model YAML, generated docs, reference scenarios, policy checks, and changelog. UOR-specific geometry and `uor.*` names stay outside it.

No issue, Slack post, or PR has been opened by this unit. This document is preparation, not an announcement of acceptance.
