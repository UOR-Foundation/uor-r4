# ORDER: one-model-end-to-end, claude-204 (a held-out-phrasing panel, and its census)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `a36ef5663`.

## Goal

The panel the item's "What is missing" names.
- **Why:** the census (claude-202, `443a980e5`) found that neither DEV nor Fresh V2 holds new templates.
- **What claude-203 settled:** M-world v2's phrasing split holds by construction. TRAIN draws `Split::Train` only, and paraphrases equal to any template are refused. The split covers user-turn phrasings only; reply templates are shared.

1. **The writer.** `m-world corpus world=v2` gains `cell=PHRASING,VALUE`, each `train` or `dev`. The default `train,train` is today's corpus, byte for byte.
   - A non-default cell draws `conversation_in(rng, Cell::new(..))` under the same probe screen.
   - It refuses `exclude=` and `chat=`.
   - It writes under `OUT/<phrasing>_phrasing-<value>_value/`, and the manifest records the cell.
2. **Three panels.** 400 conversations each, at the default mix, generated with the pinned tokenizer:
   - **P-dev** (`dev,dev`, seed 2041): the held-out-phrasing panel;
   - **P-ret** (`train,dev`, seed 2042): trained phrasings, new values;
   - **P-train** (`train,train`, seed 2043): the positive control, drawn as TRAIN was.

   They go to the HF store under `inputs/claude/heldout-panels-2026-10-29/`, with `SHA256SUMS`, never into the repo.
3. **The census of each.** `train-in-space --census 1` gains `--panel NAME=DIR` (repeatable). Each extra panel is loaded unpinned: its sha256 is computed and recorded, its header and BOS are checked, and claude-202's M3–M6 are counted.

**The reading, fixed before any panel is generated:**
- A panel holds new templates if its masked prompt-template share at K = 1,024 is at most 0.5 (claude-202's rule).
- **The instrument check:** P-train must *not* hold new templates (share above 0.5), as DEV does not. If P-train passes the new-template bar, the census or the writer is at fault, and the unit stops.
- **P-dev is the held-out-phrasing panel** if it holds new templates.
- **Its masked reply-template share is reported, not judged.** Reply templates are shared by construction.
- **The two risks claude-203 named,** both counted, not judged:
  - **near-duplicate paraphrases:** P-dev's prompt 8-gram share in TRAIN;
  - **chat-v0 coincidences:** P-dev's exact-prompt share in TRAIN.

## Cancel probe (run first)

- **Does a dev-cell writer or panel already exist?** No.
  - `corpus_v2` hard-codes `Split::Train` (`crates/uor-r4-training/examples/m-world.rs:1210-1217`).
  - No UORT panel of `dev_phrasing` cells is in the HF store: `scripts/pod/uor-store ls inputs/` lists none.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-training/examples/m-world.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`

## Done when

1. **`cell=` works.**
   - **Byte-identity:** `cell=train,train` and no `cell=` write byte-identical splits at the same seed. A test, or a sha comparison recorded in the REPORT, shows it.
   - A non-default cell refuses `exclude=` and `chat=`.
2. **The three panels exist in the HF store,** with their sha256s in the REPORT.
3. **The census report** (`docs/labs/one-model-end-to-end/census-panels-2026-10-29.json`) holds the three panels' M3–M6. P-train holds no new templates. P-dev's verdict, P-ret's and both risks are given.
4. **Tests:**
   - the cell parser;
   - `--panel` parsing;
   - the unpinned loader's header and BOS refusals.
5. **Checks:**
   - the float crate's suite and the training crate's `m-world` example build pass;
   - `loop gates --full` is green;
   - one notebook row, and a REPORT, negatives first.

## Stop and ask if

Stop and decide (never wait) if:
- P-train passes the new-template bar: an instrument fault, so find it first;
- P-dev fails the new-template bar: then the phrasing split leaks at the turn level, and the REPORT says so with the risks' counts. The fallback, newly written phrasings, becomes the next unit;
- the writer needs a library change.
