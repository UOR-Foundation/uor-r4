# [codex] The source, mixer and writer each lose information on the released trunks

On Wick's RTX 4090, both unchanged M1 models emitted token 1155 (`ĠG`) at answer position zero on all 16 diagnostic prompts. Their natural orbit pointer and the alternate pair-nearest identity-frame Pick both missed the asked first value token on all 16. Supplying the correct occurrence to the native pointer bonus still produced zero correct opening tokens at either seed. Every arm scored 0/16 complete exact replies under the registered immediate-EOS criterion. This is a fixed diagnostic on the released Relation checkpoints. Each has four blocks, context 192, eight kept cells and one Relation sweep over eight active leaves. No parameters changed. [Sheet](ROLE-SHEET.md) · [Prompt IDs and exact spans](panel.json) · [Human-readable prompts](panel-key.json)

## Where the answer is lost

1. **Selection.** In `distinct-direct-ann-first-ask-ann`, the prefix has 31 tokens. Red is ID 677 at offset 7, and blue is ID 924 at offset 12. Both selectors match the key at offset 3 and take its successor, ID 2241 (`ĠAnn`) at offset 4. The alternate pair match has sign-code distance 12. For the opposite question the read and recurrence codes change at seed 1 (9241 to 9401 and 29312 to 29320), but the chosen source and all ten generated IDs stay identical.
2. **Mixing.** For the Ann question seed 1, red is already in the sparse candidate pool at score 682. Forcing the correct occurrence through the existing pointer raises red to 938. Token 1155 still scores 59,826. Seed 2 gives the same target scores and a winner of 56,122. Across all 16 cases the winning score exceeds the correctly boosted opening token by 16,196–58,888 at seed 1 and 14,532–56,077 at seed 2. The target cell is excluded in all 16 opening decisions at both seeds. The score competition is the first observed downstream failure even after an oracle supplies the right occurrence.
3. **Identity at the writer.** Preserving native selection and mixing, then setting the pre-writer token to the correct ID, makes all 16 opening tokens correct at each seed. Only 12/16 complete value spans survive this intervention. At offset 1 in all four `red kite` cases the writer receives 1999 (`Ġkite`), admits its cell 9 and ranks [878, 1999, …]. It emits 878 (`ak`). A post-hoc static inspection of both artifacts found that these two IDs have the same cell and byte-identical 128-byte token vectors (SHA-256 `f6c6124e9d2dc248ce741dc76ff2954bf872cad2ea49fbe3940069bea7e5e8b3`). The distance tie always goes to the lower ID. Across all 4,096 IDs per model, a census of the exact (token-vector bytes, cell) pairs gives 4,008 unique pairs, 87 collision classes and a maximum multiplicity of three. The fixed ranking therefore excludes at least 88 IDs from greedy emission at each seed. This is a vocabulary-type result. Token frequencies and resulting corpus error were not measured.
4. **Stopping.** Forcing the entire correct span as emitted tokens and then releasing the model gives 12 replies with the right value followed by period/EOS and four with the right value followed by exclamation/EOS, at each seed. All 16 retain the forced value. All 16 miss the registered next token, immediate EOS. On these cases after the supplied value the remaining error is punctuation policy.

## Arm outcomes

The numbers below are counts out of 16. “Opening” and “value” refer to fed output IDs, including deliberate controls. Finished means the model eventually emitted EOS within the 16-token cap. Slash entries give seed 1 / seed 2. Roles with `red fox` and `red kite` intentionally share an opening ID; first-token agreement alone cannot establish correct role selection.

| Intervention | Correct opening | Whole correct value | Exact reply | Finished |
|---|---:|---:|---:|---:|
| Native | 0 | 0 | 0 | 16 / 15 |
| Identity-frame Pick candidate with native +256 | 0 | 0 | 0 | 16 / 14 |
| Asked occurrence through native +256 | 0 | 0 | 0 | 16 / 15 |
| Other occurrence through native +256 | 0 | 0 | 0 | 16 / 15 |
| Asked source ID at pre-writer boundary | 16 | 12 | 0 | 16 |
| Other source ID at pre-writer boundary | 8 | 0 | 0 | 16 |
| Correct span fed, then free | 16 | 16 | 0 | 16 |
| Supplemental: native pointer removed | 0 | 0 | 0 | 16 / 15 |
| Supplemental: first asked ID fed, then free | 16 | 8 | 0 | 16 |
| Supplemental: other span fed, then free | 8 | 0 | 0 | 16 |

The Pick intervention substitutes the selected successor identity in the native mixer. The upstream Relation read still computes its original vector. The direct writer arms preserve the natural pointer and the mixed choice and change only which token is passed into the geometric writer. Every rollout follows its own fed prefix. The forced-token receipts distinguish what the model predicted from what the experiment supplied. After the controlled span, the pointer, writer and emission all resume ordinary operation.

## Checks and evidence

Two independent selector calculations (CUDA root L1/2 and host sign-code XOR/popcount, plus the native orbit selection) agreed on 1,167/1,108 steps. The ordinary Rust serving path reproduced all native-mode read vectors, candidates, IDs and loss words on 713/674 steps, including steps after interventions expired. On unchanged native prefixes the original untraced CUDA path with changed target labels reproduced all decisions on 179/166 steps. Reloading seed 1 in a fresh process reproduced its entire original JSON byte for byte. A fixture changed by one whitespace byte was refused with exit 1 before scoring and produced no receipt.

The pinned 4090 reported driver 595.99.02 and 20,522 MiB free before the run. Measurements were collected 15:51:21–15:51:25 UTC on November 18, 2026. The first processes took 1.16/1.13 s elapsed and peaked at 1,045,036/1,028,028 KiB RSS. These timings include file loading, copying and verification. No training or inference speed benchmark was performed. The separately recorded seed-1 replay used the same binary. The test command from the run root was:

```sh
target/x86_64-unknown-linux-gnu/release/d11-scorer role-swap inputs/seed1.d11 inputs/panel.json results/role-01/seed1.json
```

[Seed 1 full-step receipt](role/seed1.json) · [Seed 2 full-step receipt](role/seed2.json). The two committed files dictionary-encode repeated trace objects. With the ordered panel and the output IDs, each step reconstructs its entire input prefix. Each row carries the earliest wrong ID offset or null. Expand the dictionary for the original data using `jq '.step_dictionary as $d | del(.step_dictionary) | .rows |= map(.steps |= map($d[.]))'`. Both expanded data sets were compared in full with canonical-key JSON from the raw receipts. Binary SHA-256 `60a586901368641d8096388c58b2002b999b20ac04c0df49718c2c741038daaa`. Raw seed 1 and its replay SHA-256 `0e9de96a495c97450ffd03f6f05caf57d73c8d181f1a3abb3e8565d42a01ad70`; raw seed 2 SHA-256 `57b4eb0a35a468d61f71c5f302598f560155b643189b187c1745d608d4446d20`. The receipts bind the CUDA cubin, PTX, artifact and panel hashes. Tokenizer checksums are in the key/sheet and run input manifest.

**Implication.** A correct next experiment must isolate learned source choice, source influence through the specific mixer and exact token identity through a multi-token copy. Increasing the existing scalar bonus alone leaves the `kite` same-cell/identical-vector counterexample intact. Preserve the source occurrence and exact ID through a controlled copy action, advance a causal span cursor, and test with role swaps and the action disabled before attributing any learned recall. These fixed examples qualify no M2 panel, P3 training result, S0–S6 artifact or general prose capability.
