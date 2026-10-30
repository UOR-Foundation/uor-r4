# FACTS: documentation status at 30 Oct 2026

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). ORDER: [ORDER.md](ORDER.md). Base: `ec9a790fe5be73cd673f082eba86ced551b22d12`.

Every public number in this refresh must link to this sheet's committed evidence. This is a factual baseline, not a new measurement or a capability claim.

## 1. Milestone state and the M1 artifact

- **Project stage:** pre-alpha, **1 of 8 milestones complete**. M1 #2029 is CLOSED. M2 and M3 are in progress; M4–M8 are not started. No useful conversation or product release is established.
- **Closing artifact:** antigravity-110's saved `trunk-joint-pointer-v1-d11`, a 19-table `uor-r4.lut-discrete-trunk/1` discrete model, exported as two seeded `UORLUT01` files:
  - seed 1 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` (86,536,132 B);
  - seed 2 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de` (84,943,328 B).
- **Composition:** Flock `FrozenZeroV1`, discrete Hamilton gate, discrete quaternion recurrence, four-block orbit-root causal pointer, and post-rotation outer-L1 readout. This is not the separately float-trained FBS trunk.
- **Evidence split:** `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` covers bars (a)–(e) and is not called six-bar evidence. `docs/labs/m1-heldout-phrasing-transfer/evidence/panel-score.json` supplies bar (f). #2029 and `docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md` join them into the six-bar close.

### M1 measured results

