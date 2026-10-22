# M2 Fresh Held-Out Reply Panel (`m2-fresh256`, `#2849`)

This lab note records the single-model (`saved437`) within-panel spread audit on the development panel (`m2-reply512`), the construction and overlap-exclusion audit of the replacement 256-row held-out reply panel (`m2-fresh256`), and the sealed artifact digests registered on [`#2030`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6347165682).

## 1. Why a replacement fresh panel was needed

On 10 October 2026, the `saved437` native cross-state field (`de4a3234…`) scored `437 / 512` complete correct replies on the frozen development panel (`m2-reply512`, `b9661606…`) but `0 / 128` (`124` output-limit stops, `4` wrong predictions) on the single fresh transfer draw (`c5a643ff…`; see [`archive/labs/m2-fresh-437-2026-10-10/README.md`](../../../archive/labs/m2-fresh-437-2026-10-10/README.md)). Two defects prevented reusing that 128-row draw for future M2 qualification:

1. **Exposure and size:** The 128-row draw was opened during the 10 October audit (`0 / 128`), reused `15` of the `24` development literals (`FRESH_VALUES[0..8]` plus `VALUES[8..23]` from [`crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs`](../../../crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs)), and had only `128` rows (below the `>= 200` fresh-row floor in `CHARTER.md` §4.3 / D22 §3).
2. **Unstated development noise:** `docs/harness/panels/m2-reply512.json` recorded `"noise": "unstated"`, leaving no measured within-panel spread or cluster-sampling standard error against which to size the fresh panel and verify the `>= 40%` acceptance bar.

## 2. Measured within-panel spread on `m2-reply512` (`saved437` parent)

The standalone pure-integer Rust instrument in [`instrument/`](./instrument) reconstructs the exact `256` development histories (`512` question rows across `32` sixteen-row unordered bank blocks) from the deterministic round-robin specification ([`crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs`](../../../crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs)) and cross-checks them on disk against [`archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json`](../../../archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json) (`.endpoints.final.complete_ids`, `437` IDs) and [`archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json`](../../../archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json). Because only a single M2 checkpoint (`saved437`) exists, multi-seed / checkpoint-to-checkpoint training spread remains `unstated`; the table below records the single-model (`saved437`) deterministic replay spread and within-panel paired-variant / 16-block cluster-sampling spread:

| Quantity | Measured value | Notes |
|---|---|---|
| Complete replies on `m2-reply512` (`saved437`) | `437 / 512` (`85.35%`) | `length2: 116/128`, `length4: 114/128`, `length8: 90/128`, `update: 57/64`, `reassert: 60/64` |
| Deterministic evaluator replay spread | `0 / 512` (`0.00%`) | Greedy integer realizer and membership-only grader are 100% deterministic; multi-seed training spread is `unstated` (`1` checkpoint exists) |
| Histories with both `job` and `home` complete | `186 / 256` | `65` discordant histories (one role passes, one fails); `5` histories fail both roles |
| Chronology spread (`forward` vs `reverse`) | `212 / 256` vs `225 / 256` | `\|\Delta\| = 13` rows across `256` pairs (`61` discordant pairs) |
| Role-swap spread (`swap0` vs `swap1`) | `215 / 256` vs `222 / 256` | `\|\Delta\| = 7` rows across `256` pairs (`65` discordant pairs) |
| Query-wording spread (`wording_a` vs `wording_b`) | `214 / 256` vs `223 / 256` | `\|\Delta\| = 9` rows across `256` pairs (`57` discordant pairs) |
| Queried-role spread (`job` vs `home`) | `219 / 256` vs `218 / 256` | `\|\Delta\| = 1` row across `256` histories (`65` discordant histories) |
| Stratified half-panel split (`16` even vs `16` odd blocks of `256` rows) | `219 / 256` vs `218 / 256` | `85.55%` vs `85.16%` (`\|\Delta\| = 1` row / `0.39%`) |
| Single-model 16-block (`256-row`) cluster SE at `p = 85.35%` | `6.24` rows (`244 bps`) | Design effect $\text{DEFF} \approx 1.22$ vs i.i.d. binomial SE `5.66` rows |
| Cluster-adjusted SE at the `40%` (`103 / 256`) acceptance bar | `8.64` rows (`338 bps`) | vs i.i.d. binomial SE `7.84` rows (`306 bps`); two-sided $2\sigma$ band $\pm 17$ rows ($\pm 6.6\%$) |

