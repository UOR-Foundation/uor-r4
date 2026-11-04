# REPORT: content-readout serving sidecar, deepseek-140

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `70f45367d`.

## What did not work

- **Serving parity did not improve language output.** The content arm remained
  at **73/64** and **58/39** top-1/depth, produced **0** exact references and
  capped all 103 replies on both seeds.
- **The readout remains sparse.** It fired on 103/1,648 generated decisions on
  seed 1 and 113/1,648 on seed 2. The native or point-key fallback handled the
  other 1,545 and 1,535 decisions.
- **The serving artifact is now two files.** `UORLUT01` bytes remain unchanged
  by the readout, but a deployer must carry both `serving.uorlut` and its
  artifact-bound sidecar.
- **This is CPU/serving integration evidence, not GPU or useful-chat
  evidence.** It does not establish termination, fresh phrasing or semantic
  correctness.
- **Attempt 3 was an instrument failure.** It passed the same parity and
  control assertions but ran the focused tests inside the chat target, which
  grew to 2,246,424,310 bytes against the registered 2 GiB limit. Attempt 4
  uses a separate control target and preserves attempt 3 as evidence.

## What changed

The positive modulo-256 content readout now has an exact integer serving path.
`IntegerStackModel::serve_step` obtains the normal native writer or point-key
fallback and then applies the sidecar's nearest-example target only inside its
training-fitted Hamming radius. Raw `step` and `step_native` do not consult the
readout.

The sidecar uses schema `uor-r4.stack-content-readout/1`, binds the complete
serving artifact SHA-256, and rejects wrong artifacts, sign-mode or reference-end
payloads, BOS targets, malformed widths and thresholds, unknown fields,
out-of-vocabulary targets and oversized input.

`uor-chat --content-readout PATH` attaches the table for one-shot, REPL,
prompt-ID and OpenAI-loopback serving. The model is configured before all four
paths consume it.

The CPU content driver now writes the training-derived turn-memory artifact
and sidecar, then checks every generated evaluation decision against an
independent `IntegerStackModel` carrying that sidecar.

Session snapshots bind the canonical attached-readout hash. In the synthetic
no-turn-memory FBS control, a same-readout restore succeeds while a model with
no readout or a different readout refuses before changing state. The public
artifact keeps its pre-existing refusal to restore turn-memory sessions.
`UR4SESS2` carries that identity, while legacy `UR4SESS1` snapshots remain
byte-identical when neither side has a readout.

