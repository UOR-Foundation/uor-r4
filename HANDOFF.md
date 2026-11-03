<!-- rendered by uor-harness loop render from 83a77eb5577a4689e33a0c2bc72754384401ce66; do not edit -->
# HANDOFF

Rendered from `docs/roadmap/items/*.md` at the commit in the stamp above. Each open-item row gives the first sentence of the item's `Done when` and `Cancel if` sections; blocked items are listed once, in the blocked table, with their owner; `not recorded` means the item has no such section or key yet.

## Open items

Items not `held`, `decision` or `done`. Held and decision items are listed once, in the blocked table below.

### ready (23)

| Item | Title | Done when | Cancel if | Size | Stands |
|---|---|---|---|---|---|
| cpu-training-pilot | Run and grade the bounded CPU training pilot | - The pre-registration exists before compute and names pass, fail and stop outcomes. | Any dependency still fails parity or deterministic resume. | not recorded | ready; after cpu-discrete-training-path, cpu-shared-vector-train, cpu-quantization-exact-memory, cpu-whole-path-exact-updates |
| docs-status-refresh | Bring every status page, picture and tracker up to 31 Oct 2026 | 1. | not recorded | not recorded | ready |
| infra-loop-harness | Build the loop harness (the loop subcommands of uor-harness) | Every PR in the build plan on [the loop page](docs/loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. | not recorded | not recorded | ready |
| integer-long-context | Serve prompts and conversations past the 192-token window | 1. | not recorded | not recorded | ready; after integer-hamming-read |
| m2-train-grounded-replies | Train the student for grounded replies from exact memory | M2 bar: '>= 256/512 saved questions and >= 40% of genuinely fresh ones, with the existing graders unchanged'; the student stays on the discrete path unless the owner decides otherwise (D41); it exports to D11 with the export gap reported (D37). | not recorded | not recorded | ready; after m2-donor-corpus-generation, m2-fresh-heldout-panel, m2-pin-train-mask-sheet, m1-close-saved-model, m1-d11-exporter-discrete |
| m3-qualify-memory | Qualify durable memory on the selected native model | On the selected artifact: | not recorded | not recorded | ready; after m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests |
| m4-export-uorlut01-cli | Serve the chosen model as UORLUT01 through the uor-chat CLI | M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. | not recorded | not recorded | ready; after m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory |
| m5-laptop-full-path-measurement | Measure M1-class laptop cost on the accepted served model | M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". | not recorded | not recorded | ready; after m4-export-uorlut01-cli |
| m6-code-task-rustc-verifier | Add the rustc --test verifier the frozen M6 code-task bar names | 1. | The M6 qualification pre-registration (#2865) decides, as a bar decision card under D68, to drop or reword the `rustc --test` verifier clause; or the frozen `m6-reasoning` panel or its code-task oracle is superseded, re-frozen for a different code-task design, or re-sealed. | not recorded | ready; after m6-reasoning-panel-freeze |
| m6-qualify-reasoning-coding | Qualify reasoning and coding on the served model | M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory. | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism |
| m7-parity-library-cli-browser | Build the library, CLI and browser bit-identity parity harness | M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. | not recorded | not recorded | ready; after m7-studio-connect-wasm-engine, m4-export-uorlut01-cli |
| m8-independent-verification | Independently verify the alpha on consumer hardware | A third party reproduces the install and the M1 to M7 bars from the checksum-pinned artifact: the serving-side bars on consumer hardware, and the panel-scored bars on a GPU as CHARTER §3.6 requires. | not recorded | not recorded | ready; after m8-release-packaging |
| m8-release-packaging | Prepare the checksummed alpha release package and install path | M8 bar ([`ROADMAP.md`](ROADMAP.md) line 21): "One versioned, checksum-pinned model file, declared alpha by the owner". | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser |
| object-store-consumer-m3 | Wire the object store into the model and run the durable-memory bars | A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. | not recorded | not recorded | ready |
| one-model-end-to-end | Score one model end to end on prompts it has not seen, with tokens per second | 1. | not recorded | not recorded | ready; after turn-memory-reply |
| point-key-integer | Serve the point key's opening in the integer artifact on the laptop | The model throughout is (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3, the ones claude-217 to claude-220 scored, exported to the integer artifact. | not recorded | not recorded | ready; after one-model-end-to-end, turn-memory-integer, integer-retention-rates, integer-nearest-readout |
| scale-probe | Measure how ability grows with width and data | 1. | not recorded | not recorded | ready; after one-model-end-to-end |
| served-rounding-cost | Bring training as served back inside the 2x operational gate | 1. | The served graph's cost is not reproducible at the pinned source and data, or a repair changes any exported value. | not recorded | ready |
| studio-ide-harness | Deliver the native-style Studio IDE harness and real connection workflows | - The Studio presents a restrained Mac-inspired interface in dark/light themes and desktop/mobile layouts, with accessible controls, truthful readiness, and preserved non-model workflows. | not recorded | not recorded | ready; after studio-typed-shell-upgrade |
| studio-typed-shell-upgrade | Migrate the Studio to a typed lifecycle shell without changing its engine | - A foundation PR adds the typed lifecycle store, narrow engine/persistence adapters, and fixture-backed tests without changing the live page, engine/worker/tokenizer/`studio_api.js`, publisher, or real browser data. | not recorded | not recorded | ready; after m7-studio-live-model-sources |
| training-step-cost | Split the training backward, and run a cheaper rival search against (a) | 1. | not recorded | not recorded | ready; after fbs-cost-profile |
| turn-memory-integer | Serve the turn memory from the integer artifact on the laptop | 1. | not recorded | not recorded | ready; after route-plus-memory |
| writer-admission | Make the native writer's admission hold at every seed | 1. | not recorded | not recorded | ready; after one-decoder |

## Blocked

Items with status `held` or `decision`, each listed once. The owner is `owner` for a decision or for `open_to: owner`, otherwise the item's `open_to`.

| Item | Title | Status | Owner |
|---|---|---|---|
| m2-multi-turn-bar-number | Register a number for the multi-turn reply bar blocked under R14 | decision | owner |
| d41-driver-math-pilot | Optimize the driver math and run the ~1M-token pilot (D41 item 3) | held | lab |
| e8-orbit-memory-sigmoid-free-iteration | Build a sigmoid-free gate for E8 orbit memory and rerun it on GPU | held | lab |
| hopf-retained-fibre-test | Run the Hopf retained-fibre language test | held | lab |
| integer-nearest-readout | Serve the trained model's readout in the integer artifact | held | lab |
| m1-w4-float-credit-sheet | Register and run one float-credit training sheet (W4) | held | lab |
| m2-donor-corpus-generation | Generate the one-time offline donor answer corpus (distill-d1) | held | lab |
| native-geometric-reregister-hinge | Re-register the native geometric engine with the hinge loss | held | lab |
| r-full-epoch-revisit-schedules | Test full-epoch and frequent-token revisit schedules | held | lab |
| r-lawful-pointer-forced-copy-oracle | Build a lawful pointer, then run the forced-copy oracle | held | lab |
| r-vsa-hamming-binding | Test discrete Hamming binding (VSA) on the current writer | held | lab |
| read-binding-on-lawful-read | Port read binding to a Flock or Relation read | held | lab |

## Not doing

none recorded
