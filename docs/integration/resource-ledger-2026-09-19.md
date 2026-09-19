# Cumulative model-time ledger reconciliation — 2026-09-19

Owner: Casey · Recorded by: Zed (agent) · Authority: `AGENTS.md` resource rules and the
standing owner authorization of 2026-09-06.

Ledger file: `.uor-models/native-joint-learning-2026-09-04/model-time.json`
(this is the shared cumulative ledger; an issue, session or worktree never resets it).

## State before this reconciliation

`{"cumulative_ms":135186255,"limit_ms":140000000}` — file mtime 2026-09-17 00:41:38,
i.e. **before** the 2026-09-18 training run. No increment had been recorded for that run.

## Charges now recorded

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-18 | Full-corpus 555M-token native geometric training | 3,982,420 ms | **Reported, not independently receipted.** Taken from the "3,982.42 seconds (~66.4 minutes)" figure asserted in the 2026-09-18 `current-state.md` entry. No training log, ledger increment or result artifact with the measured figures was found in the tree. Charged anyway rather than left unrecorded, because an uncharged run is exactly the silent-drift failure the resource rules prohibit. The underlying artifact and corpus do exist (`native_geometric_prose_model.rgm` mtime 2026-09-18 17:02, `tinystories_train.u16` mtime 2026-09-17 23:10), so the work is not in doubt; only the duration figure is unverified. |
| 2026-09-19 | `ablate-prose` Card P7 sweep | 258,600 ms | **Measured.** 22.9 s smoke run (512 positions, 4 ablations) + 235.7 s full run (2,048 positions, 9 ablations), as printed by the tool and recorded in `docs/evidence/native_geometric_p7_ablation_2026-09-19.txt`. Corpus loading/mapping is included. Build and test time is engineering, not model time, and is not charged here. |
| 2026-09-19 | `ablate-prose` instrument smoke + coarse-tier removal verification | 22,000 ms | **Measured.** Instrument smoke at 1,024 positions (~44 s incl. 3 ablations), the stripped-artifact verification run at 2,048 positions (21.2 s), and `strip-coarse`'s 864-comparison scoring self-check (~2 s). Recorded in `docs/evidence/native_geometric_p7_coarse_strip_2026-09-19.txt`. |

**New cumulative: 139,449,275 ms.** Remaining against the prior ceiling: 572,725 ms
(~9.5 minutes) — insufficient for the next stage.

## Extension recorded before use

Per the standing owner authorization (2026-09-06), necessary local model/time/storage
allowance extensions are already authorized. The complete projection is recorded here
before use, as required.

**Reason.** Finish Stage 1 (confirm the three HARMFUL ablation verdicts on wider position
counts and a second disjoint slice) and run the Stage 2 evaluation work (re-evaluate after
removing the coarse lattice tier, and sweep the candidate shortlist against BPB). The
remaining 9.5 minutes cannot fund either.

**Projection.**

| Item | Estimate |
|---|---|
| Confirmation sweep: 2 disjoint 8,192-position slices × 9 ablations at ~4× the 2,048-position cost | ~1,886 s |
| Stage 2: coarse-tier removal re-evaluation and shortlist sweep (32/16/8 candidates) | ~1,800 s |
| Retry/checkpoint contingency (diagnose and re-run, no blind repeats) | ~1,800 s |
| **Total projected** | **~5,486 s ≈ 91.4 min** |

**Increment recorded: 7,200,000 ms (2 hours)**, rounding up for the contingency.

**Updated limit: 147,200,000 ms.** Remaining after the extension:
147,200,000 − 139,427,275 = **7,772,725 ms (~129.5 min)**.

## Storage

No model artifact was created or modified by the 2026-09-19 work. New retained storage is
limited to two debug binaries (`target/debug/ablate-prose`, `target/debug/attribute-recall`,
~5.3 MB combined, recreatable) and two tracked text files. The **128 MiB storage stop
margin is untouched**, and no temporary or retained model storage was added. No external
or paid compute was used.

### Storage added later the same day by the P7 coarse-tier removal

| Item | Bytes | Recreatable |
|---|---:|---|
| `.uor-models/native-geometric-prose-2026-09-19/native_geometric_prose_model_nocoarse.rgm` | 963,950 | yes, from the original artifact + `strip-coarse` |
| `target/debug/strip-coarse` | ~3 MB | yes, from source |

Net change to the retained model store: **+963,950 bytes**, and the artifact it was derived
from is 1,728,000 bytes larger, so the two together are still smaller than a hypothetical
retrained pair. The **128 MiB stop margin remains untouched**; no deletion, no cleanup and
no paid compute were performed.

## Not done

No destructive deletion, no cleanup of prior artifacts, no paid compute. The 2026-09-18
training figure remains unverified and should be replaced with the real elapsed time if a
receipt is located; that would be a correcting entry, not a rewrite of this one.
