# ORDER: one-model-end-to-end, claude-370 (rows addressed by the turn's token pairs, trained by the output error)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `7068363b4cbb789b27519cea4148fd95723d6eb9`.

## Goal

**Give the float trainer one table of learned rows addressed by the exact pairs of tokens in the user's turn, train it by the output error, and read whether the trained model then answers what TRAIN teaches.**

**Why now.** Three readings point at the same gap.

- claude-367 (PR 3958): a table of pair experts counted from TRAIN lifts the good everyday replies of three saved models from 134 to 137 of 239 to 170 or 171 held out and 191 to 194 as served. The answers are in TRAIN; the trained model does not hold them.
- The binding finding (13 Nov): the replies that fail need the conjunction of two tokens of the turn ("capital" and "Italy"). The pair experts get 57 of 57 where a single-token overlap gets 11.
- claude-346 and claude-366: learned rows addressed by the model's own state, or by one token, add about 30 exact replies of 1,108 and the single-token control scores the same. Neither address names a pair.

No unit has trained a row whose address is a pair of turn tokens. Every pair key built so far (the adjacent-pair experts of ADR-0003, the last-two-atoms store, the sieve frames, claude-367's experts) holds counts, copied values or fixed frames.

**The mechanism.**

- **The address.** For every unordered pair of different tokens in one user turn, the product of the two tokens' primes (the first `vocab` primes by token id, the rule claude-367's experts use). One pass over TRAIN's user turns at start-up counts every pair; a pair seen in at least `MIN` turns gets a row. The dictionary is exact: a pair not in it reads nothing.
- **The rows.** One table, `pairs × width`, zeros at the start, so the trunk starts unchanged.
- **The read.** For each reply, the sum of the rows of the pairs in the user turn it answers, added to the residual stream once, at the first layer's seam (after its recurrence and MLP), at every position of the reply: from the last token of the assistant header to the token before the next user marker. A turn whose user marker lies outside the window names no pairs.
- **The training.** The rows learn by the trainer's own output error through the trunk, with their own optimizer at the trainer's rate. Nothing else in the recipe changes.
- **Decoding.** A cached step adds the bag the fill pass left for its row, so replies decoded step by step are the replies of the whole-window forward.

**The steps, in one unit.** Build it in `uor-r4-float-train` with tests on the GPU. Write the pre-registration sheet and get a non-author FIT. Train four models on the owner's own GPU server `wick` (two seeds, with and without the rows) and score them in float on two panels. Serving and export are a later unit, as they were for claude-346.

**Rules check** (CHARTER §3). Geometry only: the address is the project's prime route (semiprime of two token primes), the rows live in the residual stream's own space. The read is a gather and a sum: no dense matrix product, no squashing function. Training runs on a GPU. Nothing is served. No neural comparator: the control is the same recipe without the rows.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a learned table addressed by pairs of turn tokens and trained by the output error, already built or already read.
- **Result (at base):** none.
  - `crates/uor-r4-core/src/prime_route_attention.rs` keeps occurrence counts on adjacent pairs and is not joined to a model. `crates/uor-r4-training/src/stack_prime_route.rs` stores an exact value at the semiprime of the last two tagged atoms; nothing at the address is learned. `sieve_frame_compiler.rs` holds 84 fixed frames.
  - `crates/uor-r4-float-train/src/e8_address.rs` (claude-346) addresses rows by the position's own state or its own token: `apply` reads nothing of another position.
  - `docs/mechanisms/exact-memory.md` lists "the address as the model's actual read, with the store and read trained in from the start" under what has never been tested.
- **What earlier work warns of:** exact token keys did not carry to unseen phrasings in the sieve compiler (0 of 220), and a higher admission rate did not raise answers in the exact-memory line. Here a turn names many pairs, so a new phrasing still names the pairs of the words it shares; the reading by kind says whether that is enough.

## Claim

Written by hand, as claude-359 to claude-369 were: `uor-harness loop claim` refuses while this lab's worktrees hold the item. No code of this unit exists at this order's commit.

## Files in scope

- `crates/uor-r4-float-train/src/pair_rows.rs` (new), `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-370/**`, `docs/harness/prep/**` (this unit's sheet), `docs/research/notebook/float-training.md`

## Done when

1. `--pair-rows MIN` exists with tests that fail without the rule they pin:
   - with the rows at zero the forward is the forward without them, bit for bit; without the flag nothing is created or saved;
   - the bag of a reply is the sum of the rows of its turn's pairs and nothing else; it is added at reply positions only; a pair outside the dictionary and a turn cut by the window add nothing;
   - a step's gradient is zero on every row no pair of the batch names; the index tensors carry no gradient history;
   - a reply decoded with cached steps equals the whole-window reply.
2. The sheet is committed with a non-author FIT before any training run.
3. The REPORT gives the exact replies of every model on both panels, the replies by kind and by need, the replies that did not end, the trainer's speed, and replies word for word; negatives first.

## Bars

Fixed here, before any model of this unit is trained.

- **Runs.** Recipe: claude-358's `p` recipe (the recipe of the saved models p-s1 to p-s3), 16,384 steps. Arm `b`: the recipe. Arm `pair`: the recipe with `--pair-rows 2`. Seeds 2 and 3. TRAIN, the tokenizer and the place kit are claude-358's.
- **Panels.** P14: the seed-14 panel (1,108 scored replies). P23: claude-366's seed-23 panel. Both scored in float by the trainer's own scoring pass. A reply is exact when its tokens are the panel's.
- **The number.** For each seed, exact replies of `pair` minus exact replies of `b`, over the two panels together. G is the mean of the two seeds' gains, as a share of the two panels' rows.
- **Adds:** G of 3.5 points or more, each seed's gain above zero, and replies that did not end (stopped by the token cap) up by no more than 1 point on `pair`.
- **Does not add:** G under 1.5 points.
- **Not called:** anything between.
- Read without a bar: replies by kind and by need on P14, the rows a step touches, steps per second.

## Stop rules

- The recipe, the panels, `MIN` and the bars are not moved after a model of this unit is scored.
- A run starts on `wick` only when no other lab's job is on its GPU; a healthy run is never cancelled for a budget.
- No rented pod unless `wick` cannot run the job. Nothing is served as a product.
