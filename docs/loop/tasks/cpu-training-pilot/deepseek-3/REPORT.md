# REPORT: make public CPU corpus training restartable, deepseek-3

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `20422589a`. Tested head: `636f5e1ed`.

## What did not work, first

- **The earlier ranking negative is unchanged.** This unit adds no new training configuration and does not reinterpret the unchanged 278/384/396 and 224/285/292 panels as a capability gain.
- **The restart control does not close #3506.** The sealed complete-reply and genuinely fresh-phrasing inputs still require credentials and remain `NOT_RUN`. A configuration that clears a registered capability bar is also still missing.
- **The initial ORDER treated this as a pre-registered capability run.** That was over-broad: Charter §4.5 requires a FIT sheet for GPU training/evaluation and runs a milestone claim rests on, while exempting builds, finite tests and host checks. The draft sheet was removed and the ORDER was corrected to the host-determination scope before this REPORT; no approval is fabricated for the completed control.

## What changed

- Added an optional eleventh `cpu-corpus-driver` argument for a bound interval checkpoint while preserving the existing ten-argument fresh-run command.
- Kept `<steps>` as the total target: a step-64 checkpoint resumed with target 256 executes exactly 192 more attempts.
- Wrote a sidecar binding beside every interval and final checkpoint. It binds the source artifact, token and mask hashes, manifest hash, tokenizer, trainer seed, split controls, completed step and checkpoint hash.
- Refused a resume before continuation when any bound input, the step count or the checkpoint bytes differ.
- Extended the driver tests to compare an uninterrupted run with an interval-resumed run and to exercise source, token, manifest, seed, split and checkpoint-hash tampering.

## Pre-FIT diagnostic measurements

The initial six-process comparison ran before the pre-registration sheet reached
the test tree and before it had a fresh non-author FIT review. It is retained
under [evidence/prefit-diagnostic](evidence/prefit-diagnostic/) as an executable
instrument diagnostic only. Its byte equality does not satisfy the registered
post-FIT outcome and is not the evidence used to close the restart claim.

The same pinned public inputs from the earlier pilot were used: both raw dual-c131 artifacts, public tokens and mask, trainer seed `1`, evaluation stride `2`, cap `1024`, and checkpoint interval `64`. For each artifact seed, a separate process ran 256 uninterrupted attempts; another process stopped at 64 and exited; a third process resumed that bound checkpoint to total step 256.

Filesystem hashes from the six run receipts were compared directly. `resume-comparison.json` records the structured comparison.

| Artifact seed | Resumed from / executed | Uninterrupted final artifact | Resumed final artifact | Final checkpoint equal | Panel equal | Export exact |
|---|---:|---|---|---|---|---|
| 1 | 64 / 192 | `db58f4efb3f4279e9c06f1667e1b014b70edf62ba43f0faaf37869c04886e50c` | `db58f4efb3f4279e9c06f1667e1b014b70edf62ba43f0faaf37869c04886e50c` | yes: `d9e0f9d7…` | yes: `278/384/396` | yes |
| 2 | 64 / 192 | `6781fe807a1a1e47d386b50a78d7f0367e405da58c2cc5038b6b9967e0d23570` | `6781fe807a1a1e47d386b50a78d7f0367e405da58c2cc5038b6b9967e0d23570` | yes: `85235546…` | yes: `224/285/292` | yes |

- Seed 1 interval checkpoint: `1ba0d8febe534d6f933b150a45a48c1ca8a87cfc6ba1bd4036debb504690f314`; resume finished 24 accepted / 168 refused remaining steps at 4,760.043 context-tokens/s.
- Seed 2 interval checkpoint: `de9deb91e32e85cdcbbc07d45114ad73d6708a0b922a2ec0afeed26b4ccfa5bf`; resume finished 17 accepted / 175 refused remaining steps at 5,020.964 context-tokens/s.
- Every uninterrupted and resumed arm had zero CPU/serving mismatches over 262,144 query coordinates, 1,024 tokens and 8,192 kept cells.
- Binary SHA-256: `cc367b9f0bd38a1ccc39d70815a01f18c24885a3435ec5bacb436554228ca670`.
- Machine: AMD Ryzen 9 9950X, 16 cores / 32 threads; Rust/Cargo 1.97.1; Ubuntu kernel 7.0.0-42-generic.

Evidence and its verified SHA-256 inventory are in [evidence](evidence/).

### Decision

**The pre-FIT diagnostic was byte-identical on both seeds but is not a
registered pass.** It establishes that the six commands are executable and
produced no observed continuity difference. The public restart disposition
remains pending until the pre-registration is FIT and the six runs are repeated
against the reviewed source and binary.

The program-level usefulness verdict is now explicit:

- **Useful for development:** yes, as a fast deterministic mechanism and export testbed. The Ryzen 9 9950X performs about 4.8-5.1k context-tokens/s of rank-margin update work, and exact public parity/restart controls complete in minutes.
- **Useful as demonstrated language training:** no. The only pre-registered public training configuration produced zero top-1/top-8 gain on both seeds.
- **Ready for a qualification path:** no. GPU remains the qualification authority; complete replies and fresh phrasing are not graded, and no tested CPU configuration has cleared a capability bar.

## Checks run at the head

- Focused driver tests: 3/3, including uninterrupted-versus-resumed byte/panel equality and six binding tamper cases.
- Public process restart: six runs across two seeds, with `sha256sum` equality on both final artifacts and both final checkpoints.
- Evidence `sha256sum -c SHA256SUMS`: all committed evidence files verified.
- Full CPU crate suite at tested head `636f5e1ed`: 111 library tests and 3 driver tests passed; 4 release benchmarks ignored.
- Strict Clippy and formatting at tested head `636f5e1ed`: pass.

## Skills used

No repository skill was invoked. The work used the D84 CPU program, the existing bounded pilot, and the charter's host-check exemption.

## Still open

- The registered post-FIT restart comparison and its evidence receipt.
- Credentialed sealed complete-reply and genuinely fresh-phrasing grading.
- A new pre-registered configuration that clears a stated capability bar without admission or export loss.
- A genuine trained FBS float shadow, if a lawful snapshot becomes available.

## Plain summary

The pre-FIT diagnostic of the CPU trainer stopped after an authenticated
checkpoint and finished in a fresh process with exactly the same model bytes
and evaluation reading. That is a promising instrument diagnostic, not the
registered restart result. The post-FIT comparison remains outstanding, and
neither outcome changes the earlier zero-gain training result into capability.
