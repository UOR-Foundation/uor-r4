# Tropical Features of uor-addr

The tropical realization is the **characteristic-1 carrier** for κ-derivation — the
idempotent / max-plus sibling of the characteristic-0 `ring` carrier (ADR-039 /
Amendment-43). It plugs into the same catamorphism, sealing, replay, and composition
machinery as every other realization; what is specific to it is the carrier, its
canonical form, and — the focus of this document — the **structure of its codomain**.

Grounding: the mathematics is from `characteristic_1_constructions.md` (results R1–R13,
verified). "tropical"/"characteristic 1" appear nowhere in the UOR-Framework wiki, so
this is an *extension* — a proposed carrier for the documented capability, not a
documented feature. Scope of each claim is marked in §5.

Runnable exhibits (all against the published `@uor-foundation/uor-addr`), in
`packages/addr-demo/src/`:
- `tropical-realization.mjs` — the carrier, `canonical_bytes`, σ-projection, relabeling-invariance, witness replay (§1, §2).
- `tropical-content-address.mjs` — the orthogonal `(κ, cycle-profile)` coordinates, and addressing faithful to the chosen equivalence (§3.3).
- `tropical-codomain-fiber.mjs` — the κ-fiber as a poset with `W*` maximal and the spectrum varying across it (§3.2).

---

## 1. The object (carrier)

A **max-plus weighted relation** on `n` vertices: a matrix `W ∈ ℝ_max^{n×n}` over the
characteristic-1 semifield `ℝ_max = (ℝ ∪ {−∞}, ⊕ = max, ⊗ = +)`, defining trait
`x ⊕ x = x`. `W_{ij}` is an edge weight, `−∞` is no edge. Admitted in the **stable
regime** (max cycle mean ≤ 0), where longest paths converge. Weights are integers in the
reference encoding (rationals admit a fixed scaled encoding). Vertex identities are
incidental: the carrier is taken up to relabeling.

## 2. The canonical form (domain → normal form)

- **Kleene-star closure** `W* = I ⊕ W ⊕ W^{⊗2} ⊕ …`, the all-pairs **longest-path**
  closure, which in the stable regime converges and is **idempotent**: `W* ⊗ W* = W*`.
  It is the characteristic-1 normal form of the relation, analogous to a projection.
- **Tropical content-address** `κ(W)` = the sorted multiset of finite off-diagonal
  entries of `W*`. It is **permutation-invariant**: `κ(σ·W) = κ(W)` — a canonical
  invariant of the weighted relation up to vertex relabeling.
- **`canonical_bytes` layout** (minimum-information, the analogue of `ring`'s
  `[witt_level]||[coeffs]`): `[0xC1][version][varint n][varint m][zigzag-varint κ_1..κ_m]`.
  Demonstrated in `packages/addr-demo/src/tropical-realization.mjs`; byte-identical for a
  relation and any relabeling of it, so the σ-projected label inherits the invariance.

## 3. The codomain — what the tropical address space *is*

Where the framework reads the κ-derivation codomain in operator-geometry coordinates (the
Atlas image inside E₈, ADR-059), the **tropical** κ-derivation lands in the
characteristic-1 reading of that codomain, which has concrete, computable structure.

### 3.1 The space
The codomain is the space of canonical max-plus closures up to relabeling — a quotient of
tropical space: each point is a sorted finite multiset of longest-path weights. It is
**lossy by design** (many-to-one): distinct relations share a κ. This coarseness is the
defining codomain feature, not a defect (cf. §4 equivalence).

### 3.2 Stratification — the κ-fiber (the analogue of operator-geometry strata)
The codomain is stratified by **κ-fibers**. At the finest granularity, the **same-`W*`
fiber** is `{ W : closure(W) = W* }` for a fixed idempotent closure `W*`: a **finite,
fully mappable poset** under the max-plus order `≤`, with `W*` its *maximal* element
(every member satisfies `W ≤ W*` with the same closure) and the transitive reductions as
its minimal elements. Within it the extremal/longest-path structure is fixed (so κ is
constant) while the cyclic structure is free (so the cycle-mean spectrum varies). The
**address-level fiber** is coarser still — it unions same-`W*` fibers whose `W*` share the
off-diagonal multiset κ, including relabelings (which have conjugate `W*`). This is the
concrete, computable counterpart of "operator-geometry coordinates stratifying the Atlas
image": a poset stratification you can enumerate (R11).

Exhibited in `packages/addr-demo/src/tropical-codomain-fiber.mjs`. For the 3-cycle
closure `W* = [[0,−1,−2],[−2,0,−1],[−1,−2,0]]` (κ = `[−2,−2,−2,−1,−1,−1]`), the same-`W*`
fiber has **8 members forming a Boolean lattice B₃** (rank sizes 1-3-3-1) on the three
optional shortcut edges: `W*` (all six edges) is the maximal element, the bare 3-cycle
`{0→1,1→2,2→0}` the minimal one, and the cycle-mean spectrum takes **four distinct values**
across the fiber — from `{−1}` at the bottom to `{−2,−3/2,−3/2,−3/2,−1}` at `W*` — all at
the single constant κ. Same address, different property, with the poset and its maximum
explicit.

