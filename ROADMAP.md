# UOR-R4 Geometric Language Model — Roadmap

**North Star Mission: Geometric Intelligence.** Build a working geometric language model with **NO transformers**, **NO matrix multiplication in the runtime/inference path**, and **NO floating-point arithmetic in the runtime**. The runtime is integer, fixed-point, and geometric operations only. These runtime constraints are HARD. Training, data preparation, analysis, and offline tooling may use floating-point and matrix multiplication, but nothing transformer, float, or matmul may leak into the serving path.

---

## 1. Multi-Lab Collaborative Structure & Working Agreement

Research is conducted across three parallel, coordinated labs:
- **Director & Lab 1 (Claude)**: Architecture direction, sparse geometric memory addressing (2D Galois lattice), hyperbolic cache memory, and pre-registered ablation ladders.
- **Lab 2 (OpenCode)**: Discrete geometric classification, exact signed $H_4$ roots, read groups, and continuous-to-discrete reader interfaces.
- **Lab 3 (Anti-Gravity)**: Zero-matmul wide integer serving pipeline, register-resident blocked kernels, streaming chat continuity, and Apple Silicon hardware invariant verification.

### Working Agreement

1. **Expert Mode**: Real mathematical, computer-science, and engineering analysis. Prefer novel reasoning and small decisive experiments over literature tourism.
2. **Consolidate, Don't Proliferate**: Survey what exists before starting anything new; finish one coherent path rather than starting another. No new top-level engines without director approval.
3. **Testing Discipline (Light, Not Tight)**: Smoke tests, property checks, one decisive experiment per hypothesis. Do NOT gold-plate tests or chase CI green. Quarantine flaky or peripheral failures, note them, keep the main line moving. Never sit in a testing loop.
4. **Zoom-Out Protocol (Recursive)**: When stuck for ~45+ minutes or finishing a work unit, stop, reread this mission plus `ROADMAP.md`, and write a 3–5 sentence reassessment: *Does this serve geometric intelligence?* Then explicitly choose: continue, pivot, or prune. Mark dead paths dead in the roadmap instead of letting them linger.
5. **GitHub Hygiene**: Canonical repository is `UOR-Foundation/uor-r4`. Work on feature branches (`lab/<lab>/<topic>`), commit with clear messages, push regularly (at least at end of each session), open draft PRs for cross-lab review. Keep main green; merge only with director approval.
6. **Disk Discipline**: Keep the checkout lean — clean stale build artifacts and caches, don't vendor large datasets. Prefer the attached external SSD (`/Volumes/UOR-Workspace`) for build artifacts, datasets, and checkpoints. Never silently fill the disk.
7. **ROADMAP Maintenance**: Maintain `ROADMAP.md` at the repo root and update it constantly — every direction change, completion, or pruning.

---

## 2. Active Research Tracks

