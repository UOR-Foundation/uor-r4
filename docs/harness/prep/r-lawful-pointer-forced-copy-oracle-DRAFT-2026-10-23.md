# Pre-Registration Sheet: Lawful Pointer Forced-Copy Oracle (`r-lawful-pointer-forced-copy-oracle`)

- **Roadmap item:** [`docs/roadmap/items/r-lawful-pointer-forced-copy-oracle.md`](../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md) (Issue `#2880`)
- **Mechanism:** [`docs/mechanisms/pointer-key.md`](../../mechanisms/pointer-key.md) (`pointer-key`, status: `PARKED`)
- **Sheet type:** D32 discrete integer evaluation sheet (CHARTER §3.1–§3.5, §4.3, §4.5)
- **Design and finite controls:** [`docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`](../../labs/r-lawful-pointer-forced-copy-oracle/README.md), [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../crates/uor-r4-training/src/lawful_pointer.rs)

---

## Bar

**Status: PROPOSED, NOT REGISTERED** (awaits owner/PM bar registration on `#2880` under CHARTER §5.2 before any GPU oracle evaluation is released).

The forced-copy oracle evaluates three serving-time conditions (`N`, `O1`, `O2`) on a bound discrete checkpoint equipped with `LawfulPointerHead` (`crates/uor-r4-training/src/lawful_pointer.rs`) over the frozen 256-row held-out memory/reply panel (`m2-fresh256`, `128` opposite-role history pairs):

1. **Identity check (`N`, `CopySpanOracle::Off`):**
   - `CopySpanOracle::Off` must reproduce the bound discrete checkpoint's own recorded unforced memory score bit-for-bit (`0 / 256` token mismatches and `0 / 256` row-verdict mismatches against the bound checkpoint's baseline evaluation). Any mismatch on condition `N` halts the run immediately as an `instrument` defect.
