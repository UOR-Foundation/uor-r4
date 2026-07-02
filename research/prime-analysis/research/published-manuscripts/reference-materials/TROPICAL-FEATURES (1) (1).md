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
- `tropical-codec.mjs` — the shared canonical encoder (run-length `canonical_bytes`) + σ-projection transport used by the rest.
- `tropical-realization.mjs` — the carrier, `canonical_bytes`, σ-projection, relabeling-invariance, witness replay (§1, §2).
- `tropical-content-address.mjs` — the orthogonal `(κ, cycle-profile)` coordinates, and addressing faithful to the chosen equivalence (§3.3).
- `tropical-codomain-fiber.mjs` — the κ-fiber as a poset with `W*` maximal and the spectrum varying across it (§3.2).
- `e8-seal.mjs` (+ `e8_complete.py`) — the E₈ root-graph κ and Coxeter cycle-mean spectrum, computed and sealed as witness-verified uor-addr labels (§3.5).

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
  `[witt_level]||[coeffs]`): **run-length over the sorted κ multiset** —
  `[0xC1][version][varint n][varint d][ (zigzag value)(varint count) ]×d` over the `d`
  distinct values. A sorted multiset *is* its run-length data, so this is the minimum-
  information form (and scales to highly repetitive κ such as a graph distance
  distribution). Implemented once in `tropical-codec.mjs` and shared by every exhibit.
  Byte-identical for a relation and any relabeling of it, so the σ-projected label inherits
  the invariance.

## 3. The codomain — what the tropical address space *is*

Where the framework reads the κ-derivation codomain in operator-geometry coordinates (the
Atlas image inside E₈, ADR-059), the **tropical** κ-derivation lands in the
characteristic-1 reading of that codomain, which has concrete, computable structure.

### 3.1 The space
The codomain is the space of canonical max-plus closures up to relabeling — a quotient of
tropical space: each point is a sorted finite multiset of longest-path weights. κ is the
relation's identity **relative to the extremal distinction-surface**: relations that coincide
in that frame share a κ and differ only in a complementary frame (the cyclic coordinate,
§3.3), which is itself tracked. There is no frame-independent relation for κ to be measured
*against* — choosing the extremal coordinate is choosing what sameness is relative to, and a
finer frame is reached by compiling the residue (§3.3, §4), not by recovering a lost absolute.
That relativity is the defining codomain feature.

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

### 3.5 The E₈ codomain: computed, sealed, and the residual open question
The principled relationship between this codomain and the framework's operator-geometry
(E₈/Atlas) codomain runs through the **zero-temperature limit** (R7),
`lim_{β→∞}(1/β)·log ρ(e^{βW}) = max cycle mean`, which is exactly **ultradiscretization**
(`×→+`, `+→max`) — the *same* operation that sends the quantum group U_q(E₈) at `q→0` to
its **Kashiwara crystals**, and geometric crystals to combinatorial ones (string /
Berenstein–Zelevinsky polytopes). So under precisely doc 1's notion of tropicalization,
**E₈ has a literal tropical limit: its crystal / string-polytope combinatorics** (established
Lie theory). The two coordinates of E₈'s graph-and-dynamics tropicalization are computed and
sealed through uor-addr here (`packages/addr-demo/src/e8-seal.mjs`, math in `e8_complete.py`):

- **Extremal / closure coordinate** — the 60° root graph (240 roots, degree 56,
  **diameter 3**): tropical κ = `{−1 ×13440, −2 ×43680, −3 ×240}` (the 240 distance-3 pairs are
  exactly the antipodal `(α,−α)`), Weyl-invariant; its tropical eigenvalue is the degenerate
  `−1`. uor-addr κ-label `sha256:94c6db53…`, witness-verified.
- **Spectral coordinate** — the Coxeter element on the 240 roots: order **30 = h** (the
  Coxeter number), **8 orbits each of length 30** (Kostant), trace `−1`, non-degenerate
  cycle-mean spectrum `{−1 ×3, 0 ×4, +1 ×1}`. The Coxeter number appears as the **tropical
  prime-cycle length** (§3.4). uor-addr κ-label `sha256:0e4b174e…`, witness-verified.

