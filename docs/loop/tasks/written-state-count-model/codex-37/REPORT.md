# [codex] Reserved-TRAIN refit using the deployed full-C compact table

## What failed and what remains open

An intentionally altered in-range mass on the finite tape failed at original ordinal 0. The correct value was 4,143,392,400 Q48 and the changed value was 4,143,392,257. All 22 count and total words still agreed there. This checks that a plausible mass alone cannot pass the independent reader. [First failing row](replay/evidence/neg.log)

The captured features contain the target-dependent counts and probability mass for grading. The geometric Pick successor must still be chosen from the preceding prefix before comparison with the target. The first capture establishes the input to that join. It supplies no first-generated-token or through-EOS reply score.

## Reserved-TRAIN count result

The two fresh Wick RTX 4090 processes each read the same physical threshold-318 artifact once. Both emitted all 15,947,584 non-BOS targets, including each of the 4,060 EOS targets, in the original record order. Their entire 2,232,661,872-byte outputs matched. SHA-256 `add9aa306fd70d87e02048e2a2a32d23f6f9e9be89bb8ff8011c64690042f4dd`. The original UORT, compact table and full-C digest are bound in the feature header. [Pre-registration](count/SHEET.md) · [Independent FIT](count/FIT.md) · [Launch decision](count/GO.json)

| Verification | Measured result |
| --- | --- |
| Primary batch width 65,536 | 244 launches, 1 upload, 14.57 s wall, 239,732 KiB peak host RSS |
| Cold batch width 65,521 | 244 launches, 1 upload, 14.07 s wall, 240,056 KiB peak host RSS |
| Independent native reader | 12,500 selected ordinals, 287,500 exact words, 9.50 s wall |

The independent comparison covered both ends of every record, both sides of every interior primary batch boundary, and an interior sample specified before dispatch. Every tape row also passed framing, count/total and Q48 range checks. The two GPU traversals give whole-stream reproducibility under shifted batching. Native numerical parity was measured on the declared sample. No DEV split was used. [Native receipt](count/evidence/native.json) · [Output manifest](count/evidence/capture.sha256)

## Native classes and finite GPU optimizer

The three-class and distance-split carriers store 7,392 and 12,320 Q32 words respectively at order ten. The legacy format keeps its original bytes. The five-class format identifies the split on load. Under the ordinary client's identity-code Pick reader, the D37 manifest must name the identity preparation and the arm matching its carrier. A permuted-code control, exact-match control, disabled control or swapped arm label is rejected on that path. The targeted serving test exercised those cases on Wick with `p1-count-serving` enabled. Two earlier invocations without that feature selected zero tests. [Manifest and reader](../../../../../crates/uor-r4-integer/src/discrete_count.rs)

On the 128-row invented fixture the 4090 matched all 97 objective sums and all 717,024 legacy and 1,195,040 split Q32 weight words across optimizer steps. The fixture activates buckets 527 and 879. The truncated row tape was rejected before a trace was created. [Finite GPU parity](fit/evidence/finite/parity.txt) · [Registered fit](fit/FIT-FINITE.md)
