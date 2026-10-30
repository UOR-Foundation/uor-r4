# Do the copy heads make the reply loop? (claude-215, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-215](../../../loop/tasks/one-model-end-to-end/claude-215/ORDER.md). Sheet: [one-model-end-to-end-reply-scope-2026-10-30](../../../harness/prep/one-model-end-to-end-reply-scope-2026-10-30.md), FIT in round 2 and pre-run PASS.

## The run

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1, 2 and 3.
- **The arms:** greedy decoding with the memory on, under three scopes of the Hamming read: `full` (as trained), `copy` (the copy heads hold to the prompt) and `all` (every head holds to the prompt). Each arm decodes 256 replies per split and seed, on DEV and on the new panel p-r (seed 2071).
- **The hardware:** one RTX 5090 (`vast-58379979`), built at `4b0b8955d`. Binary sha256 `92160ea57e799e52ae8f03cf5adc905820c3340c4805dbbb78b53c02c5fdad3b`.
- **The instrument checks:**
  - 228 CUDA tests pass on the pod, among them `a_cut_keeps_every_pick_before_it` and `the_read_scope_starts_as_greedy_and_clears_its_cut`;
  - every input matches its store sha256;
  - the binary reproduces claude-211's 15 registered files byte for byte, with the three seeds run side by side;
  - all 6 scoped p-r files differ from greedy's, so both scopes ran;
  - every scoped reply's first token equals greedy's, 256 of 256 at every seed and scope, as the cut requires.
- **Sealed** at `results/claude/one-model-end-to-end-reply-scope-2026-10-30/run-4b0b8955/`: 88 files, with a `SHA256SUMS` whose sha256 is `502d54524e5f5f89ab23f8095be8d98f9e6411e2011560ab6526887a896b5e4b`, read back OK.
- **Graded** on the laptop by `train-in-space --read-grade 1` at `4b0b8955d`: [read-grade.json](read-grade.json), sha256 `238eb90790162e5fb600d8fded1b30a6d8de5b54d3dc89067a440b6301c0de4f`.

## Tokens per second

- **Every arm decodes at 28–41 tok/s per process,** with three seeds sharing the GPU. This pod was slower than claude-213's: its 8-sample probe took 93 s against 25 s.
- **The scope costs nothing:** the `copy` and `all` arms are as fast as greedy, at the same seed.
- **This is not the served speed.** The integer path is not built yet.

## The registered outcome

**`all`: reduces, content lost. `copy`, which decides nothing, reads the same.**

At all three seeds, on p-r:
- **Fewer loops:** the `all` arm has fewer looping replies than greedy, beyond decision 4's band.
- **Decision 4 holds.**
- **Content is not kept at seed 2.** There, greedy's verbatim share `v` is 0.511, near the references' 0.563. `all`'s is 0.386: a gap of 0.177, against greedy's 0.052 plus `tol` (0.100), which is 0.152. At seeds 1 and 3 the content is kept.
- **The power floor is met:** greedy loops in 63, 70 and 92 of 256 p-r replies, all at or above 40.

As the sheet registered, "reduces, content lost" changes no decode and makes no causal claim. It unlocks one follow-up only: training with the same cut, so that the trained model meets the cut.

## p-r, the deciding panel

The references' `v_ref` is 0.563, their repeat rate is 0.00, and their mean length is 12.8 tokens. Each cell gives seeds 1, 2 and 3.

| Arm | Looping replies, of 256 | Complete | `v` | Repeated-4-gram rate | Mean length |
|---|---|---|---|---|---|
| greedy | 63, 70, 92 | 213, 188, 203 | 0.385, 0.511, 0.374 | 0.526, 0.633, 0.566 | 21.5, 24.9, 25.8 |
| copy | 29, 50, 62 | 239, 240, 239 | 0.343, 0.400, 0.318 | 0.258, 0.271, 0.268 | 16.1, 16.9, 18.2 |
| all | 31, 39, 68 | 236, 246, 240 | 0.347, 0.386, 0.317 | 0.293, 0.208, 0.272 | 16.8, 15.1, 18.4 |

**Paired, under `all`:**
- 34, 33 and 39 replies loop under greedy but not under `all`;
- 2, 2 and 15 loop under `all` but not under greedy.

**Unchanged replies:** 46, 165 and 129 of 256 are identical to greedy's under `all`, and 54, 185 and 155 under `copy`.

**On DEV,** whose prompts are TRAIN-templated (`v_ref` 0.57), the loops fall too, but by less:
- greedy has 46, 28 and 29 looping replies, and `all` has 31, 21 and 24. Only seed 1 is beyond the band.
- `v` stays near `v_ref`: 0.544–0.569 under `all`, against greedy's 0.560–0.595.

## What it shows

- **Holding the read to the prompt removes about half of greedy's loops on new phrasings, at every seed.** The paired counts run strongly one way: 33–39 replies stop looping, and 2–15 start. Completion rises by 23 to 58 replies, and the repeat rate falls from 0.53–0.63 to 0.21–0.29.
- **The copy heads alone carry most of it.** `copy` leaves 29, 50 and 62 looping replies, against `all`'s 31, 39 and 68: fewer at seeds 1 and 3, more at seed 2. With no type-only arm, this says no more than that the copy heads' read of the reply takes part.
- **The reading's limits:**
  - **Replies end sooner.** Mean lengths are 15–18 tokens against greedy's 21–26, though both are above the references' 12.8. Greedy's are long because its loops run to 64 tokens.
  - **The loops are interrupted, and the interruption is often salad.** In the fixed reading of seed 1's first 20 p-r replies, reply 1's loop "A friendundrot hear aboutdant: A friendundrot hear …" becomes "A friendundrot hear aboutdant: A friendunday." and ends. Replies 7 and 17 now end in "…all all all all soundededay.".
  - **Some loops survive:** replies 2, 11 and 12 loop under every arm.
  - **One reply improves:** reply 19 under `all` is "Got it, your friend is Pleddrurgrod.", the prompt's name, where greedy wrote "Got it, your lucky number is P.".
  - **The beginnings are unchanged.** A reply's first sentence is mostly greedy's: the model was trained to read its reply, and decoding it with that read removed takes it off what it learned.
- **"Content lost" rests on one seed, through a confound the sheet did not fix.** `v` counts only complete replies. Greedy completes the fewest (188 at seed 2), so its `v` is measured on the prompts where it did not loop. On the prompts where both greedy and an arm complete, exact matches to the reference are the same for both: 5 and 5, 3 and 3, 7 and 7. That is too few to settle content either way. The registered outcome stands as written.

## What it unlocks

As the sheet registered for "reduces, content lost":
- **No decode change.** The served model still decodes with the read as trained.
- **No causal claim.** The cut removes loops, but it may also have pushed the model off what it was trained on.
- **One follow-up:** train with the same cut, so that a reply's read is held to the prompt in training as in serving, and the model learns to reply without reading its own reply. Then this panel's measures can be repeated on a new one.
