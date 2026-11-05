# REPORT: cpu-training-pilot, deepseek-159

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `330185de6`.

## Status

Decision: **REJECT** for the fixed exact-body generated-stop construction. The
same eight-prototype sign-signature gate learned termination on the filtered
training trajectories (**7/11** exact on both artifact seeds), but transferred
only **1/13** exact dev replies. Seed 2 also produced **9** premature dev stops
against the registered four-reply limit. This closes exact-body filtering with
this gate and threshold family; it does not reject geometric termination,
copy-boundary learning or terminal-token construction.

## What did not work

- The gate's dev exact counts failed the seven-reply floor on both seeds:
  **1/13** for seed 1 and **1/13** for seed 2.
- The dev paired controls failed on both seeds. The true gate added one exact
  reply over shuffled, all-reference and no-gate, below the registered
  two-reply paired minimum.
- Seed 2 emitted **9 premature dev stops**, above the four-reply 1% limit.
  Seed 1 exactly reached the limit at **4**.
- Seed 2 fired at the reference position on three dev trajectories, but only
  one had an exact body. Seed 1 fired once there and that body was exact.
- The all-reference control retained its deepseek-157 failure: zero exact dev
  replies on both seeds.
- The exact-body train set contains only **11** terminal trajectories, or 11
  distinct terminal signatures. The fit is possible and nonzero, but the
  training result is in-sample; the held-out result is the decision.

## What changed

`cpu-copy-generated-stop` now accepts `--exact-body`. In that mode:

1. it validates and replays the sealed longest-first trajectories exactly as
   deepseek-157 did;
2. it admits a training trajectory only when its generated prefix through the
   normalized reference length equals the reference;
3. it labels the reference-length decision terminal and earlier decisions
   nonterminal;
4. it fits eight terminal and eight nonterminal 256-bit sign prototypes with
   the existing Hamming score and training-only 1% false-stop limit;
5. it records the selected row counts, distinct signatures per class, margin
   and fitted stops;
6. it retains the class-preserving shuffled gate, deepseek-157 all-reference
   gate and no-gate controls;
7. it reports every reply's fired step, stop class, exact status and unchanged
   selected-token trace hash.

The first firing emits BOS. Otherwise the sealed longest-first token is fed
back through exact `CpuFbsForward`. The decoder does not read the reference.

## Measurements

Selected training census and fit:

| Artifact seed | Exact terminal | Exact nonterminal | Distinct terminal | True terminal stops | True false stops / limit | Margin |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 11 | 308 | 11 | 7/11 | 0/308 (limit 3) | 69 |
| 2 | 11 | 308 | 11 | 7/11 | 0/308 (limit 3) | 67 |

Complete-reply simulation:

| Artifact seed | Panel | Oracle exact | True exact | True premature | True at reference | True no fire |
|---:|---|---:|---:|---:|---:|---:|
| 1 | train | 11 | **7** | 1 | 7 | 445 |
| 1 | dev | 13 | **1** | 4 | 1 | 444 |
| 2 | train | 11 | **7** | 2 | 7 | 444 |
| 2 | dev | 13 | **1** | 9 | 3 | 437 |

Controls:

| Artifact seed | Panel | Shuffled exact | All-reference exact | No-gate exact |
|---:|---|---:|---:|---:|
| 1 | train | 0 | 0 | 0 |
| 1 | dev | 0 | 0 | 0 |
| 2 | train | 0 | 0 | 0 |
| 2 | dev | 0 | 0 | 0 |

Both fresh-process repeats were byte-identical to their first run. The final
runs took **9.09, 9.12, 9.05 and 8.98 s** and peaked at **198,040 KiB**.
The incremental release build took **0.58 s** and peaked at **243,692 KiB**.
The final analyzer binary is **2,019,440 bytes**, SHA-256
`2ccf94a93eb289243b8e4b6457ca2de71aacf6c237f31adbdce0b8392db37e3c`.

Source report hashes:

- train: `28a8caaffef6d815da86a6e84207729f3acd41892ced8a205cfe5248d7c2f62a`;
- dev: `e2485b3b4012a4c5102b817e3758517502213d126cbb136a46552df18b936d17`.

Raw attempt-7 manifest SHA-256:
`b27f0ec3ed2c3ab9b4d02283e8c3df2b9dd48b2b304a47dc48a62e09dae024e6`.
Published evidence manifest SHA-256:
`65937627a3000747756e6f095a6aa0cd065e4c2bdb4ee9e2f3251f79450aac59`.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores.
- **80** binary tests passed; the focused generated-stop binary passed its
  **6** tests.
- Scoped package Clippy with `--all-targets --no-deps -- -D warnings`,
  rustfmt, D23/D25/D32 ratchets, claim wording and `git diff --check` passed.
- Every input hash, source report hash and unchanged selected-trace hash
  matched.
- Both artifact seeds used train panel 2711 and evaluated unchanged on dev
  panel 2712.
- No serving, integer runtime, CUDA, float-training or Claude-owned source
  changed.

## Size

Against base `330185de6`, this unit changes one CPU-train binary (**+146 /
-17**), nine task, preregistration, notebook, roadmap, state and lab-record
files (**+474 / -0**), and 12 published evidence files (**+130,909 / -0**):
**22 files, +131,529 / -17**. No binary, corpus or scratch artifact is tracked.

## Skills used

No system-provided Codex skill applies. Project CPU research, evidence and
review procedures were used.

## Still open

- Do not retry the same eight-prototype gate with another label filter derived
  from the reference horizon.
- The next termination unit should train on copy phrase boundaries or use a
  continuation objective with an independent boundary signal.
- Improve terminal-token reconstruction before expecting a stop gate to
  recover even the 11/13 oracle ceiling.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

Filtering to exact bodies lets this gate fit the small successful set, but the
state does not transfer. It finds one complete dev reply and makes too many
early stops. Exact-body labeling with this prototype family is closed; the
next attempt needs a better boundary or terminal objective.
