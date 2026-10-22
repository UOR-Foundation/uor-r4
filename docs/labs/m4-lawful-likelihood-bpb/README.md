# A lawful likelihood for bits-per-byte scoring — design note (deepseek-8, repaired as deepseek-9)

Status: **design only, under the PM's §3.2 reading.** No code is changed; the item stays `ready`. Posted on #2861 before any code, as item step 2 requires. This version answers the counted CHANGES record at `e9748f1`.

## The problem

The integer model emits no probabilities, so the M4 bar (held-out quality at or below 0.90 bits per byte) cannot be scored. The historical integer read normalizes Q24 geometric scores through a fixed Q31 **exp table** (`crates/uor-r4-integer/src/geometric_read.rs`, `EXP_TABLE_LEN = 8194`) — a normalized-exponential form, banned by CHARTER §3.2. The Q48 sampling contract (`sampling.rs`, `PROBABILITY_ONE = 1<<48`) accepts masses summing exactly to one, but nothing lawful produces them. `docs/research/open-questions.md` records: "A lawful likelihood is still unbuilt."

## What the project already has

- **Q24 integer potentials**: `geometric_potential.rs` scores candidates with signed Q24 table reads and exact integer addition on the fixed signed H4 geometry. `MAX_ABS_SCORE < 2^39` (asserted in that file).
- **A Q48 mass contract**: `sampling.rs` requires per-token masses summing exactly to `1<<48`; greedy, categorical and Min-P selection consume it.
- **Prior art for no-normalization reads**: the Flock read keeps candidates by hard integer rank and is explicit that "nothing is normalised, no scores are exponentiated, and no weights are multiplied" (`docs/mechanisms/flock-read.md`); the softmax-free read experiments' learned rank table is the nearest historical relative (`archive/` record, +0.0185 BPB in its arm).
- **Host evaluation paths**: `crates/uor-r4-discrete`'s `eval-corpus` counts/reports, and `data/panels/` (frozen conversational panels, `MANIFEST.sha256`).

## Proposed construction: potential-ratio lawful likelihood (PRLL)

1. **Scores unchanged.** At a scored step, `s[0..V)` are the model's Q24 signed integer scores on the admitted causal prefix. No exponential and no table is applied.
2. **Non-negative potentials, order-preservingly.** `phi[i] = (s[i] - min(s)) + 1` in exact i64. The shift preserves order and ties exactly; every `phi[i] >= 1`.
3. **Every token gets a legal minimum, then the ratio.** With `V` candidates and `R = 2^48 - V` (positive for every vocabulary this model uses: `config.rs` fixes `vocab_size = 4096`):
   - `base[i] = 1`;
   - `quota[i] = floor(phi[i] * R / sum(phi))`, in **u128** (`phi*R` can approach `2^88`);
   - give the remaining `R - sum(quota)` units one each to the largest fractional remainders (`phi[i]*R mod sum(phi)`, ties by **lowest token id**);
   - `mass[i] = 1 + quota[i] + (0 or 1)`, so `mass[i] >= 1` and `sum(mass) = V + R = 2^48` exactly.
   The `+1` floor is what makes zero mass impossible: with `V=4096`, `MAX_ABS_SCORE < 2^39` and 4,095 maximal scores, `sum(phi) < 2^52`, so the minimum-potential token's proportional quota can floor to 0 — the base unit is what keeps its bits finite.
4. **Bits, in integer fixed point.** For a scored token with `mass m in [1, 2^48]`: `bits = 48 - log2(m)`.
   - `log2(m)` is computed exactly in structure and approximately in the fraction: integer part `k = floor(log2 m)` from the bit length; fractional part by 12 repeated-squaring rounds (normalize `m/2^k` to `x in [1,2)`, square and halve 12 times, reading each comparison bit).
   - Accumulate `bits` scaled by `2^12` as integers: `bits_q12 = 48*4096 - (k*4096 + fractional_q12)`. The per-token error is at most `2^-12` bit; `bits per byte = sum(bits_q12) / (4096 * bytes)`, reported as an exact integer numerator and denominator and never as a rounded float.
   - **No float anywhere**, so §3.7's float boundary does not apply and the discrete crate can host the scorer.

### Why this contains no banned function

- No softmax, log-softmax, softplus, sigmoid, tanh, GELU, normalized exponential, or renamed/table form of one: the normalizer is a linear ratio of non-negative integer potentials, with no exponential map and no exp table.
- No squashing: the map is identity-or-shift, not a bounded curve.
- No dense matmul or GEMM: per-candidate integer arithmetic plus one exact u128 ratio per candidate.
- **Division is not in the artifact.** The D11 artifact emits integer **potentials only**; the ratio and the remainder distribution run in the offline GPU scoring harness, so no integer-divide instruction is emitted in the artifact. If an artifact-side normalizer is ever wanted, `stack_div_u128` (`stack/kernels.rs:304`, restoring divide) is the existing audited way, with the u128 width stated above.
- **The geometry claim is softened.** The board's Normalization slot reads "**GAP: none.** Snapping to an E8 root may serve, but no record says so" (`docs/mechanisms/README.md`). PRLL is a *scoring* normalization applied to the model's own H4 potentials; it does **not** claim the model has a normalization mechanism, and the board's gap stands. The Flock read's no-normalization rank read is the closest existing precedent, not a proof of equivalence.

