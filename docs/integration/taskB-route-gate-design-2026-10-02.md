**Design note for Task B (learned route gate) — recorded before implementation so the decisions survive a session boundary.**

Card: Claude's #1512 card of 01:04Z, item B. I hold ownership of the pointer path (`PrimeRoute`, `route_*`, `ranked_attention`, `PointerMixture`, `next_scores`, `pointer_route` grammar) for the duration.

**Mechanism as specified by the card.**
```
where the route admits sources:  attention = (1 − r)·a_soft + r·a_route
where nothing is admitted:       attention = a_soft
r = sigmoid(w_r · h + b_r)
new variables: pointer.route_gate.weight [1, width], pointer.route_gate.bias [1]
gradient reaches r, the learned scores (through both terms) and the copy gate
```

**What I have verified about the existing code.**

1. `pointer_side` (`geometric_stack.rs:2821`) builds the pointer's `side` as `Tensor::cat([query, key, gate], 0)` with a matching bias, so `side` is exactly `[query | key | gate logit]` and is `2*dim + 1` wide.
2. `pointer_query(side, dim, row)` (`:7842`) extracts the query slice by stride; `pointer_key` (`:7849`) extracts the key.
3. `pointer_attention` (`:7938`) is the single dispatch point: `Some(route) if route.ranked` → `ranked_attention`, `Some(route)` → `route_attention`, `None` → `pointer_weights`.
4. `ranked_attention` (`:665`) already implements exactly the fallback the card wants: **empty admission → `pointer_weights(learned, None)`**, i.e. the soft pointer.
5. `PrimeRoute` (`:472`) already has two optional fields added the same incremental way — `ranked: bool` and `admission: RouteAdmission`, both `#[serde(default, skip_serializing_if = …)]`. **The gate should follow that established pattern exactly.**
6. Parameter registration is additive at three sites: `shapes.insert` (`:972-975`), the initial-value match (`:1108-1109`), and `p.get(...)` in `pointer_side` (`:2828-2837`).
7. `PointerMixture` (`:7984`) carries `time, dim, score, select, route, ids, targets, weights`, and `rule(beta)` builds the `PointerRule`.

**Two design decisions I am making, with reasons.**

**(A) The gate logit will NOT go into the `side` layout.** `side` is `2*dim + 1` wide and that width is baked into `pointer_query`/`pointer_key` strides, the `PointerMixture` row stride, and the saved-artifact tensor inventory. Extending it to `2*dim + 2` would change every stride and every saved pointer model — a much larger, riskier change than the card asks for, and it would break the existing saved heads the card says to reuse (`init=claude-a1-d18/arm-L1b-1`). **Instead `r` is computed in the forward pass from the hidden state and carried on `PointerMixture` as an extra per-row vector**, alongside the existing `weights`. Rationale: the card says `r = sigmoid(w_r·h + b_r)` where `h` is the hidden state, which is available where `side` is produced; and `PointerMixture` already carries per-row side data.

**(B) The gate is `PrimeRoute.route_gate: bool`, not a new enum arm.** The card names the CLI `pointer_route=ngram-gated:W` / `prime-gated:W`, i.e. **gated is a variant of the existing admission kinds, not a third admission kind.** So: `route_gate: bool` with `#[serde(default, skip_serializing_if = "is_false")]`, matching how `ranked` was added. A gated route implies ranked ranking (the card's formula mixes `a_route`, which is the ranked route's attention).

**Implementation order (each step independently checkable).**
1. `PrimeRoute.route_gate: bool` + serde, and `parse_pointer_route` grammar for `:gated` variants.
2. Variables `pointer.route_gate.weight [1, width]` / `.bias [1]`, registered at the three sites; zero-initialised weight so the model starts at `r = 0.5`… **no — see caveat below.**
3. `PointerMixture.route_gate: Option<Vec<f32>>` (per-row logits), threaded from the forward pass.
4. Gated branch in `pointer_attention`: `(1-r)·a_soft + r·a_route` on admitted positions; `a_soft` when nothing is admitted.
5. Tests following `a_ranked_route_trains_the_scores_it_ranks_by`: f64 reference, finite differences over **every** pointer variable including the route gate, under both Dot and Lorentz; and **`r = 1` must equal the ranked route exactly**.

**Caveat I must resolve before fixing the initial value.** At `w_r = 0, b_r = 0` the gate is `r = sigmoid(0) = 0.5`, which is *not* the identity: a freshly initialised gated route would immediately halve the soft pointer's contribution. That would perturb every existing `init=` run the moment the flag is set, and it would make the "exact at zero learned scores" property that `PrimeRoute` currently guarantees untrue for the gated variant. **The correct initialisation is `b_r` very negative (or `w_r = 0, b_r = -large`) so that `r ≈ 0` and a gated route starts as the soft pointer**, preserving both the identity property and comparability with the ungated arm. I will confirm the chosen constant against the existing `POINTER_GATE_BIAS` convention (`:1109`) before committing.

**Honest scope statement.** This is a code-and-test task, and its *result* is a training comparison that **cannot run while the workspace drive is being rebuilt** and that needs the slot. So Task B delivers: the mechanism, the grammar, the tests, and a committed branch — with the training run explicitly deferred and labelled as such. I will not present the code as a capability result.
