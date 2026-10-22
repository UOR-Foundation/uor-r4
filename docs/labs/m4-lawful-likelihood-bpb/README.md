# A lawful likelihood for bits-per-byte scoring — design note (deepseek-8, 22 October 2026)

Status: **design only**. No code is changed in this unit; the item stays `ready`. This note is posted on #2861 before any code, as the item's step 2 requires.

## The problem

The integer model emits no probabilities, so the M4 bar (held-out quality at or below 0.90 bits per byte) cannot be scored. The historical integer read normalizes Q24 geometric scores through a fixed Q31 **exp table** (`crates/uor-r4-integer/src/geometric_read.rs`, `EXP_TABLE_LEN`) — a normalized-exponential form, which CHARTER §3.2 bans. The Q48 sampling contract (`sampling.rs`, `PROBABILITY_ONE = 1<<48`) accepts a normalized mass vector, but nothing lawful produces one. `docs/research/open-questions.md` records: "A lawful likelihood is still unbuilt."

## What the project already has

- **Q24 integer potentials**: `geometric_potential.rs` scores candidates with signed Q24 table reads and exact integer addition on the fixed signed H4 geometry. This is the geometric quantity to build on.
- **A Q48 mass contract**: `sampling.rs` requires per-token masses summing exactly to `1<<48`; greedy, categorical and Min-P selection already consume that contract.
- **Exact integer machinery**: fixed-point tables, checked arithmetic and restoring-division/shift kernels in `stack::kernels`, plus the `eval-corpus` counting path in `crates/uor-r4-discrete`.
- **A held-out panel corpus**: `data/panels/` (frozen conversational panels, `heldout-clean-ids.txt`), for the later registered scoring unit.

## Proposed construction: potential-ratio lawful likelihood (PRLL)

1. **Take the existing geometric scores, unchanged.** For a step, let `s[0..V)` be the model's Q24 integer scores (signed) for the candidate tokens on the admitted causal prefix. No exponential and no table is applied to them.
2. **Make them non-negative potentials, order-preservingly.** `phi[i] = (s[i] - min(s)) + 1`, in Q24 integers. The subtraction is exact i64 arithmetic; `min(s)` is the lowest candidate score; the `+1` keeps every candidate representable and strictly positive. This is the identity-or-shift map: order and ties are preserved exactly, and argmax(s) = argmax(phi).
3. **Normalize by the ratio of potentials.** `mass[i] = floor(phi[i] * 2^48 / sum(phi))`, then distribute the remainder `2^48 - sum(mass)` one unit at a time to the largest fractional remainders, ties resolved by lowest token id. The result sums to exactly `2^48` and each mass is a non-negative integer, so it is a lawful probability over the vocabulary in the Q48 contract's own arithmetic.
4. **Bits from the mass.** For a scored target token `t`, `bits = -log2(mass[t] / 2^48) = 48 - log2(mass[t])`. The scorer computes this per token; `bits per byte = total bits / total scored bytes`. `log2` is a measurement operation (offline scoring), not a model mechanism, and none of §3.2's banned forms is used.

### Why this contains no banned function

- No softmax, log-softmax, softplus, sigmoid, tanh, GELU, normalized exponential, or a renamed/table form of any of them: the only normalizer is a linear ratio of non-negative integer potentials, with no exponential map and no exp table.
- No squashing: the map is the identity-or-shift (`x - min + 1`), not a bounded curve.
- No dense matmul or GEMM: per-candidate integer subtraction/addition and one exact ratio per candidate; nothing dense.
- The geometric object is preserved: scores and potentials are the model's own H4 table values, and the construction is the L1 radial projection of the potential vector onto the probability simplex — a geometric normalization, not a distribution from a different family.

### Why it is "lawful" (testable properties)

1. Every mass is a non-negative integer (`>= 0`).
2. The masses sum to exactly `2^48` — checked, never assumed; the largest-remainder step is what makes this exact.
3. Order is preserved: `s[i] < s[j]` implies `mass[i] <= mass[j]`, and argmax is unchanged (ties by lowest token id).
4. Equivariance: permuting candidates permutes masses exactly; adding a constant to every score changes nothing.
5. Uniform scores produce exactly uniform masses (with the remainder rule deterministic).
6. The construction uses no banned operation in its source (a source audit plus a compiled-instruction audit where the crate's policy requires one).

### Finite host controls (before any GPU run)

1. **Tiny alphabet, enumerated**: V=4 with hand-written Q24 scores; enumerate all four masses, assert exact sum `2^48`, assert each expected value by hand.
2. **Uniform case**: all-equal scores give `mass[i] = 2^48 / V` plus a deterministic remainder distribution; assert exact sum and the exact per-token vector.
3. **Monotonicity/argmax**: property test over random Q24 vectors; assert monotonicity, argmax and the tie rule.
4. **Permutation equivariance**: shuffle candidates; assert masses permute exactly.
5. **BPB arithmetic**: a multi-token sequence with known masses produces exactly the analytic bits per byte; a uniform baseline over V tokens scores exactly `log2(V)` bits per token.
6. **Count-table baseline through the same scorer**: a unigram table over the same vocabulary is normalized by PRLL and scored by the same code, so the instrument is compared against a non-model baseline built the same way.

## Open choices for the PM and FIT review (not decided here)

1. **Normalization family.** PRLL (shifted-linear) is the simplest lawful candidate and preserves order. A polynomial distance kernel (e.g. `1/(1+max-s)`) or an occurrence-count ratio from `geometric_occurrence_read` are alternatives; each would be its own pre-registration arm, not a silent substitution.
2. **Calibration.** A non-exponential normalizer is not temperature-calibrated; whether the 0.90 BPB bar should be read as "a lawful likelihood, uncalibrated" or must wait for a calibrated family is a PM/owner call, not this note's.
3. **The held-out stream.** The item says the sources do not name it: the pre-registration must name the file, byte count and sha256 (candidates: `data/panels/` and the registered held-out corpus).
4. **Arithmetic in the scoring path.** Exact division is used once per candidate in the scorer; whether the compiled-instruction audit forbids the division instruction on this path (it is offline scoring, not artifact serving) is a fit-review question.
5. **Where the code lives.** Probable home: the integer crate's scoring path plus an `eval-corpus`-side BPB scorer; the design note is silent on the final crate boundary until the family is chosen.

## What this unit does and does not do

- Does: the design note and its record (this page), the issue post, the ORDER and REPORT, and the cancel probe. The item stays `ready`.
- Does not: change any code or test, run any model or GPU work, name the held-out stream, register an instrument, or claim BPB numbers. The next unit chooses the family with the PM/FIT and implements the finite controls; the registered scoring run needs its own pre-registration, non-author FIT review and owner release.

## Sources

- Item: [m4-lawful-likelihood-bpb](../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861), CHARTER §3.2, §3.7, §4.8.
- Historical read and its exp table: `crates/uor-r4-integer/src/geometric_read.rs`; potentials: `crates/uor-r4-integer/src/geometric_potential.rs`; Q48 contract: `crates/uor-r4-integer/src/sampling.rs`.
- Open question: `docs/research/open-questions.md` ("A lawful likelihood is still unbuilt").
- Panels: `data/panels/README.md` and `MANIFEST.sha256`.