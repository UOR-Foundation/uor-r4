# Prep sheet: explicit system-memory serving bridge, 2 November 2026

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../roadmap/items/m4-export-uorlut01-cli.md).
Order:
[deepseek-1](../../loop/tasks/m4-export-uorlut01-cli/deepseek-1/ORDER.md).
Source base: `e0647330314841ddf84ba3bd7cfd57e57c03680e`.

TEST FITNESS: PENDING

## Question and prior evidence

Training places a `System: Memory: {value}.` turn between a recall question
and the assistant marker. Product `uor-chat` previously could not encode that
turn. The public protocol-2 baseline therefore failed both `Pickle` and tomato
memory probes, and Claude-259 independently identified the absent recall line
as a serving-path gap.

This test asks only whether the explicit memory value, encoded through the new
bridge, changes the integer model's output on two fixed factual questions and
whether a `Memory: none.` control changes an unknowable question. It does not
test automatic extraction.

## Base artifact

- Stack-chat source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/stack/chat.rs`,
  SHA-256 `3820ae6fe15c5fd6486b47e99b1dba7ecc4216c99a4a65e448fa71a4622b9826`.
- CLI source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-integer/src/bin/uor-chat.rs`,
  SHA-256 `215a0e9584d9416e786f61167a139584c4278cd34302d2945e0ca2e024871f41`.
- Protocol encoder source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-tokenizer/src/dialogue.rs`,
  SHA-256 `d3f70a92165ae08a1a359bf77fb83c1f0fa95fe609bbaa51d723758a01254a56`.
- Dependency lock:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/Cargo.lock`,
  SHA-256 `9ab8edb75b2a651430bda0f70edc3116d070c272b7c20ca91e85df9e386a27eb`.
- Toolchain pin:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/rust-toolchain.toml`,
  SHA-256 `0a4b55522951f5475fcdd9ebcbfbbe650bfac272e8baff599585f0e8a120d5ee`.
- Executable:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-memory-target/debug/uor-chat`,
  SHA-256 `a7ad4b83547943bd8902bd13b64596cd8699d6bf2c367347ee9c59d13b02f23c`.
- Build:

```sh
CARGO_TARGET_DIR=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/chat-memory-target \
  /home/callard/.cargo/bin/cargo build --locked -p uor-r4-integer \
  --features serving-wasm --bin uor-chat
```

Before inference, pin the source, executable, `Cargo.lock`, toolchain and all
artifact hashes below. Re-hash the executable after the run. Stop before model
output if any pin differs.

## Artifacts

- Seed 1 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 1 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-1.json`,
  SHA-256 `1ca1ad2fe36ca43b4cc7deeb46e49c275f5172720d2e2fe30fafca2c17ce1514`.
- Seed 2 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Seed 2 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-2.json`,
  SHA-256 `5c09890c37475e1fdc2ae5cb6e70c4613200a0bd6c59a1eafe7f3b4a4311f72c`.

Each tokenizer's `stack_sha256` must equal its matching artifact SHA-256.

## Data split

Every invocation uses `--protocol 2 --threads 16 --temperature 0
--max-tokens 16`.

| Id | User turn | Explicit value |
|---|---|---|
| `recall-name` | `What did we name the new puppy?` | `Pickle` |
| `recall-grow` | `What does grandpa grow?` | `tomatoes` |
| `absent-breakfast` | `What did I eat for breakfast yesterday?` | `none` |

Each prompt runs under two arms:

- `off`: omit `--memory`;
- `on`: pass `--memory PICKLE` with the value in the table.

Both artifacts run both arms. Seed 1 repeats both factual `on` arms from fresh
processes. The CLI's bridge wraps a value as `System: Memory: VALUE.` through
the same protocol-2 encoder as training.

## Steps

Use a new output root:

`/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/m4-memory-bridge-2026-11-02/`

If it exists, stop. Use fresh `seed-1/`, `seed-2/` and `seed-1-repeat/`
subdirectories. For every invocation run:

```sh
/usr/bin/time -v -o OUT.time \
  "$BIN" --stack "$ARTIFACT" --tokenizer "$TOKENIZER" --protocol 2 \
    --threads 16 --temperature 0 --max-tokens 16 ${MEMORY_ARGS:-} \
    --say "$PROMPT" >OUT.json 2>ERR.log
```

Do not reuse an output basename. Use `<arm>-<prompt-id>`. The `off` arms leave
`MEMORY_ARGS` unset; the `on` arms set it to `--memory <value>`. The seed-1
repeat runs only `on-recall-name` and `on-recall-grow` in fresh processes.

## Readings

For every run record:

- exit status, exact wall time and peak RSS;
- returned protocol schema, artifact hash, user text, memory line and reply;
- reply text, exact reply ids, stop reason, history depth and elapsed seconds;
- whether the reply contains the factual value case-insensitively;
- whether each `on` reply-id list differs from its matching `off` control;
- whether the two factual `on` outputs change from off on each seed.

The recorded `memory` field must equal `Memory: <value>.` on every `on` run and
be null on every `off` run.

## Bar and decision

- **Instrument pass:** every pin and status matches; all 14 processes exit 0;
  all schemas are protocol 2; all stops are `eos`, `short_cycle` or
  `max_new_tokens`; text decodes from ids; the seed-1 repeat is exact; and all
  output files are retained.
- If both factual memory-on replies equal their off controls on both seeds,
  report `NO_OBSERVED_EFFECT` for this explicit-injection configuration.
- If any factual memory-on reply id list differs from its off control, report
  `MEMORY_LINE_CONSUMED` as an integration observation. Separately report
  whether the expected value is present.
- The absence control is descriptive. It cannot establish whether absent-store
  reads should consult a log sieve.

No model-quality bar is set. This test distinguishes an unconnected serving
format from complete model failure; it does not qualify chat.

## Minimum test power

Two initialization seeds, two factual questions, one absence control, two arms,
and two repeated seed-1 factual arms produce 14 replies. This is enough to
detect whether the explicit line reaches the model path and whether it visibly
changes any fixed answer. It is not enough to estimate language or memory
quality.

## Cost

Local Ryzen 9 9950X only, one process at a time, 16 threads, no GPU and `$0.00`
external spend. The panel is projected below one minute and 1 MiB. Stop on any

## Stop rule

Stop on any pin mismatch, malformed record, command over 60 seconds, protocol
other than 2, peak RSS over 1 GiB or non-exact seed-1 repeat.

## Claim boundary

This test can show that an explicit `System: Memory:` turn is encoded and can
change deterministic integer outputs. It cannot show that facts are
automatically extracted, that a reply is correct, that the model generalizes,
or that the artifact is ready for useful chat.

## Non-author TEST FITNESS review

TEST FITNESS: PENDING
