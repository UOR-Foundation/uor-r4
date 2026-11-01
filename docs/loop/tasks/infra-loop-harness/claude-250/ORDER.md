# ORDER: infra-loop-harness, claude-250 (the files F's and T's trace sides need)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), the umbrella that edits roadmap items. Base: `03800ffd2`.

Setup row: 2 (item-file maintenance: a scope change to a gap item).

## Goal

This unit corrects [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md)'s `scope:` (#3462) for the next part of Q5's trace instrument: F's and T's sides, under claude-247's [decision card](../../integer-nearest-readout/claude-247/DECISION.md). The guard enforces an item's scope as it stands on `main`, so the scope must merge before the code that needs it (claude-248's precedent).

**Why.** Casey's call on 1 Nov (Claude's chat, recorded on #3462, `6564833770`): "Finish Q5 first". R's side merged in claude-249 (#3499). F's and T's sides record the card's five kinds from `train-in-space --score-rows`, on the GPU. A read-only scout of `origin/main` found that three of the files they need are not in the scope.

**Three files added, each named by what the card requires of it:**
- `crates/uor-r4-float-train/src/flock_read_memory.rs`: kind 3, the turn memory's lookup. `TurnMemory::stored` (`table.get(&key)`) and `stored_reply` run inside `apply_readout`, so the key, what was stored, and whether the row is the opening are known only there. T's memory must also be rounded as exported, and no helper does that today.
- `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`: kind 5, the rate bucket. `rate_select_fwd` is the op that launches `cuda_stack.cu`'s selection kernel. The card records the entry "as the decision reads it", and forces it, so the op must carry the recorded and forced entries.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: the options. `--score-rows`'s arguments are parsed there, so a `--decision-trace` and a `--force` option are too.

**Two files removed:** `crates/uor-r4-integer/src/wasm_serve.rs` and `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`.
- The browser readout they carry was built in claude-231. Q5's remaining work (F, T, the replay driver, the grader and the sheet) touches neither file.
- Both are in an open Studio item's scope: `m7-studio-live-model-sources`, whose orders kimi-1 to kimi-3 are open on PR #3509. While both items list them, neither lab can claim the other's item.
- A later unit that needs either file adds it back the same way.

## How this unit was claimed

`uor-harness loop claim infra-loop-harness --lab-topic inr-scope` refused at 22:59Z on 1 Nov: "scope docs/roadmap/items/*.md overlaps docs/roadmap/items/cpu-discrete-writer-core.md of item cpu-discrete-writer-core (PR #3511 …)". Before that, Kimi's #3509 held it the same way. The umbrella's `docs/roadmap/items/*.md` overlaps every open item's own file, so this item cannot be claimed while any other lab has an order open.

Casey, in Claude's chat, 1 Nov, about 23:00Z, verbatim: "I told all of the labs never to hold you up because you are doing work on the main track, if kimi is holding on to something or any other lab is, you may supercede them to get whatever done".

So this unit was claimed by hand, with the writes `loop claim` makes (`loopclaim.rs` `write`): the worktree `.worktrees/claude-inr-scope`, the branch `claude/inr-scope` off `origin/main` at `03800ffd2`, this ORDER, the claim commit, the push and a draft PR. Every other check of `loopclaim.rs` `check` passed by hand:
- (a) the item is on `origin/main` with `status: ready`;
- (b) no open order names it: the only open PRs are #3509 (Kimi, `m7-studio-live-model-sources`) and #3511 (DeepSeek, `cpu-discrete-writer-core`);
- (c) the item has a `scope:` line, and it lists no shared file;
- (e) Claude has no open PR.

Only (d), the overlap, was superseded. **No open PR changes a file this unit edits.** #3509's and #3511's file lists hold neither `integer-nearest-readout.md` nor any of the five files named above. #3509 changes `docs/roadmap/README.md` and `docs/roadmap/issues.json`, which are shared files open to every loop PR, so a conflict there is resolved by re-rendering. Nothing of Kimi's or DeepSeek's is edited, closed or pushed to.

## Cancel probe (run first)

- **Are the three files already in the item's scope?** None of them is, at the base.
- **Does an open order hold any of the five?** Check every open PR's ORDER and the lab worktrees. A shared file is allowed: the claim's overlap check orders the units.
- **Do the three files exist at the base?** Check each with `git cat-file -e`.
- **Does Q5's remaining work need either WASM file?** Check the card's sides and the item's "What is missing".
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-250/**`

## Done when

1. **The item's `scope:`** gains the three files and loses the two, and nothing else changes in it. Its "What is missing" names the change.
2. **`cargo xtask roadmap check`** passes, and the render is current.
3. **A fresh exact-head review passes.** It checks each added file against the card's kinds and the code that runs them, and each removed file against Q5's remaining work.
4. **`loop report --check`, `loop pr-check --local` and `loop gates --full`** pass.

## Stop and decide (never wait) if

- the review finds that F's or T's side needs a file not named here. Then that file needs its own reason, and this unit adds only the three;
- another open ORDER holds one of the files.