### Why it is lawful (testable properties)

1. Every mass is a non-negative integer **at least 1**.
2. The masses sum to exactly `2^48` — checked, never assumed.
3. Order and argmax are preserved (ties by lowest token id).
4. Relabeling candidates permutes masses exactly; ties are keyed by token id, not slot index, so the mapping is well defined before and after a permutation (a tied control is required).
5. Equal scores give exactly equal masses **when `V` divides `2^48`** (true for every power-of-two vocabulary, including 4096); otherwise the remainder units are deterministic and the masses differ by at most one, which the control states explicitly.
6. The source uses no banned operation; the scorer's bit arithmetic is integer only.

### Finite host controls (before any GPU run)

1. **Tiny alphabet, enumerated**: V=4 hand-computed Q24 scores; exact expected masses, exact sum `2^48`.
2. **Worst-case zero-mass check**: V=4096, one score at `-MAX_ABS_SCORE` and the rest at `+MAX_ABS_SCORE`, the true token the minimum — its mass must be exactly 1 and its bits exactly 48 (finite), the trap the first note missed.
3. **Uniform case**: power-of-two V (4096) gives exactly equal masses; a non-power-of-two V gives deterministic ±1 remainder masses exactly as specified.
4. **Ties**: two candidates with equal scores across the remainder split; ties resolved by token id, asserted in both slot orders.
5. **Monotonicity/argmax** and **permutation equivariance** property tests over random Q24 vectors.
6. **BPB arithmetic**: a synthetic sequence with known masses yields exactly the analytic `bits_q12` and bits per byte; a uniform baseline over V tokens scores `log2(V)` bits per token.
7. **Count-table baseline**: the primary baseline is the count model's own maximum-likelihood probabilities (`count_i / total`) through the same bits arithmetic, **not** through the PRLL shift. A secondary control runs add-one-smoothed counts through PRLL and is labelled "add-one smoothed", because the shift changes a minimum count above zero.

## Crate choice

- **Pure scorer**: `crates/uor-r4-integer`, beside the Q24 potentials and the Q48 contract it consumes; it is the crate that already owns the score/value arithmetic and `stack_div_u128`.
- **Evaluation integration and the GPU run's report**: `crates/uor-r4-discrete`'s `eval-corpus` path, which already owns the panel scoring and the report schema. The integer crate must not gain GPU dependencies.

## What the PM decides (not this note)

1. **Is a linear full-alphabet ratio within CHARTER §3.2?** The ban's stated reason is that squashing functions "hide a dense global normalization", and `sum(phi)` over the full alphabet is such a normalization. The item itself requires an exact sum over the alphabet, so this reading is the PM's call — asked on #2028 as a decision card; this note calls nothing lawful until answered.
2. **Is `-log2(mass)` a measurement or the banned "cross-entropy"?** This note reads it as a measurement (cross-entropy is banned as an *implicit softmax*); a card is needed only if disputed.
3. **The held-out stream** (file, byte count, sha256) — the pre-registration names and freezes it.
4. **Calibration** — PRLL is lawful but not temperature-calibrated; whether the 0.90 bar may be read through an uncalibrated lawful likelihood is a PM/owner call.
5. **Family choice** — PRLL vs a polynomial distance kernel vs an occurrence-count ratio; each is its own pre-registration arm.

## What this unit does and does not do

- Does: the revised design note and its record, the issue post, ORDER and REPORT, the cancel probe, and the two PM questions.
- Does not: change code or tests, run any model or GPU work, name the held-out stream, register an instrument, or claim BPB numbers. The item stays `ready`; implementation and registered scoring are later units.

## Sources

- Item: [m4-lawful-likelihood-bpb](../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861); CHARTER §3.2, §3.6, §3.7, §4.8.
- Scores: `crates/uor-r4-integer/src/geometric_potential.rs`; Q48 contract: `sampling.rs`; restoring divide: `stack/kernels.rs:304`; vocabulary: `config.rs:86`.
- Banned historical path: `crates/uor-r4-integer/src/geometric_read.rs` (Q31 exp table).
- Normalization gap and Flock precedent: `docs/mechanisms/README.md:142`; `docs/mechanisms/flock-read.md`.
- Open question: `docs/research/open-questions.md:102`.