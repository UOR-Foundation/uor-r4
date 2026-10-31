# FACTS: documentation status at 30 Oct 2026

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). ORDER: [ORDER.md](ORDER.md). Original base: `ec9a790fe5be73cd673f082eba86ced551b22d12`; current PR base: `bd9660fa76cf14e5a96d9c87b53788797d6d2fbd`. Earlier exact-head reviews inspected `f6acd4b8a9e01acfca4506ac070f72349cfe46de`, `9fb906dc37ab6fc1ec2b36d5ae007c44b1053394` and `95cb7388ada0386e9c3c1bea31f1766809f7938b`. Those are historical provenance, not the final head; the final head is the commit named in the PR and its fresh review.

Every public number in this refresh must link to this sheet's committed evidence. This is a factual baseline, not a new measurement or a capability claim.

## 1. Milestone state and the M1 artifact

- **Project stage:** pre-alpha, **1 of 8 milestones complete**. M1 #2029 is CLOSED. M2 and M3 are in progress; M4–M8 are not complete. M7 runtime/harness work is in progress on a synthetic stand-in. No useful conversation or product release is established.
- **Closing artifact:** antigravity-110's saved `trunk-joint-pointer-v1-d11`, a 19-table `uor-r4.lut-discrete-trunk/1` discrete model, exported as two seeded `UORLUT01` files:
  - seed 1 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` (86,536,132 B);
  - seed 2 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de` (84,943,328 B).
- **Composition:** Flock `FrozenZeroV1`, discrete Hamilton gate, discrete quaternion recurrence, `PositionHolonomyV1`, `OrbitRootSnapNormV1`, four-block orbit-root causal pointer, and post-rotation outer-L1 readout. The normalization is discrete, not the FBS RMS norm, and this is not the separately float-trained FBS trunk.
- **Evidence split:** `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` covers bars (a)–(e) and is not called six-bar evidence. `docs/labs/m1-heldout-phrasing-transfer/evidence/panel-score.json` supplies bar (f). #2029 and `docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md` join them into the six-bar close.

### M1 measured results

