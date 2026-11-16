# REPORT: one-model-end-to-end, codex-21 (the read trained on the stream the artifact serves)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-20](../codex-20/REPORT.md).

## What did not work, first

- **The same-row fit is not a generalization reading.** Fitting all 256 windows of the FBS dump
  reaches 1,015/1,024 (`ext-s1`) and 1,023/1,024 (`ext-s2`), but the read was fitted on exactly the
  rows it is scored on. The number that answers the order is the held split below (0.859 / 0.889);
  the same-row fit is only an upper reference and is reported as such.
- **Shift 4 diverges on `ext-s1`'s FBS dump**: 0 accepted steps and a loss of 1,258,285,780. Shift
  12 converges slowly (740/1,024 at step 512, 909/1,024 at 2,048 at `ext-s2`). The chosen shifts are
  8 (`ext-s1`) and 4 (`ext-s2`) — the same choices the float stream made, because the credit is
  normalized from the stream's own maximum and does not carry the scale.
- **My first splitter counted words where the dump counts bytes**, so the trainer refused with
  `a read dump ends early`. The split is by window: 128 windows train, 128 held.
- **A test asserted that a split window continues the session.** It does not: a window is a session,
  so the second window's rows are its own. The test now pins that, not a continuation.
- No language or quality claim; two real states, one seed each, one window family.

## What changed

- `crates/uor-r4-integer/src/stack/session.rs`: `IntegerStackModel::q31_read_rows(&[u32], time)` —
  the artifact's own stream at the read, one `x_in` row per token, each window in a fresh session.
  This is the dump's core, so the bin has no logic of its own to drift.
- `crates/uor-r4-integer/src/stack/q31_read.rs`: two tests — the rows equal the steps' `x_in`
  (`read_rows_are_the_steps_x_in`), and a model with no read or a zero window refuses
  (`read_rows_refuse_an_artifact_without_a_read`).
- `crates/uor-r4-integer/src/bin/fbs-read-dump.rs` (+ the `Cargo.toml` entry): steps a dump's ids
  through the parsed artifact and writes a `Q31ReadDump` — the artifact's `x_in` rows, its Q12
  writer table, the ids — beside the probe's `.ask/.other/.bucket` targets.

The read is retrained, not rebuilt: the identity start copies the dump read's payload and gain, so
the trained read differs from the state's read only in the gates (`ext-s1`: 2,707 of 6,153 bytes,
all inside the gate words; payload and header identical).

## Instrument check

`fbs-read-dump` reproduces the artifact exactly. The dump's own read (`--start dump`, 0 steps) on
the dump it wrote, through the integer session, gives **224 + 216 of 512+512** — the same 440 asked
and 21 + 30 = 51 other of `read-panel` on the artifact (`codex-20`, 34 s host run per artifact). Two
builds of the dump are byte-identical (`f293bd10…`), which is the order's cancel condition, passed.

## Measurements

Held split: windows 0–127 train, 128–255 measured. `ext-s1` trains at shift 8, `ext-s2` at shift 4,
both 2,048 steps, identity start; every count is out of 512 held rows (256 a bucket).

| State | Read | Stream | asked on held rows | train half | `K=1` other |
|---|---|---|---|---|---|
| ext-s1 | the state's own (`/14`, C1) | FBS | 224/512 (0.4375) | — | 0 |
| ext-s1 | FBS-trained | FBS | **440/512 (0.859)** | 486/512 (0.949) | 0 |
| ext-s2 | the state's own (`/14`, C1) | FBS | 329/512 (0.643) | — | 0 |
| ext-s2 | FBS-trained | FBS | **455/512 (0.889)** | 505/512 (0.986) | 0 |

Same-row upper reference (all 256 windows, fitted and scored on the same rows): `ext-s1`
1,015/1,024 (0.991), `ext-s2` 1,023/1,024 (0.999). The float stream's own C1 reading is
504/496 (0.977) at `ext-s1` and 496/494 (0.967) at `ext-s2`.

**The C2′ half does not move.** The FBS-trained reads choose the other fact on at most 7 of their
512 held rows, and on **0 of the identifiable `K=1` rows** (0 of 22 at `ext-s1` held, 0 of 24 at
`ext-s2` held). That is the codex-18 ceiling again: the
gate family expresses the asked fact, not the other one. The relation `/15` artifact is unchanged
and still exact on `K=1`: **47/47** (`ext-s1`), **50/50** (`ext-s2`), 431 and 439 of 1,024 rows,
asked 0.

## The artifacts

| Artifact | Read | sha256 (first 8) |
|---|---|---|
| `ext-s1.uorlut01` | the state's own (`/14`) | `35b01437` |
| `ext-s1-fbsread-held.uorlut01` | FBS-trained on windows 0–127 | `f02a423f` |
| `ext-s1-fbsread.uorlut01` | FBS-trained on all 256 (upper reference) | `2dfd5328` |
| `ext-s1-relation.uorlut01` | relation `/15` | `8bfcf013` |
| `ext-s2.uorlut01` | the state's own (`/14`) | `af45407d` |
| `ext-s2-fbsread-held.uorlut01` | FBS-trained on windows 0–127 | `cd74e3f5` |
| `ext-s2-fbsread.uorlut01` | FBS-trained on all 256 (upper reference) | `f93a64b0` |
| `ext-s2-relation.uorlut01` | relation `/15` | `6a0ec6e7` |

The `/14` and `/15` records stay exclusive: each artifact carries one read.

## What it means

The 0.428 pick parity in `codex-20` was the read's input, not the read's capacity. Retrained on the
stream the artifact actually serves, the same gate family picks the asked fact on 0.859 / 0.889 of
held windows, up from 0.4375 / 0.643 — and it does so with the payload and gain of the original
read, so the served value row is untouched. The remaining 0.11–0.14 is the FBS stream's own
sign disagreement with the float stream (0.887 in `codex-20`), i.e. the integer trunk's read
fidelity, and it is bounded by that stream, not by the read family.

Everything C2′ asks for on the other fact still comes from the relation `/15` read, which is exact
on the identifiable rows in both artifacts and never touches the trunk's stream.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 686 passed,
  1 ignored (2 new).
- `cargo test --locked -p uor-r4-native-vector` — 39 passed.
- `fbs-read-dump --artifact ext-s1.uorlut01 --dump dump-ext-s1-s24 --out …` — twice, byte-identical.
- `native_read_train` — 3 shifts at each state, 2,048 steps, plus the 128/128 split; the device step
  is the default. The first run (`ext-s1`, shift 8, 512 steps) checked host = device on its first
  two steps; the later runs did not repeat that check, and the step it validates is unchanged.
- `read-panel` on the four held-trained and two same-row artifacts — 34 s each, host.

## Still open

- The FBS trunk's read input: 0.887 sign agreement with the float stream. That is a trunk-quality
  question, and it now has a measured ceiling on the read's reach.
- A bound export (stack contract + tokenizer) for this run; the artifacts here are on the explicit
  `--legacy-artifact` path.
- The 1B read is claude's line; the discrete read-binding hinge still needs its Flock trainer.
