# ORDER: one-model-end-to-end, claude-373 (pair rows weighed by how few replies their pair is seen with)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `fccad269b8e81100a82d21082e7eae4aa9055874`. Follows claude-370 (PR #3961).

## Goal

**Weigh each pair row of claude-370's table by one over the number of different replies its pair is seen with in TRAIN, train it the same way, and read whether the fact replies claude-370 left wrong now come out right.**

**Why now.** claude-370's pair rows add 96 and 108 exact replies of 2,218 at two seeds, but the gain is mostly the turn's own words carried into the reply. The fact forms it was aimed at barely moved: capital 0 and 1 of 20 on P14, adding and counting 0 in every model. Its REPORT names the cause and this unit's step ("Still open"): "A pair of frame words and a pair that holds the key word get the same share of the bag. The next unit should weigh rows by how few replies their pair is seen with". In "What is the capital of Italy?" the pair (what, is) is seen with thousands of different replies and (capital, Italy) with about one, yet both add a full row.

**The mechanism.**

- **The weight.** One pass over TRAIN's user turns, the same pass that counts the dictionary. For every kept pair, R is the number of different replies (the reply's tokens, from the header's last token to the token before the next user marker) of the turns that hold the pair. The pair's weight is 1/R. It is a fixed number per row, counted once at start-up and saved with the rows. It is not learned.
- **The read.** The bag of a reply is the sum over its turn's pairs of weight × row, in place of claude-370's plain sum. Everything else is claude-370's: the address (the product of the two tokens' primes), where the bag is added, the zero start, the training by the output error and the rows' own optimizer.
- **The flag.** `--pair-weight replies` with `--pair-rows MIN`. Without it the forward, the saved files and the training are claude-370's, bit for bit.

**Rules check** (CHARTER §3). Geometry only: the address stays the prime route (two token primes' product). The weight is an exact count's reciprocal, a fixed scale on each row. There is no learned gate and no squashing function, and division is allowed (D81). The read is a gather, a scale and a sum: no dense matrix product. Training runs on a GPU. Nothing is served. The control is claude-370's arm `pair` at this unit's pin, not a neural comparator.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a pair-addressed learned table weighed by reply counts, already built or already read.
- **Result (at base):** none. `crates/uor-r4-float-train/src/pair_rows.rs` sums unweighted rows. claude-367's and claude-368's pair experts (`crates/uor-r4-integer/src/pair_experts.rs`) weigh counted votes by a share and a floor, but nothing at those addresses is learned, and they are read only after the reply is formed.
- **What earlier work warns of:** claude-367 found that experts on pairs of common tokens lose held-out rows whose specific phrasing is missing. Down-weighing common pairs could cost claude-370's copying gain, which is why the bar below also guards the exact count.

## Claim

Written by hand, as claude-359 to claude-372 were. No code of this unit exists at this order's commit.

## Files in scope

- `crates/uor-r4-float-train/src/pair_rows.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `crates/uor-r4-float-train/src/geometric_stack_native.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-373/**`, `docs/harness/prep/**` (this unit's sheet), `docs/research/notebook/float-training.md`

## Done when

1. `--pair-weight replies` exists with tests that fail without the rule they pin:
   - R counts different replies, once per different reply, across turns; a pair seen only with one reply has weight 1;
   - a reply's bag is the weighted sum of its turn's rows and nothing else; without the flag the bag, the saved files and the forward are claude-370's bit for bit;
   - the weights are saved and loaded with the rows, and half of a weighted checkpoint is refused;
   - a reply decoded with cached steps equals the whole-window reply.
2. The sheet is committed with a non-author FIT before any training run.
3. The REPORT gives, for every model: exact replies on both panels, the fact and number rows (exact, and ended with the answer word), replies by kind and by need, replies that did not end, the trainer's speed, and replies word for word. Negatives come first.

## Bars

Fixed here, before any model of this unit is trained. The sheet may sharpen them before its FIT, never after a result is read.

- **Runs.** claude-370's recipe: claude-358's `p` recipe, 16,384 steps, `--pair-rows 2`. Arm `pair`: that recipe. Arm `rare`: that recipe with `--pair-weight replies`. Seeds 2 and 3. Same TRAIN, tokenizer and panels, same GPU kind for every model.
- **Panels.** P14 (1,108 scored replies) and P23 (1,110), scored in float by the trainer's own pass.
- **The number.** The fact and number rows: 101 on P14 and 87 on P23, 188 per model, as claude-370's `subst` table counts them. A row counts when the reply ends with the answer word. For each seed, `rare` minus `pair`.
- **Adds:** each seed gains 10 or more of 188, and exact replies over the two panels fall by no more than 1.5 points (33 of 2,218) at either seed.
- **Does not add:** a mean gain under 4.
- **Not called:** anything between.
- Read without a bar: exact facts, replies by kind and by need, replies that did not end, the weight census (R by quantile), steps per second.

## Stop rules

- A run that does not finish, or a scoring pass that is an instrument failure (claude-370's I-8), is repaired and run again from a new root. It is never read as a negative.
- No arm, seed or step count is added after a result is read.
