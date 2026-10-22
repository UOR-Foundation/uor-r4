# M2 Fresh Held-Out Reply Panel (`m2-fresh256`, `#2849`)

This lab note records the development-panel (`m2-reply512`) noise audit on the sealed `saved437` parent (`de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61`), the construction and overlap-exclusion audit of the replacement 256-row held-out reply panel (`m2-fresh256`), and the sealed artifact digests registered on [`#2030`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6347165682).

## 1. Why a replacement fresh panel was needed

On 10 October 2026, the `saved437` native cross-state field (`de4a3234…`) scored `437 / 512` complete correct replies on the frozen development panel (`m2-reply512`, `b9661606…`) but `0 / 128` (`124` output-limit stops, `4` wrong predictions) on the single fresh transfer draw (`c5a643ff…`; see [`archive/labs/m2-fresh-437-2026-10-10/README.md`](../../../archive/labs/m2-fresh-437-2026-10-10/README.md)). Two defects prevented reusing that 128-row draw for future M2 qualification:

1. **Exposure and size:** The 128-row draw was opened during the 10 October audit (`0 / 128`), reused `15` of the `24` development literals (`FRESH_VALUES[0..8]` plus `VALUES[8..23]` from `crates/uor-r4-core/src/native_geometric/curriculum.rs`), and had only `128` rows (below the `>= 200` fresh-row floor in `CHARTER.md` §4.3 / D22 §3).
2. **Unstated development noise:** `docs/harness/panels/m2-reply512.json` recorded `"noise": "unstated"`, leaving no measured spread or cluster-sampling standard error against which to size the fresh panel and verify the `>= 40%` acceptance bar.

## 2. Measured noise on `m2-reply512` (`saved437` parent)

The standalone pure-integer Rust instrument in [`instrument/`](./instrument) reconstructs the exact `256` development histories (`512` question rows across `32` sixteen-row unordered bank blocks) from the deterministic round-robin specification (`archive/labs/m2-cross-state-2026-10-10/geometric-native-bank-observation-plan.rs`) and cross-checks them against [`archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json`](../../../archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json) (`.endpoints.final.complete_ids`, `437` IDs) and [`archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json`](../../../archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json):

| Quantity | Measured value | Notes |
|---|---|---|
| Complete replies on `m2-reply512` | `437 / 512` (`85.35%`) | `length2: 116/128`, `length4: 114/128`, `length8: 90/128`, `update: 57/64`, `reassert: 60/64` |
| Evaluator replay spread | `0 / 512` (`0.00%`) | Greedy integer realizer and membership-only grader are 100% deterministic |
| Histories with both `job` and `home` complete | `186 / 256` | `65` discordant histories (one role passes, one fails); `5` histories fail both roles |
| Chronology spread (`forward` vs `reverse`) | `212 / 256` vs `225 / 256` | `\|\Delta\| = 13` rows across `256` pairs (`61` discordant pairs) |
| Role-swap spread (`swap0` vs `swap1`) | `215 / 256` vs `222 / 256` | `\|\Delta\| = 7` rows across `256` pairs (`65` discordant pairs) |
| Query-wording spread (`wording_a` vs `wording_b`) | `214 / 256` vs `223 / 256` | `\|\Delta\| = 9` rows across `256` pairs (`57` discordant pairs) |
| Queried-role spread (`job` vs `home`) | `219 / 256` vs `218 / 256` | `\|\Delta\| = 1` row across `256` histories (`65` discordant histories) |
| Stratified half-panel split (`16` even vs `16` odd blocks of `256` rows) | `219 / 256` vs `218 / 256` | `85.55%` vs `85.16%` (`\|\Delta\| = 1` row / `0.39%`) |
| Empirical 16-block (`256-row`) cluster SE at `p = 85.35%` | `6.24` rows (`244 bps`) | Design effect $\text{DEFF} \approx 1.22$ vs i.i.d. binomial SE `5.66` rows |
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

### Overlap exclusion and structural balance verification

`verify_m2_fresh256_panel()` in [`instrument/src/lib.rs`](./instrument/src/lib.rs) enforces the following checks against all `72` historical exposed literals (`VALUES[0..32]` in `curriculum.rs`, `FACTOR_VALUES[0..8]`, `FRESH_VALUES[0..8]`, `FRESH2_VALUES[0..8]`, `FRESH3_VALUES[0..8]`, and the `8` untouched transfer literals in `geometric-native-bank-observation-plan.rs`, covering `m2-reply512` `b9661606…` and the opened 128-row draw `c5a643ff…`):

