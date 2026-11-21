# DECISION: the saved question reader as M2's saved model (`CHARTER.md` §5.1)

- **Lab:** `claude`, 21 Nov 2026 (UTC). Casey asked for this card in chat: "Write the card".
- **Milestone:** M2, issue #2030. **Native-space items:** [NATIVE-SPACE.md](NATIVE-SPACE.md).
- **Decision review:** recorded at the end of this card.
- **Authority:** §5.1: a status or scope change goes ahead on the lab's own whole-project decision review and is logged for Casey, who can reverse it. One part of this card is not the lab's to decide, because it reads §3.5 and §3.6; it is marked "for the owner" below. This card writes M2's `base_artifact` in `docs/harness/milestones.json`; only `headline.current` and `secondary[0].current` wait for his answer. [claude]

## What this card does not claim

- **M2 is not closed.** M2's own text asks for "one unified model". The reader and the trunk share an artifact and a vocabulary; the trunk's transported state takes no part in the read. [claude]
- **Wording transfer is not shown.** TRAIN is 20,000 synthetic cases a seed, drawn by `conjunction-probe m2-format` from the panel family's published wording (four statement openings, twelve questions, two reply openings). The fresh panel uses the same wording with other values and histories. So the result shows transfer to new values, which is where the last candidate fell from 437 of 512 to 0 of 128. It does not show a reply to a question phrased another way. [claude]
- **The development count is in-distribution.** `m2-reply512` is in the same wording family TRAIN was drawn from, so 512 of 512 there is expected and says little by itself; the last candidate had 437 of 512 there and 0 of 128 fresh. The fresh read is the informative one. [claude]
- **The TRAIN generator is this lab's, and its separation from the fresh panel is by construction, not by a check.** `m2-format` and `reader-grammar.json` were written by this lab (commit 658708b9a) from the panel family's published openings and the development inputs. TRAIN values are drawn from 2,534 whole-word tokens that no development case uses. The panel's fresh literals were not read, so a single pool token may also be a word of a fresh literal, and a whole fresh literal of two or more tokens appearing as a TRAIN value was estimated at about 5 in 10,000 a literal a seed, not measured (the sheet, lines 32–33). [claude]
- **TRAIN ran outside any D84-named CPU program.** §3.5 allows CPU training as development research in an owner-authorized, named, separately scanned module. `conjunction-probe` is not one. [claude]
- **Not trained on the corpus.** The reader was not trained on D87. Nothing here is evidence about the 1B-equivalent model. [claude]
- **CPU only.** TRAIN is a host count pass and every read was scored on a CPU (the laptop, aarch64 macOS; the replay also on wick's host, x86_64 Linux). No GPU ran. [claude]
- **Overwrites are not shown.** The fresh panel's inputs carry a history's two current records only. [claude]
- **The grade has a mapping this lab added.** The panel's labels accept the bare value; the reader replies in the asked role's opening, which the development labels accept. `--grade` removes that opening before the registered grader runs unchanged. [claude]
- **Same-lab review only.** The sheet's four fitness reads and this card's review are fresh-context readers of this lab. No other lab has read either. [claude]
- **One implementation.** Host, `wasm32-wasip1` and the D11 path compile one Rust source. No CI job builds the integer crate's `reader-serving` feature. [claude]
- **No reader ablation.** The reader is memory and nothing else, so with its rows removed it replies nothing. The controls are the panel instrument's four heuristic replies, scored check-only and not runs of the reader: `copy_first_stated`, `copy_last_stated`, `role_swap_distractor` and `best_constant_reply`, each 0 of 128 pairs. [claude]

## The decision (the lab's)

The lab selects, as M2's saved model, **reader A carried in a D11 artifact**: [claude]

| Part | Identity |
|---|---|
| Artifact | `uor-r4.lut-discrete-question-reader/1`, sha256 `604fcd03da9aa400a28756037e11fc5dce7bc95400ccac5c68a733c8d1a7e4dd`, 122,111,038 bytes |
| Reader inside it | `UORRDR04`, sha256 `44574adb84fd0c4cd30e8c182cf0f835b5d0348618b6b248b23c2c5742972c3c`, 35,574,650 bytes; TRAIN seed 1, tokens sha256 `bfbf5812…0bfc24`, read choices `pair open precise,across` |
| Parent inside it, unchanged | the seed-1 M1 artifact `seed1.d11`, sha256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, 86,536,132 bytes |
| Replicate | reader B, TRAIN seed 2, sha256 `078dcfdc…810c2b`, in artifact `cf9bf6ee…adce1d` |

This card writes it into M2's `base_artifact`, with the note that the trunk takes no part in the read. The artifact is built by `question-reader-d11 pack` from files whose hashes are above; it is not committed (§3.13) and is on the laptop at `local/claude/d11-reader/`. [claude]

## The counts this model has

| Measure | Bar | Reader A | Reader B | Source |
|---|---|---|---|---|
| `m2-reply512`, replies token-for-token the reference (development) | 256 of 512 | 512 of 512 | 512 of 512 | sheet `docs/harness/prep/m2-fresh256-reader-read-2026-11-21.md`; report `local/claude/m2-fresh-sheet/dev-s1.json` (`count.exact` 512, `count.ended` 512) |
| `m2-fresh256`, pairs both complete and distinct | 52 of 128 | 128 of 128 | 128 of 128 | notebook row of 21 Nov, `docs/research/notebook/m2-grounded-reply.md`; one pre-registered read, the panel is spent for this line |
| `m2-fresh256`, rows complete | 104 of 256 | 256 of 256 | 256 of 256 | same row |
| Panel instrument controls on `m2-fresh256` (four heuristic replies, not reader runs) | below the bar | 0 of 128 pairs each | | `docs/labs/m2-fresh-heldout-panel/README.md` |
| Trained reader against the reader read from its file, raw step words | equal | 0 prompts differ | 0 prompts differ | [NATIVE-SPACE.md](NATIVE-SPACE.md) §3 |
| The same words and reply ids from the D11 artifact | equal | equal (`440d7cd0…`, `4554ffd9…`) | equal (`4c0a0ed4…`, `4554ffd9…`) | [NATIVE-SPACE.md](NATIVE-SPACE.md) §3 |
| `m2-reply512` answered from the D11 artifact, replies token-for-token the reference | 256 of 512 | 512 of 512 | 512 of 512 | `question-reader-d11 words … 48`: `replies_equal_to_reference` 512, 6,664 reply ids; `local/claude/d11-reader/words48-s{1,2}.json` |

All of it was measured on CPU. The development count is measured twice: from the reader file (reply limit 48) and from the D11 artifact itself (limit 48, last row; the hash rows used limit 64, and every reply ends before either limit, 6,664 ids in each run). The fresh count was read from the reader file before the artifact existed, and the panel is not read again. It carries to the artifact only because the artifact holds that file byte for byte under a SHA-256 binding and answers through the same source; that is an argument, not a second measurement. [claude]

## For the owner: does a CPU count stand as M2's headline

§3.5: a CPU program "may not substitute CPU output for a GPU qualification bar unless a later owner decision says so". §3.6: "panel scoring and milestone evaluation remain GPU research compute". The reader's TRAIN is a count pass of seconds on a CPU, and its replies are integer row reads on a CPU; there is no GPU kernel for either. Casey's 16 Nov goal names a model "that can be trained and served on this laptop". The lab does not read that as the later owner decision §3.5 asks for. [claude]

The same sections bear on TRAIN: it ran on a CPU outside any D84-named program. So the question to the owner is one: do this reader's CPU-trained, CPU-scored counts stand as M2's headline. The headline change is a proposal, and it is the one part of this card that waits: [claude]

| Field in `docs/harness/milestones.json`, M2 | Now | Proposed |
|---|---|---|
| `headline.current` | 0 | 512 |
| `secondary[0].current` (fresh, percent) | 0 | 100, with the note kept and extended: "new values, same wording as TRAIN; read from the reader file the artifact carries byte for byte; fresh acceptance in other wording remains unqualified" |
| `status` | in-progress | in-progress (unchanged) |

If the answer is no, the counts stay in the notebook as development research and the unit that moves the headline is a GPU reader held to the replay's word streams. Neither answer stops the work below. [claude]

**Answered, 21 Nov 2026 (UTC), Casey in chat, to the question "Do the reader's CPU-trained, CPU-scored counts stand as M2's headline (512 of 512 dev)?": "No, GPU first".** The proposed counts are withdrawn. `headline.current` and `secondary[0].current` stay 0. The counts above stay in the notebook as development research. The next unit is a GPU reader: the read, then TRAIN, as CUDA kernels on wick's 4090, each held bit for bit to the host word streams of [NATIVE-SPACE.md](NATIVE-SPACE.md) §3, and the M2 counts scored from it under a sheet with a non-author FIT (§4.5). [claude]

## The GPU count, 21 Nov 2026 (UTC)

What this does not show, first [claude]. `m2-reply512` is a development panel in the wording TRAIN is drawn from, and this reader line was developed on it; the host reader already answered it 512 of 512 and the GPU path was already equal to the host in finite kernel tests, so the count was expected and is not a blind result. It says nothing about fresh transfer: the fresh number stays 0 and `m2-fresh256` was not read. The trunk takes no part in the read. The GPU TRAIN is about 27 times slower than the host, and the host orders its launches. The two fitness readers are the same lab; no other lab has read the sheet.

Sheet: [`m2-reply512-gpu-reader-2026-11-21.md`](../../../harness/prep/m2-reply512-gpu-reader-2026-11-21.md), FIT on its second non-author read, pushed (commit f526662d1) before the run. Run: wick, RTX 4090, `runs/conj/gpuscore1/`, binary `conjunction-probe-v39` (`d863d3e8…a6e1b8`), every input sha256 as pinned.

| | Reader A (seed 1) | Reader B (seed 2) |
|---|---|---|
| GPU TRAIN: bytes are the host file; same on a second run | yes (`44574adb…972c3c`); yes | yes (`078dcfdc…810c2b`); yes |
| GPU TRAIN kernel time; host TRAIN | 55.0 s; 2.0 s | 54.8 s; 2.0 s |
| Prompts read on the GPU; GPU read equal to the host | 512 of 512; yes | 512 of 512; yes |
| Replies equal to the reference, id for id (`gpu_count.exact`) | **512 of 512** | 512 of 512 |
| Replies ending on EOS | 512 | 512 |
| Control: each reply against the next prompt's reference | 0 of 512 | 0 of 512 |

Result against the sheet's bar: PASS for reader A (bar 256 of 512); reader B agrees. Decision [claude]: the owner's condition ("No, GPU first") is met for the development count, so M2's `headline.current` is set to 512, reader A's count. `secondary[0].current` stays 0 and `status` stays in-progress. This is reversible by the owner; it is logged on #2030. A fresh-context decision review of this change (same lab) returned SURVIVES with no required fix. §4.3 asks for a fresh transfer number beside a development gain: the only fresh read of this reader line was scored on CPU and its panel is spent, so a GPU-scored read of a newly drawn panel is owed and is item 2 below.

### The order-free TRAIN as the GPU trainer, 21 Nov 2026 (UTC)

What this does not show, first [claude]. No milestone number moved: the headline was 512 and is 512. The panel is the same development panel, and this reader had read it twice under no sheet before the sheet was written, so the count was known. Nothing here is a fresh read. The host still finds the reply runs and pair positions.

Under the sheet [`m2-reply512-gpu-free-reader-2026-11-21.md`](../../../../harness/prep/m2-reply512-gpu-free-reader-2026-11-21.md) (FIT with notes on one non-author read, same lab), run `runs/conj/gpuscore2/` on wick, binary v42 [claude]: reader A's order-free file written on the GPU is the host's order-free file byte for byte (`9db7e834…e1227b`), the same on a second run and with the replies reversed; `512 / 512` replies equal the reference, `512 / 512` prompts read on the GPU, GPU read equal to the host, control `0 / 512`. Reader B (`ce961568…b89578`): the same counts. GPU TRAIN kernel time 1.15 s and 1.16 s in 48 launches each (the in-order rule: 55 s, 8,042 launches). Every input and both host files carried the sheet's pins.

Result against the sheet's bar: PASS. Decision [claude]: the order-free TRAIN (two passes) is the GPU trainer of the M2 reader from here on, and reader A's order-free file is the saved reader that later sheets train and read. The in-order reader of the section above remains the reader the 512 was first counted on; both give 512 of 512. Reason: the in-order rule's launch count grows with the stream and it gave no result on the corpus-worded stream, where the order-free rule wrote the host's file in 9.5 s. Cost to undo: none; the in-order kernels and their sheet are unchanged. This choice changes no bar, status or scope.

### The fresh number from a second draw, 21 Nov 2026 (UTC)

What this does not show, first [claude]. The panel read here, `m2-fresh256b`, is not the panel `milestones.json` names (`m2-fresh256`); that one was read once by this reader line on a CPU and is spent. `m2-fresh256b` is a second draw from the same instrument and construction: new literals, drawn by a fresh-context agent of this lab, not by another lab. The values are new; the wording is the wording TRAIN is drawn from, so transfer to other wording is unmeasured. The reader control (reader N, a TRAIN of one case) was not run: its GPU TRAIN stopped on "no form was seen eight times". The trunk takes no part in the read, so this is not one unified model. The host still finds the reply runs and pair positions for TRAIN, and reads every prompt as the parity reference. The 4 and 2 incomplete rows were not read by cause, because the panel stays sealed from the reader's author; the grader's counters say only that every reply ends on EOS and carries its opening. The sheet's fitness read was same-lab.

Under the sheet [`m2-fresh256b-gpu-reader-read-2026-11-21.md`](../../../../harness/prep/m2-fresh256b-gpu-reader-read-2026-11-21.md), committed before the run (commit `6348d3858420c242999dda9ef60eebfcd99eb192`), run `runs/conj/gpufresh1/` on wick, binary v43 [claude]:

| Measure | Bar | Reader A (`9db7e834…e1227b`) | Reader B (`ce961568…b89578`) |
|---|---|---|---|
| `m2-fresh256b`, pairs both complete and distinct, GPU-written replies | 52 of 128 | 124 of 128 | 126 of 128 |
| `m2-fresh256b`, rows complete | 104 of 256 | 252 of 256 | 254 of 256 |
| GPU TRAIN bytes are the pinned file, and the same on a second run | true | true | true |
| `reader-gpu` exit 0, `equal` true, prompts read on the GPU | 256 of 256 | 256 of 256 | 256 of 256 |
| Grade output's reader, inputs, labels, panel and literals sha256 against the pins | equal | equal | equal |
| Swapped-pair check of the grading path | 0 of 128 | 0 of 128 | 0 of 128 |
| Reader control N | under 13 of 128 | not run | not run |

Result against the sheet's bar: PASS, reader control not run. The reported numbers are the lower of the two readers: 124 of 128 pairs, 252 of 256 rows.

Why this is not a CPU-scored count under the owner's "No, GPU first" [claude]. The tables were written by the GPU TRAIN kernels and are byte for byte the pinned reader file. Every reply id was written by the GPU read kernel, and all 256 prompts were read to their end on the GPU. The laptop's CPU ran the panel instrument only: it removes the asked role's fixed opening and compares each reply's ids with the sealed labels. It computes no part of a reply. The host read that `reader-gpu` also runs is the parity reference; the graded replies are the kernel's.

The decision [claude]. The sheet says a PASS unlocks this question and does not answer it. The change to `docs/harness/milestones.json`, M2:

| Field | Now | Set to |
|---|---|---|
| `secondary[0].current` (fresh, percent) | 0 | 96 (124 of 128 pairs, the lower reader, rounded down) |
| `secondary[0].note` | "Fresh acceptance remains unqualified; M1 does not establish complete replies." | "96 = 124 of 128 pairs on m2-fresh256b, a second draw of the registered construction with new values, read once under a sheet by GPU-trained readers whose replies a GPU kernel wrote (claude-443 decision card, 21 Nov 2026). Same wording as TRAIN; the drawer was a fresh-context agent of the same lab; the one-case reader control was not run; the trunk takes no part. Fresh acceptance in other wording remains unqualified; M1 does not establish complete replies." |
| `fresh_panel` | m2-fresh256 | m2-fresh256 (unchanged; the registered panel stays the registered panel) |
| `headline.current` | 512 | 512 (unchanged) |
| `status` | in-progress | in-progress (unchanged) |

- **Reason.** The fresh number has stood at 0 since the previous candidate fell from 437 of 512 to 0 of 128 on a fresh draw. This reader was fixed first, the panel was drawn after, by an agent that never saw the reader, and it was read once. The bar is the one the owner registered for `m2-fresh256` (#2030, comment 6350135678). The count comes from the GPU path the owner asked for. The number is recorded with its limits in the field's own note, so the field does not claim more than the run shows.
- **Turned down.** Leaving the field at 0 until another lab draws a panel: the number would misstate what was measured, and another lab's draw remains open as an audit (§5.5) with a new draw, which this card invites. Setting 100 or the row percentage (98): the registered bar is the pair count, and the lower reader's pair count is 124. Changing `fresh_panel` to the second draw: the registered name is the owner's record of which panel the bar was set on. Closing M2: items 1, 3, 4 and 5 of the list below are open, and wording transfer is unmeasured.
- **Cost to undo.** Low: one number and one note. No run depends on either. Casey can reverse it on #2030.

What changes in the list below: item 2 is met in part (a panel drawn fresh, read on the GPU, with the panel's four heuristic controls and the pair metric as the shuffled-role control); the wording half of item 2 and a reader control that can train remain open.

## What M2 still needs before it can close

1. The reader keyed by the trunk's transported state, so the artifact holds one state and not two tables (D85; M2's "one unified model").
2. A panel in wording the reader was not trained on, drawn fresh (§4.3), with the no-memory and shuffled-role controls of [claude-409's bar](../../written-state-count-model/claude-409/DECISION.md) R.
3. A whole-state save, so TRAIN continues from the file.
4. Another lab's read of the sheet and of this card.
5. CI on PR #4136, the full discrete suite, and a CI job for `reader-serving`. [claude]

## Reason, alternatives, cost to undo

- **Reason.** M2's headline has stood at 0 because no saved model was selected. This is a saved model with a lossless file, a replay check, a D11 carrier, the development bar met and the fresh bar met on a sealed panel read once under a sheet written first. The previous candidate failed exactly the fresh read this one passes. [claude]
- **Turned down.** Closing M2 on these counts: the model is not unified and wording transfer is unmeasured. Moving the headline without the owner: §3.5 and §3.6 as written. Leaving the model unselected until a GPU reader exists: the selection and the remaining list are useful now, and cost nothing to reverse. Selecting the bare reader file: M2's text asks for D11. [claude]
- **Cost to undo.** Low. The selection is one field; the proposed counts are two numbers. No run depends on either. [claude]

## Decision review

A fresh-context reader of this lab, read-only, given this card at commit 931323ff5ff7877ca6048f4c373f83b0c933f626 and the files it cites, and none of the reasoning behind it. Verdict: **SURVIVES-WITH-FIXES**. It found the §3.5 and §3.6 deferral correctly placed, and the selection sound as a selection ("unified" is a closing condition). Its eight findings, each fixed in this text: [claude]

1. The card said both that nothing in `milestones.json` changes and that `base_artifact` is replaced. Fixed: the card writes `base_artifact` with the trunk note; only the two counts wait.
2. "The same 512" for the artifact was inferred across runs at different reply limits. Fixed by measuring it: the artifact answers `m2-reply512` 512 of 512 at limit 48.
3. Proposing 100 for a field labelled "genuinely fresh" overstated it. Fixed: the proposed value keeps and extends the field's note.
4. The card did not say whether TRAIN ran under a D84-named program. It did not; that is now a stated limit and part of the question to the owner.
5. The TRAIN generator's authorship and its separation from the fresh panel were not stated. Now stated, with what was estimated and not measured.
6. The development count is in-distribution. Now said.
7. Two of the panel's four controls were listed, without saying they are not reader runs. Fixed.
8. The notebook row of 21 Nov says there is no D11 export. A dated follow-up row is added; the old row is not edited.

Not checked by the reviewer: the `m2-format` generator against the panel generator (the panel's literals are sealed from this line), reader B's local reports, and the sheet's full text. [claude]