### Track 1: Sparse Geometric Memory Addressing (2D Galois Lattice / Product-Key Memory)
- **Goal**: Replace dense MLPs with sparse geometric memory addressing using $H_4$ and $E_8$ fixed codebooks to achieve per-token parameter sparsity under the D5 serving contract.
- **Current Hypothesis**: Moving geometry from dense read scores to sparse memory indexes with fixed $H_4$ (600-cell, 120 icosians) and $E_8$ (240 roots) codebooks reduces parameter reads per token while matching or improving loss over dense Dot/Lorentz baselines.
- **Owning Lab**: Lab 1 (Claude / Director)
- **Status**: Active. PR [#1437](https://github.com/UOR-Foundation/uor-r4/pull/1437) open with `StackConfig.memory`, fused top-$k$ selection, and pre-registered 5-arm comparison plan. Cycle-4 main baseline comparison running.
- **Next Decision Point**: Score Cycle-4 comparison and evaluate Cycle-5 memory arms against pre-registered threshold ($\ge 0.02$ nats improvement).

### Track 2: Exact Signed $H_4$ Integer Classification & Discrete Read Groups
- **Goal**: Provide an exact common-scale i32 classifier over all 120 signed $H_4$ roots without normalized floating-point arithmetic or hardware multiplier instructions.
- **Current Hypothesis**: Scoring 14 fundamental root families under exact integer arithmetic enables canonical directional classification across the full i32 domain, eliminating floating-point dependencies from discrete geometric routing.
- **Owning Lab**: Lab 2 (OpenCode)
- **Status**: Active. PR [#1435](https://github.com/UOR-Foundation/uor-r4/pull/1435) open with exact signed $H_4$ integer classifier in `crates/uor-r4-integer/src/h4.rs`. 3 focused checks pass; strict ARM64 opcode audit passes (0 Class I multipliers, 0 Class II dividers, 0 Class III floats).
- **Next Decision Point**: Verify training donor-order check and decide on integrating exact $H_4$ relation lookup table (`inverse(q) * k`) with learned reader scoring.

### Track 3: Zero-MatMul Wide Integer Serving & Streaming Chat Continuity
- **Goal**: Serve learned geometric dialogue models on Apple Silicon with strictly zero multipliers, zero dividers, zero floats in runtime symbols, latency $\le 4.0$ ms/tok, and peak RSS $< 35$ MB.
- **Current Hypothesis**: Four-row blocked product table reuse with register-resident accumulators and zero-allocation wide affine projections accelerate width 576/1152 evaluation on ARM64 while preserving strict D0-b hardware invariants and exact streaming dialogue history across turns.
- **Owning Lab**: Lab 3 (Anti-Gravity)
- **Status**: Active. PR [#1434](https://github.com/UOR-Foundation/uor-r4/pull/1434) (dialogue continuity and streaming generation in `uor-chat`) and PR [#1436](https://github.com/UOR-Foundation/uor-r4/pull/1436) (four-row blocked product table reuse and zero-allocation wide affine) delivered and synthesized in `lab/anti-gravity/geometric-intelligence-synthesis`. Benchmarks: 0.349 ms `project_vocab`, 1.0008 ms/tok streaming latency, peak RSS 7.06–8.27 MB, 0 multipliers/dividers/floats.
- **Next Decision Point**: Evaluate recovered continuous relations vs quantized representations on the dialogue evaluator, and integrate exact $H_4$ discrete lookups into wide serving.

### Track 4: Response-Aware Dialogue Code-Choice Fitting (Precision Recovery)
- **Goal**: Recover semantic relations lost during nearest-neighbor integer quantization of continuous dialogue children via response-aware legal integer neighbor code learning.
- **Current Hypothesis**: Response/EOS-weighted objective over full-sequence shards with one global rounding penalty and clipped Adam updates over 5.34M choosable coordinates preserves continuous child semantic relations (entity recall, color, role relations) that independent rounding erases.
- **Owning Lab**: Cross-Lab / Fourth Lab Scaffolding (coordinated by Lab 3)
- **Status**: Checkpointed at 231/512 updates (260,430 targets) on `/Volumes/UOR-Workspace/uor-r4-lab/fourth-lab-sequential-code-choice-fit-1`. Continuation runner and spec prepared; integer conversion bridge validated.
- **Next Decision Point**: Complete 58-turn dialogue observation on the 231-step checkpointed codes to verify whether entity recall (Momo, green) recovers before scheduling the remaining 281 continuation updates.

---

## 3. Pruned and Dead Tracks

Paths explicitly pruned to maintain research focus and prevent cyclical experimentation:
- **Recent64 Training Loop**: Dead path. Withdrawn under [D9](docs/integration/DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract). Truncating context to 64 tokens violated the long-horizon context contract without improving efficiency. Replaced by full-context language learning.
- **Unweighted Reader Comparison without Termination Objective**: Pruned under [#1422](https://github.com/UOR-Foundation/uor-r4/pull/1422) and [#1424](https://github.com/UOR-Foundation/uor-r4/pull/1424). Unweighted reader loss masked termination failures and distorted parameter attribution.
- **Floating-Point Matrix Multiplication in Runtime**: Dead path. Prohibited by owner-adopted [D0-b](docs/integration/DECISIONS.md#d0-b--mathematical-linear-maps-in-native-runtime). Serving runtime must remain integer, fixed-point, and table-lookup based.
- **Local Selector Tuning Scaffolds (A1–A4)**: Historical negative evidence. Retained as scaffolds, but selector-only tuning is no longer the active learning ladder.

---

## 4. Capability Direction

The canonical project plan ([docs/integration/project-track.md](docs/integration/project-track.md)) owns the ordered roadmap and acceptance. Live issue [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) is the programme tracker; [docs/integration/current-state.md](docs/integration/current-state.md) owns retained artifacts and changing results.

| Order | Responsibility / Live Issue | Focus & Milestone Goal |
|---|---|---|
| 01 | [#1139](https://github.com/UOR-Foundation/uor-r4/issues/1139) — Learn contextual phrase and role binding | Bounded binding evidence; broader role transfer across context. |
| 02 | [#1140](https://github.com/UOR-Foundation/uor-r4/issues/1140) — Learn shared state transitions and compositional emission | Reusable computation and language construction without head proliferation. |
| 03 | [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) — Integrate geometric model and learn general prose | Integrated native architecture and broad linguistic learning beyond copy attention. |
| 04 | [#962](https://github.com/UOR-Foundation/uor-r4/issues/962) — Develop conversation and identity-scoped durable memory | Meaning surviving turns, corrections, and restarts with user/project isolation. |
| 05 | [#954](https://github.com/UOR-Foundation/uor-r4/issues/954) — Qualify grounded correctness, conflict handling and abstention | Distinguish supported claims from missing/contradictory evidence for reliable reasoning. |
| 06 | [#955](https://github.com/UOR-Foundation/uor-r4/issues/955) — Qualify generalized multi-step reasoning | Composition of accepted operations on novel problems without shortcut memorization. |
| 07 | [#1088](https://github.com/UOR-Foundation/uor-r4/issues/1088) — Develop executable Rust coding and controlled workspace use | Working code generation and repair in real workspace contexts. |
| 08 | [#963](https://github.com/UOR-Foundation/uor-r4/issues/963) — Scale quality with complete-path M1 latency, energy and memory | Verification of latency $\le 4.0$ ms/tok, RSS $< 35$ MB, and complete-path energy savings on consumer hardware. |
| 09 | [#964](https://github.com/UOR-Foundation/uor-r4/issues/964) — Establish scoped serving, geometry and artifact guarantees | Formal contracts for executed serving operations and mathematical invariants. |
| 10 | [#1172](https://github.com/UOR-Foundation/uor-r4/issues/1172) — Complete native capability API and WASM model runtime | Expose native capabilities to application runtimes and web targets. |
| 11 | [#1173](https://github.com/UOR-Foundation/uor-r4/issues/1173) — Run native geometric model in GitHub Pages AI Studio | Client-side local geometric model execution in the browser. |
| 12 | [#965](https://github.com/UOR-Foundation/uor-r4/issues/965) — Qualify, release and iteratively improve local model | Alpha release integrating conversation, coding, and reasoning on consumer laptops. |

---

## 5. Historical Roadmap Notes

The [historical roadmap archive](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/ROADMAP.md) preserves earlier stage locks, fixed timers, and process sequences. Research records ([docs/RESEARCH.md](docs/RESEARCH.md)), the architecture audit ([docs/integration/architecture-2026-09/README.md](docs/integration/architecture-2026-09/README.md)), and the project map ([docs/PROJECT_MAP.md](docs/PROJECT_MAP.md)) locate their source and outcomes.
