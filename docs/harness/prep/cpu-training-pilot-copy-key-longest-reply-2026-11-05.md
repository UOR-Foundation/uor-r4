# Longest-first copy selection in complete replies

**Status:** `TEST FITNESS: PENDING`

## Question

Does deepseek-149's small exact-copy advantage survive when longest-first
selection controls every writer decision in a complete generated reply, rather
than only isolated teacher-forced targets? This decides whether Claude's next
trained copy read should preserve depth-three/four candidates now, or whether
the next dependency is reply termination rather than more copy depth.

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution. The CPU result cannot substitute for a GPU
qualification bar.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `3ac601f7948e` |
| panel-generator source | `4e42ffca5` (`docs/labs/one-model-end-to-end/instrument`) |
| tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| public FBS seed 1 | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| public FBS seed 2 | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Fixed construction

At each generated decision, form the last 192 tokens of the prompt and every
generated token. An exact suffix candidate at depth `1..=4` uses:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

For `0 < s < t`, the latest matching `s` predicts `x[s]`. Missing history at the
start repeats `x[0]`, as in deepseek-149. The selector may replace only the
token already chosen by the artifact writer. The selected token is fed back to
the same exact CPU forward path; the writer, memory, point key, stopping
contract and artifact remain unchanged.

The five arms are:

- `longest_4_to_1`: deepest available exact match, else the writer token;
- `k2_backoff_k1`: depth two, else one, else the writer token;
- `fixed_depth_1`: depth one, else the writer token;
- `random_available_depth`: seeded uniform choice over available depths, else
  the writer token;
- `writer`: no override.

Generation uses a 32-token cap.

## Data split

Two new m-world-v2 panels are generated after FIT and GO:

- `copy-key-reply-tt-2701`: `cell=train,train`, 128 conversations, seed `2701`;
- `copy-key-reply-dd-2702`: `cell=dev,dev`, 128 conversations, seed `2702`.

Both public artifact seeds run on both panels. Every recovered prompt/reply
example is evaluated; no parameter is fit. Each panel must provide at least 200
examples. The examples are held out from this unit because no panel at either
seed has been opened before this registration.

## Instrument gate

Before a result is read:

- all five pinned input hashes match;
- both public artifacts parse through `CpuFbsForward`;
- every prompt fits the artifact context and every reference fits the 32-token
  cap;
- every decision has a writer token, selected token and either a copy depth or
  a writer fallback;
- a random-depth choice occurs only among available exact depths and changes at
  least one longest-first decision per panel;
- a fresh process reproduces every arm reading, reply hash and decision hash
  after path and timing fields are excluded;
- finite tests pass for latest-match order, depth precedence, fallback, BOS,
  cap and context stopping.

Any failure is an instrument or power fault, not a mechanism negative.

## Bar

Complete-reply transfer passes only if, for both artifact seeds on both panels,
`longest_4_to_1` beats each of `k2_backoff_k1`, `fixed_depth_1`,
`random_available_depth` and `writer` on both exact-reference replies and
non-premature BOS completions by the paired two-sigma rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

Selection-surrogate transfer is positive but termination-limited only if the
complete-reply bar fails while longest-first beats `k2_backoff_k1`,
`fixed_depth_1` and `random_available_depth` on correct reference body positions
by the same rule for both artifact seeds on both panels, with no loss of
non-premature BOS stops against `writer`.

Every other outcome is negative for this fixed complete-reply writer override.
It must not be generalized to exact copy, deeper suffix keys or the geometric
path.

## Minimum test power

At least 200 complete replies per panel and two independently trained artifact
seeds. The paired unit is one prompt/reply example. The random control must be
non-vacuous on each panel. A fresh process must reproduce the final reading.
The two-sigma bar is based on observed paired discordance, not a single-row
difference.

## Stop rules

Stop on artifact, tokenizer or panel hash drift; fewer than 200 examples;
nondeterminism; a vacuous random control; a prompt/reference outside context or
cap; malformed trace; resource breach; or any need to modify serving, integer,
CUDA, float-training or Claude-owned source.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and exact-head review | 900 s |
| release build and finite controls | 300 s |
| generator build and two panel draws | 120 s |
| four artifact/panel runs and one repeat | 720 s |
| evidence and delivery | 360 s |
| total increment | **2,400 s** |

- Build jobs: 16; build RSS cap: 4 GiB.
- Each reply run: 32 Rayon workers, 900 s wall cap and 1 GiB RSS cap.
- Retained evidence cap: 64 MiB; fresh target cap: 16 GiB.
- Previous admitted cumulative projection: `72,347.677487699 s`.
- New admitted cumulative projection: **`74,747.677487699 s`**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT` and
is not treated as zero.

## Steps

After FIT and GO, use a fresh output root:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-longest-reply
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05
PANELS=$OUT/panels
BUILD=$SOURCE/target/deepseek/copy-key-longest-reply
GENERATOR_TARGET=$SOURCE/target/deepseek/copy-key-panel-generator
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
IMPL_PIN=3ac601f7948e
GENERATOR_PIN=4e42ffca5
```

The implementation must remain byte-identical to `IMPL_PIN` under `crates/`,
the workspace manifests and `third_party/`. The generator must remain
byte-identical to `GENERATOR_PIN` under
`docs/labs/one-model-end-to-end/instrument/`.

Build `cpu-copy-key-reply` and `heldout-panel`, generate both panels, run all
five arms for each artifact on each panel, repeat one artifact/panel pair in a
fresh process, verify hashes and bars, seal `SHA256SUMS`, and record elapsed
time, peak RSS, target size and retained evidence size.

## Rollback boundary

The unit adds an isolated CPU policy module, reply evaluator and binary. Reverting
it removes only this screen; no artifact, serving format, CUDA source or
Claude-owned file changes.

## TEST FITNESS

TEST FITNESS: PENDING

Pending fresh-context review.
