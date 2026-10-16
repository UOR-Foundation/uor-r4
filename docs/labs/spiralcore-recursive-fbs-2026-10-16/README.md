# SpiralCore "Recursive FBS Engine" — independent specification review and replayable conformance/measurement test

DeepSeek lab (`deepseek/spiralcore-fbs-replay-20261016`, owner request in the 16 October 2026 DSH
session on the owner's checkout) · 16 October 2026.

**Reviewed object.** `SpiralCore_Recursive_FBS_Engine_Source.zip`
(SHA-256 `08562de311fe1c4d0e93512337919766a46aec70fbff48bff402acf7bea45c58`) containing
`SpiralCore_Recursive_FBS_Engine.tex`, `..._QA.json` and `README.txt`, and the delivered PDF
(`SpiralCore_Recursive_FBS_Engine.pdf`, SHA-256
`9e43a27cc774aa583228d58eb7a0adbb0ac2328084c994197babfe8654f87841`, 6 pages, 312,425 bytes).
Paper: *SpiralCore Recursive FBS Engine — Exact Transformation Rules, Recovery Invariants, and
Embedded Replay Manifests*, SpiralCore research notes, 15 October 2026, v1.0. Staged copies under
`local/spiralcore-recursive-fbs-2026-10-16/` in the owner checkout (gitignored).

**Claims under test.** The developer's verbal claim to the owner: this mechanism proves *"full
compression and inflation of data over a transferred packet, along with full fidelity of knowing
the structure of the data with an attention mechanism"*, and it is *"a complete mechanism"*. The
paper states a narrower result.

**What ran.** Local CPU only, no model compute, no GPU, no pod, no rental, no training and no hold
release (D24 non-model work: source, document and codec testing). A new review crate
[`crates/uor-r4-spiralcore-fbs-replay`](../../../crates/uor-r4-spiralcore-fbs-replay)
implements the paper's Sections 1–7 and Appendix A from the specification text; 20 conformance
tests plus two measurement receipts were produced. New GPU cost \$0.

## 0. Verdict

| # | Claim | Verdict | Decisive evidence |
|---|---|---|---|
| 1 | The specified mechanism is complete and re-implementable from the paper alone | **SUPPORTED** | Implemented from the text with no reference code; every equation, worked example and boundary convention reproduces (§3) |
| 2 | Exact recovery `D(E(x)) = x` ("inflation"), including boundary conventions | **SUPPORTED** | Exact on a 45-point size × content matrix, at 0/1/2/82/83/164/165 bytes, for nesting `n` = 0…16 and absorption `k` = 3 (§3.2–3.4) |
| 3 | Section 6 nested literal tails recover every layer; Section 7 anchor+manifest recursion | **SUPPORTED** (decodability) | `D(C16(P0)) = D(P0) = x` and `sigma^-1(D(P_{k+1})) = (P_k, M_k)` verified exactly; §7 had not been run before (§3.4) |
| 4 | Counting laws `N = Σ 3^ℓ(r)`, `\|R_N\| = s_3(N)`, `W(N) = (N − s_3(N))/2` | **SUPPORTED — and content-free** | Verified for N = 1…2999 and the level histogram equals the base-3 digits of N (§3.1). These restate **Legendre's formula / Kummer's theorem**; no compression follows ([Legendre](https://en.wikipedia.org/wiki/Legendre%27s_formula), [Kummer](https://en.wikipedia.org/wiki/Kummer%27s_theorem)) |
| 5 | "Full compression … over a transferred packet" | **REFUTED BY MEASUREMENT** as stated | `\|E(x)\| = 1.06 × \|x\|` on every realistic corpus (15 corpora), while the packet carries ≥ 99.9 % of the payload **verbatim**; gzip/xz reduce the same bytes 2.7–5× (§4) |
| 6 | "Full fidelity of knowing the structure … with an attention mechanism" | **NOT SUPPORTED BY THE ARTIFACT** | The words *attention*, *compress*, *compression*, *entropy*, *inflate*, *transfer*, *network*, *telecom*, *standard* occur **0 times** in the paper; there is no query/key/value, no learned parameter, no similarity and no pooling. Structure detection is exact-subtree interning, bounded by `period/gcd(82, period)` (§4.3) |
| 7 | "A complete mechanism" | **PARTIALLY SUPPORTED** | Complete *as a lossless re-encoder* with a correct proof; the compression and attention halves of the claim are absent from the artifact (§8) |

**One-line summary.** The artifact is a correctly specified, exactly invertible
**hash-consed grammar codec over a fixed ternary (base-3 counter) parse**, serialised with an
LEB128-style varint and a bespoke magic. It is real, it round-trips, and it is close to the paper's
own careful wording — but it is **not** a compressor and contains **no** attention mechanism. The
paper itself says so: *"The invariant is exact recovery, not a fixed encoded size"* and *"the tested
whole-packet passes grew"*.

## 1. What the artifact actually is

The paper's mechanism, restated exactly:

1. Split `x` into atoms of `q = 82` bytes (eq. 1.1–1.2).
2. Intern node definitions by **exact definition equality**: `Leaf(a) = (0,a)`, `Parent(u,v,w) =
   (1,u,v,w)`; a parent binds three children of **equal level**; level(leaf) = 0, level(parent) =
   h+1 (eq. 2.1–2.6).
3. Feed atoms into an ordered frontier and repeatedly rewrite a **trailing** equal-level triple into
   its parent (eq. 3.1–3.4). This is base-3 carry normalisation.
4. Packet `E(x) = ((1,F1),(2,F2),(N,F3), T)` — three pinned frontier codes plus the dictionary
   `T` (eq. 4.1–4.3).
5. Inverse `D(P) = Expand_T(r_1) ‖ … ‖ Expand_T(r_s)`, with `D(E(x)) = x` (eq. 5.1–5.5).
6. Section 6: nest by encoding each level's *dictionary* as the next level's payload; §7: encode
   `sigma(anchor, manifest)` together.

Two mathematical clarifications this review adds (both verified, neither contradicts the paper):

- The frontier is a base-3 numeral written **most-significant-first**: levels are *non-increasing*
  left to right, and the multiplicity of level `h` is the digit `d_h`. (N = 4 → `[level 1, level 0]`.)
- The paper's "final atom count" and "root-level inventory" checks are **the same check**: a level-h
  subtree expands to exactly `3^h` leaves, so `Σ 3^ℓ(r)` *is* the leaf count. The invariant is
  therefore one fact, stated twice (test `root_inventory_and_atom_count_are_the_same_check`).

What it is **not**: there is no entropy coder, no probability model, no learned parameter, no
attention, no error detection, no negotiation and no shared-state protocol in the paper.

## 2. The instrument (replayable)

`crates/uor-r4-spiralcore-fbs-replay` — Rust, no dependencies, `#![forbid(unsafe_code)]`, integer
and byte arithmetic only (no floating point in the codec path; f64 appears only in printed ratio and
entropy statistics).

- `src/lib.rs` — Section 1–7 + Appendix A: `V(n)`, `Code(R)`, `G(f)`, `tau(T)`, encoder with
  interning and carries, wire writer, canonical parser with the reference checks, `Expand`,
  `nest` (C_n), `sigma`/`absorb` (§7).
- `src/sha256.rs` — SHA-256 (FIPS 180-4) with published test vectors, so receipts bind the same
  digest space the developer's manifest uses.
- `src/corpus.rs` — deterministic corpora (no OS randomness).
- `src/bin/replay.rs` — measurement harness: input size, `\|E(x)\|\), dictionary size, **verbatim
  payload bytes**, leaf/parent interning statistics, carries, round-trip verdict, 16-step nested
  size, absorption chain, `gzip -9`/`xz -9`/`zstd -19` baselines, order-0 entropy bound, and
  SHA-256 of input and packet.
- `tests/conformance.rs` — 20 tests, each named for the paper section it checks.

Gates run: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
(20/20 pass). One instrument defect was found and fixed during this review: the first version of the
external-compressor helper wrote the whole input from the calling thread while the child blocked on a
full stdout pipe, which deadlocks once the compressed form exceeds the pipe buffer. It hung the
multi-megabyte run; the helper now feeds stdin from its own thread. All reported compressor numbers
come from the fixed instrument; the 64 KiB set completed under the buggy version too, and its
numbers are unchanged under the fix.

Reproduce:

```sh
cargo test -p uor-r4-spiralcore-fbs-replay --offline
cargo run  -p uor-r4-spiralcore-fbs-replay --offline --release -- \
  --size 65536 --nested 16 --absorb 3 --out <dir> AGENTS.md Cargo.lock README.md
```

## 3. Conformance: every claim in the paper, checked

All 20 tests pass. Full list in the crate; the load-bearing results:

### 3.1 Counting laws and the frontier's shape
- `\|R_N\| = s_3(N)` and `W(N) = (N − s_3(N))/2` hold for **N = 1…2999**; the level histogram
  equals the base-3 digits of N; levels are non-increasing (test
  `section_3_frontier_counts_are_base_three_digits`).
- The carry schedule is **content-independent**: equal-length inputs of different content produce
  identical frontier level sequences (test `parse_shape_depends_only_on_length_not_content`). The
  parse shape is fixed by the byte length, not by the data.

### 3.2 The recovery identity
`D(E(x)) = x` is exact across sizes {0, 1, 2, 3, 41, 81, 82, 83, 84, 163, 164, 165, 166, 1000,
8192, 16400} × {zeros, random, ramp} and on every corpus in §4. Encoding is injective on 200
samples.

### 3.3 Appendix A, exactly
- `Code([1]) = 65,793` and `Code([1,2]) = 16,908,546` — the paper's two worked examples reproduce
  byte-for-byte (`01 01 01` and `01 02 01 02`).
- `V(n)` is the shortest unsigned base-128 varint; boundary conventions (empty input → three
  `(0, Code([]))` pins and empty `T`; one atom → duplicated first pin; `N = ⌈m/82⌉`) all hold.
- The reference parser rejects bad magic, truncation, trailing bytes, unknown tags, out-of-range leaf
  lengths, non-causal child references and unequal child levels, and accepts (as the paper implies)
  noncanonical duplicate definitions.

### 3.4 Recursion (Sections 6–7)
- `decode(Q_j)` returns exactly `tau(T_{j-1})` for every layer `j`, and `decode(Q_0) = x`, for
  `n` ∈ {0,1,2,3,8,16}.
- `sigma^-1(D(P_{k+1})) = (P_k, M_k)` exactly for a 3-step absorption chain with a 334-byte manifest
  — the packaging the paper says was *"not separately run in these receipts"*. It works, and it
  grows at every step.

## 4. Measurement: compression, inflation, structure

Receipts: [table (64 KiB set)](evidence/setA-64KiB/table.md) · [receipt.json](evidence/setA-64KiB/receipt.json) ·
[large real corpora](evidence/setB-real/table.md). Command in §2.

### 4.1 The compression claim fails on every realistic input

| corpus | input B | `E(x)` B | ratio | verbatim payload B | gzip -9 | xz -9 | zstd -19 |
|---|---:|---:|---:|---:|---:|---:|---:|
| uniform random | 65,536 | 69,827 | **1.0655** | 65,536 | 65,574 | 65,600 | 65,549 |
| low-entropy 4 symbols | 65,536 | 69,827 | **1.0655** | 65,536 | 19,750 | 18,544 | 16,535 |
| base64 alphabet | 65,536 | 69,827 | **1.0655** | 65,536 | 49,652 | 50,444 | 49,249 |
| structured records | 65,536 | 68,985 | **1.0526** | 64,716 | 2,394 | 1,128 | 1,281 |
| `AGENTS.md` | 46,272 | 49,269 | **1.0648** | 46,272 | 18,354 | 17,320 | 17,616 |
| `Cargo.lock` | 74,318 | 79,115 | **1.0645** | 74,236 | 17,816 | 15,724 | 15,752 |
| `docs/integration/DECISIONS.md` | 125,024 | 133,304 | **1.0662** | 125,024 | 47,078 | 42,908 | 43,732 |
| `spiralcore_operator.rs` | 53,471 | 56,871 | **1.0636** | 53,389 | 11,398 | 10,568 | 10,691 |
| the paper's own extracted text | 14,100 | 14,942 | **1.0597** | 14,100 | 5,840 | 5,664 | 5,672 |
| zeros | 65,536 | **164** | 0.0025 | 100 | 96 | 140 | 22 |
| period-82 cycle | 65,536 | **164** | 0.0025 | 100 | 383 | 232 | 105 |
| period-96 cycle | 65,536 | 4,332 | 0.0661 | 3,954 | 394 | 248 | 119 |
| 0…255 repeated | 65,536 | 12,285 | 0.1875 | 10,514 | 597 | 372 | 280 |
| 4 MiB concatenated repo markdown | 4,194,304 | 4,531,602 | **1.0804** | 4,190,204 | 1,341,837 | 982,580 | 1,022,754 |
| 14 MB `git archive` tar of a source tree | 13,998,080 | 14,858,514 | **1.0615** | 13,694,434 | 2,976,711 | 1,944,060 | 1,980,390 |

Reading:

- **Unique content expands by ~6 %.** The +4,291 bytes on 65,536 random bytes decompose as: leaf
  definition overhead (tag + length varint) ≈ 1,600 B, parent definitions ≈ 2,772 B, pins and
  framing ≈ 44 B (the receipt carries every field; this is arithmetic on it, not an estimate).
- **The packet contains the payload in the clear.** For non-repeating input, *every* distinct atom
  is stored verbatim as `Leaf(a) = (0,a)`; the verbatim column is ≥ 99.9 % of the input. Nothing is
  reconstructed from a compact description, which is why recovery is exact and why unique data
  cannot shrink.
- **The only shrinking cases are inputs that repeat exactly on the 82-byte grid.** The number of
  distinct atoms for a period-`p` corpus is `p / gcd(82, p)` (+1 for the final short atom):
  period 82 → 2 distinct atoms; period 96 → 49; 0…255 → 129. That identity is asserted in
  `builtin_corpora_are_deterministic`, and it is the scheme's entire detection power.
- **The gap widens with size.** On 4 MiB of real repository markdown the packet is 4.53 MB against
  xz's 983 KB (4.6× larger); on a 14 MB source-tree tar it is 14.86 MB against xz's 1.94 MB (7.6×
  larger). The scheme's overhead is per distinct atom and does not amortise.
- **It is not an entropy coder.** The 4-symbol corpus has an order-0 bound of 131,069 bits
  (16,384 bytes) and the scheme emits 69,827 bytes because its atoms are 82 random draws over the
  alphabet and never repeat; gzip emits 19,750. Structure that is not *exactly equal and
  atom-aligned* is invisible.

### 4.2 "Inflation" is real, and growth is the rule
Exact recovery holds everywhere. The recursion grows monotonically:

| case | `E(x)` | `Q0` after 16 nested literal tails | absorption k=1..3 |
|---|---:|---:|---|
| random 65,536 B | 69,827 | **193,927** (2.96×) | 74,766 → 80,032 → 85,663 |
| `AGENTS.md` 46,272 B | 49,269 | 135,803 | 52,826 → 56,628 → 60,677 |
| zeros 65,536 B | 164 | 732 | 552 → 956 → 1,379 |

The paper's own Appendix B reports the same shape: 13,513+334 → 34,905 (2.58×) and
279,620+334 → 786,081 (2.81×). Nesting is a packaging operation, not a compressor — every level
stores the previous level's dictionary as its tail.

### 4.3 "Knowing the structure" is exact-subtree dedup, not attention
The dictionary is a DAG whose nodes are interned by exact definition equality. Its detection power is
precisely: *repeated content that is exactly equal and aligned to the fixed ternary parse*. For
periodic input that is `p/gcd(82,p)` distinct atoms; a shift, an insertion or a period coprime with
82 destroys it (period 96 keeps only 49 of 800 atoms distinct — better than random, but only because
`gcd(82,96) = 2`). Fixed-size chunking with a fixed hierarchy is the known weak form of
deduplication; content-defined chunking exists to remove exactly this sensitivity
([Data deduplication](https://en.wikipedia.org/wiki/Data_deduplication)).

Measured on the six real corpora above (363,791 bytes total): **4,438 distinct leaves for 4,440
atoms — 2 reused atoms in 363,791 bytes (99.95 % novel)**. On `AGENTS.md`: 565 atoms, 565 distinct
leaves, **0 reuse** — in 46 KB of heavily repetitive markdown the scheme finds not one repeated
82-byte block. Four hard limits follow: exact equality only (one differing bit makes a new leaf);
82-byte atom granularity only; a repeat is shared only if both occurrences start at atom indices
congruent modulo `3^h`; and the schedule is a fixed function of the atom count, with no
data-dependent search. There is no approximate, offset-free, rolling or semantic matching.

### 4.4 Why these are not arbitrary gates
- Conformance bars are the paper's own equations, worked examples and boundary conventions — not
  thresholds chosen by this review.
- The compression question is answered against the *definitional* reference (`\|E(x)\| vs \|x\|`)
  plus standard compressors run on the same bytes; no pass/fail bar is invented.
- The only inequalities asserted in tests are information-theoretic necessities (incompressible data
  cannot shrink) or derived bounds (`p/gcd(82,p)`, `Σ3^ℓ = leaves`).
- The codec is deterministic and exact, so there is no noise to average: D22's seed/row policy is
  about noisy experiments and is vacuous here. Breadth instead comes from 15 corpora (9 synthetic +
  6 real files) and a 45-point size × content round-trip matrix.

## 5. The developer's own evidence, audited

`SpiralCore_Forward_Replay_Test_Evidence/` (15 October 2026; 46/46 SHA-256 entries verify, zip and
directory byte-identical — the bundle is *internally* clean).

| Case | Input | Carrier | Retained | Round trip | Size change |
|---|---:|---:|---:|---|---|
| integer_random_64KiB | 65,536 | 65,658 (`FBSINT1`) | 66,060 | exact | **+122 B (+0.19 %)** carrier, +524 B (+0.80 %) retained |
| integer_random_corrupt | 65,536 | 66,060 | 66,060 | rejected | detected only via the transported `meta.expectedSHA256`; the codec's own re-admission check passes on corrupt data |
| recipe_1GB | 1,000,000,000 | 83 B recipe (+422 B metadata) | 505 | exact | the "input" is a **69-byte phrase repeated**, and the phrase is stored verbatim in the recipe |
| retained endpoint (1 GB) | declared | **absent** | — | **unverifiable** | the 3,397,727-byte zlib figure exists only in prose; no manifest, endpoint, restore receipt or `stream_results.json` is in the bundle |

- No compression **ratio** is computed anywhere in the bundle (search for "ratio": zero hits).
- No compressor baseline runs on the bundle's own inputs; no seeds; one run per case.
- The receiver is a fresh process but not an independent one: the payload travels in the file it
  reads, and acceptance compares against the sender's own digest.
- The bundle's carrier format (`FBSINT1`, per-root level/length + minimal big-endian integer) is
  **not** the paper's Appendix A format (`FBS7DAG1`, three pins + `tau(T)`). The paper's specified
  wire format is therefore not evidenced by any supplied artifact; its overhead is ~6 %, the
  developer's own carrier ~0.2 %. Both expand on unique data.

## 6. External context: is this networking/telecom, and is it new?

The owner's belief that the bottom layer is *"networking and telecom standards"* is **not supported**,
and the mathematics is **classical**.

**Standards.** No IETF RFC, ITU-T Recommendation, IEEE/ISO standard or 3GPP specification defines an
"FBS engine" with these semantics, and `FBS7DAG1` appears in no registry (IANA ROHC profile ids,
PPP numbers, IPComp CPIs, EtherTypes) — it is a bespoke magic. The scheme's actual primitives are a
**LEB128-style varint** (DWARF §7.8 / protobuf / WebAssembly lineage), a bespoke magic + tagged
record, a **hash-consed DAG** and a **base-3 counter**. Real framing standards supply what this
format omits: ISO/IEC 13239 HDLC and RFC 1661/1662 PPP (flag + escaping + FCS), IEEE 802.3 (FCS),
RFC 791/8200/9293 (lengths and checksums), ITU-T X.690 TLV. The format is self-describing
(magic + explicit counts) but **not** self-synchronising: the magic can occur inside a payload and
there is no escaping. Real link-compression standards add negotiation, resynchronisation, error
detection and expansion fallback — RFC 1962/1979 (PPP CCP/Deflate), RFC 3173 (IPComp), RFC 5795
(ROHC) — none of which is present.

**"Ternary" is not a line code.** Real ternary signalling is 4B3T/MMS43 (ETSI TS 102 080), 8B/6T,
AMI/HDB3 (ITU-T G.703), PAM-3 (JEDEC JESD239); 2B1Q is four-level. The paper's ternary is *parse-tree
arity* — arithmetic, not transmission.

**Prior art.** The scheme is a **fixed-shape straight-line program (SLP) / grammar compressor** with
content-based interning — i.e. hierarchical deduplication over a fixed 82-byte grid with
content-addressed nodes. Because the parse shape depends only on N and never on content, it is
strictly weaker than Re-Pair, BPE, Sequitur and LZ, all of which choose rules from the data
([Larsson & Moffat](https://doi.org/10.1109/5.892708), [Charikar et al.](http://web.cs.ucla.edu/~sahai/work/web/2005%20Publications/TransOnInfoTheory2005.pdf),
[Lohrey](https://www.math.aau.at/AofA2024/assets/slides/Lohrey.pdf)). An analysis contributed by the
external research leg of this review (a derivation from cited facts, not a citation): on
Fibonacci/Sturmian words the smallest SLP has Θ(log N) rules while the fixed-shape DAG has Θ(√N)
nodes, so no finite approximation ratio to the smallest grammar can hold. The counting laws are
**Legendre's formula at p = 3** and Kummer's theorem on carries — correct, 19th-century, and
content-free.

**The bound that settles the packet claim.** If a packet has at most B bits, it can name at most 2^B
inputs (pigeonhole); at least a `1 − 2^-c` fraction of n-bit strings cannot be compressed by c bits
([Ferbus-Zanda & Grigorieff](https://arxiv.org/abs/1010.3201)). The only escape is decoder side
information, which must be counted — the standard model-based-compression accounting, where the
authors themselves report that counting the model destroys the ratio
([Delétang et al. §3.2](https://arxiv.org/abs/2309.10668)). The nested-tail and §7 recursions do not
help: `\|P_{k+1}\| ≥ \|P_k\| + \|M_k\| + framing` because the previous packet is stored literally.

## 7. Internal context: what this repository already knew

- The repo has **never seen this paper** (no occurrence of "Recursive FBS" / "FBS Engine" anywhere);
  this record is its first review.
- "SpiralCore" in-repo names the exact v63 octonion/Cl(0,6) convention and its signed BFS transport
  ([geometry.md](../../geometry.md), [spiralcore_operator.rs](../../../crates/uor-r4-core/src/spiralcore_operator.rs)),
  status `IMPLEMENTED_CONTROL_ONLY`; "FBS" is used as the *Atomic Ternary FBS* binder/frontier
  abstraction and is never expanded. The paper's ternary binder/frontier matches that abstraction —
  not a telecom protocol.
- The repo's own float SpiralCore/FBS projection already uses the same frontier model
  (`MAX_ATOMS = 6560 // 3^8 - 1: at most sixteen frontier roots`), which independently corroborates
  the base-3 accounting in §3.1.
- The repo has already reviewed an analogous SpiralCore compression claim and reached the same shape
  of verdict: unique content expands 2.17× in the object-store records; *"It is only compression when
  the receiver already holds ancestry"*; *"The discrete model path has no transfer step to
  compress."* ([independent review](../spiralcore-attention-independent-review-2026-10-14/README.md),
  [D34](../../integration/DECISIONS.md)).
- Attention claims for SpiralCore were previously left unqualified, with the replacement head
  explicitly unmeasured ([native review](../spiralcore-native-attention-review-2026-10-14/README.md)).

## 8. Verdicts, and what would change them

**The steelman that survives.** Read charitably and precisely, the artifact is a *complete, total,
exactly-invertible, self-describing container codec*, closed under absorbing its own output: one
outermost packet carries an entire lineage of data **and** instructions, and a receiver holding only
that packet plus the published rules recovers every ancestor bit-exactly
(`sigma^-1(D(P_{k+1})) = (P_k, M_k)`), with a genuinely unbounded ratio on exactly-periodic,
atom-aligned input (65,536 → 164 B, independent of input length). That is a real result — under the
word **packaging/provenance**, not *compression*. Every clause of it is either in the paper or
measured here.

**SUPPORTED (real, creditable).** The specification is precise enough to reimplement exactly from the
prose and equations alone, including boundary conventions and a canonical parser — a genuine quality
bar that most external submissions fail. `D(E(x)) = x` is a correct theorem (trivial, but correctly
proved), the nested-tail and anchor+manifest constructions work, and Appendix B is unusually candid
about the limits.

**REFUTED BY MEASUREMENT.** "Full compression of data over a transferred packet." The packet is
~6 % *larger* than unique input, carries the payload verbatim, and only shrinks on data that repeats
exactly on the fixed 82-byte/ternary grid. Compressors on the same bytes reduce 2.7–5×. No
compression ratio is measured in any supplied artifact.

**NOT SUPPORTED BY THE ARTIFACT.** "Full fidelity of knowing the structure of the data with an
attention mechanism." The paper contains no attention mechanism at all (0 occurrences of the word),
no learned parameters and no similarity function; the *"attention"* half of the claim must come from
some other document, and none of the supplied material supplies it.

**NOT SUPPORTED.** "A complete mechanism", if "complete" means compression + attention. It is
complete for lossless re-encoding.

What would change the verdicts:

1. A declared input class and a measured size table `\|E(x)\|/\|x\|` including framing, metadata
   and any shared or pre-agreed state, against gzip/xz on the same corpus.
2. The three cited reports [1][2][3] (currently behind ChatGPT library links) and the reference codec,
   so the numbers can be reproduced rather than quoted.
3. A named attention mechanism (queries/keys/values or an equivalent) with a stated role, an
   executed artifact, and an integer-exact decoder path if a compression claim is attached to it.
4. A non-expansion guarantee (copy uncovered bytes verbatim) if the packet is meant for a link.

## 9. Provenance and gaps

- **QA record does not describe the delivered PDF.** `..._QA.json` claims `pages: 6`,
  `pdf_bytes: 269647`, `pdf_sha256: 884fcc48…`; the delivered PDF has 6 pages but 312,425 bytes and
  SHA-256 `9e43a27c…` (+42,778 bytes). `README.txt` states *"The QA record identifies the checked
  PDF"* — as delivered, it does not. Page count matches, so the document is probably a later
  re-render; the QA claim is nevertheless unbound to the artifact.
- The zip contains only the `.tex`, the QA JSON and the README; there is no codec source, no test
  harness, and none of the three cited reports.
- The paper's Appendix B numbers (13,513+334 → 34,905; 279,620+334 → 786,081) are quoted from
  report [2]; report [2] as supplied does not contain the "descendant" figures, so those remain
  **UNVERIFIED**.
- The developer's retained-endpoint claim is **UNVERIFIABLE** from the bundle (files absent).

## 10. Costs and status

Local CPU work only: one Rust crate, 20 tests, three measurement runs. No GPU, pod, rental, training
or hold release; no change to any model artifact or headline. New external cost \$0. The crate is a
review instrument, not a model component and not a D32 instrument.

**Next (owner decision).** If the owner wants the *compression* claim examined further, the decisive
follow-up is (1) above on a declared corpus with the developer's own encoder; if the interest is the
*attention* claim, the artifact that contains it must be named first — this paper does not.
