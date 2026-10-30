# UOR-R4 Geometric Language Model

*An open research project: can geometry replace a transformer's run-time machinery in a language
model that runs on a laptop?*

[![License: MIT](https://img.shields.io/github/license/UOR-Foundation/uor-r4)](LICENSE)
[![Rust 1.97.1](https://img.shields.io/badge/rust-1.97.1-orange?logo=rust)](rust-toolchain.toml)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-yellow)](STATUS.md)
[![Milestones: 1 of 8 complete](https://img.shields.io/badge/milestones-1%2F8%20complete-blue)](#where-we-are)
[![Last commit](https://img.shields.io/github/last-commit/UOR-Foundation/uor-r4/main)](https://github.com/UOR-Foundation/uor-r4/commits/main)
[![Hugging Face](https://img.shields.io/badge/%F0%9F%A4%97-Hugging%20Face-ffcc4d)](https://huggingface.co/caseyallard/uor-r4-geometric-214m)

![Four panels read left to right: a sentence split into numbered pieces; each piece placed as a point on a geometric object; the newest piece looking at only a few nearby earlier points; a few drawers opening near what was read, with the nearest word inside them picked. A band underneath reads "whole numbers only: adds, compares and table look-ups".](docs/figures/start/hero-how-it-thinks.svg)

*How UOR-R4 is designed to predict the next word. The picture shows the design, not a model that already works well.*

## What this is

Chat assistants run on **transformers**: programs that spend most of their effort multiplying very
large grids of decimal numbers. UOR-R4 bets that a different toolkit can do the same job far more
cheaply: rotations of a geometric object, whole-number addresses built from prime numbers, small
lookup tables and an exact memory. The goal is a fast language model that trains quickly and runs on
your own laptop or in your browser, with nothing sent to a server. The project is open research and
**pre-alpha**: many parts work and pass their checks, but **no model built under its rules holds a
useful conversation yet.** [What UOR-R4 is](docs/start/what-is-uor-r4.md) explains the bet in five minutes.

## The three products

None of the three ships today. The saved model's panel qualification does not yet provide useful conversation or integrated clients ([products](docs/products/README.md)).

- **[The model](docs/products/language-model.md):** one integer model file for local serving, with integer multiply and divide allowed by D80/D81. On 30 Oct 2026 the saved discrete `UORLUT01` artifacts passed M1's six bars at both seeds ([closing evidence](docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)). Separately, the float-trained FBS trunk was exported under `lut-stack/5` and served on the M1 laptop on 29 Oct (claude-182). Neither result qualifies useful conversation.
- **[Web client](docs/products/web-client.md):** the Studio page (the project's chat-style web interface) in your browser, running the model on your own device as WebAssembly. Today the browser engine and the Studio page both exist but are not yet connected.
- **[Desktop client](docs/products/desktop-client.md):** a Tauri app (a desktop shell around the same web page), running the model natively with durable memory and offline use on M1-class laptops. Today it is a written specification only.

## Where we are

**30 Oct 2026:** pre-alpha, **1 of 8 milestones complete**. [M1 (#2029)](https://github.com/UOR-Foundation/uor-r4/issues/2029) is closed: one saved discrete model, exported as two seeded `UORLUT01` artifacts, passed all six registered bars. This is a scoped panel result, **not useful-chat qualification**. The separate float-trained FBS trunk line continues toward one model that produces complete replies; see [PLAN.md](PLAN.md) and the [pipeline map](docs/model/pipeline-map.md).

[The closing run](docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md) reproduced **7,002 / 7,001** correct integer top-1 guesses on DEV's 14,287 rows, above the unigram table's 1,226 and T2's 6,300. On the fresh held-out-phrasing panel p-f (15,953 rows), it scored **7,850 / 7,872**, against T2's 7,375, exceeding the paired band of 81 at both seeds. The same artifacts served at median **10,876 / 10,566 decode tokens/s** on an Apple M1 under that report's 192-token prefill/64-token decode protocol.

**Limit:** 62.5% of p-f's reference replies are verbatim TRAIN replies. That is reference-data overlap, **not a measured generated-copy rate**: bar (f) tests new prompt phrasings leading to known kinds of reply, not new reply kinds or useful conversation.

<details>
<summary>Historical K16 result — 19 Oct 2026</summary>

The following discrete-path measurement predates the M1 closing artifacts. It asks for one next-token guess at each of 14,287 positions in 400 held-out documents:

| Who is guessing | Correct first guesses (of 14,287) | Source |
|---|---:|---|
| Selected trained UOR-R4 model (K16-A), seeds 1 and 2 (two separate random starts) | 65 and 463 | [K16 outcome](docs/labs/k16-outcome-call-2026-10-19/README.md) |
| Unigram table: always guess the most common token | 1,226 | [count tables](archive/labs/host-diagnostics-2026-10-17/README.md) |
| Next-word table (T2): the token that most often followed the previous one | 6,300 | [count tables](archive/labs/host-diagnostics-2026-10-17/README.md) |

The K16 best seed's 463 correct guesses was below both count tables. It remains dated evidence, not the current-best compliant result.

</details>

The full picture, with every number sourced: [where we are](docs/start/where-we-are.md).

## Where we're going

M1's saved-model path is complete under the six bars in [CHARTER §4.8](CHARTER.md#4-evidence), including D82's held-out-phrasing transfer. Full conversation and durable memory remain open; the remaining milestone routes are in [PLAN.md](PLAN.md#the-route-m2m8).

**30 Oct 2026.** The separate float-trained FBS trunk line follows the owner's end-to-end order: carry exact turn memory through the reply, serve the parts in integers, complete the text path (R7), then train/export/serve and score one whole model on prompts with tokens per second; cost follows. The latest completed trained-cut run returned **“reduces, content lost”** and kept `r-drop` as the recipe ([claude-216](docs/loop/tasks/one-model-end-to-end/claude-216/REPORT.md)). The [claude-217 review](https://github.com/UOR-Foundation/uor-r4/issues/3407#issuecomment-6524939582) records FIT, pre-run GO and launch, **not a result**. The remaining [one-model protocol and integer-serving measures](docs/roadmap/items/one-model-end-to-end.md) are still required. This work does not inherit useful-chat qualification from M1.

## How it works

Each transformer part gets a geometric or exact replacement that avoids softmax and matrix
multiplication. Statuses come from the [mechanism board](docs/mechanisms/README.md): **ACTIVE** is
on the current model path, **READY** is built and tested but never trained in, and **PARKED** means
the tested forms failed or broke a rule while the mechanism stays open.

| Transformer part | Geometric replacement in UOR-R4 | Status |
|---|---|---|
| Embedding table (a starting vector per token) | [Prime-zeta codebook](docs/mechanisms/prime-zeta-codebook.md) and [address cells](docs/mechanisms/address-cells.md): whole-number addresses from primes and zeta-zero phases | ACTIVE |
| Attention (which earlier tokens matter) | [Route](docs/mechanisms/route-read.md), [Flock](docs/mechanisms/flock-read.md) and [Relation](docs/mechanisms/relation-read.md) reads: walk an address tree, rotate a few neighbours into one frame, follow copy links | ACTIVE; none of the reads has yet been shown to help on its own |
| Feed-forward layers (most of the multiplying) | [Hamilton rotation gate](docs/mechanisms/hamilton-gate.md) and [SpiralCore/FBS projection](docs/mechanisms/spiralcore-fbs.md) | READY; never trained |
| Softmax output and cross-entropy loss | [Admission](docs/mechanisms/admission.md) (open a few address "drawers", called cells) and a [rank-margin hinge](docs/mechanisms/rank-margin.md) | ACTIVE |
| Backpropagation optimizer | [Discrete learning](docs/mechanisms/discrete-learning.md): try each legal rotation of one entry, keep it only if the score strictly improves | ACTIVE |
| Matrix-multiply runtime | [Integer serving (D11, the whole-number serving contract)](docs/mechanisms/lut-serving.md): adds, shifts, compares and table reads, and no float; since D80 and D81 (28 Oct 2026) the integer multiply and divide are allowed | ACTIVE engine; `lut-stack/5` exports and serves the FBS trunk exactly (claude-181, claude-182) |

Position and normalization now have replacements on the FBS trunk: position is the SpiralCore transport spin, and normalization is RMS norm with an integer square root. Both are trained and served there. The saved discrete model beats the count tables under M1's registered panel bars; this does not establish each replacement's independent effect or useful-chat capability. The board lists 28 mechanisms:
12 ACTIVE, 5 READY and 11 PARKED. Part by part: [transformer vs geometry](docs/start/transformer-vs-geometry.md).

## Explore

- **[Start here](docs/start/what-is-uor-r4.md)**, with a [glossary](docs/start/glossary.md) and [FAQ](docs/start/faq.md)
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
