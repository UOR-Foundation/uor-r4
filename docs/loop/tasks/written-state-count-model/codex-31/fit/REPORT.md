# [codex] A1 refit status and loaded-image audit

**The genuine reserved-TRAIN fit is held.** No fitted A1 payload, real-prompt fitted-mass replay, or fitted-weight footprint measurement was produced. The 59,180-byte order-ten layout is the size implied by the existing format. Neither DEV stream was accessed.

## What stopped production

On the leased RTX 5090, the v2 A1 executable passed its finite numerical and malformed-input checks. Its linked-library and PTX checks also passed. The supplemental disassembly of the actual loaded sm_120 image found that the assembler had lowered the typed integer quotient through `I2F.U32.RP`, `MUFU.RCP` and `F2I.FTZ.U32.TRUNC.NTZ` in both the responsibility and maximization kernels. The disassembly also contained all-zero `HFMA2` idioms. The arithmetic lowering blocked the archive-backed production fit despite its exact finite result. The disassembly SHA-256 is `b0460c8405151a25f4e0df8934f796a42a03be260d87965366788ac39215d116`. The extracted sites hash to `0d6eaf30c11e64593cffe369af367c071093452208f8a7e9b468c701772f449d`.

The corrected [CUDA source](device.cu) replaces the variable divide with integer restoring division. The reduced-width trial quotient is an upper bound. Its correction compares the product against the original full-width numerator. The corrected source SHA-256 is `76e5aaf08732c2dcc70fa46be7a25bbf4a7d139472af76b5b7510ee8a3a64ffe`. Separate changes remove variable integer division from the KN source and the ordinary encode/carrier index operations. The existing shared index implementation matched 132,800 host encode quotients and 200,156 carrier remainders, including admitted widths and maximum unsigned operands. [Exact revised FIT](FIT.md)

## Observed checks

| v2 artifact or check | Observed value |
| --- | --- |
| GPU and job | RTX 5090, `vast-61565679`, build and finite job ended 03:24:59 UTC |
| Host executable SHA-256 | `9f1be86bae835b4ad3da80a3a275cc72e15e2cb816c3f2055e57aaf437c89dc8` |
| SM120 executable SHA-256 | `d153d7201d6436ee281b6941b94ddd75628ec7ecb60a277b3f51e85d7864b433` |
| Finite row tape SHA-256 | `bbde4182b1d12cce0c0c2d038eb0206295fcad31e74679d5a0332e19f3087681` |
| Finite trace SHA-256 | `7242ac91eb8968bf3980f2d453362097bc4b6c1333a8f3b60d7fb548e061c875` |
| Finite numerical comparison | 128 invented targets, 97 exact Q12 sums, 717,024 exact Q32 words |
| Finite truncated-input control | A 15-byte row tape was refused as `invalid row tape header`. No trace was created. |
| Supplemental diagnostic PTX | Four kernel entries, zero rejected tokens. This check missed the lowering above. |

On the replacement RTX 5090 `vast-61568255`, the v3 loaded-image audit found zero data-dependent floating or tensor opcodes in the rebuilt A1 executable, KN, raw lookup and ordinary Pick images. The 128-target CUDA finite differential again matched all 97 Q12 sums and 717,024 Q32 words. The truncated-row check again refused the input. The separate 17-step client finite fixture matched all 4,095 candidate words per step and rejected a truncated row. The job ended at 03:39:09 UTC. The same audit found 11 disallowed opcodes in the n-gram counter, so that build did not produce a full C1 artifact or run the reserved-R fit.

The v4 build fixed the counter's histogram stride at the actual 256×256 launch. Its loaded n-gram counter, raw, KN, ordinary Pick and A1 images passed the disassembly check. V4 reproduced the exact C128 count artifact and its 12,678 lookup probes. V4 repeated the A1 and client finite comparisons and both malformed-row refusals. The combined finite job exited 0 at 03:43:42 UTC. The A1 executable SHA-256 was `9ee5329eedb146fed090a76e96cabf3c69497cdee3b37ad8bdb513e54ec03ae3` and the Rust driver was `e146430310a1e6465e779ae6d03d7ab7de0364204380e5f1c122ce5d22338997`. The production inputs still require full C1 materialization and independent restoration before A1 dispatch. [Counter repair review](../count/FIT-REPAIRED.md)

The public [source and registered procedure](README.md) validate the pinned C1, R, D11 and compact inputs and all 22 historical count-order entries. The real run would require equality to all 97 archived A1 pass sums and 306 archived fitted Pick words, followed by byte readback and original-target CUDA versus native reload checks. The archived 1% report carries only the Pick column. Equality to the remaining historical weight words cannot be read from it. A0, C1, C2 and C3 remain the established quality controls and were outside this artifact-reproduction attempt. [A1 scope and bars](SHEET.md) · [Historical A1 receipt](../../claude-439/run2/pick-p10.json)
