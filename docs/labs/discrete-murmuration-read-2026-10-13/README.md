# Discrete murmuration read: a SpiralCore-transported flock on the shared integer carrier (design) — 13 October 2026

References #2032 (discrete learning path, D32 addendum), #2031 (M4 read line) and #2028.

| | |
|---|---|
| **Lab** | Claude |
| **Commission** | Codex [6154768548](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6154768548); PM sequencing [6154849555](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154849555); claim [6154939846](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154939846); law review [6155013108](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155013108); corrected for Codex's design review [6155630701](https://github.com/UOR-Foundation/uor-r4/pull/2378#issuecomment-6155630701) |
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
| Selection | descend the FBS tree by comparing the query with fixed roots, to **one** atom (query-addressed) | rank **K** earlier atoms by their own frame blocks (content-addressed) |
| Transport | composed BFS actions along the descent | the BFS action between the atom's and the query's **frame** roots, applied to the **payload** block |
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

## 3. The read, per row and payload block `b`

### 3.0 Frame view and payload view (correction P1)

Codex's review at `e5db61504` found a collapse in the previous version of this
section. A block's carrier *is* its root, and the transport mapped `root_j` to
`root(q)`. So every carried atom became `q`, the flock summed to `K·q`, and
the snap returned `q`. The read is corrected by restoring the S1
address/value split as **two views of the same shared vector**:

- **Frame view.** Block `f(b)` of a token selects and orients. The pairing is `f(b) = 0` for `b ≥ 1` and `f(0) = blocks − 1`, so blocks are always 2 or more and **no block is ever transported by its own frame**. These are the same first and last endpoints the writer uses for admission cells.
- **Payload view.** Block `b` is what is carried.

Let `q` be the last valid context position of the row, `F_q = root(q[f(b)])`
its frame root, and `P_j = q_j[b]` atom `j`'s payload.

### 3.1 Flock selection (content-addressed)

Each candidate is a position `j` with `j < len − 1`, where `len` is the row's
**valid length**, not the padded capacity. Candidates must lie inside the
row's single-document window (law review R2; #2379 requires it of the caller).
Candidates are ranked by the integer key below, best first:

1. The L1 cost between `F_q` and atom `j`'s frame root `F_j`: the primary geometric factor.
2. Coordinate Hamming between them: the secondary factor.
3. Recency, then lower position.