These are distinct labels — κ ⊥ spectrum, realized on E₈ itself, both now first-class
content-addresses. **What this does not settle:** whether the κ-derivation codomain (the
Atlas image in E₈) tropicalizes to *this* tropical-κ codomain. The graph tropicalization is
one coordinate (the extremal/metric frame); it does not carry the classical adjacency
spectrum `{−4,−2,8,28,56}` — that lives in a complementary frame (the Coxeter spectrum,
§3.4), the two being orthogonal coordinates rather than one approximating the other — and the
metric coordinate is a *different* object from the crystal limit, so the right form of the
open question is sharper than "does E₈ have a tropical limit": it is **does E₈'s crystal /
string-polytope codomain coincide with the tropical-κ codomain of weighted relations?** The
uor-addr API *seals and addresses* these objects (done above) but cannot *prove* that
identification — it is a theorem-or-refutation, not a computation, and R7 bridges
transfer-operators-to-cycle-means, not root-systems-to-codomain-identity. Settling it
requires constructing the β-family deforming the operator-geometry codomain to the tropical
one with the Atlas's E₈ structure at finite β, which neither the wiki (ADR-059 is a deferred
conceptual-reading commitment) nor doc 1 builds.

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
| Zero-temperature bridge to the characteristic-0 codomain | **verified** (R7); shown to be the ultradiscretization that also defines E₈'s crystal limit |
| E₈ tropical coordinates (root-graph κ; Coxeter cycle-mean spectrum) computed and sealed as uor-addr labels | **computed & witness-verified here** (`e8-seal.mjs`, `e8_complete.py`) |
| E₈ has a literal tropical limit = its crystal / string-polytope combinatorics | **established Lie theory** (Kashiwara crystal limit `q→0`; Berenstein–Kazhdan ultradiscretization) |
| Coincidence of E₈'s crystal codomain with the tropical-κ codomain | **open** — the sharpened form of the original question; a theorem-or-refutation, not decided by content-addressing (the API seals and addresses but does not prove it) |

The tropical objects and theorems are classical tropical / dynamical mathematics; the
contribution is their assembly into a characteristic-1 carrier for κ-derivation and the
characterization of its codomain.

---

## 6. What it adds to uor-addr — utility and relevance

Every realization the suite ships — json, sexp, xml, asn1, cbor, schema, gguf, onnx,
codemodule, ring — canonicalizes a *serialization*: it normalizes byte-level presentation
(whitespace, key order, NFC, encoding) and addresses the representation, so two relabelings a
format's canonicalizer does not erase still get different κ-labels. Tropical addressing adds
the suite's first **semantic, symmetry-quotient carrier**: it addresses an object *up to a
mathematical equivalence* — vertex relabeling — via an algebraic normal form (the Kleene-star
closure), not a byte normalization. Everything below follows from that one shift: identity
modulo a chosen *mathematical* symmetry rather than modulo serialization noise.

### What it adds to the capability
- **A new equivalence granularity, with faceted coordinates.** The closure-lossless taxonomy
  had T1 (byte-identical), T2 (κ-label-identical / replay), T3 (outcome-coarse). Tropical
  addressing inserts a *structural* tier between T2 and T3 — "same extremal/longest-path
  structure up to relabeling" — provably **orthogonal** (κ ⊥ spectrum) to the cyclic facet.
  An object gains two independent addressable coordinates `(κ, cycle-profile)`, deduped,
  diffed, or certified along either without committing to the other.
- **A wider domain of addressable objects.** A graph, schedule, dependency DAG, or state
  machine was addressable before only through its serialization, hostage to node naming.
  Tropical addressing gives a faithful, cross-language, relabeling-invariant address for the
  **extremal / critical-path structure of any weighted relation** — extending uor-addr's
  reach from "serialized data + ring elements" to "dynamical and relational systems, up to
  their natural symmetry."
- **The characteristic-1 leg of the algebraic-carrier axis.** `ring` is the characteristic-0
  algebraic carrier; tropical addressing is its characteristic-1 (idempotent / max-plus)
  sibling. Together they span both characteristics, and the κ-derivation codomain's
  operator-geometry reading gains its zero-temperature shadow as a *native addressable
  coordinate*, not merely an interpretation.

### Concrete uses
- **Dedup / caching up to symmetry** — a κ-keyed store collapses relabeled graphs, schedules,
  or netlists; matching is **exact relative to the chosen coordinate** (same κ ⇒ same identity
  in the extremal frame), and finer identity is reached by also keying on the residual
  coordinate, not by treating the κ as an approximation of some absolute object.
- **Verifiable provenance of structure** — seal a build graph or workflow as a `Grounded`
  tropical κ-label with a replayable trace: offline-checkable attestation of *dependency
  structure*, not just file hashes, with no recomputation and no trust in the producer.
- **Cross-construction certificates** — build an object two independent ways, seal both,
  compare for a `Certified` "same object up to the chosen equivalence."
- **Semantically-targeted diff** — diff against κ to fire only on changes that move the
  worst-case / critical-path structure, while the spectral coordinate catches the throughput
  changes κ ignores.
- **Optimizing the framework itself** *(proposed)* — the resolver ψ-pipeline is a monotone
  reduction over a dependency structure, i.e. a max-plus / critical-path object, so tropical
  κ of a model's resolver-dependency graph is a relabeling-invariant **convergence /
  critical-path signature**: usable for compile-time dedup of monomorphizations that agree up
  to relabeling, for bounding convergence-tower depth, and as the coarse tower stratifier the
  wiki already posits.

### Why the E₈ result is relevant here
The deep dive showed the framework's *own* headline codomain object is tropically
addressable: E₈ yields two orthogonal, witness-verified uor-addr labels — extremal
(`sha256:94c6db53…`, the root-graph distance closure) and spectral (`sha256:0e4b174e…`, the
Coxeter cycle-mean spectrum, with the Coxeter number as the prime-cycle length). So tropical
addressing is not confined to toy relations; it reaches the operator-geometry codomain
itself, turning ADR-059's deferred conceptual-reading commitment into structure that is
concretely **sealed, comparable, and falsifiable**. Being able to address and diff the
tropical signatures of E₈-structured objects is precisely the precondition for ever *testing*
the codomain identification (§3.5) instead of only asserting it.