Attempt 1 at pin `a463a1c8d` passed the parity arm but predated session
identity. It remains under [`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/evidence-attempt-1/)
with its original report, checksums and resource ledger. Attempt 2 at pin
`fe253c5ca` passed the parity arm but omitted the required loopback sidecar
identity and is retained under
[`evidence-attempt-2/`](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/evidence-attempt-2/).
Attempt 3 at pin `819482378` passed every parity and control assertion but
grew its shared chat target above the registered ceiling; it remains under
[`evidence-attempt-3/`](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/evidence-attempt-3/)
with its own [resource ledger](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/attempt-3-resource-ledger.json).
Attempt 4 at the same implementation pin isolated the controls and is the
deciding run below.

After deepseek-141 merged the item scope, this branch rebased onto fresh
`main`. The measured runtime source is now commit `881b8ad95`; its crate source
outside tests is identical to executed pin `819482378`. The later `d847ba71c`
review-repair commit adds parser-refusal tests and records only.

The preserved attempt-3 metadata records the registration timestamps; the
evidence directory's filesystem birth and final-manifest mtimes were
`2026-11-04T18:51:47Z` and `2026-11-04T18:53:02Z`. Its sentence about checking
all targets was a pre-run plan; the attempt-4 recipe actually rechecks the
control and chat targets after tests, while the isolated driver target is
checked immediately after its build.

## Measurements

Both seeds used the registered 98-training/103-evaluation split with 1,120 and
1,085 content decisions respectively.

| Public seed | Serving artifact SHA-256 | Rows | Targets | Radius | Chosen / fires / fallbacks |
|---:|---|---:|---:|---:|---:|
| 1 | `ed06a88b…d47daffc` | 894 | 207 | 537 | 1,648 / 103 / 1,545 |
| 2 | `2c071845…4eaf7e842` | 894 | 207 | 533 | 1,648 / 113 / 1,535 |

Decision-for-decision parity:

| Public seed | Prompts | Decisions | Token mismatches | Decision mismatches | First mismatch | CPU = serving digest |
|---:|---:|---:|---:|---:|---|---|
| 1 | 103 | 1,648 | **0** | **0** | none | `54a91121…f89eed93` |
| 2 | 103 | 1,648 | **0** | **0** | none | `f40ea8b7…ad56a813` |

The compared fields were chosen token, native/point fallback, complete
2,048-bit signature, nearest distance, readout-fired state, complete reply
tokens and stop class.

Sidecars:

- seed 1: `e60a909e14a8fa2ab310a9a6255fa995920f51202471936661077c8f7ea290e7`;
- seed 2: `20534cdd3c4710b77e3df26d18da33edc55810269bbda3358f10cc1426d4fdd6`.

The prompt-ID path loaded the seed-1 `serving.uorlut` and sidecar, recorded
both hashes, and generated `[2564, 1, 1, 1]` over four capped tokens. Its
readout-bound session snapshot was 47,368 bytes with SHA-256
`efc24c3d6136a9dad5e165061a15fbacb70395cbb4d71aebb1df6e238aed94c9`.
A real one-shot text turn generated `[563]` (`small`) and exited at the
one-token cap. The REPL completed one turn through the same sidecar. The
OpenAI loopback tests covered health, model, capability, chat, completion,
Responses, and final-stream JSON.

Build and runs:

- driver build: **20 s**, peak RSS **1,224,168 KiB**, target
  **266,917,268 bytes**;
- `uor-chat` build: **12 s**, peak RSS **1,199,584 KiB**, target
  **104,717,444 bytes**;
- focused controls: **33.47 s**, peak RSS **2,430,812 KiB**, isolated target
  **2,141,709,077 bytes**; 7 integer and 18 loopback tests passed;
- walls: **3.19/3.15/3.23 s** at 32 Rayon workers for seed 1, seed 2 and the
  seed-1 repeat;
- run RSS maxima: **228,344/228,096/227,932 KiB**;
- driver and chat binary SHA-256:
  `811da311338b477c62a9c0f628cf86b7c4e2737be1e37b6973b035b939b75d29`
  and
  `31f562fd07e1c2993e8363ee2ec3b1a54f291296b99a92e459bd5d65dc727c96`;
- committed deciding evidence: **49,502,554 bytes**; all four attempt trees
  retain **197,982,559 bytes**, below the registered 224 MiB limit;
- evidence `SHA256SUMS`:
  `15cd38a930e17025d4227568d1b4d1b48740ed6f266c682599cc85565a209262`.

Compact evidence:
[comparison.json](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/evidence/comparison.json).
The source pipeline occupied **49,502,554 bytes**.

## Checks run at the head

- Full `uor-r4-cpu-train` release suite: **246 passed, 4 expected ignores**,
  with all binary tests green.
- Full `uor-r4-integer` serving library suite: **463 total, 462 passed and 1
  expected ignore**.
- The attempt-4 control target ran **7** focused integer/session tests and
  **18** OpenAI-loopback tests, all passing.
- The built release CLI independently completed a prompt-ID smoke, a text
  one-shot and a one-turn REPL with the exact sidecar path and hash; a sidecar
  missing `includes_reference_end` was refused before any step.
- Focused tests cover sidecar strictness, BOS refusal, voting and fallback,
  raw-step isolation, complete CPU/serving parity, same-readout session
  restore, missing/different-readout refusal, legacy `UR4SESS1` and
  `UR4SESS2` codec identity.
- The post-run parser-refusal suite directly covers wrong schema, sign mode,
  empty rows, malformed and oversized thresholds, unknown readout/row fields,
  trailing input and the 64 MiB bound; the full integer suite is **463 total:
  462 passed and 1 expected ignore**.
- Strict `uor-r4-cpu-train` Clippy with `--no-deps -D warnings`, targeted
  `rustfmt --check` on every touched Rust file, and the source/docs-scoped
  `git diff --check` passed. The complete evidence diff retains the captured
  test outputs' leading/trailing blank lines.
- The integer crate's blanket all-warning Clippy still reports 51 pre-existing
  diagnostics outside this unit. Its no-dependency Clippy correctness gate
  passed, all-target compilation and the full serving library tests passed,
  and the failure list contains none of the new content-readout diagnostics.
- Both evidence manifests, the normalized repeat, generated JSON and
  `serving.uorlut`/sidecar files matched.
- Claim wording, docs catalog, roadmap schema/rendering, prep-check, D23 and
  D25 passed. The full harness check exited 0 with no errors and 160
  repository-wide warnings, with no warning attached to this unit.

## Skills used

No system-provided Codex skill applies. Independent FIT and GO reviews used
separate subagents.

## Still open

- Complete replies and a geometric termination rule.
- Fresh-phrasing and semantic-correctness panels.
- GPU parity and throughput for sidecar serving.
- Claude's P3 decode-cache unit and later end-to-end model evaluation.

## Plain summary

The geometric content table can now travel with an exact local model and drive
the real integer serving selector with byte-for-byte agreement to the CPU
trajectory. This removes the serving-integration gap for the positive content
readout. It does not solve completion or produce useful chat: the same sparse
content decisions and capped, inexact replies remain.