## 3. Panel design and overlap exclusion (`m2-fresh256`)

`m2-fresh256` comprises `256` fresh question rows (`128` multi-turn histories across `16` sixteen-row unordered role-value bank blocks, `32` ordered `(job, home)` banks, and `128` opposite-query pairs), preserving the exact 5-stratum proportions of `m2-reply512` at half scale (`>= 200` rows per `CHARTER.md` §4.3):

| Stratum | Unordered blocks | Histories | Question rows | Target literal pool | Structure |
|---|---:|---:|---:|---|---|
| `length2` | `4` | `32` | `64` | `FRESH256_LEN2` (`10` two-word literals) | `2` assertion turns (`job`, `home`) + `2` queries |
| `length4` | `4` | `32` | `64` | `FRESH256_LEN4` (`10` four-word literals) | `2` assertion turns (`job`, `home`) + `2` queries |
| `length8` | `4` | `32` | `64` | `FRESH256_LEN8` (`10` eight-word literals) | `2` assertion turns (`job`, `home`) + `2` queries |
| `update` | `2` | `16` | `32` | `FRESH256_LEN4` overwriting `FRESH256_UPDATE_INIT` (`4` two-word literals) | Initial `2`-word assertion overwritten by `4`-word `update` (`4` turns) + `2` queries |
| `reassert` | `2` | `16` | `32` | `FRESH256_LEN2[..8]` | `3` assertion turns where turn 2 reasserts turn 0 + `2` queries |
| **Total** | **`16`** | **`128`** | **`256`** | **`34` fresh literals (`30` targets + `4` overwritten initials)** | **Bar: `>= 103 / 256` (`40.23%`)** |

### Overlap exclusion, consumer schema, and control verification

`verify_m2_fresh256_panel()` in [`instrument/src/lib.rs`](./instrument/src/lib.rs) parses the actual repository source files on disk ([`crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs`](../../../crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs) for all `64` curriculum literals across `VALUES[0..32]`, `FACTOR_VALUES[0..8]`, `FRESH_VALUES[0..8]`, `FRESH2_VALUES[0..8]`, `FRESH3_VALUES[0..8]`, and [`crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs`](../../../crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs) for the `10` untouched transfer literals = `74` historical literals total, cross-checked against [`archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json`](../../../archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json)) and verifies:

- **Exact literal overlaps against all `74` historical literals:** `0`
- **Content-word overlaps against all `74` historical literals:** `0`
- **Word 2-gram overlaps against all `74` historical literals:** `0`
- **Intra-panel duplicate content words across all `34` fresh literals (`107` distinct content words):** `0`
- **Minimum same-stratum character Levenshtein edit distance:** `10` (enforced floor `>= 8`)
- **Literal role balance (`job` vs `home`):** `true` (every literal appears an equal number of times in `job` and `home`)
- **Literal initial-position balance (`turn 0` vs `turn 1`):** `true` (every literal appears an equal number of times in turn 0 and turn 1)
- **Query wording balance:** all `6` supported current-role query templates per role are exercised, with max count spread `4` (`<= 4`)
- **Pinned `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`) token budgets and `StackStore` (`record = commit = event + 1`) schema:** `max_turn_tokens = 26` (`<= 128`), `max_target_tokens = 19` (`<= 32`), `max_total_replay_tokens = 105` (`<= 128`); both `inputs.json` and `labels.json` round-trip through `#[serde(deny_unknown_fields)]` consumer structs matching [`crates/uor-r4-core/examples/native-bank-generalization.rs`](../../../crates/uor-r4-core/examples/native-bank-generalization.rs).
- **Check-only question-blind and role-swap controls under history-pair completion (`pair_id` in `labels.json`, `pairs_both_complete_distinct` in `native-bank-generalization.rs`):**
  - `copy_first_stated`: `0 / 256` pair-verified rows (`0 / 128` histories, `< 103`; raw per-row `120 / 256` because a question-blind turn-0 copy answers both `job?` and `home?` identically)
  - `copy_last_stated`: `0 / 256` pair-verified rows (`0 / 128` histories, `< 103`; raw per-row `128 / 256` because a question-blind last-turn copy answers both `job?` and `home?` identically)
  - `role_swap_distractor`: `0 / 256` pair-verified rows (`0 / 256` raw per-row)
  - `best_constant_reply`: `0 / 256` pair-verified rows (`16 / 256` raw per-row)
