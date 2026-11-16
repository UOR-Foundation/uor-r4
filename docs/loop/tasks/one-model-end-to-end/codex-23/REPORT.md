# REPORT: one-model-end-to-end, codex-23 (the `Read` seam, the bound form, the ledger)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-22](../codex-22/REPORT.md).

## What did not work, first

- **The ADR's `select(&self, ...)` has nowhere to put the reference read's scan.** The first-pick
  read keeps an incremental history; the session owns it and `reset` clears it. Making the trait
  `&mut self` would have made a read non-shareable, so the scan rides in `ContextView` and the
  trait stays `&self`. The deviation is the view's extra field, recorded below.
- **The generic view is not yet mechanism-neutral.** It carries a `Q31ReadHistory`; the relation
  read ignores it. When the Hamming heads adapt, that field becomes an enum or the session builds
  the view, and the ADR should say which.
- **The first ledger run measured all 49,152 rows and read 0.138 / 0.182.** Most rows carry no
  target, so the all-rows agreement is not the item's measure. `compare_on` with the probe's
  `.ask` mask gives the panel's numbers exactly; the bin now takes `--targets`.
- **No contract names `/17`.** The form exists and round-trips, but `docs/model/STACK-CONTRACT.md`
  is generated from `conformance/stack/*.toml` and no contract describes a bound stack with a read.
  Until one does, a `/17` artifact is a format, not an audited composition.
- **The renumber invalidates the `/15` digests in the earlier reports.** `codex-19`–`codex-22`
  describe `/15` artifacts; the relation record is `/16` from this unit, and the artifacts are
  re-exported below.
- No run, no trained model, no language claim. This unit is seams, format and instrument.

## What changed

- **The seam** (`read_stage.rs`): `ReadMechanism` (the artifact and ledger names), `Query` (token
  and row), `ContextView` (tokens, the row's residual, the writer table, the width, the earlier
  asked slots, and the reference read's scan), and `Read::select`. `ReadOut` is unchanged: it
  already had the ADR's fixed shape. `ReadOut::none()` is the empty result.
- **The reference adapter** (`q31_read.rs`): `impl Read for ServedQ31Read` calls exactly
  `read_out`, and a test replays the `PROMPT` fixture through the adapter beside the served session
  and asserts the same pick and the same written row at every position (ADR §1 rule 2).
- **The second consumer** (`relation_read.rs`): `impl Read for RelationRead` wraps
  `select_with_table`, returns `ReadOut::none()` when the rule finds nothing, and a test compares
  it with the direct call on a `K=1` fixture and on a query with no candidate.
- **The bound form** (`format.rs`, `session.rs`): `STACK_FBS_BOUND_READ_SCHEMA =
  "uor-r4.lut-stack/17"`. `add_q31_read_uorlut01` accepts a bound `/10` and emits `/17`; the parse
  dispatch requires the binding for a `/17`; the session binds the tokenizer on `/17` as on `/10`.
  The relation read stays refused on a bound stack.
- **The ledger** (`read_ledger.rs`, `bin/read-ledger.rs`): rows, agreement, first divergence (the
  absolute row, so it names a corpus position), agreement in parts per million, and rows a second
  at a supplied clock. `compare_on` restricts the reading to the scored rows.
- **The renumber**: `STACK_FBS_RELATION_SCHEMA` is `"uor-r4.lut-stack/16"`; `/15` is free for
  claude-384's pair rows.
- **The read half of the slide**: `Q31ReadHistory::drop_head(rows)` and `ids()`, called by both
  step kernels where the token window drops its head, so the read scans exactly the served window;
  `q31_read_history_len()` and `q31_read_window()` expose it. `can_slide_tokens` now also refuses a
  pointer head, and `set_context_slide` refuses a session with a copy boost, because both index
  absolute rows — the slide's real unsoundness is the position-indexed readers, not the read.
- **The `Context` ring** (`context_ring.rs`): the probe's bounded ring and settlement gate promoted
  into the library — `ContextSpan`, a mergeable `TopK`, `ContextRing::push(span, settled)` that
  refuses an unsettled eviction and leaves the ring unchanged, `settle`, `sum`, `top`, `live`. The
  probe stays independent on purpose: a gate that shared its code with the implementation would
  only prove itself.

## Measurements

| Reading | Rows | Agree | Agreement | First divergence |
|---|---|---|---|---|
| The artifact's own read against the probe's asked target | 1,024 | 440 | 0.4297 | row 178 |
| The FBS-trained read against the same target | 1,024 | 1,015 | **0.9912** | row 748 |

`read-ledger --reference <dump>.ask.u32 --served <picks> --targets <dump>.ask.u32` reproduces the
`read-panel` numbers exactly, so the instrument and the panel agree.

- The seam: the adapter matches the served read on every row of the fixture.
- The ring: the fold read and the top-3 selection read equal full history across 40 spans and four
  wraps; an unsettled eviction is refused with the blocking position and leaves the ring unchanged;
  the stub's join is order-independent.
- The slide with the read (`tests/context_slide.rs`, 3 passed): 16 steps at context 8, the read's
  window equals the token window at every step, every pick is a live row, the run repeats exactly,
  and a copy-boosted session refuses the policy.
- `/17`: a bound export gains the read, the tokenizer sha and the contract source survive, the
  session steps and records a read trace, and a `/17` header whose binding is stripped is refused.
- The `/16` relation artifact re-reads and serves: **47/47** on `K=1`, 431/1,024 rows, sha256
  `6430c48a7d5459bc2627bed40026e1e756ee4e9f38ebff292d3d04ddc8ae99ec`.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — **700
  passed**, 1 ignored (two seam, two adapter, one bound form, five ledger, four context ring).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic` (all targets) — green
  except six **pre-existing** `adversarial_challenger_m1_token_boundary` tests that read a
  `tokenizer.json` and a bundle this checkout does not carry.
- `cargo test --locked -p uor-r4-native-vector` — 48 passed.
- `cargo test --locked -p uor-r4-discrete --features cuda --lib` — 374 passed, 8 ignored.
- `fbs-export --legacy-artifact … --relation` then `read-panel` — the `/16` artifact above.

## Still open, in order

1. **`Context` ring's serving half (I2)** — the ring, the stub and both gates are in the library
   (`context_ring.rs`); what remains is a served read consuming the stub, and the first-pick read's
   sketch sufficiency is *not* the static one proved here, because it ranks against the current
   query. That is the unit to order, and it should not claim I2 closed before it runs.
2. **A contract for `/17`** — a `conformance/stack/*.toml` that names the read section and its
   mechanism, and `stack_contract`/`stack_audit` recognising the schema.
3. **The protocol run** (`one-model-end-to-end`) — a run with contract, tokenizer and read; the
   strata, arms and verdict rule in the item's Done-when 3.
4. **Kept-set agreement** — the ledger compares picks; a set-valued read (the Hamming heads) needs
   the set form before it can be compared on equal terms.
5. **Pair rows and the Hamming heads adapt to the seam** — `/15` and the copy/ask heads are the two
   consumers that would make the seam pay for itself.