- **Exact literal overlaps:** `0`
- **Content-word overlaps:** `0`
- **Word 2-gram overlaps:** `0`
- **Minimum same-stratum character Levenshtein edit distance:** `10` (enforced floor `>= 8`)
- **Literal role balance (`job` vs `home`):** `true` (every literal appears an equal number of times in `job` and `home`)
- **Literal initial-position balance (`turn 0` vs `turn 1`):** `true` (every literal appears an equal number of times in turn 0 and turn 1)
- **Query wording balance:** all `6` supported current-role query templates per role are exercised, with max count spread `4` (`<= 4`)
- **Token budgets:** `max_turn_tokens = 13` (`<= 128`), `max_target_tokens = 10` (`<= 32`), `max_total_replay_tokens = 59` (`<= 128`)
- **Check-only heuristic baselines:**
  - `copy_first_stated`: `120 / 256` (`46.88%`)
  - `copy_last_stated`: `128 / 256` (`50.00%`)
  - `role_swap_distractor`: `0 / 256` (`0.00%`)
  - `best_constant_reply`: `16 / 256` (`6.25%`)
- **Unscored runtime input seal:** `inputs.json` contains `0` occurrences of `"accepted"`.

## 4. Sealed artifact digests and storage custody

Per [`docs/reference/storage.md`](../../reference/storage.md) and `#2849` step 4, the corpus bytes are not committed to git; the repository carries the frozen panel entry ([`docs/harness/panels/m2-fresh256.json`](../../harness/panels/m2-fresh256.json)) and the deterministic Rust generator/verifier ([`instrument/`](./instrument)), while the sealed bytes are staged at `local/antigravity/m2-fresh256-sealed/` for maintainer upload to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`:

| Artifact | Bytes | SHA-256 | Purpose |
|---|---:|---|---|
| `plan.json` | `122,745` | `698d3cf2d7208f3d66f85a1b3a77727f6380c785964f914a78ab9608cdbcba4a` | Construction plan (`uor-r4.raw-natural-reader-construction-plan/1`) |
| `inputs.json` | `410,531` | `f6f6732e1958568c3ec6aed7262045e8f447d111a71abcad54d0ca2a5950a06c` | Unscored runtime inputs (`uor-r4.native-source-bank-probe-input/1`) |
| `labels.json` | `52,573` | `ecb99eb0b196b6ba2c56e6ede5556d90b35391ffe4dc13fb6fea6b97cc444842` | Sealed oracle labels (`uor-r4.native-source-bank-labels/1`, `uor-r4.literal-role-dialogue/2`) |
| `manifest.json` | `1,183` | `4ccf1e32ad86e81c243c9c0ade18cbdd90077e3e07fd854ba01aeee5b8873779` | Bundle manifest (`uor-r4.m2-fresh256-panel-manifest/1`, `model_predictions: NOT_RUN`) |

Reproduce the audit and emit the sealed files with:

```sh
CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo test \
  --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked
CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo run \
  --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked \
  -- --emit-dir local/antigravity/m2-fresh256-sealed
```

## 5. Non-author TEST FITNESS review

A fresh-context non-author reviewer (`3f74630d-7674-4958-a496-38b96b34ab18`) audited the `m2-reply512` noise spread (`0/512` replay diff, `219/256` vs `218/256` half-panel split, `6.24`-row empirical cluster SE and `8.64`-row cluster SE at the `40%` `103/256` bar), the overlap exclusion against all `72` historical exposed literals (`0` exact, `0` content-word, `0` word 2-gram overlaps, minimum same-stratum Levenshtein edit distance `10 >= 8`), the deliberate-break negative tests, the unchanged deterministic complete-reply grader (`uor-r4.literal-role-dialogue/2`), and the sealed `inputs.json`/`labels.json`/`plan.json`/`manifest.json` SHA-256 digests, returning:

- **Verdict:** `TEST FITNESS: FIT` (before any candidate scores against `m2-fresh256`; M2 training remains `HELD` until the owner releases it).

## 6. Research notebook row (M2 grounded reply)

*(Recorded here under `docs/labs/m2-fresh-heldout-panel/README.md` because `docs/roadmap/items/m2-fresh-heldout-panel.md` scopes paths to `[data/**, docs/harness/panels/**, docs/mechanisms/native-geometric.md, docs/labs/m2-fresh-heldout-panel/**, docs/reference/storage.md]`; ready to copy verbatim into `docs/research/notebook/m2-grounded-reply.md` once that path is added to an item scope.)*

| Date | Base -> new | Mechanism | Numbers (before -> after) | Cost | Verdict & evidence |
|---|---|---|---|---|---|
| 2026-10-22 | `m2-reply512` (`b9661606…`) + opened `fresh128` (`c5a643ff…`) -> frozen `m2-fresh256` (`f6f6732e…`) | [native-geometric](../../mechanisms/native-geometric.md) | `saved437` dev512 replay spread `0/512`, half-panel split `219/256` vs `218/256`, 16-block cluster SE `6.24` rows (`8.64` rows at `40%` bar); `m2-fresh256` sealed at `256` rows (`128` histories, `16` blocks, `0` exact/word/2-gram overlaps against `72` exposed literals, min edit distance `10`), bar `>= 103/256` (`NOT_RUN`) | $0 (host Rust) | **instrument frozen** — [`m2-fresh256.json`](../../harness/panels/m2-fresh256.json), [`#2030`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6347165682), `#2849` |
