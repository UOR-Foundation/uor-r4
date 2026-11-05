# ORDER: real-text-corpus, claude-292

Lab: claude. Item: [real-text-corpus](../../../../roadmap/items/real-text-corpus.md). Base: `5bd24915e`.

## Goal

The item's done-when reads:

> 1. **The builders,** in `real_text.rs` and one bin, `prepare-real-text`, with tests and negative gates:
>    - OASST2's tree to conversations, with each filter pinned by a test;
>    - Wikipedia's parquet to BOS-opened documents;
>    - the split by source id;
>    - the tokenizer pin refused on a wrong SHA;
>    - the exclusion rule, with tests that pin an 8-word hit, a 4-to-7-word turn matched whole, and a shorter turn counted as uncovered.
>
>    The training crate's tests pass.
> 2. **The corpus and its panel,** built on the laptop (data preparation, not training; see Rules that apply).
>    - **Each manifest records** the source, revision, licence and attribution; the counts of trees, threads, articles and tokens; the rows dropped, and why; and bytes per token.
> 3. **The census against the frozen test panels** reads no shared prompt, with every excluded document counted and every uncovered short turn counted.
> 4. **Storage:** in the private store under `results/<lab>/real-text-corpus-<date>/`, with a `SHA256SUMS`, read back. No corpus is committed.
> 5. **One notebook row in `data-tokenizer.md`, and a REPORT with the negatives first.** P5's sheet unit then moves P5's Data line to real text, through a decision card.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-training/src/lib.rs`
- `crates/uor-r4-training/src/real_text.rs`
- `crates/uor-r4-training/src/bin/prepare-real-text.rs`
- `crates/uor-r4-training/tests/real_text*.rs`
- `docs/harness/prep/real-text-corpus*`
- `docs/roadmap/items/real-text-corpus.md`
- `docs/loop/tasks/real-text-corpus/**`
- `docs/research/notebook/data-tokenizer.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