### Bounds
Per §5, tropical addressing is an extension rather than a documented feature; its address is a
**coordinate within a residue-tracked tower** — identity relative to a chosen
distinction-surface, exact in that frame and refined to a finer frame by compiling the
residue (the trace / the `(κ, cycle-profile)` decomposition / the composition shapes), not an
approximation of any frame-independent object; and none of the above closes the open
identification. It makes the surrounding structure addressable and testable — which is the
useful and honest contribution.

---

## 7. A new carrier class, and what combining it enables

Tropical addressing is the **first member of a new carrier class**, not a one-off. The
realizations the suite ships — json, sexp, xml, asn1, cbor, schema, gguf, onnx, codemodule,
ring — are *syntactic / representational* carriers: each canonicalizes a serialization and
addresses the representation. The tropical carrier is the first **semantic-symmetry-quotient
carrier**: its canonical form is a symmetry-invariant *algebraic* normal form — the
Kleene-star closure, quotiented by the object's natural symmetry (vertex relabeling) — not a
normalization of presentation. That is the defining property of the class: *address an object
up to a mathematical symmetry, by an invariant normal form computed from its structure.*

### The class members are coordinates
The same relational object admits a different symmetry-invariant normal form in each algebraic
frame, and each is a relabeling-invariant **coordinate**:

| carrier | algebraic frame | coordinate it addresses | status |
|---|---|---|---|
| **tropical** | max-plus `(max, +)` | extremal / longest-path (critical path) | **realized** (this workspace) |
| min-plus | `(min, +)` | metric / shortest-path (distance) | sibling |
| Boolean | `(∨, ∧)` | reachability / connectivity | sibling |
| max-min | `(max, min)` | bottleneck / capacity | sibling |
| classical | `(ℝ, +, ×)` | spectral / cyclic (cycle-mean, eigen-data) | sibling |

These are not redundant: the extremal and spectral coordinates are *provably orthogonal*
(κ ⊥ spectrum, R10), and the others extract independent structure. Each is exact relative to
its own distinction-surface; none approximates an absolute object (§3.1). All produce κ-labels
on a shared σ-axis — they differ only in the canonical-form carrier — so they meet the σ-axis
homogeneity that composition requires.

### What combining them enables
Because every member is one coordinate, **composing them is the capability** — and the suite
is already closed under composition (the ADR-061 shapes recombine κ-labels into a κ-label).
Combining a chosen set of these carriers yields a faceted composite address that fixes the
exact mathematical equivalence under which two objects count as the same:

- **Programmable identity-up-to-equivalence.** One carrier addresses up to its symmetry (e.g.
  extremal-up-to-relabeling); a chosen *combination* addresses up to the *composite* equivalence
  (e.g. extremal ∧ reachability ∧ spectral, up to relabeling). You select the granularity of
  sameness by selecting which coordinates to compose.
- **Reconstructible, not coarsening.** Each carrier's residue relative to a finer frame is
  exactly a complementary carrier's coordinate, and composition recombines them; in the limit
  the combined coordinates are the complete relational descriptor. Per §3.1 this is a
  residue-tracked tower — coarsen by dropping coordinates for indexing/dedup, refine by
  compiling them back, with no frame-independent object lost in between.
- **A lattice of equivalences, addressable.** The byte-canonical carriers give one notion of
  identity per format. This class, composed, gives uor-addr a *lattice* of equivalences over
  one object — each node a composite of coordinates, each a first-class, witness-verifiable
  κ-label — the new capability no single carrier, syntactic or semantic, provides alone.

The E₈ result is the existence proof in miniature: one object, two coordinates of this class
(extremal `sha256:94c6db53…`, spectral `sha256:0e4b174e…`), each sealed and orthogonal —
compose them and you have addressed E₈ at the equivalence "same extremal *and* same spectral
structure," recoverable to either coordinate alone.

### Status (per §5)
The tropical carrier is realized; the class, its sibling members, and the combination
capability are the proposed generalization — consistent with the framework's
composition-closure (ADR-061) and the relational / residue framing of §3 — with the siblings
natural but not yet built.

### Realization path
The combination capability is demonstrable, not just argued, with one more carrier. The
cheapest second member is the **Boolean (reachability)** or **min-plus (metric)** carrier —
the same Kleene-star machinery over a different closed semiring (`(∨, ∧)` or `(min, +)`),
reusing the shared codec (`tropical-codec.mjs`) and σ-projection unchanged; only the closure's
`⊕`/`⊗` change. With two members realized, their κ-labels compose through the existing
composition path into a single two-coordinate composite address — turning "combining" from a
proposal into a sealed, witness-verifiable artifact, and making the lattice of equivalences
above something to enumerate and address rather than only describe. That is the smallest step
that converts §7 from claim to demonstration.
