# Phase-transition termination readout

**Status:** `TEST FITNESS: PENDING`

## Question

Does a phase-aligned transition between consecutive generated geometric states
separate reply endpoints from ordinary continuation better than the closed
current-state density and remaining-horizon representations?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving, `UORLUT01`, CUDA, float training or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `e50cdea82` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use the deepseek-137 16-key split: 98 training and 103 evaluation samples per
artifact, runner seed 1.

## Fixed construction

For each generated offset `t`, keep the current modulo-256 code as the content
readout signature. Derive a separate 2,048-bit termination transition:

```text
phase_t = popcount(previous_code(q_(t-1))) mod 64
transition_t = rotate_left64(current_code(q_t), phase_t)
               XOR previous_code(q_(t-1))
```

Offset zero uses an all-zero transition because there is no predecessor. Fit
the existing class-balanced density rule over transition states with the
unchanged maximum neighbour set `{1,2,4,8}` and generated-offset radius `2`.
Fit thresholds under the existing one-percent nonterminal false-stop limit.

The matched control permutes terminal labels through the same transition rows.
It must change the fitted gate.

## Cancel probe

Deepseek-142 closed current-code density at 19/84 terminal recall against the
26/84 floor. Deepseek-143 closed exact-offset horizon at 66/1,484 false stops
against the 14-row limit. Deepseek-144/145 closed current-plus-XOR content, not
termination. None used a phase-aligned relative transition as the terminal
state.

## Preflight

Run seed 1 first with `--stop-phase-neighbors 8`. Proceed to seed 2 only if:

- the phase rows contain at least 84 terminal and 1,484 nonterminal generated
  decisions;
- true phase density recalls at least 26/84 terminal rows;
- true phase density emits at most 14/1,484 false stops;
- the shuffled-label gate differs from the true gate; and
- source and all five public inputs match.

Preserve and seal the preflight even when it stops.

## Arms and bar

If the preflight passes, run writer, code, phase density and shuffled phase
density over complete 16-token replies on both seeds.

Both seeds must:

- recall at least 26/84 terminal rows and emit at most 14/1,484 false stops;
- beat the shuffled-label terminal arm by the prompt-level paired two-sigma
  stopping rule;
- keep every ungated content decision equal to code;
- produce at least 9 exact references and 9 BOS-complete replies, including at
  least 8 stops within the reference length;
- keep generated top-1 and matching depth not below code; and
- reproduce every non-timing field in a one-worker seed-1 repeat.

Report terminal/false counts, threshold, selected neighbours, gate hashes,
exact-reference/BOS counts, top-1/depth, openings, distinct/repeated replies,
workers, wall, RSS and transition latency.

## Instrument gates

- Phase packing uses the previous current-state code and a 64-bit rotation
  derived only from that previous code; offset zero is all zero.
- Fitting, evaluation and generation use the same transition history.
- Label permutation changes the gate.
- Ungated content remains byte-identical.
- Worker-count repeat matches all non-timing outputs.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Minimum test power

The split and model seeds are fixed. Fitting and generation are exact and
deterministic. Two trained artifacts and a one-worker seed-1 repeat prevent a
one-seed or worker-order claim. The result applies only to this public
development corpus and phase-transition construction.

## Stop rules

Stop on source/input/split drift, transition-history mismatch, degenerate or
vacuous transitions, preflight failure, vacuous label control, worker
nondeterminism, resource breach, or any need to change integer serving, CUDA,
float training or Claude-owned source.

## Decision consequence

A pass makes phase-transition termination a supported CPU candidate and
justifies a separate artifact-bound serving integration unit. A fail closes
this exact phase alignment, not all transition geometry.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and exact-head review | 900 s |
| release build, controls and two-seed preflight | 1,500 s |
| two seeds plus repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **3,600 s** |

- Build and controls: 16 jobs, 4 GiB RSS, separate 2 GiB and 3 GiB targets.
- Runs: 32 workers for seeds 1/2 and one worker for the repeat, 1 GiB RSS and
  600 s each.
- Retained evidence limit: 201,326,592 bytes.
- Previous measured cumulative is `35,945.808816343 s`; the admitted
  cumulative is `62,827.677487699 s`.
- New admitted cumulative: **66,427.677487699 s**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT` and
is not treated as zero.

## Steps

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-phase-termination
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/phase-termination/2026-11-04
BUILD=$SOURCE/target/deepseek/phase-termination-146
CONTROL_BUILD=$SOURCE/target/deepseek/phase-termination-controls-146
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=e50cdea82

test ! -e "$PIPELINE"
test ! -e "$BUILD"
test ! -e "$CONTROL_BUILD"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  .cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party
mkdir -p "$PIPELINE"

CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-content-readout-driver
CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked --release \
  -p uor-r4-cpu-train --lib
CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked --release \
  -p uor-r4-cpu-train --bin cpu-content-readout-driver
CARGO_TARGET_DIR="$CONTROL_BUILD" cargo clippy --locked \
  -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings

BIN=$BUILD/release/cpu-content-readout-driver
"$BIN" "$DUAL/dual-c131-seed-1-raw_both.uorlut" \
  "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
  "$PIPELINE/select-only-unused-1" \
  --expected-artifact-sha256 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
  --reply-cap 16 --stop-phase-neighbors 8 --seed 1 --select-only \
  >"$PIPELINE/seed-1.preflight.json"
```

Stop before seed 2 when any preflight condition fails. Seal every generated
file in the attempt directory without editing it afterward.

## Rollback boundary

The unit changes only `uor-r4-cpu-train` and research records. Reverting it
restores current-code termination behavior and leaves integer serving,
`UORLUT01`, CUDA and Claude-owned paths untouched.

## TEST FITNESS

TEST FITNESS: PENDING - M2 #2030 deepseek-146

Pending fresh-context review.

## Pre-run GO

Pending after FIT.
