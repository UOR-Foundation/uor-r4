# claude-339 early path: served artifacts and panel reports

[claude] Written by `early-commit-339.sh` as each checkpoint is exported. The owner asked for these while the
runs train (9 Nov 2026). Steps below 16,384 are trend points, not the registered read. The registered read of
step-016384 is `read-339.sh serve` after the seals, with its own pinned scorer.

The exports use the existing laptop snapshot path (`hamming-base-copy`, then `cpu-fbs-export`, then the pod's
`hamming_ask` spliced in). The `-C` artifact is `fbs-toks ARTIFACT --drop-hamming-carry --out NEW_FILE` at
`402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`. Scorer: `served-reply-panel --engine integer` at the same commit, on
`m2-fresh256` (256 rows, 128 pairs). "L-a" has no carry table, so it has no `-C` artifact.

| cell | step | artifact | asked first line | other first line | complete rows | complete-distinct pairs | EOS | BOS | cap | context | collapse | distinct replies | most common reply |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| L-aC-s1 | step-008192 | carried | 0 | 0 | 0 | 0 | 18 | 0 | 238 | 0 | 14 | 137 | 14 |
| L-aC-s1 | step-008192 | exact -C | 0 | 0 | 0 | 0 | 39 | 0 | 217 | 0 | 25 | 135 | 25 |

## Files and hashes

### L-aC-s1 step-008192

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-aC-s1/step-008192/` (fetch with `scripts/pod/uor-store get`).

```text
d94669b12a0d5740fbe7b6a29692242653c4e8e1cf24e11dbaf57d6b3df7ec98 served.uorlut
b0d02801af4c25ab8b8b8c0fd5eb6254b93ee5e921e9d04275bdfb17acf693e2 served-minus-C.uorlut
5ddd94db16e7b0d265b086a9eef0a57b61098ad2d6a17f55b850d13c66587f47 score.json
db558bbcf8eb92d20a304c6f760564dbe53441b2be6df8f3fe7b31b5c66f6551 score-minus-C.json
d6b03418c1feba015ad90b8188e8e17a9a2fa3fdfc634fcebcb487d0a08c8458 pod.snapshot.json
01e532cdd9d025d71323e511a8650d89aa758ecc9caa4f8019f51d1253bba28b served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```
