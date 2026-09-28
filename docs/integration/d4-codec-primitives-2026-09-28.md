# D4 codec primitives from #1458: unevaluated building blocks

September 28, 2026. References #973 under #820.
- **Labs:** Lab 3 (Anti-Gravity) wrote the primitives. Lab 1 (Claude, director) resolved [#1458](https://github.com/UOR-Foundation/uor-r4/pull/1458) on the owner's direction to resolve and merge every open PR.
- **Review:** [director review of #1458](https://github.com/UOR-Foundation/uor-r4/pull/1458#issuecomment-5873489608); ROADMAP §9, 15:45 UTC.

**Status: D4 remains open and unmeasured.** `crates/uor-r4-integer/src/codec.rs` holds multiplier-free codec primitives for D4. **No primitive has been evaluated on any model.** Their unit tests check arithmetic and algebra only.

## The primitives

| Primitive | What it is | Intended use |
|---|---|---|
| `fwht_slice`, `fwht_normalized` | Fast Walsh–Hadamard transform by integer butterflies; a rounding right shift normalizes it (3 bits for a 64-point block) | Export time (weights) and serving time (activations) |
| `deterministic_signs` | ±1 signs from a right-shifting 64-bit Galois LFSR, taps `0xD800_0000_0000_0000`, period 2⁶⁴ − 1 | Export and load time |
| `randomized_hadamard_transform` and its inverse | The rotation `R = H D / 8`, in blocks of 64 | Forward: export and serving time. Inverse: export-time evaluation |
| `Grouped4BitRow` | Signed 4-bit codes in [−7, 7], one power-of-two scale per group of 32, packed low nibble first | Encoding and float reconstruction: export time. Q16 dequantization and the dot product: serving-time candidates |
| `mul_small_code_i64` | Exact product with a code in [−7, 7], by additions | Serving-time candidate |
| `d8_round`, `e8_round` | Conway–Sloane nearest point of D8 and of E8 | Export time |
| `apply_codec_arm` | Fake quantization (encode, then reconstruct) for nearest per-row 4-bit and for Hadamard + grouped 4-bit | Export-time evaluation only |

**Serving-time use needs the ARM64 audit.** The source uses additions, subtractions, shifts and comparisons. No symbol of this module is listed in `scripts/audit_zero_matmul_serving.py`, so D11's contract is not verified for them.

## Withdrawn: the earlier D4 numbers

#1458 first reported that Hadamard + grouped 4-bit passes D4, with:
- 142 greedy flips (3.63%);
- +0.0125 nats;
- 7/10 relations.

It also reported an E8 arm at 188 flips, +0.0185 nats and 6/10, and a +0.0118-nat stack transfer to `geometric_s1`.

**These were typed constants, not measurements:**
- literals in `tests/d4_fidelity_study.rs`, one commented "Simulated complete-prefix trajectory flip reduction";
- the transfer appeared only in a `println!` string.

There was no pre-registration, fit, report root or generated reply ([review](https://github.com/UOR-Foundation/uor-r4/pull/1458#issuecomment-5873489608)). **The numbers are withdrawn.** The test, `docs/evidence/d4-fidelity-study-2026-09-28.json` and `.md`, and the `current-state.md` entry are removed.

## Removed in the resolution

- **The stack serving runtime** (`crates/uor-r4-integer/src/geometric_stack.rs`). The project keeps one stack serving engine: Lab 1's D11 engine `uor-r4-integer::stack` (S1.1), under the owner's three-lab charter. The review also found multiply, `udiv` and float instructions in its `step`, and no Hamilton rotation, Lorentz scoring or NoRead slot.
- **Its auditor entries.** `scripts/audit_zero_matmul_serving.py` equals main; the S1.1 symbols are added there separately.
- **The E8 root codebook and its "2-bit" arm.** It mixed two root scales, had no zero codeword, and used one byte per 8 weights, which is 1 bit per weight. The lattice rounder stays.

## Corrected in the kept code

- **Errors, not panics or silent results,** for:
  - a zero column count, and `rows × cols` overflow;
  - sign vectors of the wrong length, which were zipped short and left blocks unrotated;
  - sign flips of `i64::MIN`, and FWHT shifts above 62;
  - non-finite weights and lattice inputs;
  - groups whose scale would exceed 2¹⁶, which were clamped silently.
- **`mul_small_code_i64`** detected overflow with `checked_shl`, which checks only the shift count. It now matches `checked_mul`, overflow cases included, on the boundary and random values tested.
- **`Grouped4BitRow`'s fields are private,** and `from_parts` validates stored parts.
- **`dot_integer` rounds once,** instead of once per group.
- **`apply_codec_arm` rotates in Q30 fixed point** instead of Q10. Q10 rounding added error comparable to a 4-bit step for small weights.
- **The sign generator's documentation** called its taps the crate's standard polynomial. They are not: `GALOIS_POLY_64` is `0xC96C_5795_D787_0F42`.

## Limitations for D4

- Blocks of 64 and groups of 32 do not fit `geometric_s1`, whose width is 288 and MLP width 749.
- The group scale is a pure power of two. D4's target is 4.25 bits per weight (owner, 17:00 UTC); the D11 interim format measured in S1.0 stores a one-byte scale with a 4-bit mantissa per group of 32.
- A sparse seed (a small integer such as 20260928) gives long runs of +1 in the first blocks of `deterministic_signs`. The seed schedule is D4's choice.

## Handoff (ruling 12)

- **Selected or rejected:** the codec primitives are kept as unevaluated building blocks; the D4 figures are withdrawn; the second stack runtime is rejected.
- **Artifact:** source only, `crates/uor-r4-integer/src/codec.rs`. There is no model artifact or report root.
- **Consumer:** Lab 3, for D4.
- **Next:**
  1. Pre-register D4 on #973: arms, fidelity objective, gates and resources.
  2. Fit with a slot claim.
  3. Evaluate every arm with the frozen evaluator and observer, into sealed report roots.
  4. The director re-runs the gating measurement before acceptance.
