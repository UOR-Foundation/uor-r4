# ORDER: one-model-end-to-end, codex-52 (the writer's address, built behind a switch)

Lab: codex. The readout fix aimed at the measured defect: the writer chooses the target's cell on
**16 of the width's coordinates**.

## What the writer does today

```
cell_score = port(0) + port(width-8)          // 16 orbit roots each, summed -> 256 cells
port(s)    = -mean |q[s..s+8] - root|         // 8 lanes against each root
```

Admission admits **2.9 %** of rows while the target is the nearest table token in **15.3 %** — three
answers in four are thrown away by the address, and of those admitted the final pick is right 28 %.

## What this unit adds

`--writer-address centroids`: score every cell by the carrier's distance to the cell's **centroid**
over **all 512 lanes**:

```
centroids[c] = mean of the table's rows with id ≡ c (mod 256)     // 32 rows a cell, derived
cell_score   = -mean |q - centroids[c]| over the width
```

No new learned parameter: the centroids come from the served table and the existing identity
partition. Off by default, so every existing call keeps the port address.

## The gates

1. **Ports = today**: with the switch off, outputs and picks are bit-identical (the path is unchanged
   code).
2. **Centroids ≠ port**: on the same checkpoint and DEV, report `admitted` and `top-1`. The claim is
   that `admitted` rises toward `nearest`; if it does not, the address is not the cause and the
   writer's final pick is.
3. **Served parity** (separate unit): the integer session must compute the same centroids from its
   own table and admit the same cells; that needs a centroid table in the artifact
   (`256 × width` as i16 ≈ 256 KiB) and a bit-identical differential test. Until then this is float
   evidence only and cannot move a served claim.

## Order

1. `kept_cells` 8 vs 16 (armed, runs when the long run frees the card): breadth.
2. This unit's float A/B at the best `kept_cells`: does the whole-vector address beat the ports.
3. The integer parity unit.
4. Then the 196-item panel, on an artifact whose pick is no longer 0.8 %.
