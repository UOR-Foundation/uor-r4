# PROJECT MEMORY — UOR-R4

**Read this before any unit.** It is the whole-project picture that local files cannot give: the
mission, the hard rules, what has already been measured and where it fell short, and what the next
unit must therefore be. It summarizes and never overrides [CHARTER.md](../CHARTER.md); where they
disagree, the charter wins. Update it when a decision or a measured result changes the picture.

---

## 1. What the project is

A **geometric language model that is not a transformer**: it predicts text with geometry (prime
addresses, zeta phases, E8/H4/icosian rotations, SpiralCore/FBS routing) and exact memory, trained
and served in **one native integer state**, on a laptop or in a browser. The owner's rule:
*"USE GEOMETRY. THAT IS THE END OF IT. NEVER USE ANYTHING ELSE."*

The goal behind every ban: fast local AI that trains fast, runs fast, and reaches near-frontier
capability. **The lead metric is tokens per second**; a small quality gap is acceptable. Compute is
not the constraint: a full 16,384-step recipe is ~45 min here, and a 1B-token epoch ~20 h.

## 2. The rules that decide every design question

| Rule | Consequence |
|---|---|
| Geometry only (CHARTER §3.1) | a mechanism that is not geometric is not a candidate |
| No softmax / squashing (§3.2) | no cross-entropy, no normalized exponential, no table disguise |
| **No dense matmul / GEMM anywhere (§3.3)** | the trunk's mixing is a routed gather + gate, not a matrix |
| One persistent native state (D85, §3.7) | token addresses, carry, routes, memory keys, reads, gradients — one integer/fixed-point representation, same operators in training and serving, bit-identical replay; an F32 master plus export gap is *historical/reference only* |
| Serving is integer (§3.6) | no float or transcendental instruction in the served artifact |
| **No neural comparators (D28)** | **never propose a dense/transformer baseline to "check" the geometry** — the owner withdrew those controls (L-006) |
| GPU-first (§3.5) | GPU qualifies; the D84 CPU program is development research only |
| A negative closes a configuration, never a mechanism (§4.1) | classify instrument vs mechanism before calling anything |
| Never park, never wait (§5.1) | a lab finishes alone; an unanswered ask becomes its own decision review |

## 3. Where the milestones stand

M1 closed 30 Oct (a saved D11 model passes bars a–f) — **but 62.5 % of the reference replies are
verbatim TRAIN replies, so M1 is not chat.** M2 (grounded reply from exact memory) is blocked on the
owner for a bar number. M3 paused. M4–M8 not started. Pre-alpha: **no model holds a useful
conversation.** `STATE.md`/`HANDOFF.md` are rendered and can lag the code; the roadmap items and
notebooks are the live record.

## 4. The architecture, as it actually runs today

```
token id → embedding.weight[id]        (vocab × width; the one shared vector table)
        → per layer: rec.in / rec.gate / rec.out / mlp.gate / mlp.up / mlp.down
                     = linear() → plan.forward() → spiralcore_fbs.cu   (routed gather + sign)
                     + quaternion rotation + geometric gates
                     + reads: Hamming copy, vector, address memory, turn memory
        → writer: compatibility_scores(point, embedding.weight) + Cl(0,6) orbit roots
        → next token id
```

The projection is **a gather and a sign**: `y_o = A_o x_σ(o)` on 8D blocks with a Cl(0,6) state. We
measured a trained trunk: **1,584 of 1,584 blocks are exactly one signed permutation** (residual 0).
The integer session serves that same gather (`stack_fbs_gather`), and the k=1 export is
**byte-identical** to the float trunk's artifact.

## 5. The transport history — the single most important thing in this file

The project has repeatedly found the same thing: **the address/selection finds the right item, and
the value does not transport into the prediction.** The record:

