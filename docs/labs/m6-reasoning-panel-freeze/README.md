# M6 Reasoning and Coding Frozen Panel & Oracles (`m6-reasoning-panel-freeze` #2866)

- **Status:** Frozen and verified on finite host controls ([`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs), [`crates/uor-r4-harness/tests/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/tests/panel_m6_reasoning.rs), [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json)); GPU qualification (`m6-qualify-reasoning-coding` `#2865`) remains HELD pending M1 trunk training (`m1-trunk-assembly-and-training` `#2845`), owner bar confirmation ([`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280)), and Casey's typed D40 release.
- **Milestone:** M6 (`#2034`). **Item:** [`m6-reasoning-panel-freeze`](../../roadmap/items/m6-reasoning-panel-freeze.md) (`#2866`). **Lab:** antigravity.
- **Design note:** [`#2034` comment `6344264079`](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6344264079), [`#2866` comment `6344265209`](https://github.com/UOR-Foundation/uor-r4/issues/2866#issuecomment-6344265209). **Bar decision card:** [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280).

## 1. Seeded Panel Synthesis and Content Seal

The `m6-reasoning` acceptance panel is synthesized deterministically in pure Rust by `uor_r4_harness::panels::m6_reasoning::generate_frozen_m6_panel()` from canonical seed `M6_PANEL_FROZEN_SEED_V1 = 0x004D_365F_5245_4153` (`"M6_REAS"` in ASCII), wired into [`crates/uor-r4-harness/src/panels.rs`](../../../crates/uor-r4-harness/src/panels.rs). By generating the exact JSONL bytes in-process from the frozen seed and sealing their SHA-256 digests in [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json) and [`docs/reference/storage.md`](../../reference/storage.md), no bulk corpus file is committed to git (CHARTER §3.13):

| Stream | Generator call | Row count | Byte length | SHA-256 content hash |
|---|---|---:|---:|---|
| Unscored evaluation prompts (`M6UnscoredPromptRow`) | `generate_frozen_m6_panel().to_canonical_jsonl_bytes(false)` | `240` | `79,215 B` | `ac2034eda9b8ade5b6c760461a2c40c167955289a7f4a53c5ad1081894ab3edc` |
| Sealed oracle specification (`M6FrozenRow`) | `generate_frozen_m6_panel().to_canonical_jsonl_bytes(true)` | `240` | `206,422 B` | `d1e5e4eb295d06d711b90e79805e6e35a84caa47666baeed6e202e203d407203` |

## 2. Three Task Families and Deterministic Oracles (D28 Compliant)

Every row is graded by `grade_m6_row` / `grade_m6_panel` in [`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs) with zero neural comparator, teacher, or judge (CHARTER §3.10, D28):

1. **`Compositions` (`80` rows, `m6-comp-000` .. `m6-comp-079`) — Unseen multi-step compositions:**
   - Synthesized from SplitMix64 stream `seed ^ 0x434F_4D50_5F56_3100` over 4- to 5-digit integers in `[1_200, 78_999]` with at least `2` decimal carries (`count_decimal_carries(a, b) >= 2`) and at least `1` decimal borrow (`count_decimal_borrows(a + b, c) >= 1`).
   - `40` rows (`m6-comp-000` .. `m6-comp-039`) are 3-step register chains (`T1 = A + B` into `R0`, `T2 = R0 - C` into `R1`, `OUT = R1 + D`).
   - `40` rows (`m6-comp-040` .. `m6-comp-079`) are 3-step versioned addressed-memory chains (`R0@initial = A`, `R0@previous = R0@initial + B`, `R0@current = (R0@previous * scale) - C`, querying `R0@initial` or `R0@previous` alongside `R0@current`).
   - **Oracle (`grade_composition_row`):** Requires exact integer equality on **both** the intermediate register/memory state (`INT=<i64>`) and the final output (`OUT=<i64>`). Corrupting `INT` while keeping `OUT` exact fails with `M6OracleDiagnostic::IntermediateStateMismatch`.
2. **`AlteredConstraints` (`80` rows, `m6-alt-000` .. `m6-alt-079`) — Altered constraints:**
   - Synthesized from SplitMix64 stream `seed ^ 0x414C_5443_5F56_3100` across 4 sub-families (`20` rows each), every row verified to satisfy `expected_answer != default_habit_answer`:
     - `NonDecimalRadix` (`20` rows): 3-digit addition in base $B \in \{7, 8, 9\}$ using digits `1..B` with a mandatory base-$B$ carry so a base-10 habit reading yields a distinct wrong numeral (`default_habit_answer`).
     - `ReboundPrecedence` (`20` rows): expressions `a [+] b [*] c` where `[+]` binds tighter than `[*]` (`(a + b) * c != a + (b * c)`).
     - `ModularStepCap` (`20` rows): 2-step modular transitions where hitting `S1_raw mod M == forbidden_rem` adds `fallback_delta` before `(S1 + c) mod M`.
     - `UnsatisfiableConstraint` (`20` rows): contradictory constraints (unsigned subtraction underflow without wraparound or even-lattice parity contradiction) where the sole lawful output is `ABSTAIN:UNSATISFIABLE`.
   - **Oracle (`grade_altered_constraint_row`):** Requires exact `expected_answer` on the `60` satisfiable rows (explicitly classifying `default_habit_answer` as `M6OracleDiagnostic::HabitCaptureError` and `ABSTAIN:UNSATISFIABLE` as `FalseAbstentionOnSatisfiableRow`), and exact `ABSTAIN:UNSATISFIABLE` on the `20` unsatisfiable rows.
3. **`CodeTasks` (`80` rows, `m6-code-000` .. `m6-code-079`) — Executable coding tasks:**
   - Synthesized from SplitMix64 stream `seed ^ 0x434F_4445_5F56_3100` across 4 parameterized integer algorithm families (`20` rows each): `AffinelyScaledClampLoop`, `RadixDigitWeightedSum`, `BoundedRingStep`, and `PiecewiseParityRecurrence`.
   - **Safe execution & oracle (`SandboxProgram`, `grade_code_task_program`, `grade_code_task_row`):**
     - Candidate programs execute inside `SandboxProgram::execute`, a total 8-register integer machine with a hard instruction cap (`MAX_SANDBOX_STEPS = 256`), checked integer arithmetic (`checked_add/sub/mul/div/rem`), zero heap allocation, and zero filesystem/network/process I/O.
     - Each row also provides `SandboxProgram::to_rust_source`, emitting a standalone Rust module (`pub fn solve(x: i64) -> i64` and `#[test] fn verify_vectors()`) for offline `rustc --test` / `cargo test` verification.
     - Passing requires matching all `4 / 4` held-out `(input, expected_output)` test vectors (`V0=<y0>;V1=<y1>;V2=<y2>;V3=<y3>`) within `256` steps with zero runtime traps.

## 3. Overlap Exclusion and Lawful Count-Table Baseline Noise (CHARTER §3.10, §4.4)

- **Overlap exclusion (`M6ReasoningPanel::verify_zero_overlap`):**
  1. Every row carries the `[M6-PANEL-V1::<family>::<row_id>]` domain header and structured multi-digit / register / Rust specification disjoint from `data/tinystories_train.u16`, `v5-reply`, and `m2-reply512`.
  2. `FORBIDDEN_CONTROL_PAIRS` (`18` pairs) and `FORBIDDEN_CONTROL_TRIPLES` (`32` triples) explicitly exclude every operand tuple used in the finite CPU controls of [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) and [`crates/uor-r4-integer/src/m6_ablation.rs`](../../../crates/uor-r4-integer/src/m6_ablation.rs).
  3. Independent seeds (`M6_PANEL_FROZEN_SEED_V1` vs `M6_PANEL_FROZEN_SEED_V1 ^ 0xDEAD_BEEF_CAFE_BABE`) are verified to have `0 / 240` prompt overlap.
- **Lawful non-neural count-table baseline (`M6CountTableBaseline`, `evaluate_count_table_baseline_noise`):**
  - Evaluated across `8` independent seeded training draws (`num_seeds = 8`, `50%` sub-sampled training panel per seed):
    - **Unconditioned numeric count-table scores:** `[2, 1, 1, 2, 0, 0, 0, 0]` (`0..=2 / 240`, arising from small-modulus residue collisions on `ModularStepCap` rows; `0 / 80` on `Compositions` and `0 / 80` on `CodeTasks`).
    - **Context-keyed count-table scores:** `[12, 16, 17, 18, 14, 17, 12, 18]` (min `12 / 240`, max `18 / 240`, range `6` rows, measured half-band **`+-3 rows`**).
    - **Family-conditioned modal count-table / best-constant ceiling:** `20 / 240` (`8.33%`, hitting all `20` `ABSTAIN:UNSATISFIABLE` rows in `AlteredConstraints`, `0 / 60` satisfiable `AlteredConstraints`, `0 / 80` `Compositions`, `0 / 80` `CodeTasks`).

## 4. Research Notebook Record (Six-Column Format for `m6-reasoning-coding`)

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-22 | `m6-reasoning-panel-freeze` (`#2866`): deterministic 240-row `m6-reasoning` generator (`0x004D365F52454153`), 3-family D28 oracles (`Compositions`, `AlteredConstraints`, `CodeTasks`), overlap exclusion against `m6_exact_arithmetic.rs` controls, and 8-seed `M6CountTableBaseline` noise evaluation, CPU finite check. | `240 / 240` oracle positive controls (`1.0000`), `0 / 240` negative controls (`60 / 60` habit-capture errors flagged); count-table context baseline `12..=18 / 240` (`+-3 rows` noise), modal ceiling `20 / 240` (`0.0833`); unscored SHA-256 `ac2034ed…`, sealed SHA-256 `d1e5e4eb…`. | Proposed bar `>= 168 / 240` (`>= 56 / 80` per family) across `>= 2` seeds ([`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280)). | **positive** (panel frozen & FIT) | [`panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs), [`m6-reasoning.json`](../../harness/panels/m6-reasoning.json) |

---

## 5. Pre-Registration and Fitness Review of the Frozen `m6-reasoning` Panel (`#2866` / `#2034`)

### Bar

- **Registered in [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json) and proposed on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) for owner confirmation (CHARTER §5.2):**
  1. `overall-pass-240`: `>= 168 / 240` (`70.0%`, `+148` rows above the `20 / 240` count-table modal ceiling, `49.3x` the measured `+-3` row noise band) across `>= 2` seeds.
  2. `compositions-80`: `>= 56 / 80` (`70.0%` exact intermediate + final match on unseen 3-step compositions).
  3. `altered-constraints-80`: `>= 56 / 80` (`70.0%`, requiring `>= 36 / 60` on satisfiable altered-constraint rows plus `ABSTAIN:UNSATISFIABLE` discrimination).
  4. `code-tasks-80`: `>= 56 / 80` (`70.0%` passing all 4 held-out test vectors in the 256-step sandbox and `rustc --test` verifier).

### Base artifact

- **Panel generator & deterministic grader:** `uor_r4_harness::panels::m6_reasoning` ([`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs)) and [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json).
- **Candidate model artifact (for `#2865`):** D11 discrete artifact (`uor-r4.lut-discrete/1`) exported from the M1 trunk (`m1-trunk-assembly-and-training` `#2845`), evaluated with [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) and [`crates/uor-r4-integer/src/m6_ablation.rs`](../../../crates/uor-r4-integer/src/m6_ablation.rs).

### Data split

- **Frozen evaluation split:** `240` rows (`80` `Compositions`, `80` `AlteredConstraints`, `80` `CodeTasks`) from canonical seed `0x004D365F52454153`:
  - Unscored evaluation prompt stream: `79,215` bytes, SHA-256 `ac2034eda9b8ade5b6c760461a2c40c167955289a7f4a53c5ad1081894ab3edc`.
  - Sealed oracle specification stream: `206,422` bytes, SHA-256 `d1e5e4eb295d06d711b90e79805e6e35a84caa47666baeed6e202e203d407203`.
  - Verified zero overlap (`verify_zero_overlap()`) with `FORBIDDEN_CONTROL_PAIRS`, `FORBIDDEN_CONTROL_TRIPLES`, training corpora, and independent seeds.

### Steps

1. Run `cargo test --locked -p uor-r4-harness --test panel_m6_reasoning` to verify byte-exact SHA-256 seals (`ac2034ed…` and `d1e5e4eb…`), overlap exclusion, positive/negative oracle controls (`240/240` vs `0/240`), and the 8-seed `M6CountTableBaseline` noise band (`+-3 rows`).
2. Obtain non-author `TEST FITNESS: FIT` verification of the overlap exclusion, the deterministic 3-family oracles, and the SHA-256 content seal.
3. Keep M6 GPU qualification (`m6-qualify-reasoning-coding` `#2865`) HELD until M1 trunk training (`#2845`), owner bar confirmation on `#2028`, PM dispatch, and Casey's typed D40 release.

### Stop conditions

- Any change to `unscored_sha256` (`ac2034ed…`) or `sealed_oracle_sha256` (`d1e5e4eb…`), any overlap with `FORBIDDEN_CONTROL_PAIRS` / `FORBIDDEN_CONTROL_TRIPLES`, or any failure of the positive/negative oracle controls halts evaluation immediately.

### Cost and minimum test power

- **Cost:**
  - `preparation`: `\$0.00` (host-only deterministic Rust generator, oracles, and count-table baseline noise evaluation).
  - `run`: `\$0.00` in this PR (this item freezes the panel and releases no GPU run).
  - `controls`: `240` positive oracle controls, `240` negative/perturbed/habit-capture controls, `240` constant-abstention controls, and `8` seeded `M6CountTableBaseline` draws.
  - `evaluation`: deterministic integer and 256-step sandbox grading (`grade_m6_panel`).
  - `retries`: `0`.
  - `storage`: `0 B` bulk binary corpus committed to git; `79,215 B` unscored and `206,422 B` sealed JSONL streams reproduced in-process from seed `0x004D365F52454153`.
- **Minimum test power (D22 §3):**
  - `240` fresh rows (`>= 200` D22 §3 floor), `80` rows per family, evaluated across `>= 2` independent seeds against the measured `+-3 rows` noise band and paired McNemar discordant floor $F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$.

TEST FITNESS: FIT
