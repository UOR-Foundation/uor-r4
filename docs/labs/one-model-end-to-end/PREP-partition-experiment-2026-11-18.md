# Pre-registration: the readout experiment (address vs partition)

**Written before the numbers.** The queued address A/B is the instrument; this fixes what each
outcome will mean so the reading cannot be chosen after the fact (L-012: bar before numbers).

## The question

The readout loses three answers in four. Which part is defective?

| candidate | what it would mean | where it lives |
|---|---|---|
| **the address** | the writer ranks cells badly; a geometric address finds the right cell | `address_scores`, `--writer-address` |
| **the partition** | no address can rank the cells, because the cells are `row index mod 256` | `CELLS = 256` in `native_vector_writer.rs` |

## Established arithmetic (measured, not assumed)

- Chance: 8 kept cells × 32 slots over 8,192 tokens = **3.125 %**.
- Measured admission under `ports`: **2.9 %** (2,930 of 101,175 rows at step 8,192: 1,782 admitted is
  the step-8,192 reading) — **at, and slightly below, chance**.
- Measured pick: **0.82 %** (832/101,175); `nearest` on the same table: **15.32 %**.
- The writer's address under `ports` costs ~**0.5 k** lane-operations per token; under `centroids`,
  ~**393 k** (256 centroids × 512 lanes). The candidate inventory is ~262 k; whole-table nearest is
  8.4 M.

## The two arms (already queued, 4,096 steps each, seed 1, same data, batch 48)

1. **A — `ports`** (baseline): two 8D ports against 16 orbit roots, `c = 16a + b`.
2. **B — `centroids`**: each cell's centroid over all 512 lanes, ranked by L1 to the query.

Read from each: `admitted`, `top-1`, `nearest`. **Never the token hinge** — it is `−mean|q−v|`, so
margin 1.0 is unreachable and it cannot measure progress (claude-142).

## Gates, fixed now

- **H1 (the address is the defect):** B's admission **≥ 8.0 %** (2.6× chance) → the address is a real
  lever. Then, and only then, take the ordered pick gate: **≥ 3.0 %** (3.7× today), equal steps.
- **H0 (the partition is the defect):** B's admission **≤ 4.1 %** (1.3× chance) → the cell carries
  almost no information about the target whatever the ranking rule; eliminate the address and go to
  the partition unit.
- **Inconclusive band, 4.1 % – 8.0 %:** do not spend. Read the pick; if pick ≤ 1.2 %, treat as H0.

**Cancel condition (instrument first, before any arm is read):** two identical `ports` runs must
reproduce identical numbers. The run is deterministic, so any difference is an instrument fault and
the whole comparison is void. Check this before reading B.

## What we already know that costs money, and was not previously established

`build_hamilton_model_full` builds the table with `StackModel::new(config, device)` from a seed:
**the rows are learned, and a token's cell is its row index mod 256.** So in this line the partition
is not a knob. A geometric *partition* would require a **vocabulary → row codebook** — the placement
the discrete line already owns (`address_cells.rs`:
`StructuredV1` / `FullVectorV1` / `WiderBlocksV1` / `EMeansV1`, each with a seeded shuffled control,
and `-33.94 %` against `P8`, `-25.12 %` against its own shuffled control on TRAIN predecessor L1) —
ported to this line and pinned. **That is a larger unit than an address change and it is not what the
queued A/B tests.**

So the A/B is a test of the *address*, and a negative result is what licenses the codebook work.

## Validity caveats to carry into the reading

1. **The dev panel is 75 % verbatim TRAIN replies** (`BLOCKERS:29`). Admission measured there can
   overstate facts the model has seen verbatim. If the verbatim rows are recoverable at scoring time,
   report admission split by verbatim/non-verbatim; if not, say so in the reading rather than
   generalising.
2. **Admission is not capability.** A cell being *open* is not the token being *picked*. Both numbers
   are reported; neither alone is the readout.
3. **Cost is not free.** If B wins on admission, its address costs ~512× the port address per
   token-set. The served form must then be the M4 kept-cell shortlist (8.4 M → 262 k), and the
   artifact must carry 256 × 512 i16 centroids = **256 KiB** — one extra inventory pass — and must
   preserve the writer's tie order ("smallest cell identity on an exact tie") **bit-identically**,
   or it breaks D85 replay.

## Cost

4,096 steps at batch 48 on a free card ≈ 2 × 50 min ≈ **1.7 h**, already queued behind the long run.
The next unit (partition codebook) is **not** authorised by this document; only H1 is.

## Correction (19 Nov): the partition is cheaper than this document first said

The section above says a geometric partition needs "a vocabulary → row codebook ... ported to this
line". That overstates the work. Verified in the source and the checkpoint:

- The writer's table **is** `embedding.weight`: `geometric_stack_native.rs:1496` builds the writer
  from `self.get("embedding.weight")`, and the checkpoint carries it as exactly one tensor,
  `embedding.weight [8192, 512] F32`.
- A token's cell is its **row index mod 256**, so changing the partition is changing **which row a
  token occupies** — a permutation `sigma` on 8,192 ids. No model code changes.

So the unit is: produce `sigma` on the host (the `address_cells` placement algorithm, run once, with
its seeded shuffled control), write it into a checkpoint, and train from it with `--init-from`.

**The trap, and it is a silent one.** `fn head()` also returns `embedding.weight`, so the tensor is
the tied output projection as well as the input table. Permuting rows alone makes the model read and
score the *permuted* label: it would be trained to predict `sigma(t)` when the target is `t`, with no
error anywhere. The permutation must therefore be applied to the **corpus and dev token ids** as well
as to the table rows, which makes it a new corpus (new hashes, a new run) — not a drop-in table swap.

**Revised cost:** a permutation (seconds), a corpus remap over 385 M tokens (minutes, ~1.5 GB of new
shards), and one 4,096-step arm against its shuffled control ≈ **1–2 h of wall clock**, versus the
"port a mechanism" framing above. That is cheap enough that H0 licenses it without a further decision,
and it is still gated on H0: H1 makes it unnecessary.

## Result (19 Nov): H1, by evaluation rather than by the two training arms

The pre-registration assumed the A/B needed two 4,096-step training arms. It did not: `--init-from`
accepts `--writer-address`, so both addresses can be scored on **one checkpoint** with the trunk
fixed, which is a *stronger* instrument (any difference must come from the readout, and `nearest`
proves it did not move).

- **H1 met**: `centroids` admitted **12.01 %** and **12.11 %** on the step-4,096 and step-2,048
  checkpoints, against the **≥ 8.0 %** gate (ports: 2.28 % and 2.47 %).
- **Pick gate met**: **8,221 (8.1 %)** and **7,793 (7.7 %)**, against **≥ 3.0 %** (ports: 430, 240).
- `nearest` **identical** within each pair (10,942 and 9,266), so the trunk did not change.
- The training arms are therefore not run for the address question; only a single **feedback arm**
  remains queued (train with `ports` at 4,096 steps, batch 48) to ask whether the broken readout also
  damaged the trunk.
- **H0 is not triggered**, so the permutation work below is not authorised and stays in reserve.
