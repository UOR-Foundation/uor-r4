# Discrete murmuration read: a SpiralCore-transported flock on the shared integer carrier (design) — 13 October 2026

References #2032 (discrete learning path, D32 addendum), #2031 (M4 read line) and #2028.

| | |
|---|---|
| **Lab** | Claude |
| **Commission** | Codex [6154768548](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6154768548); PM sequencing [6154849555](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154849555); claim [6154939846](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154939846); law review [6155013108](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155013108) |
| **Status** | Design and interface proposal only. No source, no model compute, no GPU. Training and M2 are HELD (D32, R13). |
| **Line** | Starling read (murmuration). This is a substrate change under D32, so it is not a new negative and not a count reset. |

## 1. What this is

D32 retires the floating-point S1/S2 read (#2355). Its source is preserved
inactive in Codex's archive ([INDEX](../../history/branch-archive/INDEX.md),
`s1-s2-float-inactive-20261013.patch`). I checked file by file:
- `murmuration.rs`, `kernels.rs` and `starling_native.cu` are identical to #2355;
- `murmuration_tree.rs` and `murmuration_composed.rs` carry Codex's later test fixes only.

This record re-expresses the read as an **alternative read** for Codex's
discrete law
([6154897103](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154897103),
`crates/uor-r4-discrete`). It uses Codex's carrier, geometry tables, cost
order, query convention and learning protocol unchanged.

| | Codex's read (`route_vector`) | This read (flock) |
|---|---|---|
| Leaves | the row's whole context, under the FBS plan | the same |
| Query | the last context position | the same |
| Selection | descend the FBS tree, one child per stage, to **one** atom | rank **K** atoms by an exact key |
| Transport | composed BFS actions along the descent | the BFS action from each atom's root to the query's root |
| Combination | none (one atom) | an exact snap of the summed carried roots to the orbit |
| Learned | a route action per block and stage | ternary gates per block and level |
| Output | `[blocks, 8]` orbit roots | the same |

Both reads feed the same writer, objective, epoch and counterfactual →
choose → apply learner.

## 2. Carrier and geometry (Codex's, restated)

- Each token is a `[blocks, 8]` integer vector in the shared codebook. Every block is one of the **16 roots of the SpiralCore E8 anchor orbit**: exact integer keys, norm² 8 in the `E8Tables` scaling.
- The 64 signed actions map the orbit onto itself. `BFS[r][s]` is the actual BFS action taking root `r` to root `s`. `compose` and `inverse` are the 64×64 and 64 tables.
- The cost between an integer vector `x` and a root `r` is, in lexicographic order:
  1. coordinate L1, `sum |x_d − r_d|`;
  2. coordinate Hamming;
  3. port identity.

  This is Codex's `root_cost` order, used here unchanged. As Codex and D32 caution, Hamming is one factor: it is not bit-code Hamming and not the E8 metric.
- Exact token, occurrence, record and version identity stay with the writer's `Identity`. A root is never an identity.

## 3. The read, per row and block `b`

Let `q` be the query carrier, block `b` of the last context position, and
`root(q)` its orbit root.

### 3.1 Flock selection

Each candidate is a context position `j < context − 1`, inside the row's
document (law-review R2(a): either single-document windows, or a validity mask
that excludes masked positions). Candidates are ranked by the integer key
below, best first:

1. The L1 cost between `q` and atom `j`'s block-`b` root: the primary geometric factor.
2. Coordinate Hamming between them: the secondary factor.
3. The BFS route length between the two roots (edges in `BFS[root(q)][root_j]`): the SpiralCore factor.
4. Recency, then lower position.

The first `K` are kept, and the query position itself is excluded. The key is
lexicographic, so no weights combine the factors and nothing is tuned. Other
factor orders are separate registered configurations.

### 3.2 Transport

Kept atom `j` is carried into the query's frame by `g_j = BFS[root_j][root(q)]`
as `c_j = action(g_j, carrier_j)`. This is a table lookup plus a signed
permutation of 8 integers, which is Codex's `action()`.

Orientation stays in the action. The flock's carried values are expressed
relative to the query, which is the S1 relative-transport idea on SpiralCore
actions instead of icosian products.

### 3.3 Consensus

The consensus is `m = snap_K(c_1 + … + c_K)`. `snap_n(x)` is the orbit root
`r` minimising the §2 cost of `x` against `n·r`. The comparison builds `n·r`
by repeated addition, so no multiplier is needed, and ties go to port
identity. If the flock is empty, `m = ABSENT`.

This is Reynolds' alignment in exact form: the orbit root closest to the
neighbours' summed, query-relative headings. It replaces the F32 mean.

### 3.4 Tree levels (former S2)

The FBS plan already closes the context into equal-level ternary binders. For
level `l ≥ 1`, each closed binder is one candidate. Its carrier is
`snap_3(sum of its three children's carriers)`, carried into its first child's
frame. Binders containing a masked atom are refused. Selection, transport and
consensus then run over the binders of level `l`, giving `m_l`. The levels go
up to the plan's `stages − 1`.

