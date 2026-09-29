# UOR-R4 Geometric Language Model

UOR-R4 is a research language model written in Rust. It asks whether geometry
(quaternion transport, lattice codes, hyperbolic and harmonic similarity) can
replace the parts of a transformer that are expensive at run time. The aim is a
model with frontier-level quality that is stored on, and runs from, a 16 GB
laptop.

**Status: pre-alpha research.** No model in this repository holds a useful
conversation yet. General prose, reasoning, coding, frontier equivalence and
energy savings are not established. The [status page](STATUS.md) gives the
measured position of every active line of work.

## Approach

The work runs on two tracks that share one set of geometric mechanisms.

- **Track A: a native geometric chat model.** A small (≤ 30M parameter) model
  trained from scratch, served without floating point or a hardware multiplier.
  Its first milestone is a sealed conversation panel: multi-turn replies,
  recall of updated facts, and new instruction wordings.
- **Track B: geometric conversion of pretrained transformers.** Distil open
  models (SmolLM2 135M → 360M → 1.7B) into a geometric runtime: replace
  attention and compress the weights, measuring the quality gap to the teacher
  and the bytes and operations per token.

The four geometric mechanisms under test:

| Mechanism | Idea | Role |
| --- | --- | --- |
| Flock attention | Each query reads a sink, a local window and its *k* nearest keys, as starlings track about seven neighbours. Rank-table weights, no exponentials | Sparse attention and exact copy (k = 1) |
| Spherical-harmonic attention | Harmonic features of normalised queries and keys give a fixed-size recurrent state | The smooth, low-frequency part of attention |
| Quaternion / 2I transport | A recurrent state carried by unit quaternions, snapped to the 120-element binary icosahedral group | Long-range state without a key-value cache |
| E8 / 2I lattice codes | Weights stored as lattice codewords and read by table lookup | 2–4 bit weights |

A geometric mechanism stays in the model when it is within 0.02 nats of its
ordinary matrix equivalent, a rule the owner set on 29 September. That
comparison uses paired arms and at least two seeds. The plan, gates and kill criteria
are in the [plan of record](docs/plans/2026-09-29-path-to-chat.md).

## Architecture

- **The geometric stack** ([`geometric_stack.rs`](crates/uor-r4-training/src/geometric_stack.rs))
  interleaves quaternion-transport recurrence layers (`r`) and multi-head reads
  with a Lorentz or dot score (`a`). A SwiGLU MLP follows each layer. An
  ordinary transformer control runs on the same kernels.
- **Memory:** an exact, addressed store keyed by prime identities (AERM).
  Learned heads decide what to write and when to read.
- **Serving (D11):** the integer engine ([`uor-r4-integer`](crates/uor-r4-integer/README.md))
  runs 4-bit weights with integer add, shift, compare and table reads only: no
  floating point and no multiply or divide instruction. The served binary is
  audited at instruction level.
- **Training:** offline in Rust (Candle), with floating point allowed.
  Quantization-aware training reads exactly the values the export writes.

## Repository map

| Path | Contents |
| --- | --- |
| [`crates/uor-r4-training`](crates/uor-r4-training) | The geometric stack, memory, training worlds, export, and the experiment drivers in `examples/` |
| [`crates/uor-r4-integer`](crates/uor-r4-integer) | The multiplier-free serving engine and the `uor-chat` and `uor-r4-stack` binaries |
| [`crates/uor-r4-lut`](crates/uor-r4-lut) | The serving artifact format |
| [`crates/uor-r4-model-source`](crates/uor-r4-model-source) | The exact CPU reference for teacher models (Llama/SmolLM2, GPT-2) and the attention-replacement seam |
| [`crates/uor-r4-core`](crates/uor-r4-core) | Geometric primitives (R4/S3/H4, exact `Z[φ]`, prime addressing), the earlier native learner and sealed report output |
| [`crates/uor-r4-tokenizer`](crates/uor-r4-tokenizer) | The shared byte-level BPE tokenizer |
| `crates/uor-r4-graph-*`, `uor-r4-router` | The frozen TLA/R4G1 graph runtime and certified routing, under their own scoped contracts |
| [`docs/`](docs) | Plans, decisions, results and the full research history |

The [project map](docs/PROJECT_MAP.md) covers every crate and historical engine.

## Quick start

The toolchain is Rust stable, via rustup. Training data and model artifacts
are local material and are not in the repository.

```sh
cargo build --release -p uor-r4-training --example geometric-stack
cargo build --release -p uor-r4-integer --bin uor-r4-stack --bin uor-chat
cargo test -p uor-r4-training --lib geometric_stack
```

`geometric-stack` trains and evaluates stacks (run it without arguments to
list its modes). `uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS` serves
an exported stack under D11, and `uor-chat --bundle <dir>` is the interactive
session.

## How the work is run

Five labs work in parallel. Each is an AI research agent with its own GitHub
board, and the owner holds the mission, spending and final decisions.

- [Plan of record](docs/plans/2026-09-29-path-to-chat.md): tracks,
  experiments, gates, kill criteria and owners.
- [STATUS.md](STATUS.md): one row per lab, with the current item and the latest
  result.
- [ROADMAP.md](ROADMAP.md): assignments, dead paths and the anti-stall rules.
- [DECISIONS.md](docs/integration/DECISIONS.md): owner decisions.
- [Current state](docs/integration/current-state.md): measured results and artifacts.
- The [programme tracker #820](https://github.com/UOR-Foundation/uor-r4/issues/820),
  epics [#1508](https://github.com/UOR-Foundation/uor-r4/issues/1508) (Track A),
  [#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509) (Track B) and
  [#1510](https://github.com/UOR-Foundation/uor-r4/issues/1510) (infrastructure).
- [AGENTS.md](AGENTS.md): the operating rules for every contributor and agent.

**Evidence rules.** Every number comes from committed code run into a sealed
report directory. Experiments are pre-registered with a gate and a kill
criterion, negative results keep their exact scope, and claims follow the
[formal vocabulary](docs/formal_vocabulary.md).

## History

The research began with exact geometric memory and a transformerless prose
learner, then moved to learned recurrent models with integer serving. The
[README as of 29 September 2026](https://github.com/UOR-Foundation/uor-r4/blob/0fb03372/README.md)
keeps the dated results log. The [evidence index](docs/integration/EVIDENCE.md)
and the [research ledger](docs/RESEARCH.md) cover the full history.

## License

[MIT](LICENSE). Third-party research and dependencies keep their own licenses.