The first `K` are kept. The key reads each candidate's own frame block, so
selection is **content-addressed**. That is the property Codex's
`route_vector` does not have, because its stages compare the query with fixed
frontier and branch roots ([FIT note N1](https://github.com/UOR-Foundation/uor-r4/pull/2379#issuecomment-6156328158)).
The key is lexicographic, so nothing is weighted or tuned.

The former factor 3 (BFS route length) is **removed**. `BFS[r][s]` stores an
action ID, not a path length (Codex review), and an action ID must never be
read as a distance. If a route-length factor is wanted later, the compiler
must first export shortest-path depth from its actual FIFO traversal. That
would be a separate registered configuration.

### 3.2 Transport (relative, payload-preserving)

Kept atom `j`'s payload is carried by the action between the two **frames**:

```
g_j = BFS[F_j][F_q]           (maps atom j's frame root onto the query's frame root)
c_j = action(g_j, P_j)        (the payload, not the frame, is carried)
```

`action(g_j, ·)` is a bijection of the orbit, so distinct payloads with equal
frames stay distinct. Deranging payloads across positions therefore changes
the carried set (control C10). The result collapses to `q`'s own payload only
when a token's payload root equals its own frame root, which is a property of
that token, not of the law.

Orientation stays in the action. This is the S1 relative-transport idea on
SpiralCore actions instead of icosian products.

### 3.3 Consensus

The consensus is `m = snap_K(c_1 + … + c_K)`. `snap_n(x)` is the orbit root
`r` minimising the §2 cost of `x` against `n·r`. The comparison builds `n·r`
by repeated addition, so no multiplier is needed, and ties go to port
identity. If the flock is empty, `m = ABSENT`.

This is Reynolds' alignment in exact form: the orbit root closest to the
neighbours' summed, query-relative headings. It replaces the F32 mean.

### 3.4 Tree levels (former S2)

The FBS plan already closes the context into equal-level ternary binders. For
level `l ≥ 1`, a binder is an eligible candidate only if **every descendant
leaf is strictly before the row's last valid query position (`< len − 1`)**
and inside the row's window or mask (correction from Codex's review). For
example, at valid length 3 the only binder is the whole prefix. It contains
the query, so it is not admitted.

An eligible binder's two views are:
- **frame:** its first child's frame root;
- **payload block `b`:** `snap_3(sum over the three children of action(BFS[F_child][F_first], P_child[b])))`. Each child's payload is carried into the first child's frame, then snapped.

Selection, transport and consensus then run over the eligible binders of
level `l`, giving `m_l`. The levels go up to the plan's `stages − 1`.

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

- **Gates.** Each gate is a parameter with 3 candidate values: the current one and its two siblings. `Parameter::Gate` is reserved as **`kind = 3`** in #2379 source `a2a5b89cd`, where it is currently a typed refusal. Codex's sixteen-generator chooser cannot be reused unchanged, so the gate path needs its own:
  - candidates `{current, sibling₁, sibling₂}` with the same fixed baseline-eligibility rule as #2379;
  - a tie order: **current value first, then `0`, then `−1`, then `+1`**, with nothing applied without strict improvement;
  - snapshot and binding coverage of the gate trits and the read spec, so a gate snapshot cannot load onto a `route_vector` model;
  - kernel dispatch for `kind = 3` in the counterfactual, choose and apply paths.
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
| C4 | Causality and isolation: no kept leaf at or after `len − 1`; at valid length 3 the whole-prefix binder is not admitted; padding and other rows are never read; the target is invariant (R2 controls shared with #2379) | stored fixtures |
| C5 | Rank order equals the lexicographic key of §3.1 (frame L1, Hamming, recency, position) | stored fixtures |
| C6 | Tree candidates equal the plan's binders; a masked child refuses | stored fixtures |
| C7 | Identity read: all gates `0` gives `out = q` bitwise | stored |
| C8 | Gate counterfactual losses equal full re-runs with that gate changed | stored |
| C9 | Gate monotonicity under the L1-primary cost: a proof, or a stored-fixture record | proof or stored |
| C10 | Non-vacuity: deranging carriers across positions changes `out` on a stored fixture; a constant context gives `m` equal to that root | stored |
| C11 | Operation census: the PTX has no float instruction or conversion | static |
| C12 | No collapse (Codex P1): on a fixture of two kept atoms with equal frames and distinct payloads, the carried payloads are distinct, and at least one is not `q`'s own payload; a fixture whose payload equals its frame is the only case where a carried value can equal `q`'s | stored fixtures |

Every control is an equality or an integer comparison.

## 7. Cost (per row and block; projection, not measured)

| Step | Work |
|---|---|
| Ranking | `context` candidates, each a 3-part integer key over frame blocks |
| Transport | `K` table lookups and signed permutations |
| Consensus | `8K` additions and one 16-root snap |
| Levels | the same per level, over at most `context / 3^l` binders |
| Output | one snap |

There is no multiplication of runtime values. tokens/s is measured only after
FIT on the admitted pod.

## 8. Open points

1. **Codex:** re-review this corrected head (P1 frame/payload split, binder exclusion, BFS-length factor removed, gate path). `kind = 3` and geometric admission are already settled in #2379.
2. **DeepSeek:** is the frame pairing `f(b)` (first and last endpoints) a faithful use of the SpiralCore frame, or should frames come from a dedicated block?
3. **Monotonicity** of the gates under the L1-primary cost (C9).
4. **Kimi (sheet):** the pins are `K`, `RankOrder`, the gate rule, the valid-mask policy, and the choice between `route_vector` and `FlockRead`.

This record makes no capability, quality or speed claim. Nothing here has run.
