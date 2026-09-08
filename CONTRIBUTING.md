# Contributing to UOR-R4 Geometric Language Model

New development follows the
[owner-adopted immediate sequence](docs/integration/project-track.md#immediate-build-sequence).
Read the [project map](docs/PROJECT_MAP.md) and [capability/direction assessment](docs/integration/model-direction-2026-09.md). Treat roadmap order as priority and evidence dependencies, not an automatic one-task stop. Use the existing
task and consolidated issue owners; coordinate independent work when useful and
preserve dated results. Reuse the
[architecture/source audit](docs/integration/architecture-2026-09/README.md) before
introducing a new mechanism; inspect its actual source and declared limitations.

Follow [AGENTS.md](AGENTS.md), the [native geometric AI plan](docs/integration/project-track.md)
and the [current implementation](docs/integration/current-state.md). The stable
machine policy is [agent-execution-policy.json](docs/integration/agent-execution-policy.json).
Do not copy the current stage into another roadmap or restore old issue gates.

## Build the native model

Use Rust for preparation, training, artifact construction and inference.
Training may use floating point and matrix multiplication. The serving target
executes learned geometric operators through bounded state, routes and
integer/table lookup, with no serving matrix products, including lookup/add
contractions, and no transformer backbone. Deterministic geometric address/page
selection is allowed. Shared typed operators are current; expert gates remain a
conditional later option requiring capability and complete M1 cost evidence.

Prime/ordered-n-let addresses, fixed zeta phases, R4/S3/H4 transport, exact
`Z[phi]` and orientation state, typed paired-H4/icosian geometry and UOR identity
are primary model mechanisms. Keep their architectural roles separate from
measured predictive contribution. Preserve historical Python/dense references
as evidence; add no Python model or product dependency.

Both conversation/memory and coding/reasoning must earn alpha through actual
model behavior. Implement small coherent steps in the native path. A request to
complete a whole plan authorizes its necessary successive tasks; the historical
one-task stop is not a default.

## Development and delivery

1. Refresh `origin/main` and relevant live issue state; work in an isolated full
   worktree. Preserve unrelated changes and unique research artifacts.
2. Implement the next useful native behavior. Reuse existing geometry and
   runtime parts before adding abstractions or another mechanism.
3. Compile and exercise the changed path. Use focused tests for concrete
   arithmetic, state, causal, serialization and interface risks.
4. Deliver through a protected pull request with actual commands, outcomes,
   limitations and the next action. Stage named files; do not push `main` or
   bypass branch protection. Compatibility check names alone are not QA.

Typical checks are `cargo fmt --check`, `cargo check -p <touched-package>
--all-targets --offline` and `cargo test -p <touched-package> <focused-test>
--offline`. Choose relevant checks; this is not a blanket full-suite ladder.
Broader checks run when the changed boundary or a release requires them.

## Learning and resources

Configure useful context, training and evaluation windows and CPU/thread,
wall-time, RAM, new-storage and checkpoint limits. Account cumulatively across
warmup, training, evaluation, retries and resumes. Diagnose and correct failures
within the remaining authorized budget; there is no global 15-minute cutoff or
one-retry quota. Stop/checkpoint at limits, avoid unchanged blind retries, and
apply the standing local-extension authorization below before increasing a limit. External cost still requires separate explicit authorization.

Open development evaluation is part of learning; final held-out evaluation
follows design selection. Keep prior results at their exact scope, and report
resource unavailability separately from model quality. Preserve proof, measured
behavior and hypothesis distinctions without creating a proof dossier or new
ledger for every edit.


**Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance.
