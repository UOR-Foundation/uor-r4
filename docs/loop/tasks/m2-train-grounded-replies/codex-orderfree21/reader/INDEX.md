# Reader verification receipts

[REPORT.md](REPORT.md) records the Wick build and commands. [pack.json](pack.json), [header.json](header.json), and [final-replay.sha256](final-replay.sha256) bind the package, executable, patch and transcripts. [gpu-reference-match.json](gpu-reference-match.json) records equality against the existing development CUDA reply and step-word hashes.

| Ordinary binary raw-ID replay | Original expected and actual IDs, identical through EOS | First mismatch |
|---|---|---|
| Development 0 and 2, job | `[2997,1541,435,261,92,607,770,397,471,16,1]` | None |
| Development 1 and 3, home | `[617,2097,315,683,3123,2211,1202,520,16,1]` | None |

The same fact and question language entered through the real session has different framing and the prior actual assistant reply. The ordinary job session begins with the housing reply `[617,2097,315,896,1843,1251,694,312,3387,3017,3684,16,1]`. Its second fact triggers repetition until 48 tokens, so neither the job nor the home question in the three-turn session is reached. The first framing mismatch is at zero-based index 4, development ID 2485 versus protocol 2 ID 2496.

The direct two-turn job case reaches the question. Relative to the original structural oracle job IDs above, it first differs at offset 4, expected 92 versus actual 261. The actual reply is `[2997,1541,435,261,261,92,607,770,397,471,16,617]` and lacks EOS. The housing-only session never reaches the query. Its first reply begins with the original housing answer IDs through the period at offset 8, then continues at offset 9 with ID 201 where the structural answer ends with EOS 1. It reaches the 48-token cap.

The three ordinary single-turn transcripts and their exact raw-ID round trips are in `final-replay/`. The reader-disabled nurse output matches the bare parent. The mismatched tokenizer and poisoned reader receipts show refusal before generation.

## Source occurrence of the first genuine client error

In the two-turn job transcript the preserved native writer uses form 945. At reply offsets 3 and 4 the same shared-row entry selects prompt position 11 twice. At offset 4 the entering cursor is already 11. The writer emits the original 261 at that position again, where the next expected ID is 92 at position 12. The next five copied IDs advance correctly, followed by the period at position 17. At offset 11 another entry selects position 24, the first token of the previous assistant reply, and emits 617. Replaying each selected position against the growing actual prefix confirmed every copied ID. In the home transcript the writer reports no held form at the first missing stop. Its suffix fallback emits 201 at offset 9 and starts repeating assistant markup. The complete source and cursor traces are in [job](trace-job-two-turn.json) and [home](trace-just-home.json).

The final combined-feature client preserved the original two-turn job transcript byte-for-byte (SHA-256 `ace1f6ee2e4d5da9881f19529e0df33bfc2c68bc59afd9bf252fdb150feff5ed`). Its ordinary control emitted the same original ID as its sole count-state trace step on the valid D36 compact [count carrier](count-raw.json). Presenting that count carrier to the reader-only script entry failed before it emitted a transcript. At this source revision the count loader additionally pins the unwrapped seed trunk SHA, so a sidecar associated with a different wrapped reader is also refused by the loader.
