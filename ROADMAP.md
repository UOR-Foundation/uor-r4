# Roadmap

UOR-R4 marks progress with eight milestones, M1 to M8. **M1 is complete: 1 of 8.** A milestone
closes only when its full bar is met on a saved model. The evidence and limitations are in
[where we are](docs/start/where-we-are.md); the route and open questions are in
[where we're going](docs/start/where-were-going.md) and the one [PLAN](PLAN.md).

![The project road: M1 marked complete on the saved discrete model, while the current end-to-end line works toward grounded replies, durable memory and the remaining milestones.](docs/figures/start/roadmap.svg)

*M1 is complete; the current work is new-phrasing reply opening and one useful model end to end.*

| Milestone | Bar to close it | Status |
|---|---|---|
| [M1 Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) | One saved model passes the approved six-part close-grade contract at two seeds ([decision log 6515151261](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6515151261)): self-contained D11 artifact; exact export/reference agreement and planted-fault control; tokens/s; DEV integer top-1 above both count tables and the 6,324 target (T2 6,300 + 24); Fresh V2 transfer; and held-out-phrasing transfer | **complete, 30 Oct 2026** |
| [M2 Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) | ≥ 256/512 complete correct replies on DEV and ≥ 40% on a genuinely fresh panel, through one unified geometric model and a clean D11 export | in progress |
| [M3 Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | Provenance, correction, abstention, explicit reset and process restart all pass on the selected model; restored sessions continue identically | in progress |
| [M4 One served model](https://github.com/UOR-Foundation/uor-r4/issues/2032) | The chosen model exports to `UORLUT01`, with held-out quality at or below **0.90 BPB**, passes the full D11/CLI quality, session and audit bars, and reports its export gap | not complete; accepted model path not started |
| [M5 Laptop cost](https://github.com/UOR-Foundation/uor-r4/issues/2033) | Measured energy per token, memory and tokens per second on an M1-class laptop, with selected parameter access reported | not started |
| [M6 Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | Unseen multi-step compositions and executable coding tasks on the same model, with causal state/memory ablation | not started |
| [M7 API, browser and Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | Bit-identical tokens and sessions across library, CLI and browser on the accepted model; truthful client-only Studio | engine/harness delivered on a synthetic stand-in; accepted model open |
| [M8 Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | One versioned, checksum-pinned artifact meets M1–M7, with reproducible installation and independent consumer-hardware verification | not started |

M1's two-seed results are **7,002 / 7,001** on DEV (T2 6,300), **7,215 / 7,228** on Fresh V2
(T2 6,582), and **7,850 / 7,872** on the held-out-phrasing p-f panel (T2 7,375) ([M1 close](docs/labs/m1-close-saved-model/README.md), [Fresh V2](docs/labs/m1-fresh-panel-transfer/README.md), [bar-f](docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)). The p-f panel's
62.5% reference-reply overlap with TRAIN means this is a scoped language-base result, not useful
conversation. M2 is the next capability milestone. Claude-220 serves the point key inside one
float model on fresh p-w, improving templates by +41/+40/+42 at three seeds with exact claude-219
parity; the gain recalls TRAIN replies, not new wording. Claude-222 has built the integer point
table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](docs/loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](docs/loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](docs/loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](docs/loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](docs/loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](docs/loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun.
Every run goes ahead on its own pre-run review (D67, [charter](CHARTER.md) §5.1). The one plan is
[PLAN.md](PLAN.md) (D77).