2. **Proposed three-branch diagnostic partition (`N` vs `O1` vs `O2`):**
   - Let $C_N, C_{O1}, C_{O2} \in \{0, \dots, 256\}$ be the single-row complete-reply counts (and $P_N, P_{O1}, P_{O2} \in \{0, \dots, 128\}$ the `pairs_both_complete_distinct` counts) under conditions `N` (`CopySpanOracle::Off`), `O1` (`CopySpanOracle::O1SpanLearnedGate` over each row's last stated value span located by `locate_last_value_span`), and `O2` (`CopySpanOracle::O2SpanForcedGate` with `forced_gate_margin >= gate_threshold_margin`), against the 16-block cluster-sampling standard error $\text{SE}_{\text{rows}} = 8.64$ rows ($\text{SE}_{\text{pairs}} = 4.32$ pairs) measured on [`m2-fresh256.json`](../panels/m2-fresh256.json):
     - **Branch A — Addressing bottleneck (`O1` moves decisively):** $C_{O1} - C_N \ge 32$ rows ($\ge 3.70 \times \text{SE}_{\text{rows}}$, matching the $+5/40 = +12.5\%$ historical proportion scaled to $256$ rows) and $P_{O1} - P_N \ge 16$ pairs. Finding: a true slot/span address alone unlocks the copy head under its learned gate; proceed to the follow-on slot-address training item (`SlotAddressKey`, $\ge 3$ seeds, $\ge 120$ memory rows).
     - **Branch B — Gate bottleneck (`O1` stays inside noise, `O2` moves decisively):** $C_{O1} - C_N < 32$ rows while $C_{O2} - C_N \ge 32$ rows and $P_{O2} - P_N \ge 16$ pairs. Finding: supplying the true span is insufficient under the learned gate margin, but forcing both the span and the gate margin yields the stored answer; the follow-on training item must pair `SlotAddressKey` with integer gate-margin supervision.
     - **Branch C — Downstream emission / span-boundary bottleneck (`O2` fails to move decisively):** $C_{O2} - C_N < 32$ rows (within $< 3.70 \times \text{SE}_{\text{rows}}$). Finding: even when both the exact value span and an open gate are supplied by the oracle, multi-token emission or frame continuation loses the answer; pointer-copy is not reopened until span-end stop / emission selection is repaired.

---

## Base artifact

- **Lawful pointer and span-oracle implementation:** [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../crates/uor-r4-training/src/lawful_pointer.rs) (`SlotAddressKey`, `RouteAdmissionMode`, `CanonicalFoldTable`, `CopySpanOracle`, `locate_last_value_span`, `LawfulPointerHead`), verified on host by `6 / 6` unit tests (`cargo test -p uor-r4-training --lib lawful_pointer`).
- **Bound discrete checkpoint prerequisite (`PENDING`):**
  - The historical dense float pointer checkpoints (`09e9dd3d…`, `71409eac…`, `70fef0ba…` from `ptr-addr-2026-10-10`) are barred from execution by CHARTER §3.2–§3.4 (D23, D30).
  - Before this sheet transitions from `PENDING` to `FIT`, the bound discrete checkpoint (`uor-r4.discrete-spiralcore-fbs/2` or native integer memory-answering artifact) must be pinned here by exact file path, byte length, and `sha256` digest, together with the exact GPU evaluation binary (`bin/lawful-pointer-oracle-eval` or `examples/geometric-stack`) and its compiled PTX SHA-256.

---

## Data split

All evaluation rows are drawn from the sealed, zero-overlap held-out memory/reply panel [`docs/harness/panels/m2-fresh256.json`](../panels/m2-fresh256.json) (`#2849`), generated deterministically by `docs/labs/m2-fresh-heldout-panel/instrument` with pinned `ByteBpeTokenizer` (`sha256:d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`):

| File | Role | Bytes | SHA-256 |
|---|---|---:|---|
| `plan.json` | 128 histories x 2 opposite-role queries across 16 bank blocks (`length2: 64`, `length4: 64`, `length8: 64`, `update: 32`, `reassert: 32`) | `122,268` | `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715` |
| `inputs.json` | Unscored prompt histories (`0` occurrences of `"accepted"`) | `524,850` | `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a` |
| `labels.json` | Sealed expected role-value answers and `pair_id` bindings | `68,000` | `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be` |
| `manifest.json` | Panel seal manifest | `1,384` | `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964` |

- **Contamination check:** `0` exact literal overlaps, `0` content-word overlaps, and `0` word 2-gram overlaps against all `74` historical curriculum and transfer literals on disk (`docs/labs/m2-fresh-heldout-panel/README.md`).
- **Span locator rule:** For each row in `inputs.json`, `locate_last_value_span` locates the **last** occurrence of the queried role's stated value in that row's user-turn token window `[0, W)` (handling values split across multiple tokens). The located `[start, end)` span (or `None` with reason if absent) is logged per row under `copy_oracle_spans` and bound to `labels.json` by SHA-256.

---

## Steps

1. **Host finite control verification (complete in `antigravity-43`, CPU-legal under CHARTER §3.5):**
   ```sh
   CARGO_TARGET_DIR=local/antigravity/target cargo test -p uor-r4-training --lib lawful_pointer
   CARGO_TARGET_DIR=local/antigravity/target cargo check -p uor-r4-training --bins
   cargo fmt --check -p uor-r4-training
   ```
   All `6 / 6` finite unit tests in `crates/uor-r4-training/src/lawful_pointer.rs` must pass before any GPU pod is leased.
2. **GPU forced-copy oracle evaluation (`N`, `O1`, `O2`, held until `TEST FITNESS: FIT` and typed owner release under CHARTER §5.1):**
   - Check `scripts/pod/uor-pod status` first (respecting the `<= 4` pods and `<= $8/h` caps).
   - Verify all four `m2-fresh256` SHA-256 digests (`plan.json`, `inputs.json`, `labels.json`, `manifest.json`) and the bound discrete checkpoint SHA-256 on the pod before scoring.
   - Run the three deterministic conditions (`N = CopySpanOracle::Off`, `O1 = CopySpanOracle::O1SpanLearnedGate`, `O2 = CopySpanOracle::O2SpanForcedGate`) in a single pass over the `256` rows of `m2-fresh256` on an RTX 5090 pod, emitting `oracle-summary.json` and `oracle-rows.jsonl` with per-row `[start, end)` spans, `emitted_token` traces, single-row completion counts $(C_N, C_{O1}, C_{O2})$, and opposite-role pair completion counts $(P_N, P_{O1}, P_{O2})$.
   - Copy all sealed artifacts off the pod and verify SHA-256 readback before `scripts/pod/uor-pod down`.

---

## Stop

1. **Pre-run gate:** Do not start any GPU pod or run step 2 until (a) the bound discrete checkpoint SHA-256 and GPU evaluation entrypoint are pinned in `## Base artifact`, (b) the owner or PM registers the `## Bar` on `#2880` (CHARTER §5.2), (c) a non-author reviewer records `TEST FITNESS: FIT` on this sheet (CHARTER §4.5), and (d) Casey types the owner release personally with `UOR_LAB` unset (CHARTER §5.1).
2. **Baseline identity stop (`N`):** If condition `N` (`CopySpanOracle::Off`) differs on even `1 / 256` rows from the bound discrete checkpoint's unforced evaluation, halt immediately and classify the run as `instrument` failure.
3. **Span coverage stop:** If `locate_last_value_span` fails to locate a non-empty `[start, end)` span inside the context window on any row where the queried role's value was stated in `inputs.json`, halt immediately as an `instrument` failure before grading `O1` or `O2`.
4. **Single-pass evaluation cap:** The oracle is a deterministic zero-training serving intervention (`1` pass x `3` conditions x `256` rows = `768` row evaluations). No training steps, no hyperparameter sweeps, and no post-hoc gate-threshold tuning are permitted under this sheet.

---

## Cost

- **Host build and finite controls (step 1):** `$0.00` (CPU compile and `6` unit tests in `crates/uor-r4-training/src/lawful_pointer.rs`).
- **GPU oracle evaluation (step 2, after `FIT` and owner release):** `1 x RTX 5090` pod via `scripts/pod/uor-pod` for `<= 15 minutes` wall time (`768` deterministic row evaluations, zero training steps) at `<= $0.89/h`, total estimated spend `<= $0.25` (well within the `<= 4` pods and `<= $8/h` project caps).

---

## Minimum test power

- **Panel size (`>= 200` fresh rows per CHARTER §4.3):** `256` fresh held-out rows (`128` opposite-role history pairs across `16` bank blocks) in `m2-fresh256` (`plan.json` `sha256:0ed1ab98…`, `inputs.json` `sha256:1de2fc28…`, `labels.json` `sha256:3b5c5477…`). This exceeds both CHARTER §4.3's `200`-row minimum and the `120`-row floor named in `docs/roadmap/items/r-lawful-pointer-forced-copy-oracle.md`, replacing the retired 40-row `v5` memory half (`±3` rows noise).
- **Measured noise and minimum detectable effect:** On `m2-fresh256`, the 16-block cluster-sampling standard error at the `40%` bar is $\text{SE}_{\text{rows}} = 8.64$ rows ($\text{SE}_{\text{pairs}} = 4.32$ pairs), and every question-blind or role-swap heuristic (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) scores `0 / 128` pairs (`0 / 256` pair-verified rows). The proposed `+32`-row (`+16`-pair) threshold for `O1` and `O2` represents $3.70 \times \text{SE}_{\text{rows}}$ ($p < 0.001$), cleanly separating genuine address/gate gain from panel sampling noise.
- **Seeds:** The forced-copy oracle is a deterministic zero-training intervention (`N` / `O1` / `O2`) on a fixed checkpoint, so within-checkpoint evaluation variance is zero (`0 / 512` replay mismatches verified in `development-replay-audit-summary.json`). When multiple seed checkpoints of the bound discrete model are available, `N` / `O1` / `O2` are evaluated across all available trained seeds ($\ge 2$ seeds under D22 §3; $\ge 3$ seeds required for the follow-on slot-address training item).

---

## TEST FITNESS

TEST FITNESS: PENDING - M2 #2030 pointer-key

Awaiting:
1. Owner/PM registration of the `N` / `O1` / `O2` diagnostic bar in `## Bar` on `#2880` (CHARTER §5.2).
2. Pinning of the bound discrete memory-answering checkpoint SHA-256 and GPU evaluation binary in `## Base artifact` once an M1/M2 discrete checkpoint is released and trained.
3. Non-author `TEST FITNESS` review under CHARTER §4.5.
