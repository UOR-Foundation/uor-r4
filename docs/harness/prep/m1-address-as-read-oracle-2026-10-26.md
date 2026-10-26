# DRAFT M1 object-store oracle: exact read and five matched arms

- **Item:** `m1-address-as-read` (#2833). **Unit:** kimi-42, `References` only. Design: `docs/labs/m1-address-as-read/integer-read-successor.md`.
- **Run:** zero training. One RTX 5090, one 14,287-row D11 parity repeat plus an 80-row oracle instrument, estimated <15 min and about $0.30 at current offers; no retry without a new instrument classification. New output/HF path; preserve kimi-40's evidence.
- **Artifact:** T16_fz Seed 1 D11 `trunk-model.d11`, SHA-256 `cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd`; original checkpoint lineage `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4`.
- **Original GPU receipt:** `final-dev.json`, SHA-256 `a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e`; Stage T archive `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`.
- **Tokenizer:** `tokenizer.json`, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **DEV:** tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; response mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 14,287 rows.
- **TRAIN:** tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; response mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **Config lineage:** original `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d`; staged `43a316bc67bb45fe5b50bd2ceee57f6b919164c02fe45ee9e677f8819fea5d15`; actual normalized hash recorded at runtime.
- **Oracle manifest:** `docs/labs/m1-address-as-read/address-oracle.json`, schema `uor-r4.address-oracle-panel/1`; SHA-256 **TO_PIN_AFTER_GENERATION**. 40 current, 20 previous, 20 abstention; versioned sieve lexicon and limits. FIT is not claimed while this is unpinned.

## Question

Can the delivered `AddressDirectory` and existing D11 GPU scorer measure the contribution of a true exact address over deterministic sieve, no-address and no-memory controls, without confusing hard store emission with model generation?

## Arms

1. **No-memory:** store disabled; model-only baseline (L-010).
2. **No-address:** store populated but no address supplied; model-only baseline must be byte-identical to no-memory.
3. **Sieve:** deterministic prompt-to-`PrimeAddress` query from the versioned lexicon: split words, retain lexicon words in order, take primes, pair adjacent kept words. Use `read_sieve`/`read_sieve_from`. Record `sieve_address_match` separately from `sieve_value_check_pass`.
4. **True-address:** exact `read_exact`/`read_exact_from` with the registered key and `Current`/`Previous` selection. A hit emits verified content marked `ExactStore`; this is oracle assistance, not learned model copy.
5. **Random-address:** a present, exactly unequal key bound to a different value; planted-wrong control, never an accuracy arm.

The sieve selects current values only. Previous-version sieve output is descriptive and excluded from true-versus-sieve. Previous rows remain in true-versus-no-memory.

## Bars and outcomes

- Parity: all 14,287 rows status 0, prediction/top-8/admitted-count agreement with the sealed receipt, aggregate `455 / 1,484 / 5,009`, all position-strip deltas zero, and byte-identical repeat records.
- True-address hard emission: exactly **40/40 current, 20/20 previous, 20/20 typed abstention**, zero wrong-value substitutions.
- No-memory/no-address byte identity holds; random-address planted-wrong rows remain wrong; every expected value is absent from TRAIN/DEV and all model-visible panel text.
- Report true-versus-sieve, true-versus-no-memory and sieve-versus-no-memory `b`, `c`, `d` on their supported populations. Do not pool families. No BPB.
- **Instrument failure:** wrong substitution, baseline drift, missing control, leak, nondeterminism or emission error. **Ceiling:** correct store emission but model/sieve already exact; no gain is measurable and no training follows. **Address next:** correct exact oracle with poor sieve formation localizes the next question to addressing. No hidden-fact panel in this unit.

## Instrument checks and identity

Pure tests cover exact current/previous, overlapping-but-unequal active/evicted keys, invalid primes/pairs/token bindings, missing previous, absent/evicted, deterministic manifest and bounded limits. All model evaluation, including fallback, uses the GPU scorer; no host `evaluate_prefix`.

Bind source HEAD, executable/PTX/cubin/geometry/kernel/driver identities, artifact/checkpoint/tokenizer/corpus/config/panel hashes, CUDA toolkit and driver API version, and display-driver version from `nvidia-smi` or UNVERIFIED. Record per-arm rows/outcomes, slots/fetches/candidates/value bytes, resident bytes, decode/context tokens/s and complete pod uptime/cost. No energy claim.

## Stop rules

Any artifact/input/schema/hash/shape mismatch, nonzero parity status, nondeterminism, corpus or cross-row leak, missing baseline identity, unauthorized fallback or wrong substitution stops as an instrument fault. A failed hard-emission gate ends this configuration with no hidden-fact panel or training. Do not change thresholds after FIT.

## TEST FITNESS line

TEST FITNESS: NOT_FIT — manifest generation and SHA-256 pin are pending; fresh non-author review is required after those inputs exist.
