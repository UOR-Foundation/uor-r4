# Order-free reader D11 and ordinary CLI on Wick

The genuine text path dispatches the saved reader. The job and home three-turn conversations both fail before the question. The first fact receives an unrelated housing reply ending in ID 1. The second fact reaches the 48-token cap while repeating assistant markers and the housing phrase. A shorter two-turn job conversation reaches the question and emits “Your job is a azure orchard. You” with the correct source value but no EOS. Its first and last IDs are 2997 and 617, and it exhausts the reader after 12 tokens. A two-turn housing case with only the housing fact never reaches its question because the first response repeats the housing phrase and role marker until its 48-token cap. Protocol 1 reaches the cap on the second fact as well.

## Exact artifact and replay

| Item | SHA-256 |
|---|---|
| 120,755,422-byte `order-free-seed1.d11` | `2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302` |
| Embedded 34,219,034-byte reader | `9db7e8348bb0bdadee7502a35716d39dacb8e9c25e57f76338fe3fea91e1227b` |
| Embedded 86,536,132-byte seed-1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| Released trunk tokenizer used by the CLI | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| Final `uor-chat` executable | `8e752e8ddb896f93b6a0cafc70e85180cbf95cc289274be8e31113f81ab7e2e0` |

Source archive is `4372ed414487c68cdd79bb680ac8cc110c5df631`, with `ordinary-client-final.patch` applied only in this scratch tree. The reader and trunk byte ranges were independently hashed and matched the D11 envelope. The patch resolves the legacy released tokenizer pin before replacing the parent digest with the wrapper digest. It dispatches the saved reader for default read mode and adds ordinary multi-turn text script serving with actual reply IDs appended through EOS. If the reader does not produce EOS, the script reports the stop and does not synthesize the rest of the conversation. The caller supplies only user text in the ordinary path. Labeled facts, query positions, and empty assistant outputs are absent from these runs.

The loaded D11 produced the same reply stream and served step word stream as the existing Wick CUDA report on the known 512-row development fixture. The 6,664 IDs include EOS. `development-package-words.json`, `developmental-existing-gpu-reference.json`, and `gpu-reference-match.json` bind the archived GPU and current package outputs. Four exact development prompts also passed through the ordinary binary’s raw-ID mode, with first IDs 2997, 617, 2997, and 617 and all stopping at ID 1. The development fixture includes historically supplied empty assistant turns and uses different text framing from straightforward input. For the same initial job fact, the first protocol 2 token difference is offset 4, development ID 2485 and client ID 2496. See `final-replay/first-framing-difference.json`.

Three ordinary single-turn text fixtures gave identical output IDs when fed back via raw IDs. Disabling the carried reader matched the bare trunk on the nurse fixture. The training tokenizer JSON shares the released tokenizer vocab and merges but differs in decoder metadata and version. The CLI refused its mismatching digest, and loading a package with one mutated reader byte failed the binding check. Both failures emitted no reply. The release build, reader binding test, a reader-free CLI check, and a combined reader/count feature check passed. The spent evaluation panel was untouched. This run compares serving against an existing CUDA receipt and carries no new release or research qualification.

## Reproduce

From this directory on Wick, the ordinary one-shot entry is:

```sh
source/target/release/uor-chat --discrete order-free-seed1.d11 \
  --tokenizer /home/callard/uor-codex/20261118-diagnostics/inputs/tokenizer.json \
  --say 'My current job is a nurse. What is my current job?' --max-tokens 24
```

Use `--script replay/job-script-input.json --max-tokens 48` in place of the single-turn arguments for the three-turn conversation. The final transcripts and their hashes are in `final-replay/` and `final-replay.sha256`.