| Panel/contract | Seed 1 | Seed 2 | Controls and scope | Source |
|---|---:|---:|---|---|
| DEV integer top-1 | **7,002 / 14,287** | **7,001 / 14,287** | unigram 1,226; T2 6,300; both seeds beat both | [M1 close record](../../../../labs/m1-close-saved-model/README.md) |
| Fresh V2 integer top-1 | **7,215 / 14,479** | **7,228 / 14,479** | unigram 1,279; T2 6,582; paired band 82 | [Fresh transfer](../../../../labs/m1-fresh-panel-transfer/README.md) |
| p-f held-out phrasing | **7,850 / 15,953** | **7,872 / 15,953** | unigram 1,409; T2 7,375; leads 475/497 above paired band 81 | [bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| Export/reference agreement | 14,287 / 14,287 | 14,287 / 14,287 | zero export gap; planted-fault B4 passes | [close grade](../../../../labs/m1-close-saved-model/evidence/m1-close-grade.json) |
| Apple M1 decode | **10,876 tok/s** | **10,566 tok/s** | median; 192-token prefill and 64-token decode; about 2.67M prefill tok/s | [bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md) |

- **What bar (f) proves:** p-f has 0.000 of its prompt value-masked templates in TRAIN at K = 1,024 (0.050 at K = 256), so the saved model reads new prompt phrasings into known kinds of reply.
- **The limit that must stay beside it:** **0.625 (62.5%) of p-f's reference replies are verbatim TRAIN replies**. That is reference-data overlap, not a generated-copy rate, not new reply kinds, and not useful-chat qualification.
- **M1's close is reversible through the project record, not by documentation.** Documentation propagates the already reviewed close; it does not create it.

## 2. Separate FBS trunk and current open research

- The float-trained FBS trunk is a different path from the saved discrete M1 model. `lut-stack/5` exports/serves it exactly at DEV scope; D80/D81 allow integer multiply and divide while floats remain banned in serving. Its earlier M1 rates were 4,217–4,504 tok/s; do not mix those rates with the saved M1 artifacts' 10,876/10,566 tok/s.
- **Latest merged result: claude-217**, `docs/loop/tasks/one-model-end-to-end/claude-217/REPORT.md`:
  - The cycle/echo steps end every registered loop, but every content decision is **underpowered (content)**; greedy remains the served decode. Loop removal is not a quality result.
  - On held-out p-s, exact-reference replies are only 3, 1 and 4 of 256 at seeds 1–3, against 38–40 on DEV.
  - Teacher-forced p-s reply openings are **17.4%, 13.7%, 18.6%**, against **80.8%, 80.2%, 79.9%** on later tokens. On p-t the openings are 18.7%, 12.2%, 17.9% against 80.8–81.1%.
  - Exact turn memory keys on the turn's exact first token ids, so a new phrasing of a known question can find no stored key. On p-s it adds only 14–16 of 1,400 openings; on DEV it adds 322–337 of 1,226.
  - The working inference is that new-phrasing reply opening is the larger failure, not merely decode looping. The next design must address that opening/memory-key problem; no unmerged claude-218 result is claimed.
- **D67:** no blanket owner release holds every training step. The D41 donor pass is the remaining named owner-at-dispatch exception. Do not restore retired "every training step is held" language.

## 3. Historical facts to preserve, not present as current

- **K16, 19 Oct:** the selected discrete K16-A seeds scored 65 and 463 of 14,287, below unigram 1,226 and T2 6,300. K16-R collapsed to 19/20 cells. Keep this as dated negative evidence, never as the current best model.
- **Earlier M2 headline:** 437/512 on seen rows fell to 0/128 on a fresh draw. It is a panel-fitting lesson, not qualified progress toward M2's current saved-model acceptance.
- **Four 214M Hugging Face checkpoints:** historical models that used softmax and matrix multiplication; they are not the D11 product and not evidence for today's rules.
- Dated `docs/labs/**`, prior `docs/loop/tasks/**`, notebook rows, `docs/integration/current-state.md` and `archive/**` stay unchanged. `STATUS.md` and `docs/integration/NOW.md` are retired pointer pages; current plan/status goes to `PLAN.md` and rendered `STATE.md`/`HANDOFF.md`.

## 4. Product state

- **Model:** trained integer artifacts exist, including the M1 pair, but no versioned product release ships and no useful-chat model is qualified.
- **Web:** the Studio and `uor-r4-integer` worker are connected and verified on a synthetic stand-in; page assets and a stack-bound tokenizer are self-hosted. No qualified model is bundled and no M7 product parity is claimed. The old Studio deploy remains paused; the mdBook workflow publishes preview on main and production only by manual dispatch, which this unit does not perform.
- **Desktop:** the Tauri shell exists and is verified with durable-memory/session behavior on the 469,252-byte `mk-synthetic-stack` seed-5 artifact. It is not merely a specification, but it has no qualified trained product bundle.
- **M4:** a lawful likelihood (`PRLL`) is built and registered, but the GPU held-out scoring and full accepted M4 model/CLI path remain open. Do not say no lawful likelihood exists, and do not claim M4 is complete.
- **M7:** runtime/harness work on the synthetic stand-in is real; accepted-model distribution is not complete.

## 5. Mechanism and rule state

- The board has **28 mechanisms: 12 ACTIVE, 5 READY, 11 PARKED**. A whole-model M1 pass does not establish each mechanism's independent effect.
- **D11:** integer serving may use hardware integer multiply and divide (D80/D81); floats, transcendentals and dense matmul remain banned. Do not restore multiplier/divide-free wording as current.
- **D37:** float is allowed only in offline training with the export gap; the discrete path stays integer. The saved M1 model and FBS trunk must not be joined in prose or figures.
- **Exact memory:** the true-address oracle is 80/80 against 0/80 no-memory and 20/80 sieve, but that is oracle scope, not learned address formation or M3 qualification. Antigravity-113's `exact-memory.md` and `object-store.md` remain outside this unit.
- **Historical milestone cache:** current status/headline fields may move to the six-bar close; older counts, negatives and holds must remain in explicit history fields rather than disappear.

## 6. Documentation/tooling facts and known negatives

- `STATE.md`, `HANDOFF.md` and `docs/roadmap/README.md` are generated views and are not edited in the content PR; use the designated render flow after the inputs merge.
- Claim-wording covers `docs/**/*.md`, `crates/*/README.md` and root `README.md`, not every public file. Run a separate changed-public-prose scan for root/website/xtask files.
- Previous refresh negatives to retain in the REPORT: two pre-existing broken book links from `docs/research/notebook/m4-serving.md`, one old unformatted `figures_charts.rs` test, and two pre-existing xtask clippy findings. They are not silently attributed to this unit.
- Codex #3429 is closed as superseded and preserved at `ed54006e7615a5e14335a2b501c5c7ba14e14eee` on `codex/docs-current-status-20261030` and `refs/pull/3429/head`. Its useful narrow README/PLAN/item corrections may be reused with credit; its unregistered ORDER/REPORT is not carried into this PR.
- Owner approval for status-only owner paths: **"Approved Plan"**, `2026-10-30T15:30:07Z`, in Kimi's chat. No rule, bar, cap, workflow or production-deployment change follows.
- Disk is actively monitored because the user reported low space. The reusable target is about 218 MiB; free space was 34 GiB at the content baseline. Keep builds narrow and stop before the 128 MiB physical stop margin.

## 7. Active collaboration boundaries at the baseline

- #3430 (claude-218): active memory-key research files, prep, lab result and ORDER are excluded; only merged claude-217 is used here.
- #3426: touches `xtask/src/figures_charts.rs` and `figures_math.rs`; merge its fixes forward if it lands and do not overwrite them.
- #3435/#3424: discrete/source-interface code only, no documentation overlap at baseline.
- #3323 (antigravity-113): `exact-memory.md` and `object-store.md` stay untouched even though they were once considered by the older documentation refresh.

## Primary evidence

- [M1 close README](../../../../labs/m1-close-saved-model/README.md)
- [M1 bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md)
- [M1 bar-f panel score](../../../../labs/m1-heldout-phrasing-transfer/evidence/panel-score.json)
- [Latest merged one-model REPORT](../../one-model-end-to-end/claude-217/REPORT.md)
- [Pipeline map](../../../../model/pipeline-map.md)
- [Mechanism board](../../../../mechanisms/README.md)
- [Studio wiring record](../../../../labs/m7-studio-connect-wasm-engine/README.md)
- [Desktop shell record](../../../../labs/infra-desktop-client-tauri/README.md)
