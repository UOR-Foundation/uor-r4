# The three products

UOR-R4 aims to ship one thing people can use: a small, fast language model that runs on your own
laptop or in your browser, with local computation by default and no model server required. An optional user-configured API sends prompts to the endpoint the user chooses. The three products are the **model** in
one integer file, a **web client** that runs it in a browser tab, and a **desktop client** that runs
it natively with durable memory. **None ships today.** M1 is complete (1 of 8), but no useful
conversation, integrated qualified product or alpha release exists.

Labels: **proof**, **measurement**, **hypothesis** ([`docs/formal_vocabulary.md`](../formal_vocabulary.md), [glossary](../start/glossary.md)).

![One integer model file feeding a browser-based web client and a native desktop client. Local mode has no default model server; an optional user-configured API sends prompts to the chosen endpoint.](../figures/products/product-family.svg)

*One model file is intended to feed two clients sharing one interface and local storage.*

## The three at a glance

| Product | What it is meant to be | What exists today | Main gate |
|---|---|---|---|
| [Language model](./language-model.md) | A geometric language model exported as one checksum-pinned `UORLUT01` file | The discrete M1 pair and a separate exact FBS `lut-stack/5` export, with serving evidence; no conversational product release | M2–M4, then M8 |
| [Web client](./web-client.md) | A local model in the browser: download once, run in WebAssembly on the device | Synthetic parity harness plus public M1 top-8 next-token prediction in the live Studio; no chat or qualified bundle | M7 |
| [Desktop client](./desktop-client.md) | A Tauri app sharing the interface, with native engine, file access, durable memory and offline use | [`src-tauri/`](../../src-tauri/) backend, Studio host and GUI shell with seven integration controls on the synthetic stand-in; no qualified bundle | M3, M5, M7 |

[Releases and artifacts](./releases-and-artifacts.md) covers the file format, future M8 release,
Hugging Face repositories and checksum binding.

## Why the products look like this

**Local by design.** The mission is fast local AI, and the lead metric is tokens per second. A
product that needs a data centre would defeat the point.

**One file, many hosts.** D11 bans float and transcendental arithmetic and excludes dense matmul.
D80 and D81 allow integer multiply and divide. That lets one integer artifact run natively, in a
browser or on Apple silicon. Serving the exported artifact on the user's device is the product and
is allowed ([charter](../../CHARTER.md), section 3.6).

**One interface, two shells.** The browser runs the engine as WebAssembly; the desktop app wraps
the same Studio and runs native Rust. Native being faster is a **hypothesis** until a same-file
comparison exists.

## Where each product stands

| Item | State | Source |
|---|---|---|
| Milestones complete | **1 of 8 (M1)** | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| Saved discrete M1 model | DEV 7,002 / 7,001; Fresh V2 7,215 / 7,228; p-f 7,850 / 7,872; Apple M1 decode 10,876 / 10,566 tok/s | [M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| Separate FBS trunk | Exact `lut-stack/5` export at DEV scope; not joined to the M1 artifact | [pipeline map](../model/pipeline-map.md) |
| Integer serving engine, native and WebAssembly | Audits green; integer multiply and divide allowed; float remains banned | [serving dossier](../mechanisms/lut-serving.md) |
| Studio connected to the integer engine | Synthetic parity harness passes; the live Studio serves the public M1 model as top-8 next-token prediction, not chat; desktop remains synthetic-only | [web client](./web-client.md), [desktop client](./desktop-client.md) |
| Public website deploy | DeepSeek's Studio is live at `/studio/`; mdBook preview publishes on main, production only by manual dispatch | [`deploy.yml`](../../.github/workflows/deploy.yml), [`docs-site.yml`](../../.github/workflows/docs-site.yml), [web client](./web-client.md) |

A passing audit shows that a forbidden instruction is absent. It is not a trained capability, and
M1's result does not qualify useful conversation.

## Since M1 closed

The open research problem is complete replies on new phrasing. Claude-217's cycle step ends every
registered loop but leaves content underpowered, so greedy remains the decode. On p-s, reply
openings are 17.4%, 13.7% and 18.6% at three seeds, against about 80% on later tokens. Exact turn
memory keys on exact first token ids and misses reworded prompts. [Claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) now serves the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. Claude-222 has built the integer point table, search and `/8` record, but only synthetically. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [integer-nearest-readout](../roadmap/items/integer-nearest-readout.md) repairs the readout and checks parity before rerunning p-x; M2–M4 remain open.

## The order things must happen in

1. **M2 and M3** produce complete grounded replies and durable memory on the selected model.
2. **M4** exports and qualifies that model through D11 and the command line.
3. **M5** measures laptop cost, **M6** adds reasoning and coding on the same model.
4. **M7** proves library/CLI/browser parity and truthful client-only Studio behavior.
5. **M8** releases one versioned, checksum-pinned model that meets every earlier bar.

The desktop client rides on M3 for memory, M5 for laptop cost and M7 for the shared interface.
Full definitions are in the [project track](../integration/project-track.md).

## Go deeper

- [Language model](./language-model.md): the artifact, M1 evidence and M1–M8 path.
- [Web client](./web-client.md) and [desktop client](./desktop-client.md): architectures and remaining gaps.
- [Releases and artifacts](./releases-and-artifacts.md): `UORLUT01`, future release machinery, Hugging Face and checksums.
- [Serving and export](../model/serving-and-export.md) and the [integer serving dossier](../mechanisms/lut-serving.md).
- [Charter](../../CHARTER.md), sections 2 and 3.6.
