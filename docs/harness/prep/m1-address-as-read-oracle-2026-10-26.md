# M1 object-store oracle: exact read and five matched arms

- **Item:** `m1-address-as-read` (#2833). **Unit:** kimi-42, `References` only. Design: `docs/labs/m1-address-as-read/integer-read-successor.md`.
- **Run:** zero training. One RTX 5090, one 14,287-row D11 parity repeat plus an 80-row oracle instrument, estimated <15 min and about $0.30 at current offers; no retry without a new instrument classification. New output/HF path; preserve kimi-40's evidence.
- **Artifact:** T16_fz Seed 1 D11 `trunk-model.d11`, SHA-256 `cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd`; original checkpoint lineage `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4`.
- **Original GPU receipt:** `final-dev.json`, SHA-256 `a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e`; Stage T archive `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`.
- **Tokenizer:** `tokenizer.json`, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **DEV:** tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; response mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 14,287 rows.
- **TRAIN:** tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; response mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **Config lineage:** original `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d`; staged `43a316bc67bb45fe5b50bd2ceee57f6b919164c02fe45ee9e677f8819fea5d15`; actual normalized hash recorded at runtime.
- **Oracle manifest:** `docs/labs/m1-address-as-read/address-oracle.json`, schema `uor-r4.address-oracle-panel/1`; SHA-256 `a4aa548f3e38478fdc8cd4ad077d9886cefa3aaa8e2712abe8f38abe0dba4103`. 40 current, 20 previous, 20 abstention = 10 absent + 10 evicted; versioned sieve lexicon and limits; repeated generation is byte-identical.

## Question

Can the delivered `AddressDirectory` and existing D11 GPU scorer measure the contribution of a true exact address over deterministic sieve, no-address and no-memory controls, without confusing hard store emission with model generation?

## Arms

1. **No-memory:** store disabled; model-only baseline (L-010).
2. **No-address:** store populated but no address supplied; model-only baseline must be byte-identical to no-memory.
3. **Sieve:** deterministic prompt-to-`PrimeAddress` query from the versioned lexicon: split words, retain lexicon words in order, take primes, pair adjacent kept words. Use `read_sieve`/`read_sieve_from`. Record `sieve_query_address_match` (formed query equals true address) separately from `sieve_selected_address_match` (selected slot equals true address) and `sieve_value_check_pass`; never substitute selected-slot equality for address formation.
4. **True-address:** exact `read_exact`/`read_exact_from` with the registered key and `Current`/`Previous` selection. A hit emits verified content marked `ExactStore`; this is oracle assistance, not learned model copy.
5. **Random-address:** a present, exactly unequal key bound to a different value; planted-wrong control, never an accuracy arm.

The sieve selects current values only. Previous-version sieve output is descriptive and excluded from true-versus-sieve and sieve-versus-no-memory. Previous rows remain in true-versus-no-memory. Every contrast is reported separately for Current, Previous and Abstention; no pooled count or win is allowed.

## Bars and outcomes

- Parity: join by traversal position and require target-position/target-token, prediction, top-8 and admitted-count agreement on all 14,287 rows, status 0, aggregate `455 / 1,484 / 5,009`, all position-strip deltas zero, and byte-identical repeat records. The sealed receipt has no ordinal field.
- True-address hard emission: exactly **40/40 current, 20/20 previous, 20/20 typed abstention = 10 absent + 10 evicted**, zero wrong-value substitutions.
- No-memory/no-address byte identity holds; random-address planted-wrong rows remain wrong; every expected value is absent from TRAIN/DEV and all model-visible panel text.
- Report each family-specific true-versus-sieve, true-versus-no-memory and sieve-versus-no-memory `b`, `c`, `d` on its supported population. Previous is excluded from sieve contrasts. No BPB.
- **Instrument failure:** wrong substitution, baseline drift, missing/vacuous control, leak, nondeterminism or emission error. **Ceiling:** per family and contrast, the hard gate passes and the control is already exact on every row of that family; no gain is measurable and no training follows. **Gain:** hard gate passes and treatment `d > 0` for that family. **No-gain:** hard gate passes but neither ceiling nor gain holds; report explicitly and do not train. **Address next:** only a true-address gain with poor sieve query formation localizes the next question to addressing. No hidden-fact panel in this unit; every negative is configuration-scoped (D22).

## Instrument checks and identity

Pure tests cover exact current/previous, overlapping-but-unequal active/evicted keys, invalid primes/pairs/token bindings, missing previous, absent/evicted, deterministic manifest and bounded limits. All model evaluation, including fallback, uses the GPU scorer; no host `evaluate_prefix`.

One composed gate must require parity, leak-free inputs, complete identity, no-memory/no-address identity, random control and exact hard gates; a caller cannot pass on the true-address subset alone.

Bind source HEAD, executable/PTX/cubin/geometry/kernel/driver identities, artifact/checkpoint/tokenizer/TRAIN/DEV/config/panel hashes, CUDA toolkit and driver API version, and display-driver version from `nvidia-smi` or UNVERIFIED. Explicitly include TRAIN token/mask and panel hashes in identity. Record per-arm rows/outcomes, per-row slots/fetches/candidates/value bytes, resident bytes, decode/context tokens/s and complete pod uptime/cost. No energy claim.

## Cost and storage

Host preparation/build/review is lab time only. One pod at the live offer is estimated <15 min and about $0.30, with a $0.50 projected spend ceiling for this single dispatch; no retry without a new instrument classification. Output reports and row records are bounded to <64 MiB. Preserve the new tar and SHA256SUMS in HF before release/down. Record create/bootstrap/build/run/upload/retry wall time separately, actual create-to-down cost, local temporary/retained storage and available disk. No energy claim.

## Stop rules

Any artifact/input/schema/hash/shape mismatch, nonzero parity status, nondeterminism, corpus or cross-row leak, missing/vacuous control or baseline identity, unauthorized fallback or wrong substitution stops as an instrument fault. A failed hard-emission or composed gate ends this configuration with no hidden-fact panel or training. Do not change thresholds after FIT.

## TEST FITNESS line

TEST FITNESS: PENDING — the manifest is generated and pinned; fresh non-author review is required before dispatch.
