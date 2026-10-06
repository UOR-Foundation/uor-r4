# Source-bound geometric actions — October 6

PR [#1778](https://github.com/UOR-Foundation/uor-r4/pull/1778) adds an opt-in
native integer action path at numerical head
`4cc70f08e2a3062b122d3348d8a5885a33e77a80`. It preserves Copy scores and derives
Period/Stop endpoint scores for every admitted Source. SharedCue applies the
same authentic source cue to all three action types. NoTerminalCue keeps the
identical layout and Copy scores but omits the terminal cue. Legacy remains the
default and the retained model control. No DeepSeek training-normalizer seam is
changed.

## Executed development result

All arms use the same trial12 donor, development512 panel, context128 and
maximum32 generated tokens. There are zero fitted updates, no adoption/selection,
and zero predictions on the evaluation128 panel. Counts require complete
own-prefix generated answers including EOS, rather than teacher-forced accuracy.

| Arm | Complete /512 | Native equal-episode CE | Gains/losses versus legacy |
| --- | ---: | ---: | ---: |
| Legacy | 74 | 0.9544226152014282 | — |
| All-source, no terminal cue | 69 | 0.8714107967489306 | 19 /24 |
| All-source, shared cue | 72 | 0.8653232323445513 | 18 /20 |

Shared cue gains5 and loses2 versus the matched no-terminal-cue arm. Lower CE
alone does not qualify adoption: the primary complete-answer count remains below
legacy. Retain every arm and row tradeoff. This development-only comparison is
neither fresh transfer evidence nor a general attention/chat result.

Independent saved-output reviews recount every arm and tradeoff. Legacy canonical
and generation reports are byte-identical to the prior donor. All5128 canonical
Copy-score vectors are identical across arms, and20512 terminal cue offsets equal
the authentic source cue. First-source correctness is296/512 in all arms; first
emitted-token correctness is229/204/207. Shared versus legacy changes169 rollout
rows,168 first divergences from payload to terminal and1 from terminal to payload.
This supports a phase/action-space interpretation, not improved source finding.
The reviewers recommend holding further selector/terminal patches while locating
the existing learned geometric state→prediction bridge requested by the owner.

## Mechanism and limits

The bounded layout admits at most128 Copy occurrences and128 Source records,
384 actions total. A private383-row read constructor leaves the public256-context
limit unchanged; the final real Stop occupies the existing null slot. Token aliases
are marginalized only after normalization, and every action retains exact Source,
record, commit and event identity. A terminal token may have multiple contributors;
the trace does not invent a unique selected Source for that aggregate.

The public reader requires a nonempty Source. Empty and context-only banks retain
`EmptyBank`; an initially proposed fallback was unreachable and removed. Failed
checks on earlier heads were fixture/admission defects, retained separately from
model negatives. Low-level absence of an adjacent cue still gives zero cue for a
valid nonempty Source.

Shared cue cancels in within-source raw Copy-versus-terminal score differences.
Flat normalization still couples source mass to phase spectra, source length,
repetition, terminal multiplicity, and integer tail/rounding. Legacy-to-all-source
changes endpoints and terminal count; it cannot isolate an independent normalized
source prior. Raw Copy rankings at the same actual prefix are unchanged across
arms. Thus this is a relevance/emission diagnostic, not an initial-source ranking
repair. The earlier three J-quarter margin recoveries remain arithmetic only;
this donor-only probe does not generate those quarter counterfactuals.

## Exact execution evidence

Linux x86_64 host `0ec50ed5c0d3`, CUDA hidden. Exact numerical-head formatting,
34 source tests,4 reader tests,49 fitter tests, and fitter/observer release builds
passed:87 tests,229.463s,CPU8, peak own process-group RSS4,945,211,392 bytes.
Actual model probe: exit0,39.375s,CPU16, RSS6,777,573,376 bytes, no bounded stop.
This native integer instrument ran on the pod CPU; no D19 session, chat-grade,
CUDA parity, Metal test, whole-path allocation or opcode audit was performed.

Executable SHA256 `c2d15d2c7e5771490b773bc032e622035b23f37fcc4b8e92db2ffa1263a551fb`.
Config SHA256 `4b514a962b3e064eb1c0834ef2222da5ba5a695465bd2f5db2c84cd989f56318`.
Joint donor SHA256 `33c6021151e84b27cb222a333303ccff093219e46c661b0ca689bd9b92929254`.
Durable sealed report: `/workspace/uor-r4/codex/source-bound-actions/probe-1`.
Execution/check/freeze receipts are alongside it; all prior failed checks retained.
Clean committed source was reconstructed from a SHA-verified committed archive
and exact changed-file inventory before checks; executable identity was frozen
before predictions. Complete preparation/review/build/retry/model costs stay in
the precharged7200s continuation reservation. Own combined pod storage observed
9,298,367,746 bytes under12GiB with128MiB stop margin; new laptop evidence is thin.

## Owner correction and next causal dependency

After this probe, the owner challenged whether continued cue/terminal adjustments
were semantic patches instead of learning geometry that predicts. The proposed
record-credit quarter follow-up is held. Preserve its investigation as a conditional
diagnostic, not the active implementation direction.

The source supports a narrower limitation: `loss_bank_cue_joint_source_end` learns
coefficients over fixed relative roots with context, unary cue, Copy, prefix and
terminal families frozen. The observation adapter rejects joint-cue context
adjoints. This Copy/Period/Stop realizer also restricts output support to stored
source tokens and two terminal actions. Existing learned context/transport and
broader emission machinery remain in the repository; they are not missing by
fiat or newly invented. This particular instrument cannot establish a general
learned geometric next-token process.

Next investigate and implement the smallest connection from ordinary causal token
prediction error into the existing geometric state/query/transport and native
emission path. Preserve exact hard-forward export/reload behavior and old artifacts;
diagnose representation and surrogate effects rather than freezing the representation
by default. Use retrieval-containing data without supplied answer records, plus
continuations requiring a token absent from the copied context, so a pure copy
mechanism cannot pass the integrated predictive task. Expert source/history reviews
must determine the concrete existing seam before a new fit. DeepSeek's training
normalizer remains independently owned. No new integrated-learning run is claimed.

Independent integrated-learning review confirms the earlier `loss_bank` already
connects token prediction to recurrent context/root/category families and Copy/
terminal potentials; do not claim no learning exists. It predates the present
composed cue/prefix/end reader. First concrete seam is joint-cue credit into the
actual independently encoded query and cue states via existing `ContextWeights`.
A complete composed loss must also account for prefix/end latent-state paths;
a partial gradient adapter alone is not end-to-end qualification. Audit the
existing ordinary-token native readout before adopting a Generate integration.
The proposed integration and source-specific review are preserved alongside the
outcome receipts; no new implementation or learning run has occurred.
