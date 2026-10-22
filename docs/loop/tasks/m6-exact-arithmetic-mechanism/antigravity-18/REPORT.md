# REPORT: m6-exact-arithmetic-mechanism, antigravity-18

Lab: antigravity. Item: [m6-exact-arithmetic-mechanism](../../../../roadmap/items/m6-exact-arithmetic-mechanism.md), closes #2864.

## What did not work

- **Shared role-prime + place-prime factor keys cross-talked across places and masked capacity eviction:** The first draft of `OperandRole::place_key(place)` built a two-prime `PrimeFactorKey([role_prime, place_prime])` where all places of a role shared `role_prime` and all roles shared `place_prime`. Because `IntegerAddressedMemory::lookup` admits any active slot with `shared_primes >= 1` (`gcd(q, A_c) > 1`), querying an evicted `(OperandA, place 0)` slot after `OperandB` overwrote the ring matched `(OperandB, place 0)` on the shared `place_prime` instead of reaching the eviction log. Replacing shared role/place primes with pairwise-coprime two-prime keys `(nth_odd_prime(2 * slot_idx), nth_odd_prime(2 * slot_idx + 1))` where `slot_idx = (role_index << 5) + place` makes $\gcd(q_{r_1, p_1}, A_{r_2, p_2}) > 1 \iff (r_1, p_1) = (r_2, p_2)$, eliminating cross-talk and ensuring evicted operand slots always return `MemoryLookupOutcome::Evicted`.
- **Fixed-width `max_places = 4` engine rejected 5-digit carry sums in the knockout test:** Running `4789 + 5643 = 10432` on the `max_places = 4` engine built for the subtraction underflow check tripped the `addition carry overflowed engine max_places bound` guard at place 4; configuring the 16-item knockout sweep with `max_places = 8` (`wide_engine`) separated the narrow-width overflow refusal control from the multi-step causal knockout sweep.
- **No GPU evaluation was run:** Under CHARTER §4.5 and §5.1, GPU evaluation of the served M6 model remains HELD pending `m6-reasoning-panel-freeze` (`#2866`), M1 trunk training (`#2845`), owner confirmation of the proposed bar on `#2028`, a non-author `TEST FITNESS: FIT` review on [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md), and Casey's approval.

## Result

- Posted the M6 exact geometric arithmetic design note and bar decision card on [`#2034`](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6342897026), [`#2864`](https://github.com/UOR-Foundation/uor-r4/issues/2864#issuecomment-6342897842), and [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483).
- Implemented [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) (`OrbitDigitAtom`, `OperandRole`, `PlaceStepRecord`, `GeometricArithmeticTrace`, `GeometricArithmeticEngine`, `u64_to_atoms`, `atoms_to_u64`) wired into [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../crates/uor-r4-integer/src/discrete_d11.rs), [`crates/uor-r4-integer/src/native_geometry.rs`](../../../../crates/uor-r4-integer/src/native_geometry.rs), and [`crates/uor-r4-integer/src/serving_root.rs`](../../../../crates/uor-r4-integer/src/serving_root.rs):
  - Multi-digit integers in radix $B \in \{2, \dots, 16\}$ (including decimal $B=10$ and full-orbit hexadecimal $B=16$) are represented as LSB-first 8D integer atoms on the 16-root SpiralCore $E_8$ anchor orbit (`ROOTS[0..16]`), recovered via `snap(x, 1)` and verified on-orbit via `root_id`.
  - Each place transition applies the canonical 64-state $\mathrm{Cl}(0,6)$ signed coordinate permutation `BFS[r_a * 16 + r_out]` with exact inverse $g^{-1} = g^3 = \mathrm{COMPOSE}[g, \mathrm{COMPOSE}[g, g]]$ and a three-valued carry/borrow trit switch in `{-1, 0, +1}`.
  - Multi-digit multiplication executes as multiplier-free place-shifted repeated addition (`mul_atoms_shift_and_add`) over `IntegerAddressedMemory` (`[Initial, Previous, Current]` versioned accumulator slots).
- Recorded the four finite CPU control rows (in six-column notebook format) and the 8-block pre-registration specification (`TEST FITNESS: PENDING`) in [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md), and updated [`docs/geometry/spiralcore.md`](../../../geometry/spiralcore.md), [`docs/mechanisms/spiralcore-fbs.md`](../../../mechanisms/spiralcore-fbs.md), [`docs/mechanisms/ternary-xor.md`](../../../mechanisms/ternary-xor.md), [`docs/roadmap/items/m6-exact-arithmetic-mechanism.md`](../../../roadmap/items/m6-exact-arithmetic-mechanism.md), and [`docs/roadmap/README.md`](../../../roadmap/README.md).

## Files changed

- `crates/uor-r4-integer/src/discrete_d11.rs`
- `crates/uor-r4-integer/src/m6_exact_arithmetic.rs`
- `crates/uor-r4-integer/src/native_geometry.rs`
- `crates/uor-r4-integer/src/serving_root.rs`
- `docs/geometry/spiralcore.md`
- `docs/labs/m6-exact-arithmetic-mechanism/README.md`
- `docs/loop/tasks/m6-exact-arithmetic-mechanism/antigravity-18/ORDER.md`
- `docs/loop/tasks/m6-exact-arithmetic-mechanism/antigravity-18/REPORT.md`
- `docs/mechanisms/spiralcore-fbs.md`
- `docs/mechanisms/ternary-xor.md`
- `docs/roadmap/README.md`
- `docs/roadmap/items/m6-exact-arithmetic-mechanism.md`

## Gates and tests

- `~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic`: 4 passed (`control_orbit_digit_and_cl06_inverse`: `64/64` $g^4 = I$ and `256/256` root transitions + inverses; `control_ripple_carry_add_and_sub`: `32/32`; `control_addressed_memory_composition_and_multiply`: `16/16`; `control_refusal_and_causal_knockouts`: `5/5` invalid/underflow/overflow/eviction refusals and `16/16 -> 0/16` causal knockout drop).
- `~/.cargo/bin/cargo test -p uor-r4-integer --lib --features serving-wasm -- m6_exact_arithmetic`: 4 passed.
- `~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: 88 items OK.
- `uor-harness check`: `0 error`.
- `uor-harness loop gates --full`: all 6 gates GREEN.

## Remaining gaps

- GPU evaluation and served-model qualification (`m6-qualify-reasoning-coding` `#2865`) remain HELD pending `m6-reasoning-panel-freeze` (`#2866`), M1 trunk training (`#2845`), owner confirmation of the proposed bar on `#2028`, a non-author `TEST FITNESS: FIT` review on [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md), and Casey's approval.
