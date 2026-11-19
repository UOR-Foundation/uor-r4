# D25 original-prefix contextual vector diagnostic

This optional diagnostic checks which integer vector the contextual source
reader receives before scoring. It starts with the seed's original 19-table
D11 trunk and original token IDs. It reuses the production CUDA Relation
`read_vector` device function and independently implements the D11 recurrence,
causal successor, joint-pointer gate, and final signed action and snap. GPU
output includes the gate. The GPU kernel scans vocabulary IDs in ascending
order and admits the same union of query, read, recurrence, and pointer
identities as the Rust gate. The count tables, directional frames and
normalization actions come directly from the validated artifact.

The prefix file has no header. Each TSV row is an increasing unsigned row ID
followed by the original token IDs ending just before the bound assertion.
The packer accepts 1–4096 rows and 1–192 IDs per row. It preserves positions
and refuses an overlong prefix. It uses the ordinary ternary `BinderPlan`
for each prefix and applies the CUDA carry convention of padding with its
first token beyond the valid length. The packer executes no model scoring.
Seal the prefix, base artifact, binary packet and executable identities.

From the repository root, with `Q` set to the owned build directory:

```sh
CONTEXT=docs/loop/tasks/written-state-count-model/codex-25/context
CARGO_TARGET_DIR="$Q/target-context" cargo build -p uor-r4-discrete --release
GEOMETRY=$(find "$Q/target-context/release/build" -path '*/out/geometry.rs' -print -quit)
D25_DISCRETE_GEOMETRY="$GEOMETRY" CARGO_TARGET_DIR="$Q/target-context" \
  cargo build --locked --release --manifest-path "$CONTEXT/Cargo.toml"
"$Q/target-context/release/d25-context-pack" seed1.d11 original-prefix.tsv context.packet
sed '/^extern /,$d' crates/uor-r4-discrete/src/discrete.cu > "$Q/discrete_read.cuh"
nvcc -std=c++17 -O2 -gencode arch=compute_89,code=sm_89 -I "$Q" \
  "$CONTEXT/context.cu" -o "$Q/d25-context"
```

The source extraction retains the original device functions and omits the
ordinary executable's CUDA entry points. Build the GPU executable ahead of
time with CUDA 12.8.1 and inspect its cubins and SASS. Only the GPU executable
and packets produced by the host packer go to the GPU machine. Each command
requires a fresh output path.

```sh
./d25-context context.packet device-context.tsv
cmp device-context.tsv serving-context.tsv
```

The output has no header. Each TSV row consists of the row ID and all 32
signed post-gate coordinates in block order. The reference consists of the
same original row IDs and native serving context words. Candidate selection,
writing, and panel grading are outside this diagnostic.