| Line | Measured | Verdict |
|---|---|---|
| Hopf sector routing | 7 of 512 sectors occupied; 43 after a signed projection; as a retrieval filter MRR 0.0045 vs 0.0743 | PARKED (D72) |
| ADR-0003 learned four-coordinate mixer | "selected support but **did not transport adequate semantic value**": 4.44 % real-vs-permuted against a 5 % gate | `RETAIN_STORAGE_RECALL_ONLY` |
| Prime-ranked pointer | hit rate 0.377 → 0.545, but memory answers stayed 19 / 21 / 23 of 40 against a bar of 25 — "the found token did not reach the prediction" | pointer-key PARKED |
| The choice rule (#455) | hard selection 22.81 %, top-3 24.54 %, **averaging over all 25.91 %**; a fixed baseline at 26.42 % beat every learned arm | averaging is the read that works; learned hard-picking collapsed |
| Exact memory (M3 oracle) | true address 80/80, sieve 20/80, no memory 0/80 | address is not the problem |
| **The current line (D86 review, 15 Nov)** | the vector read finds the asked fact — **first pick right on 97 %** — then **averages it with three other picks**; TRAIN ≈ 39 k windows | the open diagnosis |
| **Hard-copy read, eval-only (claude-388, `#3997`)** | copying **only the first pick** passes the floor bar at a **×16 gain** (R_ask 0.984/0.969 and 0.969/0.965); at the **trained** scale it fails (0.631/0.643, 0.096/0.145). The trained write is **0.048 of the stream with one pick** (0.026 for the mean of four) — it needs ~16× that, and **no trained parameter supplies it** | the write scale is the blocker |
| **Native Q31 first-pick operator (claude-389, `#4001`)** | integer-only, picks equal to the f64 reference on **1,024/1,024 rows** at both barred seeds, CUDA twin bit-identical (49,152 rows, 12.58 M out words, 0 different) | the operator exists; **it has no gradient and no session calls it** |

**Lesson:** do not re-litigate "is the address right" and do not re-propose selection-vs-averaging
from scratch — averaging keeps the information and hard selection loses it. The unsolved piece is
the **readout**: an address/value that survives the mix, in the same integer state, with the writer.

## 6. What this means for the next units

1. **Transport/readout first**, and the failure is now measured precisely: the pick is right on
   97 % of rows and the **write is ~16× too small**. Copying the first pick passes the bar at ×16;
   the trained gain supplies 1/16 of that. So the unit is **to make the write scale train**, in the
   native state — not more data, not more layers, and not a "better" read. The native operator
   already exists (claude-389); `q31_first_pick_read` has no gradient and no session calls it.
   That is the missing piece before any scale run.
2. **Scale is still part of the answer** (D87: a 1B kept-token donor corpus from DeepSeek V4.1 Flash
   with reasoning kept; caps 4 pods / $35 h for that run). The first 457 k answered rows are already
   on disk and tokenized (see §7).
3. **Never a dense baseline.** The comparison is against *lawful non-neural baselines* (count
   tables: unigram 1,226/14,287, T2 6,300/14,287) and against the project's own earlier results.
4. **Every item must be validated before it judges a model.** This session lost several conclusions
   to: an echo counted as an answer, a `System: Memory:` crib present in the corpus but missing from
   the item, and a checkpoint chosen at the end of an overfit run. Prove the item on a source that
   has the capability (the corpus itself, or an oracle) first.

## 6b. The sheet and the writer (measured, codex-43)

- The trainer's model shape is now parameterised: `--width`, `--layers`, `--vocab-size`, `--context`,
  with the sheet's values as the defaults and `config.json` unchanged at them.
- **The native vector writer's vocabulary is 256..8192** (`CELLS 256 × MAX_PER_CELL 32`). A 32k
  tokenizer cannot be served by it; **8,192 is the ceiling** until the identity partition changes.
  An 8,192-piece tokenizer exists (`blake3:226a2016…`).
- **The writer's candidate-vector intermediate is capped at 1 GiB f32** (256 M elements). At 8 kept
  cells and 32 slots that is 2,048 rows at width 512, so `candidates` is now **row-chunked**; the
  sheet's own shape (4,096 × 256) still fits a whole panel in one call.
- A **30 M model** (512 width, 12 layers, 8,192 vocab, 512 context) builds and steps on the real
  corpus: 8 steps in 7 s at batch 8 → about **4,500 tokens/s**, so one 332 M-token epoch is **~20 h**
  on this 4090.
- **DEV**: `--dev DIR` scores an unpinned split and `--dev-tokens N` caps it. A donor DEV from
  held-out shards 3–4 (`/tmp/opencode/donor-dev`) at 200 k tokens is **1,052 windows / 101,175
  rows**, a 21× cheaper evaluation than the full 2.13 M rows. The narrow pinned `ft/dev` is a
  different tokenizer and world and must never be used for a donor run.
- **Speed, measured (18 Nov)**: the objective's whole-table scan was half the step — the cell
  objective now takes the fused top-8 pass with identical loss (0.59 → 0.30 s/step). The projection
  route precompute was then tried and **measured at 3 % (a wash) and reverted**: every thread of a
  block reads the same query, so the route's table reads already hit L2. The real cost is **per layer:
  ~63 ms at 1,024 rows**, about **200x a dense layer**, bound by per-element gather, quantise and
  gate work. **Batch amortises it** (193 µs/token at batch 8 vs 80 µs/token at batch 24). The writer's
  candidate inventory is now scored in column chunks (M2), which removed the 8,192-row cap and lets
  batch 48 run.
- **Tool hygiene**: `NativeVectorWriter`'s source hash is part of a checkpoint's saved binding, so
  **any edit to that file requires rebuilding every checkpoint-loading tool** (`fbs-snapshot`), and a
  checkpoint trained before the edit cannot be exported by tools built after it. Restart a run after
  such an edit, or its checkpoint is unexportable. The served artifact is unaffected.
- **The plan and the ledger**: [PLAN-2026-11-18](labs/one-model-end-to-end/PLAN-2026-11-18.md) (fix the
  math before anything long runs; the capability ladder C1–C8; the 20 % cost rule) and
  [COSTS](labs/one-model-end-to-end/COSTS.md) (measured stage costs and the layer ablation).

## 7. Assets and pins that exist now

| Asset | Where | Value |
|---|---|---|
| Donor corpus (prompts) | `/mnt/plexpool/uor-r4-corpus/m2-donor-corpus-2026-11-16/inputs/bank*.jsonl.zst` | 692,364 prompts, no replies |
| Donor replies | `.../api-a/shard-*.jsonl.zst` (225 shards) | 457,601 rows, `turns[{answer,reasoning}]` |
| 32k tokenizer | `/mnt/plexpool/uor-r4-corpus/tokenizer-32768-2026-11-17.json` | 32,509 merges |
| Tokenized corpus, 8k (the one to train on) | `/mnt/plexpool/uor-r4-corpus/tokens-8k-2026-11-17/` | **456,619 conversations, 388,405,825 tokens**, 274 M targets, vocab 8,192; tokens `de5c05bc…`, mask `0949302f…` |
| Tokenized corpus, 32k (unusable: the writer caps at 8,192) | `/mnt/plexpool/uor-r4-corpus/tokens-2026-11-17/` | 457,021 conversations, 332,425,562 tokens; tokens `507cf271…`, mask `4865da8d…` |
| Pinned narrow recipe | `docs/loop/tasks/one-model-end-to-end/codex-34/ORDER.md` | reproduces all nine `[memory]` fingerprints |
| Narrow corpus | `/tmp/opencode/next02-inputs/mw-balp/train` | 8.13 M tokens, sha `1fc3f4ec…` |
| Reader/writer bin | `crates/uor-r4-training/src/bin/prepare-chat-jsonl.rs` | JSONL `messages` **or** `turns` → UORT |

The trainer's model shape is compiled in: `SHEET_VOCAB 4096`, `SHEET_WIDTH 256`, `SHEET_CONTEXT 192`,
`pattern "rrrrrr"` (6 layers) — threading a 32 k vocabulary and a longer window through the read
tables, dev table, bounds and window logic is the "new sheet" unit.

## 8. Traps this project has already paid for

- **L-006**: compute spent on transformer controls the owner had withdrawn. Do not rebuild them.
- **L-007**: a failed measurement cloaked as a mechanism negative. An instrument fault is a fix.
- **L-009**: "copy or read recalled across a multi-turn conversation when the artifact has no
  reader" — a reply that looks recalled may be a copy.
- **L-010**: a memory gain measured with no no-memory control.
- **This session's own**: the panel's `value_present` scored echoing (254/256 replies shared a
  4-token span with the prompt); then the item omitted the corpus's `System: Memory:` turn, so the
  model was asked something it never trained on. Both looked like model failures and were not.

## 9. Open questions the next session must not re-derive

- The readout's failure is **not** averaging vs selection (claude-388 settled it: the first pick is
  right and copying it alone works) — it is **write scale**: 0.048 of the stream where ~0.75 is
  needed. How to train that gain in the native state, under D85's one-state rule, is the open unit.
- Does the routed-gather transport scale? Unknown at every size above 1.1 M values.
- Is the 1B corpus complete? D87 is open until it is sealed; 457 k answered rows exist today.

## 10. Test-validity incidents (keep this list current)

- **The DEV leak (17 Nov, caught before use).** The first full 8k training corpus was built from
  **all 225 shards**, including the two the DEV slice came from. A run on it would have reported a
  leak-inflated curve. Caught by checking the corpus manifest's `input_files` before trusting the
  run; stopped, and rebuilt as `tokens-8k-clean-2026-11-17` with shards 3–4 excluded. **Rule: a split
  is only held out if the training corpus's manifest does not name its shards.**
- **The echo panel (16 Nov).** `value_present` scored a reply that repeated the prompt; 254 of 256
  replies were prompt spans and 0 were answers.
- **The missing Memory turn (17 Nov).** The item omitted the corpus's `System: Memory:` turn, so the
  model was asked a shape it never trained on.