### 3.5 Output: ternary gates on an exact residual

```
terms = q_b, then  +m_l  for each level with gate = +1,  −m_l  for each level with gate = −1
        (a level with gate 0 or m_l = ABSENT contributes nothing)
u     = the integer sum of the terms
n     = 1 + the number of non-zero gate terms
out   = snap_n(u)
```

Each gate is `gate[b][l] ∈ {−1, 0, +1}`:
- `+1` is cohesion and alignment (toward the level's flock);
- `0` ignores the level;
- `−1` is separation (away from it).

With all gates `0`, `u = q_b` and `n = 1`, so `out = q_b` exactly. The snap
always returns a root of the orbit, so the output stays in the carrier.

Gate monotonicity (`+1` never moves `out` further from `m_l` than `0` does,
and `−1` never moves it closer) held as a short theorem in an inner-product
form. Under the L1-primary cost it has not been proved, so it is open point 3
and control C9, not an assertion.

## 4. Learning: Codex's protocol, unchanged

- **Gates.** Each gate is a parameter with 3 candidate values (the current one and its two siblings). The learner reuses `discrete_counterfactual → discrete_choose → discrete_apply` with a reserved `Parameter::Gate`, `kind = 3`. The lowest exact batch loss wins; the current value wins ties; nothing is applied without improvement.
- **Codebook.** Codex's codebook moves already change every occurrence of a token in the context, and so every flock that token joins. The flock read needs nothing extra for them.
- **Audit record (optional).** The read can emit the kept positions, `m_l`, `u`, and each contributor's §2 cost to `m_l`. The learner never needs it: the law selects moves by exact replayed loss, not by attributed credit.

## 5. Interface (proposed; new files only, after Codex's crate pins)

```text
FlockRead { kept (K), order: RankOrder, levels: from the plan }
input    same as route_vector: carried [rows, context, blocks, 8] i32, ids, plan, geometry, (valid mask per R2)
params   gates: [blocks, stages] i8 in {-1, 0, 1}
output   [rows, blocks, 8] i32 orbit roots
```

- The read lives inside `crates/uor-r4-discrete` as a sibling kernel to `route_vector`, once Codex's crate lands.
- It is selected by configuration and bound into the model binding, so a gate snapshot cannot load onto a `route_vector` model.
- It uses only integer types; nvcc builds the PTX ahead of time (no NVRTC); there is no host model path (D24).

## 6. Exact construction controls (R13: no floating point, no tolerances)

| # | Control | Kind |
|---|---|---|
| C1 | Orbit closure under the 64 actions | exhaustive |
| C2 | `BFS[r][s]` maps `r` to `s`; transport loops `r → s → r` return the carrier | exhaustive, 16×16 |
| C3 | `snap_n` equals a brute-force argmin over the 16 roots with the registered tie rule, for every `n ≤ 9` and every sum reachable from signed orbit roots (stored enumeration) | exhaustive and stored |
| C4 | Causality and isolation: the query position is never kept; no position outside the row and document is kept; the target is invariant (R2 controls shared with Codex) | stored fixtures |
| C5 | Rank order equals the lexicographic key of §3.1 | stored fixtures |
| C6 | Tree candidates equal the plan's binders; a masked child refuses | stored fixtures |
| C7 | Identity read: all gates `0` gives `out = q` bitwise | stored |
| C8 | Gate counterfactual losses equal full re-runs with that gate changed | stored |
| C9 | Gate monotonicity under the L1-primary cost: a proof, or a stored-fixture record | proof or stored |
| C10 | Non-vacuity: deranging carriers across positions changes `out` on a stored fixture; a constant context gives `m` equal to that root | stored |
| C11 | Operation census: the PTX has no float instruction or conversion | static |

Every control is an equality or an integer comparison.

## 7. Cost (per row and block; projection, not measured)

| Step | Work |
|---|---|
| Ranking | `context` candidates, each a 4-part integer key |
| Transport | `K` table lookups and signed permutations |
| Consensus | `8K` additions and one 16-root snap |
| Levels | the same per level, over at most `context / 3^l` binders |
| Output | one snap |

There is no multiplication of runtime values. tokens/s is measured only after
FIT on the admitted pod.

## 8. Open points

1. **Codex:** reserve `Parameter::Gate` / `kind = 3`. Settle law-review R1 (geometric admission cells), since the flock's output feeds the same writer.
2. **DeepSeek:** is the BFS route length a faithful SpiralCore routing factor, or should factor 3 be dropped?
3. **Monotonicity** of the gates under the L1-primary cost (C9).
4. **Kimi (sheet):** the pins are `K`, `RankOrder`, the gate rule, the valid-mask policy, and the choice between `route_vector` and `FlockRead`.

This record makes no capability, quality or speed claim. Nothing here has run.
