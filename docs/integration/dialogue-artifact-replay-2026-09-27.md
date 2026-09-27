# Retained dialogue artifact: offline output recovery

References #973. Fourth-lab scope under the owner's native, multiplier-free
serving direction. This work uses offline floating Rust inference to inspect
already-learned parameters; it does not widen the integer serving contract.

## Decision and retained context

The R1d dialogue study saved 21 named F32 arrays but supplied no command to
reload them for generated responses. Its width 576/read 64/context 256,
vocabulary 4096 Quaternion model is a separate response-learning vehicle from
the active width256 TinyStories reader study. The retained parameters have
SHA-256 `95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822`
and occupy 21,719,304 bytes. The fit report has SHA-256
`a192916c17d8d124e9a32c21c3b31ef6b622890b148d37549eb219d11cd641fa`.
Both remain in
`/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1d/curve-2`.

The report labels the original executable's source
`5109861c93dee0e39d3aa189c325b3a13c4ededc` and executable SHA-256
`84ec4ebe5d95b28346501129394e3cb211483c9bf2b471caa8abbc4313463e4b`.
New replay source and executable identities are recorded separately. Comparing
the legacy and current ordinary Dot/Quaternion operations supports a scoped
compatibility case; a current full-window/incremental check does not reproduce
the historical backend bit for bit.

The prior negative is preserved: after 2,237 updates and 9,162,752 sampled targets,
reported response NLL 3.0678578 remained above the matched count comparator's
2.4656426. No automatic longer fit follows. The stronger R1d interpretation
that capacity, rather than exposure, is established as the cause is not
identified by that record. Its all-point fitted floor is degenerate (-50.5),
second-half and last-eight floors differ (2.511 versus 2.941), its floorless fit
disagrees, and the observed curve is still decreasing. R1c/R1d share the
model/data seed; changed batch and F32 reduction create a distinct trajectory,
not an independent-seed replication. Close training/development loss also does
not exclude objective or data issues: arbitrary 256-token response windows can
begin inside a response without its complete user request.

A spending rule may support declining a longer run without establishing
convergence or a universal model-size floor. The methodological distinction
between fitting an observed curve and validating its extrapolation is supported
by [Alabdulmohsin, Neyshabur and Zhai, *Revisiting Neural Scaling Laws in Language
and Vision*](https://arxiv.org/html/2209.06640). That literature is not evidence
that this particular model would improve with more training.

## Implementation boundary

`training::dialogue_artifact` imports only the retained legacy continuous
Full/Dot/Quaternion configuration. It verifies the sealed report, complete
tensor inventory, element offsets, shapes, finite values and parameter bytes
before construction. Missing legacy read/admission metadata is explicitly
interpreted as Dot/Full for this schema. The model remains private; the API
exposes immutable provenance and exact-token generation. Existing canonical
checkpoint/training and integer width validation are unchanged.

The corpus manifest binds tokenizer identity and train/development token
hashes to the fit report. Mask hashes are retrospective identity observations:
the historical report did not bind those masks. The shared tokenizer's literal
role protocol encodes separators, markers and content independently.

The separate `dialogue-artifact-replay` example consumes only the existing 38
development requests from the old panel,58assistant responses including actual
intermediate replies. It skips expected-answer and fresh fields. It uses
greedy 32-token output with the existing cycle stop, exact generated IDs,
one inserted BOS, and an explicit caller EOS before another request when the
model stopped without EOS. It never truncates the complete 256-token input plus
output horizon. Each turn replays its exact prefix in a fresh session; this is
not a persistent-session speed measurement.

This changes the old panel's presentation and output cap. It is an open
development output observation, not that panel's official pass/fail result or
a held-out qualification. Raw bytes/IDs, every prediction record, stop reasons,
prefix comparison, source/executable/artifact identities and actual work are
retained in a new exclusively claimed, sealed attempt. Interrupted attempts
remain separate.

## Necessary observation and distinct decisions

Check the loader's real format/provenance risks and exact-token interface, then
compare current full-window and incremental probabilities on one loaded
development prefix (tolerance 1e-5). Inspect every actual response. Responsive
fragments preserve an integration candidate; generic or repetitive output adds
a learned-model negative without identifying its cause. Unresolved loading is
unavailable evidence. None of these automatically starts training, integer
export, a seed/decoder sweep, or a new acceptance campaign.

The [prospective work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5853030114)
records the 90-minute complete cap, 600-second/1 GiB inference ceiling, prospective
short-probe concurrency refinement, fixed requests and shared resource limits.
The frozen Dot/Lorentz/Affine study and other labs' artifacts remain separate.
Executed results will be recorded here and in the source-bound receipt.
