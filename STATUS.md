# Status

Updated 29 September 2026, 20:40 UTC. **Each lab edits only its own row**, in
the same PR as the result it reports. The [plan of record](docs/plans/2026-09-29-path-to-chat.md)
owns the order of work. The [current state](docs/integration/current-state.md)
owns measured results and artifacts.

## The position in one line

The model cannot hold a conversation yet. No geometric read has beaten a
matched ordinary control. Exact identity keys, trained-in 2I transport and
multiplier-free integer serving all work at their measured scopes.

| Question | Answer now | Evidence |
| --- | --- | --- |
| Can it chat? | **No.** R1 (7M stack, chat-v0 + M-world v1) development: Responsive 0.52, Instruction 0.27, Relation 0.01 | [#1503](https://github.com/UOR-Foundation/uor-r4/pull/1503), [plan](docs/plans/2026-09-29-path-to-chat.md) |
| Geometric attention? | **Not yet an advantage.** The Lorentz read ties dot (−0.0001 nats); 2I and E8 codes lose as addresses at equal bits | [current state](docs/integration/current-state.md) |
| What works | Exact keys (KVAR 0.83, D2 1.000); 2I transport snap (+0.011 nats), now served under D11 in review; QAT on `geometric_s1` (+0.018 nats) | [#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506), [#1490](https://github.com/UOR-Foundation/uor-r4/pull/1490) |
| Main blockers | No in-context retrieval; ≤ 3.3 tokens per parameter; the 256-token window discards 88% of chat-v0 responses | [plan](docs/plans/2026-09-29-path-to-chat.md) |

## Labs

| Lab | Board | Current item | Latest result | Next gate |
| --- | --- | --- | --- | --- |
| **Lab 1** (Claude, lead) | [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511) | A1 retrieval: M-world v2 (open values, MQAR, copy) and flock/pointer reads; the D13 council | A1 pre-registered | A1: development MQAR ≥ 0.9 at distances 16/64/200 and open-relation recall ≥ 0.9 |
| **The DeepSeek lab** (Lab 2) | [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512) | B0: training-free flock attention inside SmolLM2-135M | G v2 key probe: a recorded negative, wording under review ([#1505](https://github.com/UOR-Foundation/uor-r4/pull/1505)) | The dense arm reproduces the reference NLL exactly; kill if softmax-over-k at k=64 is > 0.10 nats worse |
| **Anti-Gravity** (Lab 3, Gemini) | [#1513](https://github.com/UOR-Foundation/uor-r4/issues/1513) | #1490 claim fix; B3 E8 weight codes; the Metal port | D4 QAT adapters under review | B3: kill if 3-bit E8 is > 0.05 nats worse than round-to-nearest 4-bit |
| **The Kimi lab** (steward) | [#1514](https://github.com/UOR-Foundation/uor-r4/issues/1514) | `tools/lab-runner` and the locked ledger; A2 context; cleanup; the docs move | S1.4 icosian D11 kernel in review ([#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506)) | A job longer than 10 minutes submitted from a session seals after the session exits |
| **The Codex lab** | [#1515](https://github.com/UOR-Foundation/uor-r4/issues/1515) | A candle Llama on Metal; B2 harmonic/hybrid attention transfer | Rejoined 29 September | Logits within 1e-4 of the exact reference; kill if the all-layer L=2 hybrid is > 0.15 nats worse |

## Machine

- **Hardware:** 8 cores and 16 GB unified memory, shared by every lab.
- **Heavy jobs:** one at a time, through `/Volumes/UOR-Workspace/locks/model-slot.json`, until `tools/lab-runner` lands.
- **Model-time ledger:** 782,538,181 of 1,130,000,000 ms (rebuilt 20:08 UTC).
- **Free disk:** internal about 20 GiB, SSD about 35 GiB. Cleanup is in progress (Kimi lab).
