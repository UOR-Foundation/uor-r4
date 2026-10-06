# Input-derived physical Source reference

The geometric potential attribution records every admitted occurrence and its
raw score, but target-token membership alone cannot identify the correct record.
The preserved 512-row prose panel lacks its historical answerability receipt.
The opt-in `--reference-only` mode in `geometric-bank-generate-panel` derives a
fresh evaluation reference from the authenticated inputs instead of inventing
historical provenance or supplying a selected record to inference.

```text
geometric-bank-generate-panel --reference-only \
  --input <sealed-root-containing-inputs-and-labels> \
  --tokenizer <bound-tokenizer.json> --out <new-reference-root>
```

Selection consumes raw query/cue bytes and validates the existing Source
namespace/entity/relation contract. It chooses the last matching physical
segment: event IDs remain opaque provenance and may repeat or decrease. Only
after selection does it require membership of the derived prose in the existing
Current answer labels. A stale-answer label fails the instrument; it cannot
redirect selection. The default prose author retains its historical
maximum-event behavior; this opt-in mode makes its separate chronology explicit.

The sealed output contains only the reference, report and manifest. It binds
the input manifest, input bytes, labels, tokenizer and executable. Each row
preserves segment/event/record/commit, independently recompiled emission views,
and canonical response token byte intervals. Original-token and emission-view
offsets remain distinct. Seam-crossing tokens are marked, not silently treated
as complete literal occurrences. No model is loaded, no prediction is generated,
and neither inputs nor labels are rewritten.

This bounded authoring rule supports the declared current-job/current-residence
grammar and rejects multiple namespaces/entities or unsupported queries. It is
not a general language parser or runtime attention mechanism. The existing
construction panel has two competing roles but no same-relation version pairs;
same-value reassertion tests protect provenance only and do not establish that
answer-token loss learns occurrence identity. Exposure remains unchanged and no
held-out claim follows from deriving this receipt.

Validation is pending at authoring. Focused checks cover answer-label steering,
byte intervals, repeated-value physical occurrences, opaque events and legacy
default preservation. The actual 512-row derivation must also pass before this
reference can support post-forward source-score contrasts. Existing fit and
four-control native predictions remain separate evidence.

References [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and
[PR #1792](https://github.com/UOR-Foundation/uor-r4/pull/1792).