### 3.3 Coordinates — the orthogonal pair (κ, cycle-profile)
The complete descriptor of a relation up to relabeling factors into two **provably
independent** coordinates (R9, R10):

- **κ** — the *extremal / longest-path* coordinate (what the address captures).
- **cycle-profile** — the *cyclic* coordinate: the multiset of `(length, weight)` over
  simple cycles, which determines the cycle-mean spectrum.

Neither determines the other (same-κ/different-spectrum **and** same-spectrum/different-κ
classes both exist). So the tropical address space has two orthogonal axes — extremal vs.
cyclic — the characteristic-1 analogue of distinguishing strata in the codomain.
Demonstrated in `packages/addr-demo/src/tropical-content-address.mjs`.

### 3.4 Spectral coordinate — the cycle-mean spectrum and its prime/zeta structure
The cyclic coordinate's invariant is the **cycle-mean spectrum**, the characteristic-1
analogue of eigenvalues. Its features:

- The dominant value is the **tropical (max-plus) eigenvalue** = the max cycle mean
  (Karp) — the Perron analogue (R4).
- The **primitive cycles** are the *tropical primes* — the closed orbits of the relation
  (R5). The dynamical zeta factors over them exactly as ζ factors over the primes:
  `∏_{primitive γ} (1 − t^{|γ|})^{-1} = 1/det(I − tB)`, verified term-by-term (R5), and is
  rational (Bowen–Lanford). So the codomain natively carries a **prime/zeta structure** —
  the characteristic-1 root of the prime-theoretic origin of the Atlas program.
- Codomain **symmetry**: `spectrum(W) = spectrum(Wᵀ)` for every relation (R12) — a genuine
  reversal theorem, the tropical analogue of a zeta functional equation.

### 3.5 Bridge to the operator-geometry (E₈/Atlas) codomain
The principled relationship between this codomain and the framework's operator-geometry
codomain is the **zero-temperature limit** (R7):

```
    lim_{β→∞} (1/β) · log ρ(e^{βW})  =  max cycle mean.
```

The classical (characteristic-0) transfer-operator codomain *degenerates exactly* to the
tropical (characteristic-1) one as temperature → 0; `log-Σ-exp` over cycles becomes `max`
over cycles. So the tropical address space is the **zero-temperature shadow** of the
characteristic-0 codomain. This is a real, verified bridge between characteristics; it is
**not** a claim that the UOR Atlas's specific E₈ structure has a literal tropical limit —
that is an open research question (§5), stated here as a bridge, not asserted as a result.

## 4. Suite-integration features (inherited from being a realization)

- **Equivalence tier.** Tropical κ-identity is a relabeling-invariant κ-label equivalence
  (a T2-style tier *for this carrier*), and the deliberate coarseness places a *structural*
  equivalence — "same extremal/longest-path structure up to relabeling" — between
  byte-identity (T1) and outcome-coarse (T3), provably orthogonal to the cyclic coordinate.
  It enriches the framework's closure-lossless taxonomy with an extremal-path facet.
- **Composition closure.** Tropical κ-labels compose under the ADR-061 shapes
  (G2 product, F4/E6/E7/E8) like any κ-label — a composition is a typed input the ψ-pipeline
  consumes through the σ-projection, so the tropical address surface is **closed under
  composition**. σ-axis homogeneity is required (all operands share the tropical σ-axis).
- **Sealing + replay + zero-cost.** A tropical address is a `Grounded` value carrying a
  content fingerprint and a trace, replay-verifiable offline (no re-hash, no author), and
  compiled to zero-cost native code — the documented core capability, unchanged.

## 5. Scope and provenance

| Element | Status |
|---|---|
| κ-derivation, sealing, replay, T1/T2/T3 taxonomy, composition-closure | **documented framework capability** (wiki ADR-058, ADR-061, §8) |
| `ring` as the characteristic-0 algebraic carrier | **documented** (ADR-039 / Amendment-43) |
| Tropical/characteristic-1 **carrier** and its `canonical_bytes` | **extension** (not in the wiki; prototyped in this workspace) |
| Codomain structure: κ-fiber poset, (κ, cycle-profile) orthogonality, cycle-mean spectrum, tropical primes/zeta, reversal symmetry | **verified mathematics** (characteristic_1_constructions.md, R4–R13) |
| Zero-temperature bridge to the characteristic-0 codomain | **verified** (R7), offered as analogy to the operator-geometry codomain |
| A literal tropical limit of the UOR Atlas's E₈ structure | **open / not claimed** — stated as a research question, not a result |

The tropical objects and theorems are classical tropical / dynamical mathematics; the
contribution is their assembly into a characteristic-1 carrier for κ-derivation and the
characterization of its codomain.
