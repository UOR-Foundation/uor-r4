# Isolated runtime readiness — #1096

**2026-09-03: contract and implementation awaiting independent review; readiness
execution is `NOT_RUN`.** This is the separate child of open #1094, based on
`df2c4cb8ef47e35b6d4083d4b8da135c7676fc19`. It does not execute the adapter
comparison, prepare another population, or release withheld inputs.

## Preserved evidence and the named decision

The original [#1094 unavailable preparation](r4_text_clause_adapter_1094.md)
remains immutable: stop SHA256
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`,
binding SHA256 `baeb56a632c0e75c0574d9bad503e7849cf434039490e1fe92e3b8f0114b1f0b`.
Its authoring 320/320 and refusal 16/16 results did not qualify worker startup,
isolation, model preservation or withheld replay. The OS denied `execvp` before
Python startup. Supplied segmentation is still the qualified model entry.

**Empirical Criterion.** Can the actual comparison worker start with the fixed
Python 3.12.14 / Torch 2.7.1 CPU runtime, verify the exact source and artifact
identities, and demonstrate denied harmless file access while constructing no
model and performing zero forwards?

**Definition — candidate correction.** Keep the same source, virtual-environment
and fully resolved Python read trees and the same five exact model/codec/frame
files. Add literal read access to the interpreter symlinks hidden by `resolve()`.
Allow metadata on the exact ancestors of permitted paths, without granting
directory enumeration or other data reads there. Keep network denial and all
home-directory writes denied, including model assets. The old editable package
path stays denied; explicit `PYTHONPATH` selects the new source closure.

The static [source audit](r4_isolated_runtime_readiness_1096_sources.md) found the
omitted `cpython-3.12-macos-aarch64-none` alias and CPython's ancestor `lstat`
requirements. Neither identifies the exact cause of the previous `execvp`
denial. This single candidate tests the combined correction; success would not
separately attribute the original error to one rule.

## Frozen procedure, boundaries and budget

The implementation consists of the existing
[profile generator](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/contract.py),
the actual [worker](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/worker.py),
and the separate [readiness coordinator](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/readiness.py).
The coordinator never invokes `campaign.prepare`, imports a model definition,
or opens an actual corpus/reference payload. The worker receives no input rows
and takes its existing `--readiness-only` branch, skipping model deserialization,
construction and every batch forward. Reading accepted model files as bytes to
verify their identities is required; it is not a model load into an executable
model. Model state receipts remain null.

1. Freeze reviewed source in Git. Use the installed qualified interpreter only
   as a coordinator to prepare a new exclusive directory under
   `/Users/casey.allard/.codex/uor/issue-1096-readiness`. This identity-authoring
   step imports no Torch/model and makes no sandbox or readiness attempt.
2. Bind the source closure, five accepted assets, interpreter launcher/resolved
   file/symlinks, `pyvenv.cfg`, relevant installed Python/Torch files, runtime
   versions/settings, hardware, profile bytes and original stop receipt. No
   environment download, new venv or library change is permitted.
3. Create four harmless sentinel files in separate `probes/corpus`, `reference`,
   `history`, and `results` directories inside the new task output. They contain
   only a fixed non-research sentence. These are stand-ins subject to the same
   denied-home read rules, not accesses to the real sealed population or history.
   The parent verifies that each exists, is readable and has the frozen bytes.
4. An independent reviewer binds the exact manifest SHA256 in a separate review
   receipt with `APPROVED_FOR_SINGLE_READINESS_ATTEMPT`. Freeze, source review
   and resource authoring are reported separately from execution.
5. Run exactly one `/usr/bin/sandbox-exec` invocation of the actual worker,
   with `--readiness-only`, the bound manifest/hash and stdin closed. An exclusive
   `started.json` is the irreversible attempt lock; failure cannot reset it.
   The parent enforces 60 seconds across identity rechecks, launch and execution,
   killing/reaping the process group on timeout. No automatic retry is allowed.
6. Retain raw stdout/stderr, both worker events or the failure, exact command,
   bindings/profile/source identities, elapsed time, combined parent/child peak
   RSS bound, observed work counts and resource receipt. Preserve partial output
   on timeout. Hashes and raw evidence precede any later interpretation.

Hard limits: **one attempt, 60 seconds, 16 MiB new receipts/probes**, with a
1 MiB combined worker-output sublimit and the inherited conservative 3 GiB
combined peak-RSS ceiling. No model construction/load, forwards, optimizer
updates, downloads, corpus regeneration, withheld reads, replay, fitting,
accelerator/thread search or broad QA is admitted. Reuse Apple M1 / 16 GiB,
four intra-op threads, one inter-op thread, deterministic CPU/Accelerate and one
worker. The #1094 budget and its prior stop are not reset by this readiness task.

**Definition — reachability and cost.** This checks an executable/input-isolation
prerequisite, not a model metric. There are no language decisions to improve.
The source path contains zero model construction/forward calls on this branch.
Expected cost is seconds for file hashing and Torch import; the independent
60-second kill limit is binding. No timing calibration precedes this attempt.

## Admission, outcomes and next action

Require exit zero and exactly two events, `ready`/`done`. The initial event must
say `ARTIFACTS_READY`; both must bind the same source/artifact manifest and exact
profile, verified interpreter chain, installed runtime files and actual CPU
runtime settings. All four probes must raise `PermissionError`; successful
open, missing file or a different I/O error fails. Both events must show zero
model loads and row/batch forwards, initial/final model states must be null,
and the final audit must show zero optimizer updates and denied access. The
worker verifies source/asset bytes and the parent separately rechecks them
before launch. Exact evidence is retained; no tolerance or criterion changes
follow observation.

| Terminal | Action |
|---|---|
| `ISOLATED_RUNTIME_READY` | Close #1096 with its exact evidence and bindings after protected delivery; return the readiness handoff to still-open #1094 for separately frozen preparation and independent release review. No comparison executes here. |
| `UNAVAILABLE_ISOLATED_RUNTIME` | Preserve the specific failure and all original evidence; keep #1094 blocked and its comparison `NOT_RUN`. A separate repair decision is required; no revised profile or retry under this contract. |
| `INCOMPLETE_RESOURCE` | Preserve partial output/counts and stop; do not renew the budget or qualify unobserved steps. Keep #1094 comparison `NOT_RUN`. |

## Claims, delivery and storage

This is an empirical runtime-readiness check. It supplies no mathematical proof,
parser/model-output result, general sandbox-security guarantee, general language,
new-world transfer, geometry advantage or final-kernel qualification. The
[#1085 source audit](integration/clause-segmentation-1085-sources.md) remains
relevant to typed identity and ordered provenance; NEMESIS/W33/UOR research is
not evidence that this operating-system policy works. Preserve #1079
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, #1082's descriptive findings, #973 open and
#954 blocked.

Named checks: independent static source/contract review before execution;
the single declared readiness attempt; independent result/source/digest review;
changed-Python syntax, claim wording, document/evidence-reference integrity and
`git diff --check`. Protected queue statuses are transport acknowledgements.
Broad Cargo, model, replay and certification suites stay `NOT_RUN`.

Initial storage review found about 51 GiB free. The new isolated worktree shares
Git objects and uses existing runtime/model files. No new Cargo target, runtime
installation, model download or code-graph rebuild is needed before this decision.
Preserve the mixed original checkout, original #1094 receipts and corpus,
mode-000 withheld directory, source caches, all unique evidence and user files.
No deletion is authorized by this task.
