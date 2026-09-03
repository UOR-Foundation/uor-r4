# Native Four-fact workbench candidate — #1107

**Decision:** implement and freeze the exact final-candidate source, admit one
offline verification/build sequence independently, and stop at
`WORKBENCH_CANDIDATE_BUILT_UNQUALIFIED` if that sequence succeeds.

This child implements the dedicated `uor-r4-workbench` crate, its one
`r4-workbench` executable, the private same-executable worker, the private
comparison adapter, and the first Four-fact shell specified by
[ADR-0006](adr/0006-native-four-fact-workbench-service.md) and its
[machine contract](r4_service_contract_1105.json). It does not fit, export,
open or load a model artifact; invoke a learned forward; compare numerical
outputs; qualify a binary; bind/listen on a service socket; make an HTTP
request; or launch a browser.

## Source boundary

The same executable contains four mutually exclusive entries before build
freeze:

- `--config ABSOLUTE_PATH CONFIG_SHA256` is the only parent service entry;
- `--internal-worker` accepts no further argument and owns no listener;
- `--private-compare-host RELEASE_PATH RELEASE_SHA256 ADMISSION_PATH
  ADMISSION_SHA256` is the separately admitted empirical adapter and owns no
  listener;
- `--private-metadata` reads only the executing binary and floating-point
  environment and cannot open configuration, artifact or evidence paths.

The [private release contract](r4_workbench_private_release_1107.json) removes
two ambiguities left deliberately open by #1105. It freezes the fresh host
execution release/admission schemas and uses a stable open executable handle on
the accepted macOS target. The worker and private comparison modes must be
executed through that verified `/dev/fd` vnode and hash inherited file
descriptor 3 before model access. Unsupported platforms compile for workspace
checks but remain unavailable; there is no readiness fallback.

The private adapter first validates a fresh execution release and its separate
independent admission. It then verifies the artifact's immutable original
export release and supplies only those old provenance bytes to the current
`ComparisonAdmission`. This preserves the distinction between permission to
execute a new host and provenance of the previously exported artifact. It does
not reuse the consumed #1102 CLI, coordinator, release or qualification as new
permission.

## Evidence classes and limits

The source maps the seven HTTP routes, exact wire records, one-slot lifecycle,
strict configuration/assets, same-origin boundary, framed worker protocol,
cancel/deadline/confirmed-reap rules, and the sole
`answer_four_fact_raw_text/v1` operation. Synthetic unit tests may inspect
pure parsing and state transitions. Those are source and measured test
observations, not mathematical proof and not observations of a real service or
model.

The accepted #1105 source audit remains the research basis. NEMESIS motivates
explicit state and transition declarations but its arbitrary-network,
zero-error and energy statements are not evidence for this executable. W33
provides useful immutable-object and receipt patterns without proving
tamper-proof storage. Pinned UOR sources require separately typed content,
realization, derivation and topology identities; digest equality is an
integrity observation, not a behavioral result or injectivity proof. The MIT
donor UI informs only the dark visual tokens, rail, truthful selector and
composer placement. Its worker, cache heuristics, fallback identity and model
claims are not transferred.

The previous empirical result remains `NATIVE_REFERENCE_PRESERVED` only for the
old #1102 binary and accepted authoring population. `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`,
`TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`, `CLAUSE_ADAPTER_PRESERVED`, all retained
unavailable preparation, #973 open and #954 blocked remain unchanged.

## Build phase boundary

No compiler or test executable may run until all source/assets, lockfile,
toolchain, target, dependency closure, commands, environment, supervisor,
empty output roots, resource ceilings and receipt schemas are frozen and an
independent reviewer accepts their exact bytes. Opening a PR before that review
would trigger an unadmitted build and is prohibited.

The admitted sequence is limited to synthetic workbench unit tests, one release
build, and the zero-model `--private-metadata` observation. A first compiler
launch consumes the sequence. Any source or build-input repair requires a new
freeze and independent admission before another compiler invocation. Successful
compilation establishes a concrete binary identity only. Numerical
comparison/qualification and ordinary API/lifecycle/browser behavior remain
separate later empirical gates.

**One next action after protected closure:** freeze and independently accept an
exact fresh private-comparison release around the delivered binary and runtime
before any artifact access or model work.
