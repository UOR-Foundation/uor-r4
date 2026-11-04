# Within-cell orbit-phase termination gate

**Status:** DRAFT

## Question

The fixed point-radius gate never fired and the top-cell majority gate stopped
ordinary continuations, both prematurely. Can a query's position among the 16
orbit roots distinguish a terminal row from a continuation inside the same
writer cell?

This tests a cell-relative phase signature. It does not retune either closed
rule, use reply text as a classifier, or train on evaluation rows.

## Compute class

D84 CPU development research in `uor-r4-cpu-train` on the local Ryzen 9
9950X. It is not GPU qualification and cannot establish language capability.

## Source and inputs

| Input | Identity |
|---|---|
| implementation source | `430fa8bc7ea8888593d13dfd3e30030e24a15f87` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| seed 1 trained artifact | `c856ce21cf46a4be5a0cc02e033d9c7d0cd674bc4ab96a1185a1b9ccf3b88de4` |
| seed 2 trained artifact | `10f91f7309c3ed36e3ce76878b2c75c07bae0102b3933571ccbd8f3b40b350e6` |
| seed 1 no-gate reply digest | `ea95ef96a4adad3b3964912591ae0b9b9dfecf484aaae85cad9cb4dcbc769eec` |
| seed 2 no-gate reply digest | `09920e95529cec1e4c87975d25cd9225a339c30e11a78853262fa918ed2d767c` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split and replay

Replay deepseek-109 exactly: 16 repeated opening keys, up to 16 examples per
key, 102 training and 114 evaluation examples, 32 attempted Q24 memory
movement-coordinate updates at update seed 1, 32 Rayon workers, cached exact
trajectory, and 16 generated tokens per reply.

The trained artifact and trained no-gate reply digest must reproduce before a
new gate result is used. Corpus EOS remains normalized to one trailing
reference marker; decoder BOS remains the generated stop marker.

## Orbit signature and gate

For every prompt-final and reply-content training query:

- compute exact L1 cost from the first 8 Q12 query coordinates to each of the
  16 canonical orbit roots;
- compute the same exact costs from the last 8 coordinates;
- subtract each block's winning cost from all 16 entries;
- concatenate the two 16-value phase vectors into a 32-integer signature;
- retain the writer's top cell, the same argmin pair used by writer admission;
- count the final content row before corpus EOS as terminal and prompt-final
  or earlier reply-content rows as nonterminal.

A cell is gated only if it contains at least one terminal and one nonterminal
training signature. At decode, a query stops only if its minimum exact L1
distance to terminal signatures in its cell is strictly less than its minimum
distance to nonterminal signatures. Equal distances, missing cells and cells
with only one class do not stop.

The matched control shuffles terminal/nonterminal labels over the unchanged
signature multiset within each gated cell, preserving cell, every signature,
and both class counts. Its seed is the model seed mixed with the cell id. The
run records how many signatures changed class.

## Arms

On each seed, evaluate the same 114 prompts with:

1. `no_gate`;
2. `orbit_gate`;
3. `shuffled_orbit_gate`.

Every arm uses the trained memory artifact, the exact integer writer and the
shared `reply_stop` contract with a 16-token cap.

## Bar

On each seed, `orbit_gate` must:

- strictly exceed both `no_gate` and `shuffled_orbit_gate` in non-premature BOS
  completion, defined as `bos_complete - premature_stop`;
- not reduce exact-reference replies against either control.

A positive is bounded CPU development evidence and justifies fresh-phrasing
follow-up. A negative closes this fixed nearest-class orbit-phase rule, not all
geometric termination.

## Instrument gates

- The seed-1 and seed-2 trained artifacts and no-gate reply digests match their
  pins.
- Every gated cell has both classes; every control preserves its cell's
  signature multiset and both class counts.
- The control changes at least one class assignment before public compute.
- At least one evaluation decision enters a gated cell. The gate and control
  report gated/ungated decision counts, so a zero-stop result is called a
  mechanism negative only after the rule was reached.
- Signature tests include exact aligned roots, cell equality, tie/no-stop,
  terminal-only separation, deterministic re-derivation and malformed-width
  refusal.
- The base forward retains exact CPU/integer parity for every Q12 query
  coordinate, token and kept cell. The orbit rule is a deterministic
  post-process of that audited Q12 query and the artifact roots.
- Seed-1 repeat reproduces all non-timing gate data, reply counters, row
  outcomes and ordered reply digests.

## Cost and stop

Local workstation only; no GPU, pod or paid API. Expected below 10 CPU minutes
per seed and below 2 GiB peak RSS. Each seed and the seed-1 repeat run under
`/usr/bin/time -v timeout 600s`. A timeout, a peak above 2,097,152 KiB, or a
seed-1 projection beyond 20 total CPU minutes stops the experiment before the
next seed.

Stop on input or source drift, leakage, missing trained/no-gate replay, a
one-class or unchanged control, CPU/serving divergence, nondeterminism,
counter underflow, any edit to Claude-owned files, or evaluation-specific
threshold tuning.

## TEST FITNESS

TEST FITNESS: PENDING
