# Direct trace result: Route and active leaf

Head: codex-6 implementation. Parent contract:
[claude-371/PARITY.md](../claude-371/PARITY.md).

## Conditions

- GPU: NVIDIA GeForce RTX 4090, compute capability 8.9, driver 595.99.02.
- CUDA: 13.1, AOT target `compute_89`.
- Host: `wick`.
- Fixtures: Claude-371's Route and Flock `UORLUT01` bytes.
- Rows: all 39 prefixes of lengths 1 through 3 over three tokens.
- Build: `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1`.

The synthetic fixture provenance bytes are not production artifact hashes. The finite test uses
the qualification-only loaders recorded by codex-5; production loading remains strict.

## Trace words

| Area | Comparison |
|---|---|
| Route header | frontier count, output root, completed stages and terminal atom |
| Route stages | configured frame, parent root, selected port, goal root, composed action and next node |
| Active-leaf header | query frame, output root, residual terms and query payload root |
| Active-leaf levels | gate, eligible count, kept count, consensus root, kept nodes, transport actions and carried roots |
| Trace parity | every word for all 39 rows, both fixtures, CUDA vs CPU vs serving |
| Untraced control | no allocation, with identical carry, predictions, losses, rivals and admitted cells |

## Fault controls

| Faults | Output-only K1 | Direct trace |
|---|---:|---:|
| Route-frame alternatives: 4 words x 63 other ports | **108 seen, 144 missed** | **252 / 252 seen** |
| Flock gate alternatives: 4 words x other two trits | **3 seen, 5 missed** | **8 / 8 seen** |
| Active kept-node slots over the 39 prefixes | not exposed before this unit | **126 words** |
| Active transport-action slots over the 39 prefixes | not exposed before this unit | **126 words** |
| One-word carry fault | caught | still caught |

Result: the direct trace closes C-1 and C-4 and makes C-2's five output-only misses visible.
The bucket writer's committed source hash remains unchanged because its generated source stops at
the previously qualified helper/writer prefix.

