# ORDER: one-model-end-to-end, claude-219

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `5ec6e36a8`.

## Authority (a hand-written ORDER)

`uor-harness loop claim one-model-end-to-end` was refused on 30 Oct at 18:56Z. The scope check found `docs/roadmap/items/one-model-end-to-end.md` overlapping the docs-status-refresh item of Kimi's draft PR #3436. Its glob `crates/*/README.md` also blocked `integer-hamming-read`'s crate files.

**Casey, in claude's chat, 30 Oct (verbatim):** "screw kimi forge ahead as you were before with your plan for the single serving model".

So this ORDER is written by hand on the owner's word, as the umbrella exception's ORDERs are. **This unit does not touch any file Kimi's ORDER edits:**
- `README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, `PLAN.md`, `AGENTS.md` and `CHARTER.md`;
- `docs/README.md`, `docs/SUMMARY.md`, and every `crates/*/README.md`;
- the item file `docs/roadmap/items/one-model-end-to-end.md`;
- `docs/harness/milestones.json` and `docs/harness/claims.json`.

Kimi's ORDER, for its part, excludes Claude's active code, prep, lab, task and notebook files.

## Goal

**Serve the trunk memory key in free generation: does a key's opening carry the reply?**

**Where claude-218 left it** (merged as `5ec6e36a8`):
- **The registered row 3 shipped the sign key.** On the fresh panel p-u's missed rows, teacher-forced, it gains +46/+37/+33 over the trunk alone.
- **The point key gains about 3.7 times as much** (+125/+185/+114).
- **A forced right first token** lifts exact replies from 4/0/0 to 54/49/52 of 256.

**This unit: the keyed opening.**
- **The mechanism:** at a reply's opening where the memory's exact key misses, a trunk key names the first token when its distance is within its τ. That is the target its nearest TRAIN openings agree on, the target the memory's move would land on.
- **Then** greedy continues. Elsewhere the decode is greedy with the memory, as served.
- **Run by** `--score-rows … --keyed-decode 1`, which computes the keys before decoding.
- **The arms,** on a fresh held-out-phrasing panel p-v (seed 2111) with its census, DEV reported, (r-drop) at 3 seeds:
  - greedy;
  - the forced opening (the ceiling);
  - keyed-sign (claude-218's ship);
  - keyed-point;
  - keyed-hsign.
- **The τ values** come from claude-218's committed levels, recomputed on p-s, its development panel.

**The decision table** comes in the prep sheet, with a fresh FIT. In outline:
- the keyed arm that carries the reply beyond its band on exact and template matches, at every seed, without more loops or a DEV loss, is what the next unit trains and serves;
- if none carries though the forced arm does, the next unit trains the trunk's opening point to gather paraphrases.

## Cancel probe (run first)

Run 30 Oct, 19:40Z. **Result: instrument, all clear; the unit goes ahead.**
- **No open PR touches the in-scope code.** Of the open PRs, #3436 (Kimi) touches `crates/uor-r4-float-train/README.md` and the item file, both excluded above; #3323 (Antigravity) touches nothing in scope.
- **The inputs are in the store** (`uor-store ls`): (r-drop)'s step-16,384 checkpoints at seeds 1–3 under `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{S}/models/step-016384/`, and the panels p-s and p-u.
- **Seed 2111 and the name p-v are unused.** The panels drawn so far are p-dev, p-ret, p-train, p-f, p-f-train, p-c, p-r, p-s, p-t and p-u. Their seeds are 2001, 2026, 2051, 2052, 2061, 2071, 2081, 2091 and 2101.
- **claude-218's τ levels are committed** in `docs/labs/one-model-end-to-end/memory-key-2026-10-30/grade-p-s-levels.json`:
  - sign: 0.0078125, 0.01953125 and 0.0078125 at seeds 1–3;
  - hsign: 0 at every seed;
  - point: unbounded (null), at every seed.

  So the keyed-point arm fires on every opening the exact key misses, and keyed-hsign only at distance 0.
- **Not checked here:** whether the forced arm still lifts on p-v. That is the decision table's instrument row, read from the run.

## Files in scope

As the item's scope, less the item file itself (see Authority):
- `crates/uor-r4-float-train/src/{in_space_long.rs,bin/train-in-space.rs,flock_read_hamming.rs,flock_read_memory.rs}`;
- `crates/uor-r4-training/examples/m-world.rs`;
- `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md`.

## Done when

1. **`--keyed-decode 1` is built and unit-tested:**
   - the keys are computed before the decode loop;
   - each sampled prompt's opening is mapped to its key's name and distance;
   - the keyed arms force that name only where the exact key misses and the key fires within τ.
2. **A fresh panel (seed 2111) and its census.**
3. **A prep sheet, with a fresh FIT and a fresh pre-run review.**
4. **The GPU decode at three seeds:** sealed, graded by the sheet's table, and reported, negatives first and tokens per second first. The registered files must reproduce byte for byte.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** a registered file does not reproduce, a keyed reply does not open with its key's name where it should, or a key's name differs from claude-218's for the same opening. Repair it; it is never a negative (CHARTER §4.1).
- **Kimi's #3436 changes a file in this scope:** merge main and resolve; never edit Kimi's files.
