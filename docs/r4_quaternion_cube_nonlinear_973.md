# #973 finite-indexed R4 nonlinear block

**Date:** 2026-09-04  
**Status:** `PRE_EXECUTION_OPERATOR_FIXED`

## Implemented decision

`R4H4FrameQuaternionCubeResidualV1` replaces the dense SwiGLU call in the
accepted sparse recurrent path with one bounded, parameter-free R4
nonlinearity. The sparse candidate selector, transported learned Q/K/V/O
attention, recurrent state, tokenizer, vocabulary head, and sampler stay
unchanged. `R4SparseGeometricCandidateSoftmaxKVBindingV1` remains the dense
SwiGLU comparator over the same learned artifact.

After each attention residual, the existing post-attention RMSNorm produces
`n[B,48]`. The new state map splits it into twelve ordered R4 blocks
`n_j in R4`. For the current token's cumulative H4 frame `F_h`, using the
existing convention `F_h: local R4 -> model R4`, each block executes

```text
q_j       = transpose(F_h) n_j
C(0)      = 0
C(q_j)    = q_j^3 / ||q_j||^2                 when q_j != 0
G_h(n_j)  = F_h C(q_j)
delta_j   = G_h(n_j) - n_j
values'   = values + flatten(delta_0, ..., delta_11)
```

Writing `q=(a,v)` with `v in R3`, the f32 implementation uses the closed form

```text
C(q) = (
  a * (a^2 - 3 ||v||^2),
  (3 a^2 - ||v||^2) * v
) / (a^2 + ||v||^2).
```

The zero branch uses exact zero detection and returns zero. It introduces no
epsilon or fitted threshold. All twelve blocks are evaluated from the same
normalized input and the same current frame. The operator adds no learned
parameters or persistent state. Dense MLP tensors remain in the accepted
artifact so the two runtime arms stay byte-compatible, but this candidate
does not read or execute them.

## Why this operator

Over real quaternions, norm multiplicativity gives `||C(q)||=||q||`. The map
is odd and homogeneous over real scalars, and its residual obeys
`||C(q)-q|| <= 2||q||`. These are algebraic consequences of the declared
operator. They are not claims about f32 equality, trained behavior, language
quality, or the surrounding learned decoder.

The current H4 element selects one of 120 frame-conjugated nonlinear maps on
continuous R4 blocks. “Finite” here means a finite indexed operator bank with
fixed work and fixed state; it does not mean that hidden values are quantized.
The implementation is transitional f32 compiler-side arithmetic. Integer/table
lowering, zero-multiply serving, and allocation-free execution remain later
work.

At batch one, each layer and token evaluates twelve R4 blocks. Its declared
logical work is 384 frame-coordinate products, 168 closed-form quaternion
products, twelve reciprocals, and 48 residual subtractions. The dense
comparator reads 18,432 learned matrix weights per layer and token across its
`48 -> 128`, `48 -> 128`, and `128 -> 48` projections. These are analytical
operation counts, not latency or hardware-energy measurements.

## Donor boundary

The project knowledge index was queried for the active nonlinear-block seam.
Original sources were then inspected for that decision.

- SpiralCore v66, SHA-256
  `38449bddefd359d69a497ca4965e6d14a39f7ab2d33940e41fd9747598b3a4ea`,
  supplies finite operator-index and refusal discipline, but its implemented
  Cl(0,6) bivectors are linear signed 8x8 maps over a separate 240-root E8
  system. It supplies no R4/R8 carrier or nonlinear language block, so no E8
  operator is imported here.
- The pinned W33 chamber operators act on `Q^160`. Their rank-48 projector is a
  particular subspace, not a canonical identification with this decoder's
  `R48`, so they are not imported.
- The pinned NEMESIS material supplies an attributed Cayley-Dickson/quaternion
  product description and a useful encode/operate/decode discipline. Its wider
  ring, inverse, complexity, and physical claims are not transferred, and no
  source code is copied.
- The existing UOR/H4 sidecar supplies the validated 120-frame carrier used by
  this implementation. Exact H4 table identities and f32 model behavior remain
  different evidence layers.
- The zeta channels and signed `Z[phi]` heatmap remain separate trace/control
  hypotheses. The #970 0/6 strict-transfer result forbids treating those
  classes as an established semantic activation.

This mechanism does not revise the #967 shortest-Cayley 6/6 tie, the #970
heatmap 0/6 transfer result, or the uneven 12-token/3-token sparse trajectory
preservation recorded by the preceding checkpoint.

## Direct comparison fixed before execution

Run exactly the existing two development prompts through two no-fit arms:

1. accepted sparse recurrent attention plus dense SwiGLU; and
2. the identical sparse recurrent attention plus
   `R4H4FrameQuaternionCubeResidualV1`.

Both arms use the accepted artifact, tokenizer, exact-H4 geometry/frame
sidecar, seed 9738, top-k 40, temperature 0.8, and at most sixteen generated
tokens. Record the generated token IDs and continuations, common prefixes,
finite output/state checks, sparse source ceiling, forbidden-read counters,
dense-MLP calls, operator blocks, maximum f32 block-norm error, and maximum
residual-bound ratio. Do not fit, tune, inspect held-out data, add a third arm,
or launch a scale campaign in this task.

Passing this comparison means only that the finite-indexed nonlinear R4 path
executes end to end and bypasses dense SwiGLU while preserving the accepted
bounded sparse reader. Language usefulness, trainability, H4 advantage,
reasoning, coding, table-native execution, and architectural alpha remain
unverified hypotheses.