| Panel/contract | Seed 1 | Seed 2 | Controls and scope | Source |
|---|---:|---:|---|---|
| DEV integer top-1 | **7,002 / 14,287** | **7,001 / 14,287** | unigram 1,226; T2 6,300; bar (d) target 6,324 = T2 + 24; both seeds beat both and the target | [M1 close record](../../../../labs/m1-close-saved-model/README.md) |
| Fresh V2 integer top-1 | **7,215 / 14,479** | **7,228 / 14,479** | unigram 1,279; T2 6,582; paired band 82 | [Fresh transfer](../../../../labs/m1-fresh-panel-transfer/README.md) |
| p-f held-out phrasing | **7,850 / 15,953** | **7,872 / 15,953** | unigram 1,409; T2 7,375; leads 475/497 above paired band 81 | [bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| Export/reference agreement | 14,287 / 14,287 | 14,287 / 14,287 | zero export gap; planted-fault B4 passes | [close grade](../../../../labs/m1-close-saved-model/evidence/m1-close-grade.json) |
| Apple M1 decode | **10,876 tok/s** | **10,566 tok/s** | median; 192-token prefill and 64-token decode; about 2.67M prefill tok/s | [bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md) |

- **What bar (f) proves:** p-f has 0.000 of its prompt value-masked templates in TRAIN at K = 1,024 (0.050 at K = 256), so the saved model reads new prompt phrasings into known kinds of reply.
- **The limit that must stay beside it:** **0.625 (62.5%) of p-f's reference replies are verbatim TRAIN replies**. That is reference-data overlap, not a generated-copy rate, not new reply kinds, and not useful-chat qualification.
- **M1's close is reversible through the project record, not by documentation.** Documentation propagates the already reviewed close; it does not create it.

## 2. Separate FBS trunk and current open research

- The float-trained FBS trunk is a different path from the saved discrete M1 model. `lut-stack/5` exports/serves it exactly at DEV scope; D80/D81 allow integer multiply and divide while floats remain banned in serving. Its earlier M1 rates were 4,217–4,504 tok/s; do not mix those rates with the saved M1 artifacts' 10,876/10,566 tok/s.
- **claude-217**, `docs/loop/tasks/one-model-end-to-end/claude-217/REPORT.md`:
  - The cycle/echo steps end every registered loop, but every content decision is **underpowered (content)**; greedy remains the served decode. Loop removal is not a quality result.
  - On held-out p-s, exact-reference replies are only 3, 1 and 4 of 256 at seeds 1–3, against 38–40 on DEV.
  - Teacher-forced p-s reply openings are **17.4%, 13.7%, 18.6%**, against **80.8%, 80.2%, 79.9%** on later tokens. On p-t the openings are 18.7%, 12.2%, 17.9% against 80.8–81.1%.
  - Exact turn memory keys on the turn's exact first token ids, so a new phrasing of a known question can find no stored key. On p-s it adds only 14–16 of 1,400 openings; on DEV it adds 322–337 of 1,226.
- **Offline precursor: claude-218**, [memory-key report](../../one-model-end-to-end/claude-218/REPORT.md), merged as `5ec6e36a8f69d3b0c9acf993cc3e09acc7be4646`:
  - On p-u's 1,218 exact-memory misses, the sign key gains **+46/+37/+33** and the point key **+125/+185/+114** at three seeds. Row 3 selects the cheapest passing sign key; the point key's 1 KB-per-opening table against 32 bytes remains an explicit tradeoff.
  - A forced correct first token lifts exact replies from 4/0/0 to 54/49/52 of 256. These are teacher-forced key readings and a forced-opening arm, not a served key or free-generation result.
- **Latest merged result: claude-219**, [keyed-opening report](../../one-model-end-to-end/claude-219/REPORT.md), merged as `e44f68e65`:
  - On fresh p-v, 393 prompts at each of three seeds, the point key supplies the first reply token where exact memory misses, then greedy continues. Template pairs are **(16,3), (9,1), (14,2)**, net **+13/+8/+12**; exact pairs are **(12,3), (9,1), (14,2)**, net **+9/+8/+12**. Each passes its paired band without higher loops or lower DEV.
  - The gain is partial: keyed openings are right on **105–119 of 385 (27–31%)**, reaching **12–18%** of the forced-opening lift of +71/+66/+73 templates. Fewer loops are not a quality gain because more replies end early, including 49–58 short `Noted.` openings per seed.
  - The sign key's pairs have `b + c < 10` at every seed, so it is **inconclusive at this panel size**, not proven ineffective.
- **Latest merged result: claude-220**, [served-key report](../../one-model-end-to-end/claude-220/REPORT.md), merged on `main` as `c3d5cb684336a59cb12a8b5021430585bc3b751a`:
  - One float model now serves the point key itself on fresh p-w (1,200 conversations), reproducing claude-219's keyed files exactly on p-v and DEV. Template pairs are **(50,9), (44,4), (47,5)**, net **+41/+40/+42** at three seeds; exact pairs are **(48,9), (44,4), (47,4)**, all passing their bands without lower DEV.
  - The gain is recall of TRAIN replies, not generalization: served replies are more often verbatim TRAIN replies and have fewer new n-grams. The key closes about 18% of the forced-opening gap and is right on 28–32% of its 1,172 openings per seed.
  - The table is 218,931,272 bytes per checkpoint and scanned at each keyed opening; search is at most 32 ms with three processes sharing one RTX 5090 and at most 5.8 ms alone, both upper bounds. These are float GPU evaluations, not integer serving speed.
  - **Next:** [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450) has since completed its unit-2 instruments; unit 3's real export and measurement remain open. Client-sized search and useful conversation remain open.
- **Merged implementation: claude-222**, [integer point-key report](../../point-key-integer/claude-222/REPORT.md):
  - The integer point table, its search and the `/8` record are built, with 380 library tests, 18 negative gates and a full serving audit. The integer search uses the trunk's own Q12 points and the least-L1/commonest-target rule.
  - This is synthetic-only, not a real trained-model evaluation.
- **Latest merged implementation: claude-224**, [panel decode and grade report](../../point-key-integer/claude-224/REPORT.md), merged on `main` as `18c4c3da70f35b5c8c9348297df05cfd4d8e452d`:
  - Unit 2 built the panel prompts, integer decode and grade, with 383 integer-library tests, 185 float-library tests, 13 negative gates and a full serving audit. These are instruments, not a real TRAIN export or panel score.
  - Both integer arms stop at BOS, 64 reply tokens or the shared 192-token context, so long prompts have capped replies; the paired comparison uses the same cap.
- **Merged registration: claude-225**, [retention-item report](../../infra-loop-harness/claude-225/REPORT.md), merged on `main` as `a5eb6863a05b0177de189c7ca39da4025843c167`:
  - Before unit 3 could run, a source inspection found that (r-drop) trained with `retain_rates` on while the integer artifact served the off reading. It registered [`integer-retention-rates`](../../../../roadmap/items/integer-retention-rates.md) (#3456).
- **Latest merged implementation: claude-226**, [retention implementation report](../../integer-retention-rates/claude-226/REPORT.md), merged on `main` as `bd9660fa76cf14e5a96d9c87b53788797d6d2fbd`:
  - The `geometric-retention` flag and retained recurrence are built, with 386 integer-library tests, 186 float-library tests, nine negative gates and a full serving audit. Earlier snapshots still export byte for byte, and older engines refuse the new reading.
  - GPU parity on (r-drop) s1 is NOT_RUN and precedes unit 3's real export, fresh p-x scoring, tok/s and resident memory under their fresh sheets and pre-run reviews. The implementation is not parity or a real panel result.
- **D67:** no blanket owner release holds every training step. The D41 donor pass is the remaining named owner-at-dispatch exception. Do not restore retired "every training step is held" language.

## 3. Historical facts to preserve, not present as current

- **K16, 19 Oct:** the selected discrete K16-A seeds scored 65 and 463 of 14,287, below unigram 1,226 and T2 6,300. K16-R collapsed to 19/20 cells. Keep this as dated negative evidence, never as the current best model.
- **Earlier M2 headline:** 437/512 on seen rows fell to 0/128 on a fresh draw. It is a panel-fitting lesson, not qualified progress toward M2's current saved-model acceptance.
- **Four 214M Hugging Face checkpoints:** historical models that used softmax and matrix multiplication; they are not the D11 product and not evidence for today's rules.
- Dated `docs/labs/**`, prior `docs/loop/tasks/**`, notebook rows, `docs/integration/current-state.md` and `archive/**` stay unchanged. `STATUS.md` and `docs/integration/NOW.md` are retired pointer pages; current plan/status goes to `PLAN.md` and rendered `STATE.md`/`HANDOFF.md`.

## 4. Product state

- **Model:** trained integer artifacts exist, including the M1 pair, but no versioned product release ships and no useful-chat model is qualified.
- **Web:** the Studio and `uor-r4-integer` worker are connected and verified on a synthetic stand-in; page assets and a stack-bound tokenizer are self-hosted. DeepSeek's #3442/#3444 rebuilt and published the Studio; #3445 publicly released the M1 pair and serves seed 1 in the Studio as top-8 next-token prediction after an explicit Hugging Face download, with hash/tokenizer verification. Greedy continuation repeats tokens, so discrete chat generation is refused. The `/7` turn-memory engine is wired, but its trained-artifact parity and memory-session restore remain open. **Live verification:** gh-pages `454bb30fcab8580db3320717685319a7c80a2d9b` serves the 626,889-byte wasm `d7bcaf4047e06b5243fb7e07602829367ab67a19bde28c711d49d4f8ceec2905`, and the live manifest lists both M1 seeds. No accepted-model M7 parity or qualified conversational product is claimed. Kimi verified the existing publish and performs no production deployment.
- **Desktop:** the Tauri shell exists and is verified with durable-memory/session behavior on the 469,252-byte `mk-synthetic-stack` seed-5 artifact. It is not merely a specification, but it has no qualified trained product bundle.
- **M4:** a lawful likelihood (`PRLL`) is built and registered, but the GPU held-out scoring and full accepted M4 model/CLI path remain open. Do not say no lawful likelihood exists, and do not claim M4 is complete.
- **M7:** runtime/harness work on the synthetic stand-in is real; accepted-model distribution is not complete.

## 5. Mechanism and rule state

- The board has **28 mechanisms: 12 ACTIVE, 5 READY, 11 PARKED**. A whole-model M1 pass does not establish each mechanism's independent effect.
- **Prime-zeta negative retained:** the 25 October zero-update comparison was a configuration loss. P8 admitted 127/3,847 against 178–188 for the strongest nulls, lost to Q0, Q1, R0 and R1, tied Q2 and won only R2. Keep the mechanism ACTIVE without erasing that outcome ([record](../../../../labs/r-prime-zeta-vs-nulls/README.md)).
- **Durable memory:** the object-store consumer and frozen 384-item host-control panel are built and pass; the on-model M3 run awaits M2 selection. Host controls are not M3 qualification ([consumer record](../../../../labs/object-store-consumer-m3/README.md)).
- **D11:** integer serving may use hardware integer multiply and divide (D80/D81); floats, transcendentals and dense matmul remain banned. Do not restore multiplier/divide-free wording as current.
- **D37:** float is allowed only in offline training with the export gap; the discrete path stays integer. The saved M1 model and FBS trunk must not be joined in prose or figures. M1 uses discrete Hamilton/quaternion recurrence, `PositionHolonomyV1` and `OrbitRootSnapNormV1`, not FBS projection or the FBS RMS normalization.
- **A8 evidence scope:** finite qualification passed 5/5 with D67 PASS; the full-dose admission-preserving rerun reached Q1 `4,292 / 14,287`, `1,106 / 3,847` and `871 / 14,287` before a diagnostic stop. Stage T—not A8—reached `5,009 / 14,287` with G-Adm/G-Occ. The next admission-preserving rerun is NOT_RUN.
- **Serving evidence split:** the older close-grade bar (c) reports 11,026 / 10,573 tok/s; the bar-f remeasurement reports 10,876 / 10,566 tok/s on Apple M1. Keep the remeasurement beside bar (f), not silently substituted for the older bar (c).
- **Key tradeoff:** claude-218 selected the 32-byte sign key over the 1 KB point key under its offline row-3 rule. Claude-219's free-generation evidence selects the point key for the next integer-serving unit instead; the sign key is underpowered at p-v's size. Do not present the earlier offline selection as the current next step.
- **Sealed-label anomaly:** `fresh_transfer_gate_receipt.family` in `m1-close-grade.json` says "Stage T B=4 C=192 Relation read (FrozenZeroV1)", while the M1 candidate is Flock plus orbit-root pointer. Do not edit sealed evidence; annotate it rather than treating that label as the M1 composition.
- **Exact memory:** the true-address oracle is 80/80 against 0/80 no-memory and 20/80 sieve, but that is oracle scope, not learned address formation or M3 qualification. Antigravity-113's `exact-memory.md` and `object-store.md` remain outside this unit.
- **Historical milestone cache:** current status/headline fields may move to the six-bar close; older counts, negatives and holds must remain in explicit history fields rather than disappear.

## 6. Documentation/tooling facts and known negatives

- `STATE.md`, `HANDOFF.md` and `docs/roadmap/README.md` are generated views and are not edited in the content PR; use the designated render flow after the inputs merge.
- Claim-wording covers `docs/**/*.md`, `crates/*/README.md` and root `README.md`, not every public file. Run a separate changed-public-prose scan for root/website/xtask files.
- Previous refresh negatives to retain in the REPORT: two pre-existing broken book links from `docs/research/notebook/m4-serving.md`, one old unformatted `figures_charts.rs` test, and two pre-existing xtask clippy findings. They are not silently attributed to this unit.
- Codex #3429 is closed as superseded and preserved at `ed54006e7615a5e14335a2b501c5c7ba14e14eee` on `codex/docs-current-status-20261030` and `refs/pull/3429/head`. Its useful narrow README/PLAN/item corrections may be reused with credit; its unregistered ORDER/REPORT is not carried into this PR.
- Owner approval for status-only owner paths: **"Approved Plan"**, `2026-10-30T15:30:07Z`, in Kimi's chat. No rule, bar, cap, workflow or production-deployment change follows.
- Disk is actively monitored because the user reported low space. The reusable target is about 1.7 GiB; free space was 34 GiB at the content baseline and 27 GiB during the latest repair. Keep builds narrow and stop before the 128 MiB physical stop margin.

## 7. Active collaboration boundaries at the baseline

- #3430 (claude-218) was active at the baseline and merged at `5ec6e36a8f69d3b0c9acf993cc3e09acc7be4646` during verification. #3439 (claude-219) then merged at `e44f68e65` with the fresh p-v free-generation result. Both are included; Claude's research files remain untouched by this unit.
- #3441 registered the M7 Studio live/model-sources item; #3442/#3444 rebuilt and published it. During final repair, #3445 added the public M1 release/top-8 Studio path, #3446 served `/7` integer turn memory, #3447 redesigned the console, #3448 supplied the workable v2 Workspace/Agent interface and #3451 polished it. `origin/main` `437d1ddf5` is merged forward; their implementations and generated roadmap view are preserved. This unit changes only documentation and does not publish the Studio or edit `.github/**`.
- #3443 merged claude-220 on `main` as `c3d5cb684336a59cb12a8b5021430585bc3b751a` during final review: one float model serves the point key on p-w at all three seeds with exact claude-219 parity. #3449 then registered `point-key-integer` (#3450), #3452 delivered claude-222's integer table, search and `/8` record, #3453 registered claude-223's grading scope and unit order, #3454 delivered claude-224's panel decode and grade, #3455 registered the retention prerequisite (#3456), and #3457 delivered claude-226's retained recurrence and flag. `origin/main` `bd9660fa76cf14e5a96d9c87b53788797d6d2fbd` is merged forward; GPU parity on (r-drop) s1 precedes unit 3's real export, fresh p-x score and laptop measurements. Claude's research source and records remain untouched by this unit.
- #3426's figure tooling fixes landed through #3438 and were merged forward before figure regeneration.
- #3435/#3424: discrete/source-interface code only, no documentation overlap at baseline.
- #3323 (antigravity-113): `exact-memory.md` and `object-store.md` stay untouched even though they were once considered by the older documentation refresh.

## Primary evidence

- [M1 close README](../../../../labs/m1-close-saved-model/README.md)
- [M1 bar-f REPORT](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md)
- [M1 bar-f panel score](../../../../labs/m1-heldout-phrasing-transfer/evidence/panel-score.json)
- [Latest merged one-model REPORT](../../one-model-end-to-end/claude-220/REPORT.md)
- [Latest merged integer point-key implementation](../../point-key-integer/claude-224/REPORT.md)
- [Latest merged retention implementation](../../integer-retention-rates/claude-226/REPORT.md)
- [Pipeline map](../../../../model/pipeline-map.md)
- [Mechanism board](../../../../mechanisms/README.md)
- [Studio wiring record](../../../../labs/m7-studio-connect-wasm-engine/README.md)
- [Desktop shell record](../../../../labs/infra-desktop-client-tauri/README.md)