- **Unscored runtime input seal:** `inputs.json` contains `0` occurrences of `"accepted"`.

## 4. Sealed artifact digests and storage custody

Per [`docs/reference/storage.md`](../../reference/storage.md) and `#2849` step 4, the corpus bytes are not committed to git; the repository carries the panel entry ([`docs/harness/panels/m2-fresh256.json`](../../harness/panels/m2-fresh256.json)) and the deterministic Rust generator/verifier ([`instrument/`](./instrument)), while the sealed bytes are staged at `local/antigravity/m2-fresh256-sealed/` for maintainer upload to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`:

| Artifact | Bytes | SHA-256 | Purpose |
|---|---:|---|---|
| `plan.json` | `122,268` | `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715` | Construction plan (`uor-r4.raw-natural-reader-construction-plan/1`) |
| `inputs.json` | `524,850` | `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a` | Unscored runtime inputs (`uor-r4.native-source-bank-probe-input/1`) |
| `labels.json` | `68,000` | `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be` | Sealed oracle labels (`uor-r4.native-source-bank-labels/1`, `uor-r4.literal-role-dialogue/2`, with `pair_id` on every case) |
| `manifest.json` | `1,273` | `c930e1f0275ea0fc8c8b2afbb6062546ff11331c1b7fdfeb9ec75eca97e456bd` | Bundle manifest (`uor-r4.m2-fresh256-panel-manifest/1`, `model_predictions: NOT_RUN`) |

Reproduce the audit and emit the sealed files with:

```sh
CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo test \
  --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked
CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo run \
  --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked \
  -- --emit-dir local/antigravity/m2-fresh256-sealed
```

## 5. Non-author TEST FITNESS status

Per `CHARTER.md` §4.5 and `#2849` steps 4–5, `docs/harness/panels/m2-fresh256.json` is registered with `"frozen": false` and `TEST FITNESS: PENDING` until:

1. A separate non-author lab session records a formal `TEST FITNESS: FIT` review of the overlap exclusion (`0` exact/word/2-gram overlaps against all `74` historical literals, `0` intra-panel duplicate content words), the exact `ByteBpeTokenizer` (`d36d3e87…`) encoding, the history-pair completion control audit (`0 / 256` across all 4 question-blind and role-swap controls), and the content seal; and
2. A maintainer with HF write access uploads `plan.json`, `inputs.json`, `labels.json`, and `manifest.json` from `local/antigravity/m2-fresh256-sealed/` to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`.

M2 training and evaluation remain `HELD` (`NOT_RUN`) until the owner releases a pre-registered M2 sheet.

## 6. Research notebook row (M2 grounded reply)

*(Recorded here under `docs/labs/m2-fresh-heldout-panel/README.md` because `docs/roadmap/items/m2-fresh-heldout-panel.md` scopes paths to `[data/**, docs/harness/panels/**, docs/mechanisms/native-geometric.md, docs/labs/m2-fresh-heldout-panel/**, docs/reference/storage.md]`; ready to copy verbatim into `docs/research/notebook/m2-grounded-reply.md` once that path is added to an item scope.)*

| Date | Base -> new | Mechanism | Numbers (before -> after) | Cost | Verdict & evidence |
|---|---|---|---|---|---|
| 2026-10-22 | `m2-reply512` (`b9661606…`) + opened `fresh128` (`c5a643ff…`) -> staged `m2-fresh256` (`1de2fc28…`) | [native-geometric](../../mechanisms/native-geometric.md) | `saved437` dev512 replay spread `0/512`, half-panel split `219/256` vs `218/256`, single-model 16-block cluster SE `6.24` rows (`8.64` rows at `40%` bar); `m2-fresh256` staged at `256` rows (`128` histories, `16` blocks, `0` exact/word/2-gram overlaps against `74` historical literals, `0` intra-panel duplicate content words, min edit distance `10`, `0/256` on all 4 controls under `pair_id`), bar `>= 103/256` (`NOT_RUN`, `TEST FITNESS: PENDING`) | $0 (host Rust) | **instrument staged** — [`m2-fresh256.json`](../../harness/panels/m2-fresh256.json), [`#2030`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6347165682), `#2849` |
