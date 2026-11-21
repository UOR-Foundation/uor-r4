# Order-free D11, ordinary conversation and unseen wording

The nearest-state selector recovered zero new exact replies on the 128 unseen prompts. Six replies that the lexical selector completed regressed. Both of the new arm's exact replies came from abstaining and running the suffix fallback. On the 32 controls with familiar question wording the ordinary lexical reader completed every reply. Replacing its form choice with the state lookup reduced that to nine.

| Selector and shared writer | Familiar exact / 32 | Unseen exact / 128 | Unseen first wrong at 0 | At 1 | At 2 or later | Unseen ended in EOS |
|---|---:|---:|---:|---:|---:|---:|
| Lexical order-free | 32 | 8 | 48 | 16 | 56 | 50 |
| Trunk pre-gate Relation vector | 9 | 2 | 61 | 54 | 11 | 46 |
| Opposite-role donor state | 2 | 2 | 67 | 47 | 12 | 46 |
| Transport knockout | 2 | 8 | 64 | 0 | 56 | 16 |
| Suffix fallback | 2 | 8 | 64 | 0 | 56 | 16 |

The comparison holds the job and home facts, values, question, history and writer fixed within each arm. Across lexical versus real state, the first wrong original ID moved later in 31 cases (25 from index 0 to 1, four from 0 to 8, two from 0 to 9). It moved earlier in 60 cases. On 29 it stayed at the same index, on six an exact reply broke and on two both were exact. All offsets are zero-based. The complete required opening survived on 64 lexical and 13 real-state rows. Among the openings that survived, the first value token matched in all cases. An EOS by itself did not earn exact credit.

| Unseen wording, 32 rows each | Lexical exact | Real state exact |
|---|---:|---:|
| Alternative question | 2 | 1 |
| Declarative request | 4 | 1 |
| Exclusion first | 2 | 0 |
| Exclusion last | 0 | 0 |

In w0032 (“Which position do I work in?”) the lexical reader began with housing token 617 in place of job opener 2997. State fixed that token, then copied prompt position 61 and emitted 315 in place of job token 1541 at index 1. In adjacent w0033 (“Which place do I call home?”) the lexical reader emitted the right answer through the period and missed EOS at index 9, emitting 201 in place of 1. State moved the first error to index 0, emitting job opener 2997 in place of housing opener 617. In w0053, state broke a previously exact housing reply at that same first token.

The 128 unseen prompts produced 45 distinct live vectors and one knockout vector. Donating the paired opposite-role state left the vector identical on 62 of the 128 rows, held the form decision identical on 72 and held the complete reply identical on 78. All knockout rows abstained. The full-state arm abstained on 15. It selected forms on the remaining 113 and none reached exact EOS. Nearest selection minimizes integer L1 distance to a frozen 107-prototype index, trained on 4,096 response onsets using existing lexical form labels. The conclusions apply to this observer, selector and panel.

## Ordinary serving on Wick

The 120,755,422-byte package binds the original reader and trunk. Reloading it reproduced 6,664 original output IDs including EOS and the development step-word digest for 512 rows. Four structural prompts were replayed through the ordinary binary raw-ID entry and matched through EOS. That panel supplies historical empty assistant turns. True user-text entry differed at input position 4 (2485 vs 2496) and included actual assistant output. In both full three-turn text scripts the second reply reached the cap before the query turn. A shorter job session reached the query. Its shared writer selected source position 11 twice at offsets 3 and 4, duplicating ID 261 instead of advancing to position 12 for ID 92. After the period it selected position 24, the start of a prior assistant answer. The direct home case had no held form at the expected EOS and the suffix fallback emitted role-marker token 201. [Client checks and original source traces](reader/INDEX.md)

## Measurement and decision

The panel crosses two roles, two fact orders, two object assignments and four literal pairs. Eight complete tokenized question strings had zero occurrences in the original reader TRAIN stream. Wording families and literals recur across panel rows. The panel uses structural empty prior assistant turns. These rates characterize those paired rows. The ordinary-client transcripts independently measure what happens during free-running turns. D37's per-byte result belongs to a different trained state and endpoint.

The GPU finite checked 16 full and 16 knockout 32-word vectors against the native CPU reference, plus exact-match donor switching and conflicting-tie abstention. GPU fit and prediction used the same pre-joint-pointer Relation operator. The original integer writer ran on the host for all arms. Two graders agreed for every original token, first error and absent output over 800 trajectories. The raw prediction file on Wick is `/mnt/plexpool/uor-r4-corpus/runs/20261121-orderfree-trunk/codex-state/pod-receipts/predict-run-1/predictions.json`. The stored answer trajectories, IDs and trace addresses at each first error suffice to reproduce the table. [Per-prompt original-ID results](state/graded-v1.json) · [Group counts](state/group-counts.json) · [Panel inputs and references](panel/SHA256SUMS) · [Prediction identity](state/adjudication.sha256)

Keep the lexical selector as the control. Before attempting another state replacement, supply the correct frozen job/home form and measure what the writer can complete with ordinary framing and actual history. Independently vary the leading-space encoding and empty versus genuine assistant history. At copy transitions record whether the same source occurrence is rebound and whether the first read after the period chooses EOS or the previous assistant. A successful oracle would isolate where role selection can help. A failing oracle identifies a writer or turn-boundary defect even after role selection is correct. This is an experimental next step and changes no milestone.
