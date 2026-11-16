# The zeta and C2 rings: re-evaluation (16 Nov 2026)

Owner: "zeta angular router and hopf router should be reexplored regardless"; "The zeta ring is the
complex discrete C2 ring that contains the routed zeros"; "the UOR rings are modulo z/256z and
compatible with all of our path"; "Our archive is wrong - zeta ring does work, gcd is very fast".

This record separates the four objects that share the phrase, audits each archive negative for
whether it could have tested what the owner means, and specifies the fair re-test. It changes no
status; it is a reading of the code and the records.

## 1. The four objects

1. **The UOR ring** — `Z/2^n Z` (`docs/content/concepts/ring.md` in the vendored UOR standard):
   `neg(x) = 2^n − x`, `bnot(x) = 2^n − 1 − x`, generating `D_{2^n}`, with
   `neg(bnot(x)) = succ(x)`. n = 8 is the byte ring, `Z/256Z`. The standard's `addressing.md`
   restricts the lifting operations to `{xor, and, or, bnot}` — **addition does not lift** — so the
   ring is a legitimate *carrier* (address, cursor, accumulator) but not an algebra to sum in
   address space.
   *In uor-r4 it is not named*, but a 2-power accumulator already ships: `phases: [u16; 4]` with
   `wrapping_add` in `crates/uor-r4-core/src/native_geometric/addressed_attention/engine.rs:84,494`.
   Ring buffers: `SegmentRing<CAP>` (`crates/uor-r4-graph-runtime/src/runtime_state.rs:211`) and
   `crates/uor-r4-integer/src/stack/context_ring.rs`.
2. **`PhaseQ29`** (`crates/uor-r4-core/src/prime_route_attention.rs:337`) — signed radians at
   `2^-29`. Its `wrapping_add` **round-trips through f64 and does not close modularly**. This is a
   real defect: the type is not a ring element because its operation is not a ring operation.
3. **`UnitS3Q30`** (`:486`) — four `i32` at Q1.30, a unit quaternion; `hopf()` projects to S², and
   `rotate_common_fiber` (`:557`) is documented as the common C2 phase action
   `(z1,z2) → e^{iφ}(z1,z2)`. **This is the owner's "complex discrete C2 ring":** two complex planes
   `z1,z2` carried as a unit quaternion, with the diagonal `U(1)` as the ring and the Hopf base as
   the invariant. The closure is the fibre rotation.
4. **The routed zeros** — `zeta_phase_delta(channel, from, to) = γ_k·(ln p_to − ln p_from)`
   (`:909`), consumed at `crates/uor-r4-core/src/native_geometric/training.rs:110` as
   `phase = round(γ_k·ln p / 2π · 65536) mod 65536` per token and channel, accumulated with
   `wrapping_add` and binned into `zeta_bins: [u8; 4]` (≤ 15) as 16 input bits. Data structure:
   four complex channels per block → two real 4-tuples → an E8 root; ring: `(Z/16)^4 ⊂ Z/2^16`.

The prime-zeta *codebook* (`crates/uor-r4-discrete/src/prime_zeta.rs`) is **not** a ring: the zeros
set placement, not routing.

## 2. The archive negatives, audited

| Recorded | What it measured | Class |
|---|---|---|
| zeta admission 127/3,847 | the token's writer cell among the 8 nearest `q`, at checkpoint 0, 0 updates; the cell is a fixed function of the token id, so no routable structure can exist | **configuration + invalid for routing**; P8 at chance (≈ 8/256 = 120 of 3,847 is not the same unit, but the arm sits at its null) |
| retrieval 0.8948 vs 0.8932 | the same zeta vectors, only the 0.85 contraction differs; the task is saturated (query shares half its tokens) | **real for the contraction**, **invalid for the ring** |
| sector routing MRR 0.0045 vs 0.0743 | arm A retains 1/456 sectors (chance 0.0022), arm B 1/16 (chance 0.0625) — arm B "wins" by barely filtering | **invalid: the control is confounded by occupancy.** The content-aligned arms (0.0176 / 0.0308 / 0.0749) are the real reading, a spread-vs-retrieval frontier |
| prime pointer 19–23/40 | n = 40, ±15 pp; the matched control scored 22/40 | **invalid at this n; control matched or beat the arm** |
| mixer 4.44% vs 5% | 9 held-out positions cannot resolve 0.56 pp | **invalid; the bar was a gate, not a power calculation** |

The quote in `docs/geometry/spiralcore.md:67` ("modular Z/2^256 coefficients are neither …") reviews
a supplied **octonion** draft, not the byte ring as a carrier, and must not be cited against `Z/256`.

## 3. The fair re-test

**Data (planted, because no corpus has ring structure).** K slots, each an address `a_k ∈ (Z/256)²`
and a value `v_k ∈ {0..255}`. A query addresses a slot **one ring step** from a stored one,
`δ = ±1`, so it is never seen verbatim.

**Arms.**
1. Ring address: step `(a + δ) & 0xFF`; read the nearest by **circular distance**
   `min(|q−k|, 256−|q−k|)` — one subtract and one compare, no trigonometry.
2. **The control: the same table with every address through a fixed random bijection.** Occupancy,
   injectivity and the value distribution are identical by construction; ring adjacency is destroyed.
   Any arm-1 minus arm-2 gap therefore cannot be explained by the quota/uniqueness passes that
   produced the archive's negatives.
3. Cosine over the 512-channel zeta word-sum (the ancestor's router).
4. The exact prime-product read by gcd (the owner's "gcd is very fast").

**Measure.** Top-1 recall at δ = 0, 1, 2, and MRR; paired McNemar at δ = 1.
**Bar.** Arm 1 − arm 2 ≥ 0.10 at δ = 1 on ≥ 500 rows, floor `max(24, 2√(b+c))`.
**Cost.** A host test, ~250 lines, sub-second. No GPU.
**Falsifier.** Arm 1 ≈ arm 2 at δ = 1 ⇒ ring adjacency carries no routing value, and the archive's
negative generalises beyond its invalid controls. Arm 1 > arm 2 only at δ = 0 ⇒ injectivity again.

The probe is specified; it is **not built yet** (ordered as unit 14 below). It is the only experiment
that can settle the owner's claim without re-litigating the confounded controls.

## 4. What the ring would have to do to be the memory

- **Address**: `p = round(γ_k·ln(prime)/2π · 2^b) mod 2^b`, b ∈ {8, 16} — already computed at
  `training.rs:110`. One integer per channel; four channels = 4b bits.
- **Write**: `slot[p] ^= value` (superposition) or last-writer-wins. **XOR only** — `AA_4` forbids
  summing addresses.
- **Read**: the window `{q−δ … q+δ}`, ranked by circular distance, returning `slot[k]`. O(window),
  no trig.
- **One required fix before any of this is a ring**: `PhaseQ29::wrapping_add` must be modular in the
  exponent. Today it round-trips through f64, so no existing test could have measured closure — this
  is the most likely single reason the owner perceives the archive as wrong about a ring the code
  never actually ran.

## 5. What is ordered

- **Unit 14**: the planted phase-ring probe of §3, with the bijection control, as a host instrument.
- **Unit 15**: make `PhaseQ29` modular in the exponent (a bug fix with a closure test).
- **Unit 16**: the C2 ring as a *carrier*: the phase accumulator as `[u8; 8]` (four complex pairs)
  so a rotation is `(phase + δ) & 0xFF`, and the UOR ring's `neg`/`bnot` as the signed ring buffer's
  two involutions.
