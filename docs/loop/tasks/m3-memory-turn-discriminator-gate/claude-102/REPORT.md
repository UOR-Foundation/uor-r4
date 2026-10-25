# REPORT: m3-memory-turn-discriminator-gate, claude-102

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2855.

## What did not work, first

- **The registered bar was missed, so this configuration is closed negative (D22).** The panel stayed clean (false_assert 0/50, false_overwrite 0/50, clean 50/50), but two parts of the bar failed:
  - **R1 fell to 406/462 (87.9%), below the 95% floor.** All 56 misses are "the surviving readings disagree", on 7 update templates whose correction frame carries a word the assert frame lacks: "Actually, … is {v} now." (6 templates) and "Sorry, my lucky number is really {v}.". The marker reading (`actually` or `sorry`, plus the assert frame) takes that word into the 1–4 word value ("X now"), while the plain correction frame reads "X". The two readings agree on the expert but not on the span, so the gate refuses. The FIT had flagged this risk.
  - **R2 stayed at 0/220,** all `Unresolved`. The unseen phrasings differ from the training frames in their core wording, not in a leading marker. Marker composition has no reach on them.
- **No notebook row file.** This item names no `notebook:` page, and its scope holds none, so the row is below and in the lab README rather than in `docs/research/notebook/m3-durable-memory.md`.
- **The pre-run review asked for D29 on the `cuda` build too,** to match the sheet. The job added it before launch, and both builds passed.

## What changed

- **`crates/uor-r4-training/src/sieve_marker_gate.rs`:** markers derived as cofactors of training frames by an exact rule, with a pinned table: `actually`, `sorry` and `sorry i meant` (update); `remember`, `please note that` and `people` (neutral); sha256 `cd793994…`. It also holds `gate_frames`, `decode_gated`, the launcher and host tests.
- **`crates/uor-r4-training/src/sieve_frame.cu`:** `sieve_frame_compile_marked`. It strips up to 2 markers, runs the sieve and alignment, and requires every reading to agree. Integer only.
- **`crates/uor-r4-training/src/bin/prose-panel-gate.rs`:** 14 finite CUDA controls, and the pinned panel, R1 and R2 run.
- **`Cargo.toml`** (the binary entry) and **`lib.rs`** (the module and the unsafe-launcher note).
- **Docs:** the prep sheet, and a "25 Oct" section in the lab README.

## Measurements

One RTX 5090, run ref `63cd966ad`, pod `vast-57633156`, about 10 minutes. Outputs are in the HF store under `results/claude/prose-panel-gate-2026-10-25/`, with `record.json` sha256 `771e15aa6d904a83e86dc6af842b14cf7b2e48fb3f84739a79b0afdb82be930c`.

- **740,484 tokens/s** (lean build; 732 turns, 4,280 words, 5,780 µs from upload to download). The `cuda` build gives 735,268.
- **Panel:** false_assert 0/50, false_overwrite 0/50, clean 50/50, with 0 surviving frames.
- **R1:** 406/462 (the floor is 95%). **R2:** 0/220. **Registered bar:** not met.
- **Instrument checks:**
  - controls 12/12 (sieve) and 14/14 (gate), on both builds;
  - D29 at 5/5 on both features;
  - two launches identical, and a second build identical (`decisions.json` `66b01121…b936`);
  - PTX `75e837bc…6710`.

**Notebook row** (`docs/research/notebook/m3-durable-memory.md` format):

| 2026-10-25 | The marker-gated sieve compiler on the frozen prose panel and the world=v2 recall controls, on one RTX 5090. Six training-derived discourse markers compose with exact frames, under an exact-explanation and agreement gate. claude-102. | 740,484 tokens/s; panel 0/50, 0/50, 50/50; R1 406/462; R2 0/220; controls 12/12 and 14/14; second build identical | Panel 0/0/50 and R1 ≥ 95% and R2 > 0/220 (sheet §4) | negative (this configuration: R1 below its floor through span disagreement, and R2 unreached) | lab README; HF `results/claude/prose-panel-gate-2026-10-25/` |

## Checks run at the head

- `cargo test -p uor-r4-training --lib`: 74 passed, including the marker-table pin.
- The `sieve-cuda` stub-nvcc type-check of the lib, both binaries and the tests: clean.
- `ratchet d23`: 4451 at baseline. `ratchet d25`: 93 at baseline.
- `loop gates --quick`, and the fresh review, are in the PR record.

## Skills used

- holding-the-goal: a negative that names its own cause (span disagreement, and no reach on core wordings) is what the next configuration starts from.

## Still open

- **`m3-memory-turn-discriminator-gate` stays open.** Its work is generalizing the frame core, which R2 shows markers cannot do.
- **A narrower next configuration** keeps the marker rule but either bars base-frame words from the value or resolves agreeing experts to the shorter span. It would recover R1, but not R2.
- **The notebook row** above can be moved into the M3 notebook once an item that names that page as its `notebook:` lands it.

## Plain summary

I added a rule that lets the lawful compiler handle phrasing variations like "Actually, …" and "Remember, …". Ordinary prose still never wrote to memory. But the rule made some trained corrections ambiguous, and it understood none of the new wordings, so this attempt is recorded as a negative. The next step is generalizing the core of each phrase, not its lead-in.
