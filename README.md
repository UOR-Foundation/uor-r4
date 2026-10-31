# UOR-R4 Geometric Language Model

*An open research project: can geometry replace a transformer's run-time machinery in a language
model that runs on a laptop?*

[![License: MIT](https://img.shields.io/github/license/UOR-Foundation/uor-r4)](LICENSE)
[![Rust 1.97.1](https://img.shields.io/badge/rust-1.97.1-orange?logo=rust)](rust-toolchain.toml)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-yellow)](PLAN.md)
[![Milestones: 1 of 8 complete](https://img.shields.io/badge/milestones-1%2F8%20complete-blue)](#where-we-are)
[![Last commit](https://img.shields.io/github/last-commit/UOR-Foundation/uor-r4/main)](https://github.com/UOR-Foundation/uor-r4/commits/main)
[![Hugging Face](https://img.shields.io/badge/%F0%9F%A4%97-Hugging%20Face-ffcc4d)](https://huggingface.co/caseyallard/uor-r4-geometric-214m)

![Four panels read left to right: a sentence split into numbered pieces; each piece placed as a point on a geometric object; the newest piece looking at only a few nearby earlier points; a few drawers opening near what was read, with the nearest word inside them picked. A band underneath reads "whole numbers: adds, compares and table look-ups, with integer multiply and divide allowed".](docs/figures/start/hero-how-it-thinks.svg)

*How UOR-R4 is designed to predict the next word. The picture shows the design, not a claim of useful conversation.*

## What this is

Chat assistants run on **transformers**: programs that spend much of their effort multiplying very
large grids of decimal numbers. UOR-R4 bets that a different toolkit can do the same job more
cheaply: geometric rotations, whole-number addresses built from primes, small lookup tables and an
exact memory. The goal is a fast language model that trains quickly and runs on your own laptop or
in your browser, with local computation by default and an optional user-configured API that sends prompts to the chosen endpoint. The project is open research and **pre-alpha**:
**one milestone is complete, but no model built under its rules holds a useful conversation yet.**
[What UOR-R4 is](docs/start/what-is-uor-r4.md) explains the bet in five minutes.

## Where we are

**30 Oct 2026: M1 is complete — 1 of 8 milestones.** The saved 19-table discrete
`trunk-joint-pointer-v1-d11` model, exported as two seeded `UORLUT01` integer artifacts, passed all
six closing bars on both seeds
([M1 #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), [closing evidence](docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)). This is a measured language-base result, not useful-chat qualification.

| Test | Saved model, seed 1 / seed 2 | Controls | Outcome |
|---|---:|---:|---|
| DEV next-token top-1, 14,287 rows | **7,002 / 7,001** | unigram 1,226; T2 6,300; bar target 6,324 (T2 + 24) | Beats both and the target at both seeds |
| Fresh V2, 14,479 rows | **7,215 / 7,228** | unigram 1,279; T2 6,582 | Beats both beyond the paired band of 82 |
| Held-out phrasing p-f, 15,953 rows | **7,850 / 7,872** | unigram 1,409; T2 7,375 | Beats both beyond the paired band of 81 |

The same saved artifacts have zero export/reference gap and pass the planted-fault control. They
serve at a median **10,876 / 10,566 decode tokens per second on an Apple M1**, measured with a
192-token prefill and 64-token decode for bar (f); the older close-grade bar (c) reports 11,026 /
10,573 tok/s. Bar (f) uses prompts whose phrasing templates are absent
from TRAIN at K = 1,024. **The limit:** 62.5% of p-f's *reference replies* are verbatim TRAIN
replies. That is data overlap, not a generated-copy rate: M1 shows reading new prompts into known
kinds of reply, not inventing new kinds of reply or carrying a conversation.

### The live research question

The separate float-trained FBS trunk is working toward complete replies. Claude-217's cycle-step
experiment ends every registered loop, but the content decision is underpowered and greedy stays
the decode. Exact turn memory also misses reworded questions: its key is the turn's exact first
token ids ([claude-217](docs/loop/tasks/one-model-end-to-end/claude-217/REPORT.md)).

Claude-218 found trunk-state memory keys that improved reply openings offline.
[Claude-219](docs/loop/tasks/one-model-end-to-end/claude-219/REPORT.md) then tested those keys in
free generation on fresh p-v. Where exact memory misses, serving the **point key** for the first
reply token improves template replies by **+13, +8 and +12** and exact replies by **+9, +8 and
+12**, at three seeds. This is a real but partial gain: keyed openings are right on **27–31%** of
the 385 openings it gives, only **12–18%** of the lift from forcing the right first token. The
cheaper sign key is inconclusive at this panel size. [Claude-220](docs/loop/tasks/one-model-end-to-end/claude-220/REPORT.md)
then served the point key inside one float model on fresh p-w: templates improve by **+41/+40/+42**
at three seeds, with exact claude-219 parity. That gain recalls TRAIN replies rather than
generalizing. [Claude-222](docs/loop/tasks/point-key-integer/claude-222/REPORT.md) has now built the
integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](docs/loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade.
Before unit 3's real TRAIN export, p-x scoring and laptop measurement, [integer-retention-rates](docs/roadmap/items/integer-retention-rates.md) must make the integer recurrence match the trained model. Useful conversation
remains unproven.

## The three products

None ships today. A trained integer artifact exists; an integrated, qualified product does not
([products](docs/products/README.md)).

- **[The model](docs/products/language-model.md):** one whole-number model file for local serving,
  with integer multiply and divide allowed by D80/D81. The saved discrete M1 pair is real; so is a
  separate exact `lut-stack/5` FBS export. Neither is a released conversational product.
- **[Web client](docs/products/web-client.md):** the live Studio can load the publicly released M1
  model after an explicit click and serves top-8 next-token prediction—not chat. The synthetic
  parity harness passes; M7's accepted-model parity and a qualified product bundle remain open.
- **[Desktop client](docs/products/desktop-client.md):** the Tauri shell exists and verifies
  durable session behavior on a synthetic artifact. It has no qualified trained product bundle.

## Where we're going

M2 asks for complete correct replies grounded in exact memory; M3 then qualifies durable memory.
After that come one served model (M4), laptop cost (M5), reasoning and coding (M6), browser/API
distribution (M7) and an alpha release (M8). The immediate end-to-end work is to make new-phrasing
reply openings reachable, serve every part in integers, and measure one whole model on fresh
prompts with tokens per second. This is a plan, not a promise: [where we're going](docs/start/where-were-going.md) and [PLAN.md](PLAN.md).

## How it works

Each transformer part gets a geometric or exact replacement that avoids softmax and dense matrix
multiplication. Statuses come from the [mechanism board](docs/mechanisms/README.md): **ACTIVE** is
on a current model path, **READY** is built and tested but not trained in, and **PARKED** means the
tested forms failed or broke a rule while the mechanism stays open.

| Transformer part | Geometric replacement in UOR-R4 | Status |
|---|---|---|
| Embedding table (a starting vector per token) | [Prime-zeta codebook](docs/mechanisms/prime-zeta-codebook.md) and [address cells](docs/mechanisms/address-cells.md): whole-number addresses from primes and zeta-zero phases | ACTIVE |
| Attention (which earlier tokens matter) | [Route](docs/mechanisms/route-read.md), [Flock](docs/mechanisms/flock-read.md) and [Relation](docs/mechanisms/relation-read.md) reads: walk an address tree, rotate a few neighbours into one frame, follow copy links | ACTIVE; independent effects remain unisolated |
| Feed-forward layers (most of the multiplying) | [Hamilton rotation gate](docs/mechanisms/hamilton-gate.md) and [SpiralCore/FBS routes](docs/mechanisms/spiralcore-fbs.md) | Hamilton gate is in M1 and FBS; FBS routes are trained and served on FBS only; dated float projection form READY |
| Softmax output and cross-entropy loss | [Admission](docs/mechanisms/admission.md) (open a few address "drawers", called cells) and a [rank-margin hinge](docs/mechanisms/rank-margin.md) | ACTIVE |
| Backpropagation optimizer | [Discrete learning](docs/mechanisms/discrete-learning.md): try each legal rotation of one entry, keep it only if the score strictly improves | ACTIVE |
| Matrix-multiply runtime | [Integer serving (D11)](docs/mechanisms/lut-serving.md): whole-number adds, shifts, compares and table reads; integer multiply and divide are allowed | ACTIVE; the discrete M1 pair and the separate FBS `lut-stack/5` export both serve |

Position and normalization have replacements on the FBS trunk: SpiralCore transport spin and RMS
norm with an integer square root. M1 instead uses Flock `FrozenZeroV1`, `DiscreteHamiltonGateV1`,
`DiscreteQuaternionRecurrenceV1`, `PositionHolonomyV1`, `OrbitRootSnapNormV1`, an orbit-root causal pointer and post-rotation `OuterL1V1`—not FBS
projection or the FBS RMS normalization. M1 does not establish every replacement's
independent effect. The board lists
**28 mechanisms: 12 ACTIVE, 5 READY and 11 PARKED**. Part by part:
[transformer vs geometry](docs/start/transformer-vs-geometry.md).

<details>
<summary><strong>Historical K16 result — 19 Oct 2026</strong></summary>

Before the M1 closing artifacts, the selected K16-A discrete model scored **65 and 463** correct
next-token guesses out of 14,287, below the unigram table's **1,226** and T2's **6,300**. K16-R
collapsed to 19 or 20 occupied cells. These remain dated negative results, not the current best
model: [K16 outcome](docs/labs/k16-outcome-call-2026-10-19/README.md).

Four older 214-million-parameter checkpoints on Hugging Face predate today's rules (they used
softmax and matrix multiplication) and are kept as historical evidence, not as the D11 product.

</details>

## Explore

- **[Start here](docs/start/what-is-uor-r4.md)**, with a [glossary](docs/start/glossary.md) and [FAQ](docs/start/faq.md)
- **[Where we are, with the evidence](docs/start/where-we-are.md)** and the [pipeline map](docs/model/pipeline-map.md)
- **[Mechanism board](docs/mechanisms/README.md):** 28 mechanisms, one page each, with reopen conditions for the PARKED ones
- **[Geometry](docs/geometry/README.md):** primes and zeta phases, quaternions, E8/H4 and the 600-cell, SpiralCore, the Hopf map
- **[Research findings](docs/research/findings.md)** and **[open questions](docs/research/open-questions.md)**
- **[History](docs/research/history.md):** how the project got here, June to October 2026
- **[Archive catalog](archive/CATALOG.md):** every dated record, kept byte-identical and indexed
- **[Charter](CHARTER.md):** the rules (geometry only, no softmax, no matrix multiplication, integer serving) and why each exists
- **[Documentation hub](docs/README.md):** every page, by question

## Contribute

```sh
cargo check --locked -p uor-r4-discrete --lib
cargo test --locked -p uor-r4-discrete --lib
cargo run -q --locked -p uor-r4-claim-wording -- .
```

These checks run no model. Documentation fixes, reproductions of a recorded result and tooling are
welcome; model work is gated by the milestones. Start with [CONTRIBUTING.md](CONTRIBUTING.md). AI
research labs read [CHARTER.md](CHARTER.md) and [AGENTS.md](AGENTS.md).

## Acknowledgements

- **Alex Flom** (UOR Foundation, [@afflom](https://github.com/afflom)): the upstream UOR framework, addressing and Lean sources this project builds on, and review of its early design.
- **Ari** (UOR, [@auser](https://github.com/auser)): migrated the transformerless engine into this workspace and authored the R4G1/graph-compiler design and early certify measurement work.
- **Maura** (UOR, [@maurathat](https://github.com/maurathat)): UOR contributions including uor-addr, and a README correction here (PR #235).
- **Mark (N3mesis)**: geometry research in [NEMESIS-Theory](https://github.com/markrnd87-cmd/NEMESIS-Theory), the octonion/Fano and structure-carrying criteria that informed the substrate reviews.
- **Matthew**: author of SpiralCore (the v63 Cl(0,6) octonion operator convention and the v68/v69 material), whose signed transport is now the trunk's projection.
- **Gerrit** (DarkUnicorn, [LinkedIn](https://www.linkedin.com/in/gerrit-b%C3%B6hme-36b3a9413)): author of GoldSnnail and goldworm-coder, reviewed as external sources for state-layout patterns and evaluation ideas.
- **Wil Dahn** ([W33-Theory](https://github.com/wilcompute/W33-Theory)): theory discussions on the high-level geometry and its allowed pathing; author of the W(3,3) corpus reviewed by the labs.
- **John Haines**: theory discussions on the high-level geometry and its allowed pathing.
- **Ron Brazil**: theory discussions, engineering design, and research and development work.
- **Adam De Delva** ([LinkedIn](https://www.linkedin.com/in/adamdedelva/)): vision, relationships, unending support, product analysis, promotion, and commercialization efforts.
- **Yassine Amraoui** ([LinkedIn](https://www.linkedin.com/in/amraoui-yassine/)): research and development work.
- **Andrew Kadziolka**: development integration, assessment, analysis and conceptual design.
- **Yazan Barakat** ([@Cyborg-Hawk-AI](https://github.com/Cyborg-Hawk-AI)): research and development work on the WebAssembly D11 integer serving runtime (#1172, PR #2654).
- **The AI research labs** (Kimi, Claude, Antigravity, DeepSeek; Codex took part earlier and left the registry), which do most implementation under the owner's direction; owner and principal investigator Casey Allard.

## Citation and license

Cite the software with [CITATION.cff](CITATION.cff) (GitHub's "Cite this repository" button reads it). No
paper is claimed. [MIT](LICENSE); third-party research and dependencies keep their own licenses.
