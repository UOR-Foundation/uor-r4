# Status

Updated 3 October 2026 (Eastern Time). **Pre-alpha: useful conversation, broad reasoning,
coding, frontier capability and lower complete-path energy remain unestablished.**

The owner-adopted [active plan](docs/integration/project-track.md) prioritizes
**grounded conversation and durable memory**, then the same model's broader
language, coding/reasoning and efficient D11/D5 laptop execution. [Current state](docs/integration/current-state.md)
and live [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) own results.
This page is navigation, not an independent results ledger. See [ROADMAP](ROADMAP.md).

**Owner direction, 3 October:** no transformer baselines. Mechanisms are compared with
matched geometric alternatives and with the previous best geometric model.

| Boundary | Current evidence and remaining work |
|---|---|
|---|---|
| Exact retrieval and emission | E1 lexical sieve reports 747/747 development MQAR and independent reproduction. emit-1 (2.1M, context 384) gets 106/109 with supplied recall; oracle open relation 48/52 versus lexical sieve 1/52. These are harness results with reference history/category assistance, not a served memory assistant. |
| Semantic compiler | E3 v14 R1 + 776 raw teacher paraphrases: gated trunk head relation 1754/2098 (0.836), act 0.766; dense combined head 0.949 and word table 0.900 on relations. Original 0.9/0.95 gate fails. The raw paraphrases carry audited label errors. A saved compiler with a learned value-span head now drives the grounded session (34/52 open, 13/17 closed relation on one development cell; see current state). A combined R1-trunk head is a recorded negative there (26/52). A reviewed 394-row paraphrase derivative (#1573) awaits its paired fit. E3 training includes development-value identities, so this is not unseen-value evidence. |
| Scale ladder (Claude lab report, #820) | 20M geometric model (quaternion rotation, flat L2 read, context 384; 300M tokens of TinyStories, TinyDialogues and chat-v0): TinyStories validation NLL 1.3882 (seed 1), 1.3858 (seed 2). Learning rate 0.001 beat 0.002 by about 0.045 nats at steps 5k and 7.5k. At 8M, quaternion lanes beat U(1) lanes and the flat L2 read beat the Lorentz read. A 29M run (434M tokens) is training. These are language-modelling losses on one data mix, not conversation results. |
| Chat | **Not achieved.** Best D19 grounded session (exact store plus prime-atom log-recall sieve, `chat-8m-a`): 959 of 1,075 turns. On the 232-reply panel judged by qwen2.5:7b, 21 replies are acceptable against 8 for the deranged control. 20M chat fine-tunes are being graded. |
| Integer serving | Existing D11 stack has scoped opcode/parity evidence. emit-1's learned pointer is not exported; head-0 copy boost is a different mechanism. Dense layer/output access remains and no valid complete-path J/token result exists. Serving of the 20M/29M models has no recorded D11 check yet. |
| Geometry | Exact tables, quaternion state, trained-in 2I and historical binding components remain available. Their semantic and efficiency contribution needs matched consumer evidence; failed promotion does not retire a family. |
| Compute | Training on a rented Runpod 2×RTX 4090 pod; grading on the M1. Optional `cuda` feature for offline training is [PR #1649](https://github.com/UOR-Foundation/uor-r4/pull/1649) (18 device-parity tests reported passing at `0ebaa75b`; not merged when written). Hosted GitHub runners are reserved for `main`'s required checks. |
| Track B | Parked. #1518's original 1e-4 parity failure stands; no automatic retry or transformer serving follows. |

The first three rows are as of 1 October and have not been refreshed here; see the
[evidence review](docs/integration/grounded-memory-evidence-2026-10-01.md) and current state
for scope, hashes and newer results. D18's A1 outcome D is preserved; old A1 training does not resume.

## Coordination

- Claude: #1511 / #1552, compiler/emitter integration, training and evaluation.
- DeepSeek (its own harness; OpenCode was removed 3 October): #1512, geometry, data/read research and shared interfaces.
- Codex: #1515 / #1563, owner-reauthorized adoption and integration; activity not confirmed since 1 October.
- Support lab (Antigravity): documentation, issue hygiene and cleanup only; no model code.
  Kimi and the older boards remain historical.

Live GitHub claims and actual process handles determine activity and ownership. Use the
[shared protocol](docs/labs/protocol.md), [host guide](docs/labs/operations.md) and a prospective
resource work card before compute. Preserve live runs' reservations. No paid compute beyond the
authorized pod and no destruction of unique material.

Protected PRs require recorded exact-head review and actual scoped checks. Historical queue
status names are acknowledgements, not compile/test evidence.
