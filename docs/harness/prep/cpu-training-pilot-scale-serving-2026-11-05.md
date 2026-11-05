# Pre-registration: exact scale-serving preflight

Run ID: `cpu-training-pilot-scale-serving-2026-11-05`

## Status

NOT RUN. This sheet fixes the D84 CPU development check before the registered
compute.

## Question

At the three widths and contexts registered by Claude's `scale-probe`, can the
current exact `UORLUT01` output path export, reload and reproduce every CPU and
integer-serving decision, and what artifact size and decode throughput does
each shape impose?

This is an instrument and cost preflight. It does not train a corpus model,
qualify GPU training, measure language quality or show that a width fits GPU
memory.

## Why this is new

- `cpu-fbs-forward-bench` hardcodes width 256, MLP 384, six layers and context
  192, and does not call `audit_served_forward`.
- Existing public trained-artifact audit coverage is width 256.
- Claude's scale-probe registers widths 256, 512 and 1,024 on real text.
- No existing CPU benchmark covers all three widths, several contexts, both a
  read-free and a full Hamming/memory artifact, exact CPU/serving parity,
  repeated decision digests, artifact bytes and decode throughput.

The result gives the scale run an exact output-path reference. It cannot fill
the scale-probe quality slopes, whose qualification remains GPU research.

## Base artifact

There is no pre-existing trained artifact. Each cell constructs the fixed
synthetic FBS snapshot in the registered source, exports it to `UORLUT01`, and
reloads those bytes. Building the same cell twice must produce byte-identical
artifacts before any timing or parity reading is accepted.

## Data split

No corpus or response mask is read. Training and evaluation data are not used.
Each context length supplies one deterministic input-only token sequence,
`token[i] = (i * 73 + 19) % 4096`, to both arms. The full arm marks row 1 as
the reply start so the exact opening/reply memory lookup path executes.

## Source and binary

- base: `3ac40d6d5`;
- implementation source:
  `bf3d44dd47faaf7531edd089b464c328990291bc`;
- binary:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/harness-target/release/cpu-scale-serving-bench`;
- binary SHA-256:
  `d652a9d4056b9534000ddb2b53b027e1eb7cdd02c2c3cec38452b95d5cb866fe`;
- host: AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- execution: one process, platform default worker availability for library
  code, no GPU.

## Fixed grid

Every cell uses vocabulary 4,096, MLP 384, six layers, eight writer cells and a
seed derived as `20_261_105 XOR (width << 17) XOR context`.

| Width | Contexts | Arms |
|---:|---|---|
| 256 | 16, 64, 192 | projection, full |
| 512 | 16, 64, 192 | projection, full |
| 1,024 | 16, 64, 192 | projection, full |

`projection` has no Hamming or memory tables. `full` adds Hamming reads at
depth 1 and the final layer plus opening and reply memory. Hamming heads are
`width / 64`: 4, 8 and 16 for widths 256, 512 and 1,024.

The token sequence is deterministic:

```text
token[i] = (i * 73 + 19) % 4096
```

The full arm marks a reply start at one-based row 1. The projection arm has no
reply start.

## Steps

Run only after FIT and D67 GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-scale-serving
OUT=/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/2026-11-05
BIN=/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/harness-target/release/cpu-scale-serving-bench

test ! -e "$OUT/summary.json"
git -C "$SOURCE" merge-base --is-ancestor bf3d44dd47faaf7531edd089b464c328990291bc HEAD
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = d652a9d4056b9534000ddb2b53b027e1eb7cdd02c2c3cec38452b95d5cb866fe
mkdir -p "$OUT"
timeout 1200s "$BIN" --output "$OUT/summary.json"
```

The output path must be absent before the run. If the command fails, preserve
the error and do not reuse the root.

## Minimum power and required readings

- 3 widths x 3 contexts x 2 arms = **18 cells**;
- **three** complete exact-parity repeats per cell;
- every artifact rebuilt from the same seed and byte-identical;
- zero query-coordinate, token and kept-cell mismatches;
- CPU and serving decision hashes equal in every repeat;
- complete grid coverage:
  - `positions_checked = context`;
  - `query_coordinates_compared = context * width`;
  - `token_decisions_compared = context`;
  - `kept_cells_compared = context * 8`;
- artifact SHA-256 and bytes;
- serialized table-payload and decoded-resident bytes;
- CPU-oracle and integer-serving tokens/s from the timed final repeat;
- peak RSS and total wall time.

Negative controls:

1. width 2,048 is refused by the writer cap;
2. a truncated artifact is refused;
3. a token outside vocabulary is refused;
4. a token sequence longer than context is refused.

## Bar

The preflight passes only if:

- all 18 cells complete;
- every exact-parity and decision-hash check repeats;
- all four negative controls fail closed;
- peak RSS is below 2 GiB;
- total wall time is at most 20 minutes.

## Outcomes

- **PASS:** all registered widths and contexts use the same exact output path,
  and the measured byte and throughput tables are available to Claude's scale
  run.
- **INSTRUMENT_FAULT:** any parity, repeat, shape, control, timeout or RSS check
  fails. Do not interpret it as a language result.
- **UNSUPPORTED:** a registered width/context cannot be exported or parsed by
  the current path after the instrument class is ruled out.

No width is retuned or removed after the result. A failure closes this exact
preflight configuration and identifies the output-path defect.

## Cost and stop

- local CPU only, $0;
- one run, at most 20 minutes;
- stop above 2 GiB peak RSS;
- stop on source, binary, shape, parity, determinism, control or output-root
  drift;
- do not edit Claude-owned float-training, serving or exporter source.

## Claim boundary

The result covers synthetic structural artifacts at the registered widths and
contexts only. It does not establish real-text quality, corpus training,
GPU-memory feasibility, a scale slope or model qualification.

## Verdict

TEST FITNESS: PENDING
