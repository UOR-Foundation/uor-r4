# Roadmap

UOR-R4 marks progress with eight milestones, M1 to M8. **M1 is complete: 1 of 8.** A milestone
closes only when its full bar is met on a saved model. The evidence and limitations are in
[where we are](docs/start/where-we-are.md); the route and open questions are in
[where we're going](docs/start/where-were-going.md) and the one [PLAN](PLAN.md).

![The project road: M1 marked complete on the saved discrete model, while the current end-to-end line works toward grounded replies, durable memory and the remaining milestones.](docs/figures/start/roadmap.svg)

*M1 is complete; the current work is new-phrasing reply opening and one useful model end to end.*

| Milestone | Bar to close it | Status |
|---|---|---|
| [M1 Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) | One saved model passes all six bars at two seeds: self-contained D11 artifact; exact export/reference agreement and planted-fault control; tokens/s; DEV integer top-1 above both count tables; Fresh V2 transfer; and held-out-phrasing transfer | **complete, 30 Oct 2026** |
| [M2 Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) | ≥ 256/512 complete correct replies on DEV and ≥ 40% on a genuinely fresh panel, through one unified geometric model and a clean D11 export | in progress |
| [M3 Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | Provenance, correction, abstention, explicit reset and process restart all pass on the selected model; restored sessions continue identically | in progress |
| [M4 One served model](https://github.com/UOR-Foundation/uor-r4/issues/2032) | The chosen model exports to `UORLUT01`, passes the full D11/CLI quality, session and audit bars, and reports its export gap | not started |
| [M5 Laptop cost](https://github.com/UOR-Foundation/uor-r4/issues/2033) | Measured energy per token, memory and tokens per second on an M1-class laptop, with selected parameter access reported | not started |
| [M6 Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | Unseen multi-step compositions and executable coding tasks on the same model, with causal state/memory ablation | not started |
| [M7 API, browser and Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | Bit-identical tokens and sessions across library, CLI and browser on the accepted model; truthful client-only Studio | engine/harness delivered on a synthetic stand-in; accepted model open |
| [M8 Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | One versioned, checksum-pinned artifact meets M1–M7, with reproducible installation and independent consumer-hardware verification | not started |

M1's two-seed results are **7,002 / 7,001** on DEV (T2 6,300), **7,215 / 7,228** on Fresh V2
(T2 6,582), and **7,850 / 7,872** on the held-out-phrasing p-f panel (T2 7,375). The p-f panel's
62.5% reference-reply overlap with TRAIN means this is a scoped language-base result, not useful
conversation. M2 is the next capability milestone. Claude-218's offline sign-key probe passed on
p-u; current work trains and serves that key and tests free generation on another fresh panel.
Every run goes ahead on its own pre-run review (D67, [charter](CHARTER.md) §5.1). The one plan is
[PLAN.md](PLAN.md) (D77).
