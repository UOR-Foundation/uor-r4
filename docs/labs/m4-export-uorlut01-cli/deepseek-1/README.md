# Explicit system-memory serving bridge

> Historical run: review-driven testing changed the executable after this
> evidence was collected. The preserved first run is not authoritative; the
> FIT-reviewed rerun under
> `local/deepseek/m4-memory-bridge-2026-11-02-rerun/` replaces it.

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../../roadmap/items/m4-export-uorlut01-cli.md).
Order:
[deepseek-1](../../../loop/tasks/m4-export-uorlut01-cli/deepseek-1/ORDER.md).
Sheet:
[FIT memory bridge](../../../harness/prep/m4-export-uorlut01-cli-memory-bridge-2026-11-02.md).

## Result

The explicit system-memory line reached the model and changed the output ids in
all six seeded memory-on/off comparisons. It did **not** produce either
requested value: no reply contained `Pickle` or `tomato`.

The result is `MEMORY_LINE_CONSUMED`, not recall. On each seed, all three
memory-on prompts produced the same reply as one another, including the
`Memory: none.` absence control. The line changes the model's state; the model
does not yet use it as a selective answer. Automatic statement extraction has
not been implemented.

## Instrument

- 14/14 processes exited 0; all stderr files are empty.
- All 14 records use `uor-r4.literal-role-dialogue/2`.
- Stops: 11 `eos`, 3 `max_new_tokens`.
- Reply text independently decoded from reply ids in 14/14 cases.
- The two seed-1 memory-on repeats match the first pass in text, ids, stop,
  history and memory line. Only timing differs.
- Every on/off pair has different reply ids.
- Peak RSS: 11,612 KiB. Sum of `/usr/bin/time` elapsed values: 1.52 s.
- Raw evidence: 44 files, 32,964 bytes, under
  `local/deepseek/m4-memory-bridge-2026-11-02/`.
- Sorted SHA-256 manifest digest:
  `35e1f521ccc57cf250a288429e3c7a2e1c1ad1dc73e0e92382698cf988d868e7`.
- The raw `status.tsv` header labels its second column `au`; its values are the
  `off`/`on` arm names. The file is retained unchanged, and the transcripts
  below are the authoritative arm mapping.

## Transcripts

### Seed 1

| Arm | Prompt | Memory | Reply text | Reply ids | Stop | History | Seconds |
|---|---|---|---|---|---|---:|---:|
| off | `recall-name` | null | `small Pretty good, I M coat.` | `563,971,623,576,561,14,340,379,581,299,16,1` | `eos` | 30 | 0.049884655 |
| on | `recall-name` | `Memory: Pickle.` | `small Pur I G's greatice'm good,uc cobweb!.` | `563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1` | `eos` | 48 | 0.063372757 |
| off | `recall-grow` | null | `small Pretty good, I M coat.` | `563,971,623,576,561,14,340,379,581,299,16,1` | `eos` | 27 | 0.034826156 |
| on | `recall-grow` | `Memory: tomatoes.` | `small Pur I G's greatice'm good,uc cobweb!.` | `563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1` | `eos` | 44 | 0.057579743 |
| off | `absent-breakfast` | null | `small Pretty good, I M coat.` | `563,971,623,576,561,14,340,379,581,299,16,1` | `eos` | 35 | 0.045633145 |
| on | `absent-breakfast` | `Memory: none.` | `small Pur I G's greatice'm good,uc cobweb!.` | `563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1` | `eos` | 52 | 0.069063646 |

### Seed 2

| Arm | Prompt | Memory | Reply text | Reply ids | Stop | History | Seconds |
|---|---|---|---|---|---|---:|---:|
| off | `recall-name` | null | `Your G me legs.` | `2997,1155,519,2295,16,1` | `eos` | 24 | 0.030394925 |
| on | `recall-name` | `Memory: Pickle.` | `Your G me have all oflkreonddot in am you'm` | `2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915` | `max_new_tokens` | 48 | 0.065018688 |
| off | `recall-grow` | null | `Your G me legs.` | `2997,1155,519,2295,16,1` | `eos` | 21 | 0.026366648 |
| on | `recall-grow` | `Memory: tomatoes.` | `Your G me have all oflkreonddot in am you'm` | `2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915` | `max_new_tokens` | 44 | 0.059539019 |
| off | `absent-breakfast` | null | `Your G me legs.` | `2997,1155,519,2295,16,1` | `eos` | 29 | 0.037964676 |
| on | `absent-breakfast` | `Memory: none.` | `Your G me have all oflkreonddot in am you'm` | `2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915` | `max_new_tokens` | 52 | 0.070643732 |

### Seed 1 repeat

| Arm | Prompt | Memory | Reply text | Reply ids | Stop | History | Seconds |
|---|---|---|---|---|---|---:|---:|
| on | `recall-name` | `Memory: Pickle.` | `small Pur I G's greatice'm good,uc cobweb!.` | `563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1` | `eos` | 48 | 0.063410138 |
| on | `recall-grow` | `Memory: tomatoes.` | `small Pur I G's greatice'm good,uc cobweb!.` | `563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1` | `eos` | 44 | 0.060156761 |

## Decision

| Seed | Name changed from off | Grow changed from off | `Pickle` present | `tomato(s)` present |
|---|---|---|---|---|
| 1 | yes | yes | no | no |
| 2 | yes | yes | no | no |

The serving bridge is connected and deterministic. This configuration does not
demonstrate recall, and the fixed public artifact remains unsuitable for useful
chat.

## Audit scope

The host is x86_64. The D11 zero-multiplier audit is specified for ARM64 and
its stack-mode scalar multiply/divide exemption recognizes ARM register syntax;
the release binary
`b1be73742c75d429ef4690334393a36fc95da1bc4b4d4de21a3521f06986c487`
reports three flagged symbols containing four existing scalar `mul`
instructions; the debug binary used for the product run reports ten flagged
symbols containing 23 such instructions. Both also report two missing mandatory
symbols, and the non-TAP call-graph path cannot run because `otool` is
unavailable. The audit self-test passes, but no full native or WebAssembly audit
claim is made for this x86 run.
