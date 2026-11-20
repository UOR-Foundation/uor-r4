# D36 independent whole-context witness

The verifier decodes immutable, digest-pinned UORKNC02 inputs without calling the
materializer or its parser. It traverses every source context in canonical order and
compares each retained key and follower span byte for byte with the pruned file. It
recomputes raw and adjusted count spectra, context and follower totals, sacrificed
support and the size at the preceding integer threshold. It checks the candidate
against the compact byte cap and requires whole-file identity for all-retained
cases. The caller supplies the TRAIN digest independently of the two artifact pins.

Both files must pass the ordinary compact reader's cold load. For each order, the
first retained and first dropped context, where available, each contribute a finite
probe using a recorded follower as target. The witness checks native raw counts,
raw totals and Q48 masses against an independent integer implementation. For each
dropped context, replay after removing the oldest token must give exactly the same
candidate Q48 word and traces, with a lower context still present. The replay checks
the compact count component before any mixture, text generation or D11 packaging.

The finite corruption control changes one raw word outside spectrum bins. It
requires rejection at the original file pin, successful native load under the
altered pin, and rejection against the immutable source's exact group words. To
check omission, supply a separately verified candidate with a stricter threshold
under the original threshold and its own artifact pin. The exhaustive group
comparison must reject the missing retained key.

Build and run on Wick with an isolated target root. Keep both audited files
immutable throughout each read-only mapping.

```sh
CARGO_TARGET_DIR="$verify_target" cargo build --release --offline --locked \
  --manifest-path docs/loop/tasks/written-state-count-model/codex-36/verify/Cargo.toml
"$verify_target/release/d36-retention-verify" "$original" "$pruned" "$threshold" \
  "$compact_cap" "$train_sha256" "$original_sha256" "$pruned_sha256"
```

Append `--corruption-control` on the C128 fixture to exercise the repinned byte
alteration. The command emits a receipt only after all requested checks pass.

## Wick results

| Candidate | Cap (bytes) | Threshold | Compact bytes | Bytes at preceding threshold | Native probe pairs |
|---|---:|---:|---:|---:|---:|
| [C128 all retained](evidence/prod-c128-all.json) | 16,459,406 | 1 | 16,459,406 | — | 11 |
| [C128 registered cap](evidence/prod-c128-8m.json) | 8,388,608 | 2 | 4,517,125 | 16,459,406 | 18 |
| [C128 tied support](evidence/prod-c128-pruned.json) | 4,000,000 | 3 | 3,054,007 | 4,517,125 | 18 |
| [C128 mandatory floor](evidence/prod-c128-floor.json) | 2,074,826 | 1,963 | 2,074,826 | 2,075,296 | 11 |
| [C1 registered cap](evidence/prod-c1-256m.json) | 268,435,456 | 5 | 242,774,057 | 276,123,394 | 18 |
| [C1 D33 projection](evidence/prod-c1-t2.json) | 544,346,506 | 2 | 544,346,506 | 2,229,740,835 | 18 |

At the C128 4,000,000-byte cap, the discarded support-two tie contains 57,507
contexts. A 20-byte member of that tie would fit within the 945,993 spare bytes.
The global threshold correctly excludes the entire tie. The C128 floor artifact
also passed its own audit, then [failed the omission control](evidence/prod-c128-omission.stderr)
when substituted under the threshold-three claim at its valid digest.

The C1 registered cap retains raw support 50,848,529 of 145,015,409 and adjusted
support 16,420,012 of 102,399,038 across orders 4–10. Every original and retained
order summary emitted by the writer agreed with the independent witness on
contexts, followers, raw and adjusted support and both spectra. Emitted section
bytes, thresholds and overall byte counts also agreed. The Wick run stores the
cross-checks and executable under
`/mnt/plexpool/uor-r4-corpus/runs/20261120-whole-context-36/verify`.
These receipts characterize the compact format and finite Q48 lookups.
